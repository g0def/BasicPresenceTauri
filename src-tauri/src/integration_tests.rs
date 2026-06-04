//! End-to-end backend test: wires the real use cases (Argon2id + envelope
//! encryption + libSQL keystore/vault) against temporary databases.

use std::path::PathBuf;

use crate::build_state;
use crate::domain::error::DomainError;
use crate::infrastructure::config::{AppConfig, Argon2Params, AuthPolicy};
use crate::infrastructure::persistence::db::open_plain_db;

fn temp_config(name: &str) -> (AppConfig, PathBuf) {
    let mut dir = std::env::temp_dir();
    dir.push(format!("basic-presence-it-{}-{name}", std::process::id()));
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
        let (config, dir) = temp_config("auth");
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

#[test]
fn profile_crud_requires_unlocked_vault() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("profile");
        let state = build_state(config).await.expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");

        // Vault is still locked before login: profile operations are refused.
        assert!(matches!(
            state
                .create_profile
                .execute("Ada", "Lovelace", "Analytical Engine", None)
                .await,
            Err(DomainError::Unauthorized)
        ));

        let session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");

        // Create: the first profile becomes the active one.
        let p1 = state
            .create_profile
            .execute(
                "Ada",
                "Lovelace",
                "Analytical Engine",
                Some("Mathematician"),
            )
            .await
            .expect("create p1");
        assert_eq!(p1.first_name, "Ada");
        assert_eq!(p1.poste.as_deref(), Some("Mathematician"));

        let listed = state.list_profiles.execute().await.expect("list");
        assert_eq!(listed.profiles.len(), 1);
        assert_eq!(listed.active_profile_id.as_deref(), Some(p1.id.as_str()));

        // A blank optional field collapses to None; required fields are enforced.
        let p2 = state
            .create_profile
            .execute("Alan", "Turing", "Bletchley", Some("  "))
            .await
            .expect("create p2");
        assert_eq!(p2.poste, None);
        assert!(matches!(
            state.create_profile.execute("  ", "X", "Y", None).await,
            Err(DomainError::Validation(_))
        ));

        // Adding a second profile does not change the active one.
        let listed = state.list_profiles.execute().await.expect("list");
        assert_eq!(listed.profiles.len(), 2);
        assert_eq!(listed.active_profile_id.as_deref(), Some(p1.id.as_str()));

        // Switch active, then update.
        state
            .set_active_profile
            .execute(&p2.id)
            .await
            .expect("set active");
        let updated = state
            .update_profile
            .execute(&p2.id, "Alan", "Turing", "GCHQ", None)
            .await
            .expect("update");
        assert_eq!(updated.enterprise, "GCHQ");

        // Deleting the active profile falls back to the first remaining one.
        state.delete_profile.execute(&p2.id).await.expect("delete");
        let listed = state.list_profiles.execute().await.expect("list");
        assert_eq!(listed.profiles.len(), 1);
        assert_eq!(listed.active_profile_id.as_deref(), Some(p1.id.as_str()));

        // Logout locks the vault; profile operations are refused again.
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state.list_profiles.execute().await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn presence_set_list_delete_with_upsert_per_day() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("presence");
        let state = build_state(config).await.expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");

        // Vault still locked before login: presence operations are refused.
        assert!(matches!(
            state.set_presence.execute("nope", 0, "office").await,
            Err(DomainError::Unauthorized)
        ));

        let session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");

        let profile = state
            .create_profile
            .execute("Ada", "Lovelace", "Analytical Engine", None)
            .await
            .expect("create profile");

        const DAY_1: i64 = 1_717_200_000_000; // arbitrary UTC-midnight epoch ms
        const DAY_2: i64 = DAY_1 + 86_400_000;

        // Set a presence for DAY_1.
        let p1 = state
            .set_presence
            .execute(&profile.id, DAY_1, "office")
            .await
            .expect("set office");
        assert_eq!(p1.kind, "office");
        assert_eq!(p1.day, DAY_1);

        // Re-setting the same day upserts (no duplicate) and overwrites the type,
        // while preserving the original id and created_at.
        let p1b = state
            .set_presence
            .execute(&profile.id, DAY_1, "remote")
            .await
            .expect("set remote");
        assert_eq!(p1b.id, p1.id);
        assert_eq!(p1b.kind, "remote");
        assert_eq!(p1b.created_at, p1.created_at);

        let listed = state
            .list_presences
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].kind, "remote");

        // An unknown type is rejected by the domain.
        assert!(matches!(
            state
                .set_presence
                .execute(&profile.id, DAY_2, "carpool")
                .await,
            Err(DomainError::Validation(_))
        ));

        // A non-midnight day (not a multiple of 86_400_000 ms) is rejected.
        assert!(matches!(
            state
                .set_presence
                .execute(&profile.id, DAY_1 + 1, "office")
                .await,
            Err(DomainError::Validation(_))
        ));

        // A second day adds a second presence.
        state
            .set_presence
            .execute(&profile.id, DAY_2, "vacation")
            .await
            .expect("set vacation");
        let listed = state
            .list_presences
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(listed.len(), 2);

        // Deleting by id removes a single presence.
        state.delete_presence.execute(&p1.id).await.expect("delete");
        let listed = state
            .list_presences
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].day, DAY_2);

        // Logout locks the vault; presence operations are refused again.
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state.list_presences.execute(&profile.id).await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn deleting_a_profile_cascades_to_its_presences() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("presence_cascade");
        let state = build_state(config).await.expect("build_state");

        state
            .register_account
            .execute("bob", "password123")
            .await
            .expect("register");
        state
            .login
            .execute("bob", "password123")
            .await
            .expect("login");

        let profile = state
            .create_profile
            .execute("Grace", "Hopper", "Navy", None)
            .await
            .expect("create profile");

        const DAY: i64 = 1_717_200_000_000; // UTC-midnight epoch ms
        state
            .set_presence
            .execute(&profile.id, DAY, "office")
            .await
            .expect("set");
        assert_eq!(
            state
                .list_presences
                .execute(&profile.id)
                .await
                .expect("list")
                .len(),
            1
        );

        // Deleting the profile must cascade-delete its presences. This only holds
        // when `PRAGMA foreign_keys = ON` is set on the vault connection.
        state
            .delete_profile
            .execute(&profile.id)
            .await
            .expect("delete profile");
        assert!(state
            .list_presences
            .execute(&profile.id)
            .await
            .expect("list")
            .is_empty());

        let _ = std::fs::remove_dir_all(dir);
    });
}
