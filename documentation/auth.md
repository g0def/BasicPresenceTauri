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

- **Argon2id** (1er choix OWASP). **Deux usages distincts, sels séparés** :
  1. **Hash d'auth** (chaîne PHC) → vérifie le mot de passe. [argon2_hasher.rs](../src-tauri/src/infrastructure/crypto/argon2_hasher.rs)
  2. **Dérivation du KEK** (Key Encryption Key, 32 octets) → enveloppe le DEK. [key_service.rs](../src-tauri/src/infrastructure/crypto/key_service.rs)
  Paramètres dans [config.rs](../src-tauri/src/infrastructure/config.rs) (profil OWASP « 46 MiB » : `m=47104, t=1, p=1`).
- **DEK** (Data Encryption Key, 32 octets aléatoires) = la vraie clé du coffre `vault.db`.
- **XChaCha20-Poly1305** (AEAD **authentifié**, nonce aléatoire 192 bits) = enveloppe (wrap) le DEK avec le KEK.
- **libSQL AES-256-CBC** = chiffrement at-rest du coffre, clé = DEK (voir [turso.md](turso.md)).
- **Clé de device** (32 octets aléatoires, **scellée dans le trousseau de l'OS** — libsecret / Keychain / Credential Manager via le crate `keyring`) = chiffre **aussi** `keystore.db` au repos (AES-256-CBC). Résolue au démarrage, indépendante du compte. [device_key.rs](../src-tauri/src/infrastructure/crypto/device_key.rs)
- **Clé MAC** (32 octets aléatoires, **enveloppée par le même KEK** que le DEK) = évidence d'altération du coffre via HMAC-SHA256 du fichier (le CBC libSQL n'étant pas authentifié). [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs)

### Flux d'inscription (`register`)

[register_account.rs](../src-tauri/src/application/use_cases/register_account.rs)

1. Valider username / mot de passe (longueurs). Refuser si un compte existe déjà (mono-utilisateur).
2. `password_hash = Argon2id(password)` (sel propre, intégré au PHC).
3. `DEK = aléatoire(32o)` ; `kek_salt = aléatoire` ; `KEK = Argon2id(password, kek_salt)`.
4. `wrapped_dek, nonce = XChaCha20Poly1305(KEK).encrypt(DEK)`.
5. `MAC key = aléatoire(32o)` ; `wrapped_mac_key, mac_key_nonce = XChaCha20Poly1305(KEK).encrypt(MAC key)`.
6. Insérer dans `keystore.db` (lui-même **scellé** par la clé de device) : `{id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, wrapped_mac_key, mac_key_nonce, ...}`.
7. Le mot de passe (et le DEK/KEK/MAC en mémoire) sont **effacés** (`Zeroizing`).

### Flux de connexion (`login`)

[login.rs](../src-tauri/src/application/use_cases/login.rs)

1. Charger le compte. _(Si username inconnu : faire un hash « à blanc » pour égaliser le timing, puis renvoyer `InvalidCredentials`.)_
2. Si verrouillé (`locked_until` futur) → `AccountLocked`.
3. Vérifier `password_hash`. Échec → incrémenter `failed_attempts`, verrouiller au seuil, renvoyer `InvalidCredentials`.
4. Succès → `KEK = Argon2id(password, kek_salt)` → `DEK = unwrap(wrapped_dek, nonce, KEK)`.
5. Résoudre la **clé MAC** : `unwrap(wrapped_mac_key, …)` si présente, sinon en générer une et la persister (*backfill* unique pour les comptes créés avant la fonctionnalité).
6. **Ouvrir le coffre** : vérifier d'abord l'intégrité at-rest (HMAC) avec la clé MAC, puis déchiffrer avec le DEK.
7. Réinitialiser les compteurs, créer une **session** (token + `expiresAt = now + 15 min`).
8. Renvoyer `{ token, expiresAt, user }`.

### Le « chicken-and-egg » résolu

