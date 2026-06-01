# Turso / libSQL — Persistance offline-first chiffrée

> Voir aussi [auth.md](auth.md) pour le modèle de sécurité (chiffrement du coffre, dérivation de clé).

## Pourquoi Turso / libSQL

**libSQL** est un fork open-source de SQLite, et le moteur de **Turso**. Il permet :

- une **base locale** (fonctionne 100 % hors-ligne),
- le **chiffrement au repos**,
- et — à terme — la **synchronisation cloud** (embedded replicas) vers Turso Cloud.

Choix d'architecture : on bâtit **dès maintenant** sur libSQL pour que la synchronisation cloud (Phase 2) soit un **ajout** et non une réécriture.

## État actuel (v1) : 100 % local, chiffré

Aucune connexion réseau en v1. Tout vit dans le dossier _app-data_ de l'OS, résolu via `app.path().app_data_dir()` dans [lib.rs](../src-tauri/src/lib.rs).

Deux fichiers de base de données :

| Fichier | Chiffré ? | Contenu |
| --- | --- | --- |
| `keystore.db` | ❌ Non | Le compte propriétaire : hash Argon2id, **DEK enveloppé**, sels, compteurs anti-bruteforce. Sûr en clair (tout y est déjà protégé cryptographiquement). |
| `vault.db` | ✅ Oui (AES-256-CBC) | Les données confidentielles (présence, déplacement, CO₂ — à venir). Ouvert **uniquement** pendant une session active. |

Cette séparation résout le _chicken-and-egg_ du chiffrement dérivé du mot de passe — détaillé dans [auth.md](auth.md).

## Le crate `libsql`

Déclaration dans [Cargo.toml](../src-tauri/Cargo.toml) :

```toml
libsql = { version = "0.9", features = ["encryption"] }   # `encryption` n'est PAS activée par défaut
bytes = "1"
```

Ouverture d'une base locale chiffrée (voir [db.rs](../src-tauri/src/infrastructure/persistence/db.rs)) :

```rust
let config = EncryptionConfig::new(Cipher::Aes256Cbc, Bytes::copy_from_slice(dek)); // dek = 32 octets
let db = Builder::new_local(path).encryption_config(config).build().await?;
let conn = db.connect()?;
```

Faits d'API importants (libsql 0.9) :

- `Cipher` n'expose qu'**`Aes256Cbc`** ; la clé fait **exactement 32 octets**.
- `Connection` est `Send + Sync + Clone` (Arc interne) → **pas besoin de `Mutex<Connection>`**, on la clone.
- `db.connect()` est **synchrone** ; `execute` / `query` sont **async**.
- Garder le `Database` **vivant** (stocké dans l'`AppState` pour le keystore, dans le `VaultHolder` pour le coffre) pour que les connexions restent valides.

Fichiers concernés :

- [db.rs](../src-tauri/src/infrastructure/persistence/db.rs) — ouverture clair / chiffré.
- [vault.rs](../src-tauri/src/infrastructure/persistence/vault.rs) — cycle de vie du coffre (`open` au login, `close` au logout/expiration ; expose `connection()` pour les futurs repositories de présence).
- [account_repository.rs](../src-tauri/src/infrastructure/persistence/account_repository.rs) — repository du keystore (CRUD compte + compteurs).

## Migrations

Runner maison minimal : [migrations.rs](../src-tauri/src/infrastructure/persistence/migrations.rs).

- Une table `_migrations(version, applied_at)` **par base** ; applique en ordre les `.sql` embarqués via `include_str!`, en sautant ceux déjà appliqués.
- Fichiers SQL :
  - [migrations/keystore/0001_init.sql](../src-tauri/migrations/keystore/0001_init.sql) — table `account`.
  - [migrations/vault/0001_init.sql](../src-tauri/migrations/vault/0001_init.sql) — table `vault_meta` (placeholder ; prouve que le coffre chiffré s'ouvre et s'écrit).

**Ajouter une migration** : déposer `000X_*.sql` dans le bon dossier, puis l'ajouter à `KEYSTORE_MIGRATIONS` ou `VAULT_MIGRATIONS` dans `migrations.rs`.

## Phase 2 — Synchronisation cloud Turso

Non implémenté en v1. **Point d'isolation : tout le choix du builder vit dans [db.rs](../src-tauri/src/infrastructure/persistence/db.rs)** — c'est le seul fichier à modifier pour brancher la sync.

### Le conflit technique à arbitrer

Aujourd'hui, libsql ne permet pas *à la fois* le chiffrement local natif **et** la synchro offline dans le même builder :

| Builder | Écritures offline | `encryption_config` local |
| --- | --- | --- |
| `new_local` (utilisé en v1) | ✅ | ✅ |
| `new_remote_replica` | ❌ (écritures envoyées au distant) | ✅ |
| `new_synced_database` | ✅ | ❌ (chiffrement at-rest non supporté, beta) |

**Options Phase 2 :**

1. Garder le coffre **local chiffré** (v1) et synchroniser vers un Turso Cloud qui gère **son propre** chiffrement at-rest côté serveur.
2. Passer à `new_synced_database` (sync offline) en acceptant temporairement la perte du chiffrement local natif → compenser par une couche applicative.

À trancher selon l'évolution de libsql.

### Provisionner une base Turso (aperçu)

```bash
turso auth login
turso db create basic-presence
turso db show basic-presence --url     # => libsql://...
turso db tokens create basic-presence  # => token d'auth
```

- L'URL et le token sont des **secrets** : jamais commités, **jamais exposés au frontend**.
- Les stocker via le trousseau OS (`keyring`) ou `tauri-plugin-stronghold`, lus **uniquement côté Rust**.
- Les injecter dans `AppConfig` ([config.rs](../src-tauri/src/infrastructure/config.rs)) et les consommer dans `db.rs`.

## Vérification

- **Test d'intégration** [integration_tests.rs](../src-tauri/src/integration_tests.rs) : prouve l'ouverture chiffrée et que `vault.db` est **illisible sans la clé**.
- **Manuel** : après un login, `vault.db` existe ; `sqlite3 vault.db .tables` échoue (chiffré) ; `strings vault.db | grep <username>` ne renvoie rien.
