# Authentification & sécurité

> Voir aussi [turso.md](turso.md) pour la couche de persistance (libSQL, coffre chiffré, sync Phase 2).

## Objectif

Les données de présence sont **très confidentielles** → exigence « niveau banque ». Trois propriétés :

1. Le mot de passe est stocké de façon **irréversible**.
2. Les données au repos sont **chiffrées**, déchiffrables uniquement après saisie du mot de passe.
3. Session **courte (15 min)**, mot de passe redemandé à **chaque démarrage**.

## Vue d'ensemble : envelope encryption

Le modèle est celui des gestionnaires de mots de passe (Bitwarden, 1Password) : le mot de passe ne chiffre pas directement les données, il déverrouille une clé qui, elle, chiffre les données.

```
mot de passe ──Argon2id(kek_salt)──▶ KEK ──(dé)chiffre──▶ DEK ──chiffre──▶ vault.db
     │                                                      ▲
     └──Argon2id(sel PHC)──▶ password_hash (vérification)   │
                                                            DEK = 32 octets aléatoires
```

Composants cryptographiques :

- **Argon2id** (1er choix OWASP). **Deux usages distincts, sels et paramètres séparés** :
  1. **Hash d'auth** (chaîne PHC) → vérifie le mot de passe. Profil OWASP « 46 MiB » (`m=47104, t=1, p=1`). [argon2_hasher.rs](../src-tauri/src/infrastructure/crypto/argon2_hasher.rs)
  2. **Dérivation du KEK** (Key Encryption Key, 32 octets) → enveloppe le DEK et la clé MAC. Profil **plus fort** (`m=65536` = 64 MiB, `t=3`, `p=1`, inspiré de la 2ᵉ recommandation de la RFC 9106) : c'est la KEK qu'un attaquant détenant le keystore *et* la clé de device chercherait à casser hors ligne. Les paramètres sont **stockés par compte** (`kdf_m_cost/t_cost/p_cost`). [key_service.rs](../src-tauri/src/infrastructure/crypto/key_service.rs)
  Politiques dans [domain/policy.rs](../src-tauri/src/domain/policy.rs), valeurs dans [config.rs](../src-tauri/src/infrastructure/config.rs). Tout paramètre relu depuis le stockage (chaîne PHC, KDF du compte) est **borné** (`KdfParams::within_bounds`, ≤ 256 MiB, t ≤ 10) : un keystore altéré ne peut pas imposer un calcul de plusieurs Go. Le calcul Argon2 tourne sur le **pool bloquant** (`spawn_blocking`), jamais sur l'exécuteur async.
- **DEK** (Data Encryption Key, 32 octets aléatoires) = la vraie clé du coffre `vault.db`.
- **XChaCha20-Poly1305** (AEAD **authentifié**, nonce aléatoire 192 bits) = enveloppe (wrap) le DEK et la clé MAC avec le KEK. Format **v2** : *associated data* `"bp-wrap-v2" ‖ rôle (dek | vault-mac) ‖ id du compte` — le DEK et la clé MAC enveloppés ne sont plus interchangeables, ni transplantables d'un compte à l'autre. Les comptes au format v1 (sans AAD) sont **ré-enveloppés automatiquement au login** suivant ([credentials.rs](../src-tauri/src/application/services/credentials.rs)).
- **libSQL AES-256-CBC** = chiffrement at-rest du coffre, clé = DEK (voir [turso.md](turso.md)).
- **Clé de device** (32 octets aléatoires, **scellée dans le trousseau de l'OS** — libsecret / Keychain / Credential Manager via le crate `keyring`) = chiffre **aussi** `keystore.db` au repos (AES-256-CBC). Résolue au démarrage, indépendante du compte. [device_key.rs](../src-tauri/src/infrastructure/crypto/device_key.rs)
- **Clé MAC** (32 octets aléatoires, **enveloppée par le même KEK** que le DEK) = évidence d'altération du coffre via HMAC-SHA256 du fichier (le CBC libSQL n'étant pas authentifié). La base de référence et l'état « session en cours » sont conservés **dans le keystore scellé**, pas dans des fichiers à côté du coffre (voir [Durcissements](#durcissements-implémentés)). [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs)

### Flux d'inscription (`register`)

[register_account.rs](../src-tauri/src/application/use_cases/register_account.rs)

1. Valider username / mot de passe (longueurs, [domain/policy.rs](../src-tauri/src/domain/policy.rs)). Refuser si un compte existe déjà (mono-utilisateur).
2. `password_hash = Argon2id(password)` (sel propre, intégré au PHC).
3. `DEK = aléatoire(32o)` ; `MAC key = aléatoire(32o)` ; `kek_salt = aléatoire` ; `KEK = Argon2id_kek(password, kek_salt)`.
4. `wrapped_dek = XChaCha20Poly1305(KEK).encrypt(DEK, aad = v2‖dek‖id)` ; idem pour la clé MAC (`aad = v2‖vault-mac‖id`) — [credentials::seal](../src-tauri/src/application/services/credentials.rs).
5. Insérer dans `keystore.db` (lui-même **scellé** par la clé de device) en **une seule instruction** `INSERT … WHERE NOT EXISTS (SELECT 1 FROM account)` : deux inscriptions concurrentes ne peuvent pas créer deux comptes.
6. Le mot de passe (et le DEK/KEK/MAC en mémoire) sont **effacés** (`Zeroizing`).