Le matériel d'auth (hash + DEK **enveloppé**) vit **hors** du coffre chiffré, dans `keystore.db`. On peut donc **vérifier le mot de passe et reconstruire la clé avant** d'ouvrir le coffre. Le keystore est lui-même **scellé au repos** par la clé de device (issue du trousseau OS), résolue au démarrage — ce qui n'enlève rien au raisonnement : une fois le keystore ouvert, le matériel d'auth reste accessible avant le déverrouillage du coffre.

## Stockage : keystore vs vault

Table `account` (keystore, **scellé** par la clé de device) — [0001_init.sql](../src-tauri/migrations/keystore/0001_init.sql) + [0002_add_mac_key.sql](../src-tauri/migrations/keystore/0002_add_mac_key.sql) :

```
id, username, password_hash, wrapped_dek, kek_salt, dek_nonce,
failed_attempts, locked_until, created_at, updated_at,
wrapped_mac_key, mac_key_nonce
```

Le coffre `vault.db` (chiffré + HMAC sidecar `vault.db.hmac` pour l'évidence d'altération) ne contient encore qu'un placeholder ; il accueillera les entités présence/déplacement/CO₂.

## Session

- **Token** = 32 octets aléatoires (`getrandom`), encodés base64url. [token_generator.rs](../src-tauri/src/infrastructure/crypto/token_generator.rs)
- **En mémoire uniquement** : côté Rust `InMemorySessionStore` (`Mutex<HashMap>`, [in_memory_session_store.rs](../src-tauri/src/infrastructure/session/in_memory_session_store.rs)) ; côté React un `useRef` dans [auth-provider.tsx](../src/features/auth/presentation/providers/auth-provider.tsx). **Jamais** dans `localStorage`. → app fermée = session perdue = re-login obligatoire.
- **Tokens jamais stockés en clair** : le store indexe les sessions par **SHA-256 du token** et la copie stockée est expurgée — un dump mémoire ne révèle que des empreintes, et la recherche compare des digests (pas de signal de timing sur le secret).
- **Expiration absolue 15 min** : basée sur `expiresAt` (epoch ms), donc insensible à la mise en veille / aux changements d'horloge. Timer front : [use-session-timer.ts](../src/features/auth/presentation/hooks/use-session-timer.ts).
- **Appliquée côté backend** : chaque commande touchant au coffre passe d'abord par [require_session.rs](../src-tauri/src/application/use_cases/require_session.rs) — s'il ne reste aucune session valide, les sessions restantes sont révoquées et le **coffre est verrouillé**, puis `SESSION_EXPIRED` est renvoyé. L'expiration tient donc même si la WebView (compromise ou boguée) n'appelle jamais `check_session`.
- **Timeout d'inactivité (5 min)** : en complément de l'expiration absolue, l'absence d'interaction (souris / clavier / scroll / retour au premier plan) déclenche une déconnexion. [use-idle-timeout.ts](../src/features/auth/presentation/hooks/use-idle-timeout.ts), seuil dans [config.ts](../src/core/config.ts) (`IDLE_TIMEOUT_MS`).
- À l'**expiration ou au logout** : session révoquée + coffre **fermé** + DEK **oublié** (re-verrouillage), et la **base d'intégrité (HMAC) du coffre est rafraîchie**. [check_session.rs](../src-tauri/src/application/use_cases/check_session.rs), [logout.rs](../src-tauri/src/application/use_cases/logout.rs). Une fermeture propre de la fenêtre déclenche aussi ce verrouillage (hook `on_window_event` dans [lib.rs](../src-tauri/src/lib.rs)).

## Anti-bruteforce

[login.rs](../src-tauri/src/application/use_cases/login.rs) + colonnes `failed_attempts` / `locked_until`.

