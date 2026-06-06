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
- **Expiration absolue 15 min** : basée sur `expiresAt` (epoch ms), donc insensible à la mise en veille / aux changements d'horloge. Timer front : [use-session-timer.ts](../src/features/auth/presentation/hooks/use-session-timer.ts).
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

[profile.rs](../src-tauri/src/presentation/commands/profile.rs) — CRUD des profils de présence (stockés dans le **coffre chiffré**). Garde d'accès : aucun token transmis ; le coffre doit être **déverrouillé** (sinon erreur `SESSION_EXPIRED`). Un profil introuvable renvoie `NOT_FOUND`.

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
| `set_presence` | `profileId, day, type` | `PresenceDto { id, profileId, day, type, createdAt, updatedAt }` |
| `list_presences` | `profileId` | `PresenceDto[]` |
| `delete_presence` | `id` | `void` |

> `day` = epoch ms à **minuit UTC** (validé côté domaine) ; `type` ∈ `office | remote | vacation | holiday`. Les présences référencent leur profil par une clé étrangère `ON DELETE CASCADE` (effective grâce à `PRAGMA foreign_keys = ON` posé sur la connexion vault) : supprimer un profil supprime ses présences.

> Les commandes applicatives (`#[tauri::command]`) ne nécessitent **pas** d'entrée dans `capabilities/` (seules les permissions plugin/core en requièrent).

## Mapping Clean Architecture

**Backend** ([src-tauri/src/](../src-tauri/src/)) :

- `domain/` — entités (`User`, `Account`, `Session`, `Profile`, `Presence`), **ports** (traits : `PasswordHasher`, `KeyService`, `TokenGenerator`, `SessionStore`, `VaultManager`, `Clock`, `AccountRepository`, `ProfileRepository`, `PresenceRepository`), `DomainError`. Aucune dépendance externe.
- `application/` — use cases (`register_account`, `login`, `check_session`, `logout`, `account_exists`, `create_profile`, `list_profiles`, `update_profile`, `delete_profile`, `set_active_profile`, `set_presence`, `list_presences`, `delete_presence`) + DTOs.
- `infrastructure/` — implémentations : `Argon2PasswordHasher`, `Argon2KeyService`, `RandomTokenGenerator`, `InMemorySessionStore`, `LibsqlAccountRepository`, `LibsqlProfileRepository`, `LibsqlPresenceRepository`, `LibsqlVaultManager`, `SystemClock`, `AppConfig`.
- `presentation/` — commandes Tauri fines + `AppError` sérialisable + **composition root** dans [lib.rs](../src-tauri/src/lib.rs) (`build_state` câble tout via `Arc<dyn …>`).

**Frontend** ([src/](../src/)) :

- `core/` — [ipc.ts](../src/core/ipc.ts) (**seul** à importer `@tauri-apps/api`), `errors.ts` (`AppError`), `config.ts`.
- `features/auth/` — `domain` (entités, `AuthRepository`, use cases), `data` (DTOs, mappers, `TauriAuthRepository`), `presentation` (`AuthProvider` = composition root front, `useAuth`, `useSessionTimer`, écrans Register/Login/Home, compte à rebours).
- `features/profile/` — `domain` (`Profile`, `ProfileRepository`, use cases), `data` (DTOs, mapper, `TauriProfileRepository`), `presentation` (`ProfileProvider` = composition root front recevant `onSessionExpired`, `useProfile`, formulaire `Dialog`, et le **badge** = menu compte regroupant profils + langue + thème + déconnexion). Indépendante de `auth` : `logout` est injecté (`onSessionExpired` au provider, `onLogout` au badge) ; langue/thème viennent de `core`/`shared`.

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
- **Évidence d'altération du coffre (HMAC-SHA256)** *(atténue le CBC non authentifié)*. À la fermeture propre, un HMAC du fichier `vault.db` (clé MAC enveloppée par le KEK) est écrit dans `vault.db.hmac` ; il est vérifié à l'ouverture. Politique `IntegrityPolicy` ([config.rs](../src-tauri/src/infrastructure/config.rs)) : `WarnAndAllow` (défaut) ou `HardFail`. Un marqueur `vault.db.dirty` distingue un crash (rebaseline silencieux) d'une altération après arrêt propre. [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs).
  - ⚠️ C'est de la **détection**, pas de la prévention, et uniquement entre sessions propres. libSQL 0.9 n'expose que `Cipher::Aes256Cbc` (aucun AEAD at-rest) — un chiffrement authentifié natif reste à surveiller côté lib.
- **Timeout d'inactivité** en complément de l'expiration absolue.

## Limites connues & Phase 2

- `change_password` (peu coûteux : ré-envelopper le DEK + la clé MAC, sans re-chiffrer tout le coffre).
- Surfacer dans l'UI l'avertissement d'altération en mode `WarnAndAllow` (aujourd'hui silencieux ; `HardFail` refuse l'ouverture).
- Multi-comptes par appareil (le v1 est mono-utilisateur, cohérent avec la clé dérivée du mot de passe).

## Vérification

- `cargo test` — tests crypto (hash/verify, wrap/unwrap) + tests d'intégration : `full_auth_flow_and_encryption` (register → login → session → logout + preuve de chiffrement coffre **et** keystore + lockout), `legacy_plaintext_keystore_is_migrated_and_sealed` (migration + scellement), `tampering_with_the_vault_is_detected_under_hard_fail` (évidence d'altération). [integration_tests.rs](../src-tauri/src/integration_tests.rs)
- `pnpm test` — mapper, repository (via `mockIPC`), timer de session, flux login → Home.
- Manuel : `pnpm tauri dev` → **Créer un compte** → **Connexion** → **Home** (« Bonjour {username} » + compte à rebours).