### Flux de connexion (`login`)

[login.rs](../src-tauri/src/application/use_cases/login.rs)

Les vérifications de mot de passe (login, changement de mot de passe) sont **sérialisées** (`CredentialGate`) : des tentatives parallèles ne peuvent pas toutes passer le contrôle de verrouillage avant que le compteur ne les rattrape.

1. Refuser sans calcul un username > 64 ou un mot de passe > 1024 caractères. Charger le compte. _(Si username inconnu : hash « à blanc » pour égaliser le timing, puis `InvalidCredentials`.)_
2. Si verrouillé (`locked_until` futur) → `AccountLocked`.
3. Vérifier `password_hash`. Échec → `failed_attempts` **incrémenté atomiquement** (`UPDATE … RETURNING`), verrouillage progressif au-delà du seuil (voir [Anti-bruteforce](#anti-bruteforce)), `InvalidCredentials`.
4. Succès → `KEK = Argon2id(password, kek_salt, paramètres du compte)` → `DEK`, clé MAC = `unwrap(…, aad)` selon la version du format.
5. **Mises à niveau transparentes** (le mot de passe est disponible) : format v1 ou paramètres KDF périmés → nouveau sel, nouvelle KEK, ré-enveloppement v2 ; chaîne PHC aux anciens paramètres → nouveau hash ; clé MAC absente → générée (*backfill*). Écrit en une seule mise à jour (`update_credentials`).
6. **Ouvrir le coffre** : vérifier d'abord l'intégrité at-rest avec la clé MAC (état lu dans le keystore), marquer la session « ouverte » **avant** toute écriture (migrations comprises), puis déchiffrer avec le DEK.
7. Réinitialiser les compteurs, **purger les sessions précédentes** (mono-propriétaire), journaliser l'événement `login`, créer une **session** (token, `expiresAt = now + 15 min`, `lastSeen = now`).
8. Renvoyer `{ token, expiresAt, user, failedAttemptsSinceLastLogin, securityEvents }` — l'UI affiche une fois ce qui s'est passé depuis la dernière connexion.

### Le « chicken-and-egg » résolu

Le matériel d'auth (hash + DEK **enveloppé**) vit **hors** du coffre chiffré, dans `keystore.db`. On peut donc **vérifier le mot de passe et reconstruire la clé avant** d'ouvrir le coffre. Le keystore est lui-même **scellé au repos** par la clé de device (issue du trousseau OS), résolue au démarrage — ce qui n'enlève rien au raisonnement : une fois le keystore ouvert, le matériel d'auth reste accessible avant le déverrouillage du coffre.

## Stockage : keystore vs vault

Keystore (**scellé** par la clé de device), migrations [keystore/](../src-tauri/migrations/keystore/) :

```
account          id, username, password_hash, wrapped_dek, kek_salt, dek_nonce,
                 failed_attempts, locked_until, created_at, updated_at,
                 wrapped_mac_key, mac_key_nonce,
                 key_version (1 = sans AAD, 2 = AAD), kdf_m_cost, kdf_t_cost, kdf_p_cost
vault_integrity  (ligne unique) baseline_mac, session_open, updated_at
security_event   id, kind, at_ms   (50 dernières entrées, aucun contenu métier)
```

Le coffre `vault.db` (chiffré par le DEK) contient toutes les données métier : profils, présences, trajets, heures, notes, paramètres.

**Écritures dans le coffre** : toutes passent par `LibsqlVaultManager::write` / `unit_of_work` ([vault.rs](../src-tauri/src/infrastructure/persistence/vault.rs)) — verrou d'écriture, **connexion dédiée** par transaction (`BEGIN IMMEDIATE … COMMIT`), et transaction « ambiante » (task-local) que rejoignent tous les dépôts appelés pendant une unité de travail. Une connexion partagée entre commandes concurrentes mélangerait leurs `BEGIN`/`ROLLBACK`. Les écritures multi-dépôts d'un use case (jour + horaire, entrées + horaire, profil + profil actif, import complet) passent par le port `UnitOfWork` : tout ou rien.

## Session

- **Token** = 32 octets aléatoires (`getrandom`), encodés base64url. [token_generator.rs](../src-tauri/src/infrastructure/crypto/token_generator.rs)
- **En mémoire uniquement** : côté Rust `InMemorySessionStore` (`Mutex<HashMap>`, [in_memory_session_store.rs](../src-tauri/src/infrastructure/session/in_memory_session_store.rs)) ; côté React un `useRef` dans [auth-provider.tsx](../src/features/auth/presentation/providers/auth-provider.tsx). **Jamais** dans `localStorage`. → app fermée = session perdue = re-login obligatoire.
- **Tokens jamais stockés en clair** : le store indexe les sessions par **SHA-256 du token** et la copie stockée est expurgée — un dump mémoire ne révèle que des empreintes, et la recherche compare des digests (pas de signal de timing sur le secret).
- **Expiration absolue 15 min** : basée sur `expiresAt` (epoch ms), donc insensible à la mise en veille / aux changements d'horloge. Timer front : [use-session-timer.ts](../src/features/auth/presentation/hooks/use-session-timer.ts).
- **Appliquée côté backend** : chaque commande touchant au coffre passe d'abord par [require_session.rs](../src-tauri/src/application/use_cases/require_session.rs) — s'il ne reste aucune session valide, les sessions restantes sont révoquées et le **coffre est verrouillé**, puis `SESSION_EXPIRED` est renvoyé. L'expiration tient donc même si la WebView (compromise ou boguée) n'appelle jamais `check_session`.
- **Timeout d'inactivité** : côté UI, 5 min sans interaction (souris / clavier / scroll) déconnectent ; au retour au premier plan, c'est le **temps réellement écoulé** qui compte (une fenêtre en arrière-plan n'est plus « réarmée »). [use-idle-timeout.ts](../src/features/auth/presentation/hooks/use-idle-timeout.ts), seuil `IDLE_TIMEOUT_MS` ([config.ts](../src/core/config.ts)).
- **Inactivité appliquée côté backend** (6 min) : chaque session porte `last_seen_ms`, mis à jour par `require_session` et par `check_session` ; au-delà, la session est invalide et le coffre verrouillé — même si l'UI ne fait rien. L'UI « ping » `check_session` (au plus une fois par minute) tant que l'utilisateur est actif, pour qu'un utilisateur qui lit sans déclencher d'appel ne soit pas déconnecté.
- À l'**expiration ou au logout** : session révoquée + coffre **fermé** (après la fin des écritures en cours) + DEK **oublié**, et la **base d'intégrité du coffre est enregistrée** dans le keystore. [check_session.rs](../src-tauri/src/application/use_cases/check_session.rs), [logout.rs](../src-tauri/src/application/use_cases/logout.rs). `logout` n'agit que pour le **détenteur du token** (un token inconnu est sans effet). La fermeture de la fenêtre et tout chemin de sortie (`RunEvent::Exit`) verrouillent aussi le coffre ([lib.rs](../src-tauri/src/lib.rs)) ; avant de redémarrer sur une mise à jour, l'UI se déconnecte.
- **Modèle de session global** (décision [ADR 0001](adr/0001-session-globale.md)) : les commandes du coffre ne reçoivent pas le token ; la session est un état temporel du processus, pas une capacité.

## Anti-bruteforce

[login.rs](../src-tauri/src/application/use_cases/login.rs) + colonnes `failed_attempts` / `locked_until`.

- 5 échecs (`max_attempts`) → verrouillage 5 min (`lockout_ms`), puis **progressif** : chaque échec supplémentaire double la durée, plafonnée à 24 h (`AuthPolicy::lockout_for`, [policy.rs](../src-tauri/src/domain/policy.rs)). Le compteur n'est remis à zéro que par une connexion réussie.
- Compteur incrémenté **atomiquement** et tentatives **sérialisées** : une rafale de tentatives parallèles n'en fait passer que `max_attempts`.
- Le changement de mot de passe re-vérifie le mot de passe actuel et ses échecs comptent aussi.
- Erreur **générique** `InvalidCredentials` (ne révèle pas si le username existe).
- Hash « à blanc » pour les usernames inconnus (timing constant).

## Commandes IPC

[auth.rs](../src-tauri/src/presentation/commands/auth.rs) — fines, sans logique métier. Erreurs sérialisées en `{ code, message }`, les détails internes (SQL/crypto) sont masqués (`INTERNAL`).

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `account_exists` | – | `boolean` |
| `register` | `username, password` | `UserDto { id, username, createdAt, updatedAt }` |
| `login` | `username, password` | `{ token, expiresAt, user: UserDto }` |
| `check_session` | `token` | `{ valid, remainingMs }` |
| `logout` | `token` | `void` (sans effet si le token est inconnu) |
| `change_password` | `currentPassword, newPassword` | `void` (session requise ; ré-enveloppe DEK + clé MAC, le coffre n'est pas re-chiffré) |

### Profils

[profile.rs](../src-tauri/src/presentation/commands/profile.rs) — CRUD des profils de présence (stockés dans le **coffre chiffré**). Garde d'accès : aucun token transmis (il n'apporterait rien : il vit dans la WebView), mais chaque commande appelle `require_session` qui vérifie côté Rust qu'une session **non expirée** existe — sinon révocation + verrouillage du coffre + `SESSION_EXPIRED`. Un profil introuvable renvoie `NOT_FOUND`. La même garde s'applique aux commandes présences et CO₂/trajets.

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `create_profile` | `firstName, lastName, enterprise, poste?` | `ProfileDto { id, firstName, lastName, enterprise, poste, createdAt, updatedAt }` |
| `list_profiles` | – | `{ profiles: ProfileDto[], activeProfileId }` |
| `update_profile` | `id, firstName, lastName, enterprise, poste?` | `ProfileDto` |
| `delete_profile` | `id` | `void` |
| `set_active_profile` | `id` | `void` |

> Le profil actif est persisté dans `vault_meta` (clé `active_profile_id`). Le premier profil créé devient l'actif ; supprimer l'actif le réassigne au premier restant.

### Paramètres par profil

[settings.rs](../src-tauri/src/presentation/commands/settings.rs) — réglages **propres à chaque profil**, en colonnes typées dans la table `profile_settings` du **coffre chiffré** (migration [0010](../src-tauri/migrations/vault/0010_add_profile_settings.sql)). Mêmes gardes d'accès. Le dépôt suit le motif « défaut paresseux » : un profil sans ligne renvoie `ProfileSettings::default()`.

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `get_profile_settings` | `profileId` | `ProfileSettingsDto` |
| `set_profile_settings` | `profileId, settings` | `ProfileSettingsDto` (ligne canonique) |

> `ProfileSettingsDto` (camelCase, à plat) = `{ defaultStartMinutes, noteFont, cellDisplayMode, gridCountry, defaultCarOccupancy, includeRadiativeForcing, countBuildingEnergy, workingDaysPerYear, factorYear }`. Validé côté use case **et** par des `CHECK` SQL (enums `noteFont`/`cellDisplayMode`/`gridCountry`, bornes start/occupants/jours).

> **Heure de départ par défaut** : à la création d'un jour `office`/`remote`, `set_presence` sème la `work_day_schedule` avec `defaultStartMinutes` **si aucun horaire n'existe encore** (jamais d'écrasement d'une heure saisie). La police des notes et le mode d'affichage des cellules — auparavant en `localStorage` (niveau appareil) — sont désormais **par profil** ; seuls le **thème** et la **langue** restent au niveau de l'appareil.

### Présences

[presence.rs](../src-tauri/src/presentation/commands/presence.rs) — présences quotidiennes (stockées dans le **coffre chiffré**). Même garde d'accès que les profils : coffre **déverrouillé** requis (sinon `SESSION_EXPIRED`). Au plus une présence par `(profileId, day)` — `set_presence` fait un *upsert* sur le jour (conserve `id`/`createdAt`).

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `set_presence` | `profileId, day, type, trips?` | `PresenceDto { id, profileId, day, type, co2Kg, isEstimated, createdAt, updatedAt }` |
| `list_presences` | `profileId` | `PresenceDto[]` |
| `delete_presence` | `id` | `void` |

> `day` = epoch ms à **minuit UTC** (validé côté domaine) ; `type` ∈ `office | remote | vacation | holiday`. Les présences référencent leur profil par une clé étrangère `ON DELETE CASCADE` (effective grâce à `PRAGMA foreign_keys = ON` posé sur la connexion vault) : supprimer un profil supprime ses présences.

> `trips` (optionnel) ne porte une empreinte que pour les jours `office`/`remote` ; les autres types l'ignorent et `co2Kg` reste `null`. Le total du jour et le détail des trajets (table `presence_trip`) sont **calculés et figés** (snapshot) à l'encodage, dans **une seule transaction** — voir la section CO₂ ci-dessous. `isEstimated = true` si au moins un trajet a dû retomber sur un facteur d'émission par défaut.

> Les commandes applicatives (`#[tauri::command]`) ne nécessitent **pas** d'entrée dans `capabilities/` (seules les permissions plugin/core en requièrent).

### CO₂ & trajets domicile-travail

[commute.rs](../src-tauri/src/presentation/commands/commute.rs) — empreinte carbone des trajets, **entièrement dans le coffre chiffré** (migration [0004_add_co2.sql](../src-tauri/migrations/vault/0004_add_co2.sql)). Même garde d'accès : coffre **déverrouillé** requis (sinon `SESSION_EXPIRED`).

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `list_emission_factors` | – | `EmissionFactorDto[] { modeId, label, value, unit, category, isParam, gridVariants, scope, source }` (sans les facteurs `building`, pour le picker) |
| `list_co2_referential` | – | `Co2ReferentialDto { factorYear, radiativeForcing, factors: EmissionFactorDto[] }` — référentiel **complet** (y compris `building`) pour la page Méthodologie |
| `create_commute` | `profileId, name, roundTrip, segments[]` | `CommuteDto { id, profileId, name, roundTrip, segments[], co2Kg, createdAt, updatedAt }` |
| `list_commutes` | `profileId` | `CommuteDto[]` (chacun annoté d'un `co2Kg` indicatif) |
| `update_commute` | `id, name, roundTrip, segments[]` | `CommuteDto` |
| `delete_commute` | `id` | `void` |
| `get_presence_trips` | `presenceId` | `TripDto[] { id, modeId, distanceKm, roundTrip, occupants, co2Kg, isEstimated, position }` |

> **Référentiel de facteurs d'émission** versionné par année (table `emission_factor`, seedée pour **2025 puis 2026** — ADEME/DEFRA/SNCF/SNCB ; 2026 corrige les sources/valeurs, cf. [calcul-impact-co2.md](calcul-impact-co2.md)) avec surcharges par pays pour l'électrique (`emission_factor_grid_variant`). Les facteurs **ne sont jamais codés en dur** dans la logique ; le calcul vit dans le service de domaine pur `Co2Calculator` (testé contre les critères d'acceptation AC1→AC10). Le multiplicateur de forçage radiatif est résolu **par millésime** (1,9 ≤ 2025 ; 1,7 ≥ 2026).
> **Commute** = modèle de trajet réutilisable (segments ordonnés) rattaché à un profil (`ON DELETE CASCADE`). **Trip** (`presence_trip`) = snapshot par jour figé à l'encodage : un jour passé reste reproductible même si le référentiel ou un commute change ensuite (`factor_year` conservé pour l'audit).
> La **configuration CO₂** (pays du réseau électrique, forçage radiatif aviation, énergie bâtiment, occupation voiture par défaut…) est désormais **par profil** (colonnes de `profile_settings`, cf. [Paramètres par profil](#paramètres-par-profil)) et éditable depuis l'écran Réglages ; `set_presence` et `list_commutes` calculent sous la config du profil actif. Le référentiel (`factor_year`, défaut **2026**) n'est pas éditable par l'utilisateur, donc `list_emission_factors` / `list_co2_referential` n'ont pas besoin du profil. *(L'ancien blob global `vault_meta.co2_config` et son dépôt `Co2SettingsRepository` ont été retirés ; l'entité de domaine `Co2Settings` subsiste, embarquée dans `ProfileSettings`.)*

### Heures de travail

[work_hours.rs](../src-tauri/src/presentation/commands/work_hours.rs) — encodage des heures d'une journée travaillée, **entièrement dans le coffre chiffré** (migrations [0005](../src-tauri/migrations/vault/0005_add_work_hours.sql)→[0008](../src-tauri/migrations/vault/0008_store_end_minutes.sql)). Même garde d'accès : coffre **déverrouillé** requis (sinon `SESSION_EXPIRED`).

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `create_task_preset` | `profileId, title, description?, defaultMinutes, color` | `TaskPresetDto { id, profileId, title, description, defaultMinutes, color, createdAt, updatedAt }` |
| `list_task_presets` | `profileId` | `TaskPresetDto[]` (triés par titre) |
| `update_task_preset` | `id, title, description?, defaultMinutes, color` | `TaskPresetDto` |
| `delete_task_preset` | `id` | `void` |
| `get_work_entries` | `presenceId` | `WorkEntryDto[] { id, title, description, minutes, color, position }` |
| `set_work_entries` | `presenceId, entries[]` | `WorkDayDto { entries[], schedule? }` |
| `get_work_schedule` | `presenceId` | `WorkDayScheduleDto { startMinutes, endMinutes }` \| `null` |
| `set_work_schedule` | `presenceId, startMinutes` | `WorkDayScheduleDto` |
| `get_day_note` | `presenceId` | `DayNoteDto { markdown, html }` |
| `set_day_note` | `presenceId, markdown?` | `DayNoteDto { markdown, html }` |

> **Task preset** = tâche réutilisable rattachée à un profil (`ON DELETE CASCADE`) : titre, durée par défaut (5..=480 min, pas de 5), couleur `#RRGGBB`. **Work entry** = tâche d'une journée, *duration-stacked* (pas d'heure de début explicite ; `position` = ordre, le total du jour = somme des `minutes`). Les entrées **snapshotent** titre/description/couleur à l'encodage : éditer ou supprimer un preset ne réécrit jamais les jours passés.

> Les heures ne s'encodent que sur des jours **`office`/`remote`** (gate vérifié côté use case, qui charge la présence cible). `set_work_entries` a une sémantique **replace-all** : il remplace tout le jeu d'entrées dans **une seule transaction**, la `position` vient de l'ordre du tableau et les ids sont régénérés côté serveur (les entrées ne sont pas des handles stables).

> L'**horaire** (`work_day_schedule`, au plus une ligne par présence) ne stocke que le début saisi ; `endMinutes` est **dérivé** (`start + somme des durées`) et recalculé/stocké à chaque sauvegarde d'entrées ou d'horaire — il peut dépasser 1440 si la journée court après minuit (l'affichage *wrap*). Un jour qui cesse d'être `office`/`remote` perd automatiquement ses entrées **et** son horaire (triggers `AFTER UPDATE OF type` sur `presence`, car l'*upsert* de `set_presence` conserve l'id et ne déclenche donc pas le `CASCADE`).

> La **note du jour** (`presence.note`, Markdown libre, chiffrée dans le coffre, `NULL` = pas de note) est éditée en WYSIWYG côté front (Milkdown) mais **rendue + assainie côté backend** : comrak avec `render.unsafe_=false` (le HTML *écrit* dans la note est échappé) + coloration syntaxique syntect (pur-Rust) + Ammonia. `get/set_day_note` renvoient le Markdown brut **et** le HTML sûr ; une note vide/blanche efface la colonne.

### Export & transfert de profil

Deux sorties distinctes, toutes deux session-gated (coffre déverrouillé requis) :
- **Export tableur** ([export.rs](../src-tauri/src/presentation/commands/export.rs)) — `export_profile_data` écrit un classeur **OpenDocument `.ods`** multi-feuilles (Présences / Tâches / Trajets / Notes) pour lecture/analyse (Excel/LibreOffice). L'écriture vit derrière le port `SpreadsheetExporter` (impl `OdsSpreadsheetExporter`).
- **Transfert de profil** ([transfer.rs](../src-tauri/src/presentation/commands/transfer.rs)) — un **bundle JSON** unique, versionné, pour *partager un profil complet* et le ré-importer sans perte. L'I/O fichier + (dé)sérialisation vivent derrière le port `ProfileBundleCodec` (impl `JsonProfileBundleCodec`, dans `infrastructure/transfer/`) — les use cases n'orchestrent que des types de domaine.

| Commande | Entrée | Sortie |
| --- | --- | --- |
| `export_profile_data` | `profileId, options, labels, defaultFileName` | `ExportSummaryDto { days, tasks, trips, path }` ou `null` (dialogue annulé) |
| `export_profile_bundle` | `profileId, options, defaultFileName` | `{ days, trips, workEntries, notes, taskPresets, commutes, settings, path }` ou `null` |
| `inspect_profile_bundle` | – | `{ handle, manifest: BundleManifestDto }` ou `null` |
| `import_profile_bundle` | `handle, selection, target, conflictStrategy` | `{ profileId, daysImported, daysReplaced, daysSkipped, trips, workEntries, notes, taskPresets, commutes, settings }` |

> **Aucun chemin de fichier ne traverse l'IPC** : le backend ouvre lui-même les dialogues natifs ([file_dialogs.rs](../src-tauri/src/presentation/file_dialogs.rs)). `defaultFileName` n'est qu'une suggestion (réduite à un nom de fichier simple) ; un bundle inspecté est désigné par un **handle opaque à usage unique** (15 min). Une WebView compromise ne peut donc ni écrire un fichier où elle veut, ni sonder des fichiers. Les exports sont écrits de façon **atomique** (fichier temporaire, `fsync`, `rename`). Le **plugin dialog n'est plus exposé à la WebView** (permissions retirées de la capability).

> **Bundle** = `format: "basic-presence-profile"`, `version: 1`. Catégories cochables : jours (avec trajets / heures+tâches / notes en enfants), modèles de tâches, trajets enregistrés, paramètres. **Import flexible** : `target.kind = "new"` crée un profil (identité du bundle, surchargeable champ par champ) **ou** `"existing"` fusionne dans un profil (conflits de jours résolus par `conflictStrategy` ∈ `skip | replace`). Empreinte CO₂ figée (`co2Kg`/`factorYear`) et horodatages **préservés tels quels** (réutilise `set_for_day`, sans recalcul). L'import est validé intégralement avec les **mêmes règles** que les commandes (`domain::rules`), puis écrit en **une seule transaction** : il aboutit entièrement ou pas du tout. Le fichier est **en clair** (le destinataire l'importe dans son propre coffre chiffré) — l'UI le rappelle à l'export. *(Remplace l'ancien import de migration `import_presences`, retiré.)*

## Mapping Clean Architecture

**Backend** ([src-tauri/src/](../src-tauri/src/)) :

- `domain/` — entités (`User`, `Account`, `Session`, `Profile`, `Presence`, `Commute`/`CommuteSegment`, `Trip`/`TripInput`, `EmissionFactor`/`GridVariant`, `Co2Settings`, `TaskPreset`, `WorkEntry`/`WorkDaySchedule`), **ports** (traits : `PasswordHasher`, `KeyService`, `TokenGenerator`, `SessionStore`, `VaultManager`, `Clock`, `AccountRepository`, `ProfileRepository`, `PresenceRepository`, `CommuteRepository`, `EmissionFactorRepository`, `Co2SettingsRepository`, `TaskPresetRepository`, `WorkEntryRepository`, `MarkdownRenderer`, `SpreadsheetExporter`, `ProfileBundleCodec`), service de calcul pur `Co2Calculator`, `DomainError`. Aucune dépendance externe.
- `application/` — use cases (`register_account`, `login`, `check_session`, `logout`, `account_exists`, `create_profile`, `list_profiles`, `update_profile`, `delete_profile`, `set_active_profile`, `get_profile_settings`, `set_profile_settings`, `set_presence`, `list_presences`, `delete_presence`, `create_commute`, `update_commute`, `delete_commute`, `list_commutes`, `list_emission_factors`, `list_co2_referential`, `get_presence_trips`, `create_task_preset`, `list_task_presets`, `update_task_preset`, `delete_task_preset`, `get_work_entries`, `set_work_entries`, `get_work_schedule`, `set_work_schedule`, `get_day_note`, `set_day_note`, `export_profile_data`, `export_profile_bundle`, `inspect_profile_bundle`, `import_profile_bundle`) + DTOs.
- `infrastructure/` — implémentations : `Argon2PasswordHasher`, `Argon2KeyService`, `RandomTokenGenerator`, `InMemorySessionStore`, `LibsqlAccountRepository`, `LibsqlProfileRepository`, `LibsqlPresenceRepository`, `LibsqlCommuteRepository`, `LibsqlEmissionFactorRepository`, `LibsqlCo2SettingsRepository`, `LibsqlTaskPresetRepository`, `LibsqlWorkEntryRepository`, `ComrakMarkdownRenderer`, `OdsSpreadsheetExporter`, `JsonProfileBundleCodec`, `LibsqlVaultManager`, `SystemClock`, `AppConfig`.
- `presentation/` — commandes Tauri fines + `AppError` sérialisable + **composition root** dans [lib.rs](../src-tauri/src/lib.rs) (`build_state` câble tout via `Arc<dyn …>`).

**Frontend** ([src/](../src/)) — couches, de haut en bas (une couche n'importe que celles du dessous ; vérifié par ESLint, cf. [../Claude.md](../Claude.md) §6) :

- `app/` + `main.tsx`/`App.tsx`/`router.ts` — **composition root** : [container.ts](../src/app/container.ts) est le **seul** endroit qui instancie les dépôts `Tauri*` ; [query-client.ts](../src/app/query-client.ts) configure TanStack Query (pas de refetch/retry automatiques, `SESSION_EXPIRED` → logout **centralisé**, cache vidé au logout).
- `routes/` — fichiers de routes minces ; `pages/` — écrans qui composent plusieurs features (Réglages).
- `features/*` — une capacité métier chacune (`auth`, `profile`, `presence`, `commute`, `work-hours`, `export`, `profile-transfer`, `profile-settings`, `stats`, `methodology`, `licence`, `updater`), avec `domain` (entités, ports, use cases purs), `data` (DTO, mappers, dépôts Tauri, `commands.ts` = déclaration typée des commandes IPC), `presentation` (providers sur TanStack Query, hooks, composants) et `di.ts` (ports pris dans le conteneur). Les features ne s'importent pas entre elles, à **une** exception assumée : `presence` → `commute` (l'éditeur de segments d'un jour).
- `entities/*` — modèles et contextes partagés (`profile` actif, `presence`, `commute`, `work-day`) : ce qui était auparavant importé d'une feature à l'autre.
- `shared/`, `components/`, `lib/` — UI kit et utilitaires génériques.
- `core/` — [ipc.ts](../src/core/ipc.ts) (**seul** à importer `@tauri-apps/api`, registre de commandes **typé**), [di.ts](../src/core/di.ts) (conteneur), [query.ts](../src/core/query.ts), `errors.ts`, `i18n`, `config.ts`.

## Garanties (mappées aux exigences)

- Mot de passe **jamais** stocké en clair (Argon2id PHC).
- `password_hash` / `DEK` / `KEK` / clé MAC / `token` **jamais** envoyés au frontend.
- Données au repos chiffrées, clé **dérivée du mot de passe** (coffre illisible sans login).
- `keystore.db` **scellé** par la clé de device (trousseau OS) → fichier volé inutilisable hors de l'appareil.
- Évidence d'altération du coffre (HMAC-SHA256, référence et état de session **scellés dans le keystore**), non contournable par manipulation de fichiers.
- Mémoire sensible effacée (`Zeroizing` / `zeroize`).
- Session 15 min absolue + inactivité appliquée côté UI **et** backend + re-login à chaque démarrage.
- Aucun chemin de fichier fourni par la WebView ; IPC filtré par le **pattern d'isolation** (liste blanche des commandes, [isolation/index.js](../src-tauri/isolation/index.js)).
- Toutes les entrées bornées dans le domaine ([rules.rs](../src-tauri/src/domain/rules.rs)).

## Durcissements implémentés

- **Scellement du `keystore.db` par le trousseau OS** *(résout le brute-force hors-ligne)*. Le keystore est désormais chiffré au repos (AES-256-CBC) avec une **clé de device** stockée dans le trousseau de l'OS (libsecret / Keychain / Credential Manager). Un attaquant qui copie le fichier ne peut plus tester des mots de passe hors-ligne sans **aussi** extraire la clé du trousseau. Bootstrap + migration d'un keystore historique en clair : [keystore_bootstrap.rs](../src-tauri/src/infrastructure/persistence/keystore_bootstrap.rs) (marqueur `keystore.db.sealed`). La bascule de migration est **récupérable après crash** : l'original est renommé en `keystore.db.old` avant l'échange (jamais supprimé avant), et le démarrage restaure ce backup si le fichier principal manque — un keystore manquant n'est donc jamais confondu avec une installation neuve (ce qui perdrait le DEK enveloppé, et le coffre avec).
  - ⚠️ **Contrainte runtime (Linux)** : un Secret Service actif (GNOME Keyring / KWallet) est requis ; en headless/CI le démarrage échoue explicitement (pas de repli silencieux en clair).
  - ⚠️ **Risque inhérent** : si l'entrée du trousseau est supprimée (reset OS, réinstallation), `keystore.db` devient illisible → coffre perdu. Surfacé via `KeystoreUnrecoverable` (pas d'effacement automatique).
- **Évidence d'altération du coffre (HMAC-SHA256)** *(atténue le CBC non authentifié)*. À la fermeture propre, un HMAC du fichier `vault.db` (+ WAL éventuel, chaque segment suivi de sa longueur, étiquette de domaine `bp-vault-mac-v2`, calcul en flux) est enregistré **dans le keystore scellé** (table `vault_integrity`) ; il est vérifié à l'ouverture. L'indicateur « session en cours » (`session_open`) y est aussi conservé, posé **avant** l'ouverture et les migrations : un mismatch avec une session ouverte est un crash (toléré, signalé à l'utilisateur au login), un mismatch après arrêt propre est une altération. Politique `IntegrityPolicy` ([config.rs](../src-tauri/src/infrastructure/config.rs)) : **`HardFail` (défaut)** refuse l'ouverture (`VAULT_TAMPERED`). Un coffre **supprimé** alors qu'une référence existe est aussi refusé (il ne sera pas recréé vide). [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs), [integrity_store.rs](../src-tauri/src/infrastructure/persistence/integrity_store.rs).
  - Les anciens fichiers `vault.db.hmac` / `vault.db.dirty` (versions antérieures) ne sont consultés qu'**une fois**, pour la migration, puis supprimés : les créer ou les effacer n'a plus aucun effet. Avant ce correctif, effacer `.hmac` ou créer `.dirty` suffisait à faire accepter un coffre modifié ([ADR 0002](adr/0002-integrite-du-coffre.md)).
  - ⚠️ C'est de la **détection**, pas de la prévention, et uniquement entre sessions propres. libSQL 0.9 n'expose que `Cipher::Aes256Cbc` ; la migration vers un chiffrement authentifié (crate `turso`, AEGIS-256) est reportée tant que celui-ci est expérimental ([ADR 0004](adr/0004-report-migration-turso.md)).
- **Migrations atomiques** : chaque migration (keystore et coffre) et sa ligne `_migrations` sont appliquées dans une même transaction.
- **Démarrage robuste** : une erreur de démarrage (trousseau absent, clé de device perdue, données illisibles) s'affiche dans une **boîte de dialogue native** puis l'application se ferme ; une clé de device absente alors qu'un keystore scellé existe n'est **jamais régénérée** (ce qui détruirait toute chance de restaurer l'entrée d'origine).
- **Journal de sécurité minimal** (`security_event`, 50 entrées, aucun contenu métier) : connexions, verrouillages, arrêts non propres, altérations, changements de mot de passe ; présenté une fois après la connexion suivante.
- **Timeout d'inactivité** en complément de l'expiration absolue.

## Limites connues & Phase 2

- Surfacer dans l'UI l'avertissement d'altération si un déploiement repasse en `WarnAndAllow` (le défaut `HardFail` refuse l'ouverture avec `VAULT_TAMPERED`).
- Multi-comptes par appareil (le v1 est mono-utilisateur, cohérent avec la clé dérivée du mot de passe).
- Le mot de passe transite par les buffers de désérialisation de l'IPC Tauri avant d'être enveloppé dans `Zeroizing` (dès la commande, [auth.rs](../src-tauri/src/presentation/commands/auth.rs)) — limite résiduelle connue, non contournable sans changer le transport.

> **Auto-updater** (implémenté) : updater Tauri signé en minisign. Il **vérifie** automatiquement au démarrage (seule sortie réseau, HTTPS vers GitHub, côté Rust) ; l'**installation reste une action manuelle** depuis le badge de version. Détails et procédure : [release.md](release.md).

## Vérification

- `cargo test` — tests crypto (hash/verify, wrap/unwrap, AAD, bornes des paramètres) + tests d'intégration (dont : contournement de l'intégrité par les fichiers annexes, crash pendant une session, écritures concurrentes, verrouillage exact sous tentatives parallèles et progressif, `change_password`, mise à niveau du format de clés v1 → v2, journal de sécurité) : `full_auth_flow_and_encryption` (register → login → session → logout + preuve de chiffrement coffre **et** keystore + lockout), `legacy_plaintext_keystore_is_migrated_and_sealed` (migration + scellement), `interrupted_keystore_migration_swap_is_recovered` (crash au milieu de la bascule → restauration du backup + reprise de la migration), `tampering_with_the_vault_is_detected_under_hard_fail` (évidence d'altération), `require_session_gates_data_access` (garde de session backend). Tests unitaires de la garde (expiration → révocation + verrouillage) dans [require_session.rs](../src-tauri/src/application/use_cases/require_session.rs). [integration_tests.rs](../src-tauri/src/integration_tests.rs)
- `pnpm test` — mapper, repository (via `mockIPC`), timer de session, flux login → Home.
- Manuel : `pnpm tauri dev` → **Créer un compte** → **Connexion** → **Home** (« Bonjour {username} » + compte à rebours).