- 5 échecs (`max_attempts`) → verrouillage 5 min (`lockout_ms`), réglable dans [config.rs](../src-tauri/src/infrastructure/config.rs).
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
| `logout` | `token` | `void` |

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
| `list_emission_factors` | – | `EmissionFactorDto[] { modeId, label, value, unit, category, isParam, gridVariants }` |
| `create_commute` | `profileId, name, roundTrip, segments[]` | `CommuteDto { id, profileId, name, roundTrip, segments[], co2Kg, createdAt, updatedAt }` |
| `list_commutes` | `profileId` | `CommuteDto[]` (chacun annoté d'un `co2Kg` indicatif) |
| `update_commute` | `id, name, roundTrip, segments[]` | `CommuteDto` |
| `delete_commute` | `id` | `void` |
| `get_presence_trips` | `presenceId` | `TripDto[] { id, modeId, distanceKm, roundTrip, occupants, co2Kg, isEstimated, position }` |

> **Référentiel de facteurs d'émission** versionné par année (table `emission_factor`, seedée pour 2025 — ADEME/DEFRA/SNCF/SNCB) avec surcharges par pays pour l'électrique (`emission_factor_grid_variant`). Les facteurs **ne sont jamais codés en dur** dans la logique ; le calcul vit dans le service de domaine pur `Co2Calculator` (testé contre les critères d'acceptation AC1→AC10).
> **Commute** = modèle de trajet réutilisable (segments ordonnés) rattaché à un profil (`ON DELETE CASCADE`). **Trip** (`presence_trip`) = snapshot par jour figé à l'encodage : un jour passé reste reproductible même si le référentiel ou un commute change ensuite (`factor_year` conservé pour l'audit).
> La **configuration CO₂** (pays du réseau électrique, forçage radiatif aviation, énergie bâtiment, occupation voiture par défaut…) est un blob JSON dans `vault_meta` (clé `co2_config`), avec repli sur `Co2Settings::default()` — pas encore d'UI de réglages.

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

## Mapping Clean Architecture

**Backend** ([src-tauri/src/](../src-tauri/src/)) :

- `domain/` — entités (`User`, `Account`, `Session`, `Profile`, `Presence`, `Commute`/`CommuteSegment`, `Trip`/`TripInput`, `EmissionFactor`/`GridVariant`, `Co2Settings`, `TaskPreset`, `WorkEntry`/`WorkDaySchedule`), **ports** (traits : `PasswordHasher`, `KeyService`, `TokenGenerator`, `SessionStore`, `VaultManager`, `Clock`, `AccountRepository`, `ProfileRepository`, `PresenceRepository`, `CommuteRepository`, `EmissionFactorRepository`, `Co2SettingsRepository`, `TaskPresetRepository`, `WorkEntryRepository`, `MarkdownRenderer`), service de calcul pur `Co2Calculator`, `DomainError`. Aucune dépendance externe.
- `application/` — use cases (`register_account`, `login`, `check_session`, `logout`, `account_exists`, `create_profile`, `list_profiles`, `update_profile`, `delete_profile`, `set_active_profile`, `set_presence`, `list_presences`, `delete_presence`, `import_presences`, `create_commute`, `update_commute`, `delete_commute`, `list_commutes`, `list_emission_factors`, `get_presence_trips`, `create_task_preset`, `list_task_presets`, `update_task_preset`, `delete_task_preset`, `get_work_entries`, `set_work_entries`, `get_work_schedule`, `set_work_schedule`, `get_day_note`, `set_day_note`) + DTOs.
- `infrastructure/` — implémentations : `Argon2PasswordHasher`, `Argon2KeyService`, `RandomTokenGenerator`, `InMemorySessionStore`, `LibsqlAccountRepository`, `LibsqlProfileRepository`, `LibsqlPresenceRepository`, `LibsqlCommuteRepository`, `LibsqlEmissionFactorRepository`, `LibsqlCo2SettingsRepository`, `LibsqlTaskPresetRepository`, `LibsqlWorkEntryRepository`, `ComrakMarkdownRenderer`, `LibsqlVaultManager`, `SystemClock`, `AppConfig`.
- `presentation/` — commandes Tauri fines + `AppError` sérialisable + **composition root** dans [lib.rs](../src-tauri/src/lib.rs) (`build_state` câble tout via `Arc<dyn …>`).

**Frontend** ([src/](../src/)) :

- `core/` — [ipc.ts](../src/core/ipc.ts) (**seul** à importer `@tauri-apps/api`), `errors.ts` (`AppError`), `config.ts`.
- `features/auth/` — `domain` (entités, `AuthRepository`, use cases), `data` (DTOs, mappers, `TauriAuthRepository`), `presentation` (`AuthProvider` = composition root front, `useAuth`, `useSessionTimer`, écrans Register/Login/Home, compte à rebours).
- `features/profile/` — `domain` (`Profile`, `ProfileRepository`, use cases), `data` (DTOs, mapper, `TauriProfileRepository`), `presentation` (`ProfileProvider` = composition root front recevant `onSessionExpired`, `useProfile`, formulaire `Dialog`, et le **badge** = menu compte regroupant profils + langue + thème + déconnexion). Indépendante de `auth` : `logout` est injecté (`onSessionExpired` au provider, `onLogout` au badge) ; langue/thème viennent de `core`/`shared`.
- `features/presence/` — calendrier mensuel des présences + import. `domain`/`data`/`presentation` (`PresenceProvider`, `usePresence`, `PresenceCalendar`, `PresenceDayDialog`). L'entité de transport `PresenceTrip` est **redéclarée ici** (structurellement identique au `TripInput` de `commute`) pour ne pas faire dépendre le domaine présence du domaine commute.
- `features/commute/` — modèles de trajet réutilisables + référentiel de facteurs d'émission. `domain` (`Commute`, `EmissionFactor`, `commuteToTrips`, `CommuteRepository`, `EmissionFactorRepository`, use cases), `data` (DTOs, mappers, `TauriCommuteRepository`, `TauriEmissionFactorRepository`), `presentation` (`CommuteProvider` = composition root front, `useCommute`, `CommuteView`, `SegmentEditor`, helpers de formatage/icônes/labels).
- `features/work-hours/` — encodage des heures d'un jour `office`/`remote` (page routée `/work-hours/$day`). `domain` (`TaskPreset`, `WorkEntry`/`WorkDaySchedule`/`WorkDay`, `TaskPresetRepository`, `WorkEntryRepository`, use cases purs), `data` (DTOs miroir, mappers, `TauriTaskPresetRepository`, `TauriWorkEntryRepository`), `presentation` (`TaskPresetProvider` = composition root front pour les presets ; `useWorkDay` = état local du jour à persistance *live* (debounce + flush au démontage) ; `DayDonut`, `WorkEntryList`, `DayScheduleEditor`, `TaskPresetPanel`, helpers de géométrie/format testés). Consomme `usePresence` pour résoudre la présence du jour (même convention que `commute`).

> **Dépendance dirigée assumée `presence → commute`.** L'empreinte CO₂ étant *par conception* attachée à un jour de présence, `PresenceDayDialog`/`PresenceCalendar` consomment le `SegmentEditor`, `useCommute` et les helpers de `commute` (jamais l'inverse — pas de cycle). C'est une dérogation **délibérée** à l'isolation stricte des features, justifiée par le couplage métier réel ; à conserver unidirectionnelle. (`CommuteProvider` consomme `useProfile` comme le fait déjà `PresenceProvider` — convention admise pour le profil actif.)

