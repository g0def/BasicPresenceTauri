use std::path::{Path, PathBuf};

use libsql::{params, Database};

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::db::{
    connect, map_storage, open_encrypted_db, open_plain_db,
};
use crate::infrastructure::persistence::migrations::{self, KEYSTORE_MIGRATIONS};

/// The base columns of the `account` table as created by the initial keystore
/// migration. Migration of a legacy plaintext keystore copies exactly these;
/// any later columns (added by subsequent migrations, nullable) are recreated
/// empty and backfilled by the app.
const ACCOUNT_BASE_COLUMNS: &str =
    "id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
     failed_attempts, locked_until, created_at, updated_at";

/// Sidecar marker proving the keystore at `path` is the sealed (encrypted) form.
/// Its presence disambiguates "already encrypted" from "legacy plaintext" and,
/// crucially, from "encrypted file that fails to decrypt" (a missing/changed
/// device key) — which must be treated as unrecoverable rather than re-migrated.
fn sealed_marker(path: &Path) -> PathBuf {
    let mut p = path.as_os_str().to_owned();
    p.push(".sealed");
    PathBuf::from(p)
}

/// Remove a libSQL database file together with its `-wal`/`-shm` siblings.
fn remove_db_files(path: &Path) {
    let _ = std::fs::remove_file(path);
    for suffix in ["-wal", "-shm"] {
        let mut p = path.as_os_str().to_owned();
        p.push(suffix);
        let _ = std::fs::remove_file(PathBuf::from(p));
    }
}

/// Open the keystore as an AES-256-CBC encrypted database, sealed behind the
/// per-device key. Handles three startup situations:
///
/// 1. **Fresh install** (no file): create an empty encrypted keystore + marker.
/// 2. **Legacy plaintext keystore** (file readable in clear, no marker): copy
///    the account row into a new encrypted keystore, atomically swap, drop the
///    marker. One-time, idempotent on re-run via the leftover-cleanup.
/// 3. **Already sealed** (marker present): open encrypted; failure to decrypt
///    means the device key is gone/changed → [`DomainError::KeystoreUnrecoverable`].
///
/// The returned `Database` still needs `connect()` + `migrations::run()` by the
/// caller (idempotent); migration path 2 runs them internally before inserting.
pub async fn open_or_migrate_keystore(
    path: &Path,
    device_key: &[u8],
) -> Result<Database, DomainError> {
    let marker = sealed_marker(path);

    if marker.exists() {
        // Already sealed: must decrypt with the current device key, or it's lost.
        let db = open_encrypted_db(path, device_key).await?;
        if probe_readable(&db).await {
            return Ok(db);
        }
        return Err(DomainError::KeystoreUnrecoverable);
    }

    if !path.exists() {
        // Fresh install: create the encrypted keystore and seal it.
        let db = open_encrypted_db(path, device_key).await?;
        write_marker(&marker)?;
        return Ok(db);
    }

    // A file exists without a marker. If it reads in clear, it's a legacy
    // plaintext keystore to migrate; otherwise assume it's already encrypted
    // (interrupted first run before the marker was written) and just re-seal.
    if plaintext_readable(path).await {
        migrate_plaintext_to_encrypted(path, device_key).await?;
        write_marker(&marker)?;
        return open_encrypted_db(path, device_key).await;
    }

    let db = open_encrypted_db(path, device_key).await?;
    if probe_readable(&db).await {
        write_marker(&marker)?;
        return Ok(db);
    }
    Err(DomainError::KeystoreUnrecoverable)
}

fn write_marker(marker: &Path) -> Result<(), DomainError> {
    std::fs::write(marker, b"sealed")
        .map_err(|e| DomainError::Storage(format!("cannot write keystore marker: {e}")))
}

/// True if `path` opens as a plaintext libSQL database whose schema is readable
/// (i.e. it is NOT encrypted). An encrypted file yields a read error here.
async fn plaintext_readable(path: &Path) -> bool {
    let Ok(db) = open_plain_db(path).await else {
        return false;
    };
    probe_readable(&db).await
}

