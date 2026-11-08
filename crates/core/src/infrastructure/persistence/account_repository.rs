use async_trait::async_trait;
use libsql::{Connection, Row, params};

use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::error::DomainError;
use crate::domain::policy::KdfParams;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::infrastructure::persistence::db::map_storage;

const SELECT_COLUMNS: &str = "id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
     failed_attempts, locked_until, created_at, updated_at, wrapped_mac_key, mac_key_nonce, \
     key_version, kdf_m_cost, kdf_t_cost, kdf_p_cost";

/// libSQL-backed account repository (over the keystore connection).
pub struct LibsqlAccountRepository {
    conn: Connection,
}

impl LibsqlAccountRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

fn row_to_kdf(row: &Row) -> Result<Option<KdfParams>, DomainError> {
    let m: Option<i64> = row.get(13).map_err(map_storage)?;
    let t: Option<i64> = row.get(14).map_err(map_storage)?;
    let p: Option<i64> = row.get(15).map_err(map_storage)?;
    let (Some(m), Some(t), Some(p)) = (m, t, p) else {
        return Ok(None);
    };
    let to_u32 =
        |v: i64| u32::try_from(v).map_err(|_| DomainError::Storage("invalid KDF parameter".into()));
    Ok(Some(KdfParams {
        m_cost: to_u32(m)?,
        t_cost: to_u32(t)?,
        p_cost: to_u32(p)?,
    }))
}

fn row_to_account(row: &Row) -> Result<Account, DomainError> {
    Ok(Account {
        id: row.get(0).map_err(map_storage)?,
        username: row.get(1).map_err(map_storage)?,
        password_hash: row.get(2).map_err(map_storage)?,
        key_material: KeyMaterial {
            wrapped_dek: row.get(3).map_err(map_storage)?,
            kek_salt: row.get(4).map_err(map_storage)?,
            dek_nonce: row.get(5).map_err(map_storage)?,
            wrapped_mac_key: row.get(10).map_err(map_storage)?,
            mac_key_nonce: row.get(11).map_err(map_storage)?,
            key_version: row.get(12).map_err(map_storage)?,
            kdf: row_to_kdf(row)?,
        },
        failed_attempts: row.get(6).map_err(map_storage)?,
        locked_until: row.get(7).map_err(map_storage)?,
        created_at: row.get(8).map_err(map_storage)?,
        updated_at: row.get(9).map_err(map_storage)?,
    })
}

fn kdf_columns(kdf: Option<KdfParams>) -> (Option<i64>, Option<i64>, Option<i64>) {
    match kdf {
        Some(k) => (
            Some(i64::from(k.m_cost)),
            Some(i64::from(k.t_cost)),
            Some(i64::from(k.p_cost)),
        ),
        None => (None, None, None),
    }
}

#[async_trait]
impl AccountRepository for LibsqlAccountRepository {
    async fn exists(&self) -> Result<bool, DomainError> {
        let mut rows = self
            .conn
            .query("SELECT COUNT(*) FROM account", ())
            .await
            .map_err(map_storage)?;
        let row = rows
            .next()
            .await
            .map_err(map_storage)?
            .ok_or_else(|| DomainError::Storage("count returned no row".into()))?;
        let count: i64 = row.get(0).map_err(map_storage)?;
        Ok(count > 0)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DomainError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM account WHERE username = ?1");
        let mut rows = self
            .conn
            .query(&sql, params![username])
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row_to_account(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_owner(&self) -> Result<Option<Account>, DomainError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM account ORDER BY created_at LIMIT 1");
        let mut rows = self.conn.query(&sql, ()).await.map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row_to_account(&row)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, account: &Account) -> Result<(), DomainError> {
        let km = &account.key_material;
        let (m, t, p) = kdf_columns(km.kdf);
        // Single statement = atomic check-and-insert: two concurrent
        // registrations cannot both see "no account" and both insert.
        let inserted = self
            .conn
            .execute(
                "INSERT INTO account \
                 (id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
                  failed_attempts, locked_until, created_at, updated_at, \
                  wrapped_mac_key, mac_key_nonce, key_version, kdf_m_cost, kdf_t_cost, kdf_p_cost) \
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16 \
                 WHERE NOT EXISTS (SELECT 1 FROM account)",
                params![
                    account.id.clone(),
                    account.username.clone(),
                    account.password_hash.clone(),
                    km.wrapped_dek.clone(),
                    km.kek_salt.clone(),
                    km.dek_nonce.clone(),
                    account.failed_attempts,
                    account.locked_until,
                    account.created_at,
                    account.updated_at,
                    km.wrapped_mac_key.clone(),
                    km.mac_key_nonce.clone(),
                    km.key_version,
                    m,
                    t,
                    p,
                ],
            )
            .await
            .map_err(map_storage)?;
        if inserted == 0 {
            return Err(DomainError::AccountAlreadyExists);
        }
        Ok(())
    }

    async fn register_failed_attempt(&self, id: &str, now: i64) -> Result<i64, DomainError> {
        let mut rows = self
            .conn
            .query(
                "UPDATE account SET failed_attempts = failed_attempts + 1, updated_at = ?1 \
                 WHERE id = ?2 RETURNING failed_attempts",
                params![now, id],
            )
            .await
            .map_err(map_storage)?;
        let row = rows
            .next()
            .await
            .map_err(map_storage)?
            .ok_or_else(|| DomainError::Storage("account vanished".into()))?;
        row.get(0).map_err(map_storage)
    }

    async fn set_locked_until(
        &self,
        id: &str,
        locked_until: Option<i64>,
        now: i64,
    ) -> Result<(), DomainError> {
        self.conn
            .execute(
                "UPDATE account SET locked_until = ?1, updated_at = ?2 WHERE id = ?3",
                params![locked_until, now, id],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn reset_failed_attempts(&self, id: &str, updated_at: i64) -> Result<(), DomainError> {
        self.conn
            .execute(
                "UPDATE account SET failed_attempts = 0, locked_until = NULL, updated_at = ?1 \
                 WHERE id = ?2",
                params![updated_at, id],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn update_credentials(
        &self,
        id: &str,
        password_hash: &str,
        km: &KeyMaterial,
        updated_at: i64,
    ) -> Result<(), DomainError> {
        let (m, t, p) = kdf_columns(km.kdf);
        self.conn
            .execute(
                "UPDATE account SET password_hash = ?1, wrapped_dek = ?2, kek_salt = ?3, \
                 dek_nonce = ?4, wrapped_mac_key = ?5, mac_key_nonce = ?6, key_version = ?7, \
                 kdf_m_cost = ?8, kdf_t_cost = ?9, kdf_p_cost = ?10, updated_at = ?11 \
                 WHERE id = ?12",
                params![
                    password_hash.to_string(),
                    km.wrapped_dek.clone(),
                    km.kek_salt.clone(),
                    km.dek_nonce.clone(),
                    km.wrapped_mac_key.clone(),
                    km.mac_key_nonce.clone(),
                    km.key_version,
                    m,
                    t,
                    p,
                    updated_at,
                    id.to_string(),
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