## Garanties (mappées aux exigences)

- Mot de passe **jamais** stocké en clair (Argon2id PHC).
- `password_hash` / `DEK` / `KEK` / clé MAC / `token` **jamais** envoyés au frontend.
- Données au repos chiffrées, clé **dérivée du mot de passe** (coffre illisible sans login).
- `keystore.db` **scellé** par la clé de device (trousseau OS) → fichier volé inutilisable hors de l'appareil.
- Évidence d'altération du coffre (HMAC-SHA256 sidecar, vérifié à l'ouverture).
- Mémoire sensible effacée (`Zeroizing` / `zeroize`).
- Session 15 min absolue + timeout d'inactivité + re-login à chaque démarrage.

## Durcissements implémentés

- **Scellement du `keystore.db` par le trousseau OS** *(résout le brute-force hors-ligne)*. Le keystore est désormais chiffré au repos (AES-256-CBC) avec une **clé de device** stockée dans le trousseau de l'OS (libsecret / Keychain / Credential Manager). Un attaquant qui copie le fichier ne peut plus tester des mots de passe hors-ligne sans **aussi** extraire la clé du trousseau. Bootstrap + migration d'un keystore historique en clair : [keystore_bootstrap.rs](../src-tauri/src/infrastructure/persistence/keystore_bootstrap.rs) (marqueur `keystore.db.sealed`).
  - ⚠️ **Contrainte runtime (Linux)** : un Secret Service actif (GNOME Keyring / KWallet) est requis ; en headless/CI le démarrage échoue explicitement (pas de repli silencieux en clair).
  - ⚠️ **Risque inhérent** : si l'entrée du trousseau est supprimée (reset OS, réinstallation), `keystore.db` devient illisible → coffre perdu. Surfacé via `KeystoreUnrecoverable` (pas d'effacement automatique).
