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
| `keystore.db` | ✅ Oui (AES-256-CBC, **clé de device du trousseau OS**) | Le compte propriétaire : hash Argon2id, **DEK enveloppé**, **clé MAC enveloppée**, sels, compteurs anti-bruteforce. Scellé au repos pour empêcher le brute-force hors-ligne d'un fichier volé. |
| `vault.db` | ✅ Oui (AES-256-CBC, **DEK**) | Les données confidentielles (présence, déplacement, CO₂ — à venir). Ouvert **uniquement** pendant une session active. Intégrité au repos via sidecar `vault.db.hmac` (HMAC-SHA256). |

Cette séparation résout le _chicken-and-egg_ du chiffrement dérivé du mot de passe — détaillé dans [auth.md](auth.md). Les deux bases sont chiffrées au repos, mais par des clés **indépendantes** : le keystore par la clé de device (trousseau OS), le coffre par le DEK (dérivé du mot de passe). La clé de device est résolue au démarrage via [device_key.rs](../src-tauri/src/infrastructure/crypto/device_key.rs) ; le bootstrap/migration du keystore vit dans [keystore_bootstrap.rs](../src-tauri/src/infrastructure/persistence/keystore_bootstrap.rs).

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

- [db.rs](../src-tauri/src/infrastructure/persistence/db.rs) — ouverture clair / chiffré (le clair ne sert plus qu'à sonder/migrer un ancien keystore).
- [keystore_bootstrap.rs](../src-tauri/src/infrastructure/persistence/keystore_bootstrap.rs) — ouverture/scellement du keystore + migration one-time d'un keystore historique en clair.
- [vault.rs](../src-tauri/src/infrastructure/persistence/vault.rs) — cycle de vie du coffre (`open` au login, `close` au logout/expiration ; vérifie/écrit le HMAC d'intégrité ; expose `connection()` pour les repositories).
- [vault_integrity.rs](../src-tauri/src/infrastructure/persistence/vault_integrity.rs) — HMAC-SHA256 du fichier + marqueur `dirty` (crash vs altération).
- [account_repository.rs](../src-tauri/src/infrastructure/persistence/account_repository.rs) — repository du keystore (CRUD compte + compteurs + backfill clé MAC).
- [presence_repository.rs](../src-tauri/src/infrastructure/persistence/presence_repository.rs) — repository du vault pour les présences. Les requêtes `SELECT` embarquent une **sous-requête corrélée** pour calculer `work_minutes` à la volée :
  ```sql
  (SELECT COALESCE(SUM(minutes), 0) FROM work_entry WHERE presence_id = presence.id) AS work_minutes
  ```
  > **Décision technique** : SQLite `RETURNING` ne supporte pas les sous-requêtes corrélées, donc l'upsert utilise un jeu de colonnes réduit (`RETURNING_COLUMNS`) puis exécute une seconde requête `SELECT SUM(minutes)…` pour récupérer le total après écriture. Le champ `work_minutes` sur `Presence` / `PresenceDto` est toujours présent (0 par défaut si aucune entrée).

## Migrations

Runner maison minimal : [migrations.rs](../src-tauri/src/infrastructure/persistence/migrations.rs).

- Une table `_migrations(version, applied_at)` **par base** ; applique en ordre les `.sql` embarqués via `include_str!`, en sautant ceux déjà appliqués.
- Fichiers SQL :
  - [migrations/keystore/0001_init.sql](../src-tauri/migrations/keystore/0001_init.sql) — table `account`.
  - [migrations/keystore/0002_add_mac_key.sql](../src-tauri/migrations/keystore/0002_add_mac_key.sql) — colonnes `wrapped_mac_key`, `mac_key_nonce` (nullable ; backfill au login).
  - [migrations/vault/0001_init.sql](../src-tauri/migrations/vault/0001_init.sql) — table `vault_meta` (placeholder ; prouve que le coffre chiffré s'ouvre et s'écrit).
  - `migrations/vault/0002_add_profiles.sql`, `0003_add_presence.sql` — tables `profile` et `presence`.

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

- **Test d'intégration** [integration_tests.rs](../src-tauri/src/integration_tests.rs) : prouve l'ouverture chiffrée, que `vault.db` **et** `keystore.db` sont **illisibles sans la clé**, la migration/scellement d'un keystore en clair, et la détection d'altération du coffre.
- **Manuel** : après un login, `vault.db` existe ; `sqlite3 vault.db .tables` échoue (chiffré) ; `strings vault.db | grep <username>` ne renvoie rien — idem pour `keystore.db`.
