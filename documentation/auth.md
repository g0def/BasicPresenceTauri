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

### Flux d'inscription (`register`)

[register_account.rs](../src-tauri/src/application/use_cases/register_account.rs)

1. Valider username / mot de passe (longueurs). Refuser si un compte existe déjà (mono-utilisateur).
2. `password_hash = Argon2id(password)` (sel propre, intégré au PHC).
3. `DEK = aléatoire(32o)` ; `kek_salt = aléatoire` ; `KEK = Argon2id(password, kek_salt)`.
4. `wrapped_dek, nonce = XChaCha20Poly1305(KEK).encrypt(DEK)`.
5. Insérer dans `keystore.db` : `{id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, ...}`.
6. Le mot de passe (et le DEK/KEK en mémoire) sont **effacés** (`Zeroizing`).

### Flux de connexion (`login`)

[login.rs](../src-tauri/src/application/use_cases/login.rs)

1. Charger le compte. _(Si username inconnu : faire un hash « à blanc » pour égaliser le timing, puis renvoyer `InvalidCredentials`.)_
2. Si verrouillé (`locked_until` futur) → `AccountLocked`.
3. Vérifier `password_hash`. Échec → incrémenter `failed_attempts`, verrouiller au seuil, renvoyer `InvalidCredentials`.
4. Succès → `KEK = Argon2id(password, kek_salt)` → `DEK = unwrap(wrapped_dek, nonce, KEK)` → **ouvrir le coffre** chiffré avec le DEK.
5. Réinitialiser les compteurs, créer une **session** (token + `expiresAt = now + 15 min`).
6. Renvoyer `{ token, expiresAt, user }`.

### Le « chicken-and-egg » résolu

Le matériel d'auth (hash + DEK **enveloppé**) vit **hors** du coffre chiffré, dans `keystore.db` (non chiffré). On peut donc **vérifier le mot de passe et reconstruire la clé avant** d'ouvrir le coffre.

## Stockage : keystore vs vault

Table `account` (keystore, non chiffré) — [0001_init.sql](../src-tauri/migrations/keystore/0001_init.sql) :

```
id, username, password_hash, wrapped_dek, kek_salt, dek_nonce,
failed_attempts, locked_until, created_at, updated_at
```

Le coffre `vault.db` (chiffré) ne contient encore qu'un placeholder ; il accueillera les entités présence/déplacement/CO₂.

## Session

- **Token** = 32 octets aléatoires (`getrandom`), encodés base64url. [token_generator.rs](../src-tauri/src/infrastructure/crypto/token_generator.rs)
- **En mémoire uniquement** : côté Rust `InMemorySessionStore` (`Mutex<HashMap>`, [in_memory_session_store.rs](../src-tauri/src/infrastructure/session/in_memory_session_store.rs)) ; côté React un `useRef` dans [auth-provider.tsx](../src/features/auth/presentation/providers/auth-provider.tsx). **Jamais** dans `localStorage`. → app fermée = session perdue = re-login obligatoire.
- **Expiration absolue 15 min** : basée sur `expiresAt` (epoch ms), donc insensible à la mise en veille / aux changements d'horloge. Timer front : [use-session-timer.ts](../src/features/auth/presentation/hooks/use-session-timer.ts).
- À l'**expiration ou au logout** : session révoquée + coffre **fermé** + DEK **oublié** (re-verrouillage). [check_session.rs](../src-tauri/src/application/use_cases/check_session.rs), [logout.rs](../src-tauri/src/application/use_cases/logout.rs).

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

> Les commandes applicatives (`#[tauri::command]`) ne nécessitent **pas** d'entrée dans `capabilities/` (seules les permissions plugin/core en requièrent).

## Mapping Clean Architecture

**Backend** ([src-tauri/src/](../src-tauri/src/)) :

- `domain/` — entités (`User`, `Account`, `Session`), **ports** (traits : `PasswordHasher`, `KeyService`, `TokenGenerator`, `SessionStore`, `VaultManager`, `Clock`, `AccountRepository`), `DomainError`. Aucune dépendance externe.
- `application/` — use cases (`register_account`, `login`, `check_session`, `logout`, `account_exists`) + DTOs.
- `infrastructure/` — implémentations : `Argon2PasswordHasher`, `Argon2KeyService`, `RandomTokenGenerator`, `InMemorySessionStore`, `LibsqlAccountRepository`, `LibsqlVaultManager`, `SystemClock`, `AppConfig`.
- `presentation/` — commandes Tauri fines + `AppError` sérialisable + **composition root** dans [lib.rs](../src-tauri/src/lib.rs) (`build_state` câble tout via `Arc<dyn …>`).

**Frontend** ([src/](../src/)) :

- `core/` — [ipc.ts](../src/core/ipc.ts) (**seul** à importer `@tauri-apps/api`), `errors.ts` (`AppError`), `config.ts`.
- `features/auth/` — `domain` (entités, `AuthRepository`, use cases), `data` (DTOs, mappers, `TauriAuthRepository`), `presentation` (`AuthProvider` = composition root front, `useAuth`, `useSessionTimer`, écrans Register/Login/Home, compte à rebours).

## Garanties (mappées aux exigences)

- Mot de passe **jamais** stocké en clair (Argon2id PHC).
- `password_hash` / `DEK` / `KEK` / `token` **jamais** envoyés au frontend.
- Données au repos chiffrées, clé **dérivée du mot de passe** (coffre illisible sans login).
- Mémoire sensible effacée (`Zeroizing` / `zeroize`).
- Session 15 min absolue + re-login à chaque démarrage.

## Limites connues & Phase 2

- **AES-256-CBC = confidentialité forte mais NON authentifiée** (pas de HMAC par page comme le SQLCipher complet) → pas de détection d'altération du fichier. L'intégrité au repos est un durcissement Phase 2. Le matériel de clé, lui, reste authentifié (XChaCha20-Poly1305).
- `change_password` (peu coûteux : ré-envelopper le DEK, sans re-chiffrer tout le coffre).
- Idle-timeout en complément de l'expiration absolue.
- Multi-comptes par appareil (le v1 est mono-utilisateur, cohérent avec la clé dérivée du mot de passe).

## Vérification

- `cargo test` — tests crypto (hash/verify, wrap/unwrap) + test d'intégration `full_auth_flow_and_encryption` (register → login → session → logout + preuve de chiffrement + lockout). [integration_tests.rs](../src-tauri/src/integration_tests.rs)
- `pnpm test` — mapper, repository (via `mockIPC`), timer de session, flux login → Home.
- Manuel : `pnpm tauri dev` → **Créer un compte** → **Connexion** → **Home** (« Bonjour {username} » + compte à rebours).
