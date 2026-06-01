use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::infrastructure::persistence::db::map_storage;

const SELECT_COLUMNS: &str = "id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
     failed_attempts, locked_until, created_at, updated_at";

/// libSQL-backed account repository (over the keystore connection).
pub struct LibsqlAccountRepository {
    conn: Connection,
}

impl LibsqlAccountRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
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
        },
        failed_attempts: row.get(6).map_err(map_storage)?,
        locked_until: row.get(7).map_err(map_storage)?,
        created_at: row.get(8).map_err(map_storage)?,
        updated_at: row.get(9).map_err(map_storage)?,
    })
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

    async fn create(&self, account: &Account) -> Result<(), DomainError> {
        self.conn
            .execute(
                "INSERT INTO account \
                 (id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
                  failed_attempts, locked_until, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    account.id.clone(),
                    account.username.clone(),
                    account.password_hash.clone(),
                    account.key_material.wrapped_dek.clone(),
                    account.key_material.kek_salt.clone(),
                    account.key_material.dek_nonce.clone(),
                    account.failed_attempts,
                    account.locked_until,
                    account.created_at,
                    account.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn record_failed_attempt(
        &self,
        id: &str,
        failed_attempts: i64,
        locked_until: Option<i64>,
        updated_at: i64,
    ) -> Result<(), DomainError> {
        self.conn
            .execute(
                "UPDATE account SET failed_attempts = ?1, locked_until = ?2, updated_at = ?3 \
                 WHERE id = ?4",
                params![failed_attempts, locked_until, updated_at, id],
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
}
