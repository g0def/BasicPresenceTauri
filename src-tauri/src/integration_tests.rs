//! End-to-end backend test: wires the real use cases (Argon2id + envelope
//! encryption + libSQL keystore/vault) against temporary databases.

use std::path::PathBuf;

use crate::build_state;
use crate::domain::error::DomainError;
use crate::infrastructure::config::{AppConfig, Argon2Params, AuthPolicy};
use crate::infrastructure::persistence::db::open_plain_db;

fn temp_config() -> (AppConfig, PathBuf) {
    let mut dir = std::env::temp_dir();
    dir.push(format!("basic-presence-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let config = AppConfig {
        keystore_path: dir.join("keystore.db"),
        vault_path: dir.join("vault.db"),
        // Tiny Argon2 params so the test stays fast.
        argon2: Argon2Params {
            m_cost: 64,
            t_cost: 1,
            p_cost: 1,
        },
        auth: AuthPolicy {
            session_ttl_ms: 15 * 60 * 1000,
            max_attempts: 3,
            lockout_ms: 60_000,
        },
    };
    (config, dir)
}

#[test]
fn full_auth_flow_and_encryption() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config();
        let max_attempts = config.auth.max_attempts;
        let vault_path = config.vault_path.clone();
        let state = build_state(config).await.expect("build_state");

        // No account on first run.
        assert!(!state.account_exists.execute().await.unwrap());

        // Register the owner account.
        let user = state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
        assert_eq!(user.username, "alice");
        assert!(state.account_exists.execute().await.unwrap());

        // Single-owner device: a second registration is refused.
        assert!(matches!(
            state.register_account.execute("bob", "password123").await,
            Err(DomainError::AccountAlreadyExists)
        ));

        // Wrong password is rejected.
        assert!(matches!(
            state.login.execute("alice", "wrong-password").await,
            Err(DomainError::InvalidCredentials)
        ));

        // Correct password opens a session and unlocks the vault.
        let result = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        assert!(!result.token.is_empty());
        assert_eq!(result.user.username, "alice");

        // Session is valid, then logout invalidates it.
        assert!(state.check_session.execute(&result.token).unwrap().valid);
        state.logout.execute(&result.token).unwrap();
        assert!(!state.check_session.execute(&result.token).unwrap().valid);

        // The vault file must be encrypted: opening it WITHOUT the key cannot
        // read its schema.
        let plain = open_plain_db(&vault_path).await.unwrap();
        let conn = plain.connect().unwrap();
        let readable = match conn.query("SELECT name FROM sqlite_master", ()).await {
            Ok(mut rows) => rows.next().await.is_ok(),
            Err(_) => false,
        };
        assert!(!readable, "vault must be unreadable without the key");

        // Brute-force lockout: after `max_attempts` failures, even the correct
        // password is rejected with a lockout.
        for _ in 0..max_attempts {
            let _ = state.login.execute("alice", "wrong-password").await;
        }
        assert!(matches!(
            state.login.execute("alice", "password123").await,
            Err(DomainError::AccountLocked { .. })
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}