- **Évidence d'altération du coffre (HMAC-SHA256)** *(atténue le CBC non authentifié)*. À la fermeture propre, un HMAC du fichier `vault.db` (clé MAC enveloppée par le KEK) est écrit dans `vault.db.hmac` ; il est vérifié à l'ouverture. Politique `IntegrityPolicy` ([config.rs](../src-tauri/src/infrastructure/config.rs)) : **`HardFail` (défaut)** — un mismatch après un arrêt propre refuse l'ouverture (`VAULT_TAMPERED`) ; `WarnAndAllow` reste disponible pour des déploiements tolérants. Un marqueur `vault.db.dirty` distingue un crash (rebaseline silencieux, toléré quelle que soit la politique) d'une altération après arrêt propre. [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs).
  - ⚠️ C'est de la **détection**, pas de la prévention, et uniquement entre sessions propres. libSQL 0.9 n'expose que `Cipher::Aes256Cbc` (aucun AEAD at-rest) — un chiffrement authentifié natif reste à surveiller côté lib.
- **Timeout d'inactivité** en complément de l'expiration absolue.

## Limites connues & Phase 2

- `change_password` (peu coûteux : ré-envelopper le DEK + la clé MAC, sans re-chiffrer tout le coffre).
- Surfacer dans l'UI l'avertissement d'altération si un déploiement repasse en `WarnAndAllow` (le défaut `HardFail` refuse l'ouverture avec `VAULT_TAMPERED`).
- Multi-comptes par appareil (le v1 est mono-utilisateur, cohérent avec la clé dérivée du mot de passe).
- Le mot de passe transite par les buffers de désérialisation de l'IPC Tauri avant d'être enveloppé dans `Zeroizing` (dès la commande, [auth.rs](../src-tauri/src/presentation/commands/auth.rs)) — limite résiduelle connue, non contournable sans changer le transport.

> **Auto-updater** (implémenté) : updater Tauri signé en minisign. Il **vérifie** automatiquement au démarrage (seule sortie réseau, HTTPS vers GitHub, côté Rust) ; l'**installation reste une action manuelle** depuis le badge de version. Détails et procédure : [release.md](release.md).

## Vérification

- `cargo test` — tests crypto (hash/verify, wrap/unwrap) + tests d'intégration : `full_auth_flow_and_encryption` (register → login → session → logout + preuve de chiffrement coffre **et** keystore + lockout), `legacy_plaintext_keystore_is_migrated_and_sealed` (migration + scellement), `tampering_with_the_vault_is_detected_under_hard_fail` (évidence d'altération), `require_session_gates_data_access` (garde de session backend). Tests unitaires de la garde (expiration → révocation + verrouillage) dans [require_session.rs](../src-tauri/src/application/use_cases/require_session.rs). [integration_tests.rs](../src-tauri/src/integration_tests.rs)
- `pnpm test` — mapper, repository (via `mockIPC`), timer de session, flux login → Home.
- Manuel : `pnpm tauri dev` → **Créer un compte** → **Connexion** → **Home** (« Bonjour {username} » + compte à rebours).