/// True if the database's schema can be read (sanity check that the cipher key
/// is correct / the file is a usable SQLite database).
async fn probe_readable(db: &Database) -> bool {
    let Ok(conn) = db.connect() else {
        return false;
    };
    match conn.query("SELECT count(*) FROM sqlite_master", ()).await {
        Ok(mut rows) => rows.next().await.is_ok(),
        Err(_) => false,
    }
}

/// Copy the single account row from the plaintext keystore into a fresh
/// encrypted keystore, then atomically replace the file.
async fn migrate_plaintext_to_encrypted(path: &Path, device_key: &[u8]) -> Result<(), DomainError> {
    // 1. Read every account row from the plaintext keystore.
    let old_db = open_plain_db(path).await?;
    let old_conn = old_db.connect().map_err(map_storage)?;
    let select = format!("SELECT {ACCOUNT_BASE_COLUMNS} FROM account");
    let mut rows = old_conn.query(&select, ()).await.map_err(map_storage)?;

    struct Row {
        id: String,
        username: String,
        password_hash: String,
        wrapped_dek: Vec<u8>,
        kek_salt: Vec<u8>,
        dek_nonce: Vec<u8>,
        failed_attempts: i64,
        locked_until: Option<i64>,
        created_at: i64,
        updated_at: i64,
    }
    let mut accounts = Vec::new();
    while let Some(row) = rows.next().await.map_err(map_storage)? {
        accounts.push(Row {
            id: row.get(0).map_err(map_storage)?,
            username: row.get(1).map_err(map_storage)?,
            password_hash: row.get(2).map_err(map_storage)?,
            wrapped_dek: row.get(3).map_err(map_storage)?,
            kek_salt: row.get(4).map_err(map_storage)?,
            dek_nonce: row.get(5).map_err(map_storage)?,
            failed_attempts: row.get(6).map_err(map_storage)?,
            locked_until: row.get(7).map_err(map_storage)?,
            created_at: row.get(8).map_err(map_storage)?,
            updated_at: row.get(9).map_err(map_storage)?,
        });
    }
    drop(rows);
    drop(old_conn);
    drop(old_db);

    // 2. Build a fresh encrypted keystore at a temp path next to the target.
    let mut tmp_os = path.as_os_str().to_owned();
    tmp_os.push(".new");
    let tmp_path = PathBuf::from(tmp_os);
    remove_db_files(&tmp_path); // clear any leftover from an interrupted run

    {
        let new_db = open_encrypted_db(&tmp_path, device_key).await?;
        let new_conn = connect(&new_db).await?;
        migrations::run(&new_conn, KEYSTORE_MIGRATIONS).await?;
        let insert = format!(
            "INSERT INTO account ({ACCOUNT_BASE_COLUMNS}) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        );
        for a in &accounts {
            new_conn
                .execute(
                    &insert,
                    params![
                        a.id.clone(),
                        a.username.clone(),
                        a.password_hash.clone(),
                        a.wrapped_dek.clone(),
                        a.kek_salt.clone(),
                        a.dek_nonce.clone(),
                        a.failed_attempts,
                        a.locked_until,
                        a.created_at,
                        a.updated_at,
                    ],
                )
                .await
                .map_err(map_storage)?;
        }
        // Fold the WAL into the main file so the swapped file is self-contained.
        let _ = new_conn
            .execute("PRAGMA wal_checkpoint(TRUNCATE)", ())
            .await;
    } // drop new_conn + new_db → flush & close before the swap.

    // 3. Atomically replace the plaintext keystore with the encrypted one.
    remove_db_files(path);
    std::fs::rename(&tmp_path, path)
        .map_err(|e| DomainError::Storage(format!("cannot swap keystore file: {e}")))?;

    Ok(())
}
