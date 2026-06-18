//! End-to-end backend test: wires the real use cases (Argon2id + envelope
//! encryption + libSQL keystore/vault) against temporary databases.

use std::path::PathBuf;

use crate::application::dto::commute_dto::CommuteSegmentInputDto;
use crate::application::dto::import_presence_dto::ImportPresenceEntryDto;
use crate::application::dto::trip_dto::TripInputDto;
use crate::application::dto::work_entry_dto::WorkEntryInputDto;
use crate::build_state;
use crate::domain::error::DomainError;
use crate::infrastructure::config::{AppConfig, Argon2Params, AuthPolicy, IntegrityPolicy};
use crate::infrastructure::persistence::db::open_plain_db;

/// Fixed device key for tests: stands in for the OS-keychain-backed key so the
/// suite never touches a real Secret Service (keeps it headless/CI-safe).
const TEST_DEVICE_KEY: [u8; 32] = [7u8; 32];

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
        integrity: IntegrityPolicy::WarnAndAllow,
    };
    (config, dir)
}

#[test]
fn full_auth_flow_and_encryption() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("auth");
        let max_attempts = config.auth.max_attempts;
        let vault_path = config.vault_path.clone();
        let keystore_path = config.keystore_path.clone();
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

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

        // The keystore must ALSO be sealed at rest: opening it in clear cannot
        // read its schema (defeats offline brute-force of a stolen file).
        let plain_ks = open_plain_db(&keystore_path).await.unwrap();
        let ks_conn = plain_ks.connect().unwrap();
        let ks_readable = match ks_conn.query("SELECT name FROM sqlite_master", ()).await {
            Ok(mut rows) => rows.next().await.is_ok(),
            Err(_) => false,
        };
        assert!(
            !ks_readable,
            "keystore must be unreadable without the device key"
        );

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
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

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
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");

        // Vault still locked before login: presence operations are refused.
        assert!(matches!(
            state
                .set_presence
                .execute("nope", 0, "office", vec![])
                .await,
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
            .execute(&profile.id, DAY_1, "office", vec![])
            .await
            .expect("set office");
        assert_eq!(p1.kind, "office");
        assert_eq!(p1.day, DAY_1);

        // Re-setting the same day upserts (no duplicate) and overwrites the type,
        // while preserving the original id and created_at.
        let p1b = state
            .set_presence
            .execute(&profile.id, DAY_1, "remote", vec![])
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
                .execute(&profile.id, DAY_2, "carpool", vec![])
                .await,
            Err(DomainError::Validation(_))
        ));

        // A non-midnight day (not a multiple of 86_400_000 ms) is rejected.
        assert!(matches!(
            state
                .set_presence
                .execute(&profile.id, DAY_1 + 1, "office", vec![])
                .await,
            Err(DomainError::Validation(_))
        ));

        // A second day adds a second presence.
        state
            .set_presence
            .execute(&profile.id, DAY_2, "vacation", vec![])
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
fn presence_import_skip_and_replace() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("presence_import");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
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

        const DAY_1: i64 = 1_717_200_000_000; // UTC-midnight epoch ms
        const DAY_2: i64 = DAY_1 + 86_400_000;
        const DAY_3: i64 = DAY_1 + 2 * 86_400_000;
        const TS: i64 = 1_700_000_000_000;

        // Build an entry exactly as the command would receive it from the front.
        let entry = |day: i64, kind: &str, created: Option<i64>| ImportPresenceEntryDto {
            day,
            kind: kind.to_string(),
            created_at: created,
            updated_at: None,
        };

        // Initial import into an empty profile: everything inserted, and the
        // file's created_at is honored (with a fallback when absent).
        let s = state
            .import_presences
            .execute(
                &profile.id,
                vec![
                    entry(DAY_1, "office", Some(TS)),
                    entry(DAY_2, "remote", None),
                ],
                false,
            )
            .await
            .expect("import");
        assert_eq!((s.imported, s.skipped, s.replaced, s.total), (2, 0, 0, 2));
        let day1 = state
            .list_presences
            .execute(&profile.id)
            .await
            .unwrap()
            .into_iter()
            .find(|p| p.day == DAY_1)
            .unwrap();
        assert_eq!(day1.kind, "office");
        assert_eq!(day1.created_at, TS, "created_at preserved from the file");

        // Re-import overlapping DAY_1 (different type) + new DAY_3 with the skip
        // strategy: DAY_1 is left untouched, DAY_3 is inserted.
        let s = state
            .import_presences
            .execute(
                &profile.id,
                vec![
                    entry(DAY_1, "vacation", Some(TS + 1)),
                    entry(DAY_3, "holiday", None),
                ],
                false,
            )
            .await
            .expect("import skip");
        assert_eq!((s.imported, s.skipped, s.replaced, s.total), (1, 1, 0, 2));
        let day1 = state
            .list_presences
            .execute(&profile.id)
            .await
            .unwrap()
            .into_iter()
            .find(|p| p.day == DAY_1)
            .unwrap();
        assert_eq!(day1.kind, "office", "skip kept the original type");
        assert_eq!(day1.created_at, TS, "skip kept the original created_at");

        // Re-import DAY_1 with the replace strategy: the type is overwritten but
        // the original created_at is preserved.
        let s = state
            .import_presences
            .execute(
                &profile.id,
                vec![entry(DAY_1, "vacation", Some(TS + 99))],
                true,
            )
            .await
            .expect("import replace");
        assert_eq!((s.imported, s.skipped, s.replaced, s.total), (0, 0, 1, 1));
        let day1 = state
            .list_presences
            .execute(&profile.id)
            .await
            .unwrap()
            .into_iter()
            .find(|p| p.day == DAY_1)
            .unwrap();
        assert_eq!(day1.kind, "vacation", "replace overwrote the type");
        assert_eq!(
            day1.created_at, TS,
            "replace preserved the original created_at"
        );

        // A duplicate day within a single import keeps the counts consistent.
        const DAY_DUP: i64 = DAY_1 + 10 * 86_400_000;
        let s = state
            .import_presences
            .execute(
                &profile.id,
                vec![
                    entry(DAY_DUP, "office", None),
                    entry(DAY_DUP, "remote", None),
                ],
                true,
            )
            .await
            .expect("import dup");
        assert_eq!(s.imported + s.replaced + s.skipped, s.total);

        // Unknown type and non-midnight day are rejected by validation.
        assert!(matches!(
            state
                .import_presences
                .execute(&profile.id, vec![entry(DAY_2, "carpool", None)], false)
                .await,
            Err(DomainError::Validation(_))
        ));
        assert!(matches!(
            state
                .import_presences
                .execute(&profile.id, vec![entry(DAY_1 + 1, "office", None)], false)
                .await,
            Err(DomainError::Validation(_))
        ));

        // Logout locks the vault; import is refused.
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state
                .import_presences
                .execute(&profile.id, vec![entry(DAY_1, "office", None)], false)
                .await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn deleting_a_profile_cascades_to_its_presences() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("presence_cascade");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

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
            .execute(&profile.id, DAY, "office", vec![])
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

#[test]
fn legacy_plaintext_keystore_is_migrated_and_sealed() {
    use crate::infrastructure::persistence::db::connect;
    use crate::infrastructure::persistence::keystore_bootstrap::open_or_migrate_keystore;
    use crate::infrastructure::persistence::migrations::{run, KEYSTORE_MIGRATIONS};

    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("legacy_migration");
        let keystore_path = config.keystore_path.clone();

        // Fabricate a legacy plaintext keystore with one account row and NO
        // sealed marker (the pre-hardening on-disk shape).
        {
            let db = open_plain_db(&keystore_path).await.unwrap();
            let conn = connect(&db).await.unwrap();
            run(&conn, KEYSTORE_MIGRATIONS).await.unwrap();
            conn.execute(
                "INSERT INTO account \
                 (id, username, password_hash, wrapped_dek, kek_salt, dek_nonce, \
                  failed_attempts, locked_until, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                libsql::params![
                    "id-1",
                    "legacy-user",
                    "$argon2id$dummy",
                    vec![1u8; 48],
                    vec![2u8; 16],
                    vec![3u8; 24],
                    0_i64,
                    None::<i64>,
                    1_700_000_000_000_i64,
                    1_700_000_000_000_i64,
                ],
            )
            .await
            .unwrap();
        }

        // Sanity: the fixture is readable in clear before migration.
        let plain = open_plain_db(&keystore_path).await.unwrap();
        let pconn = plain.connect().unwrap();
        assert!(pconn
            .query("SELECT count(*) FROM account", ())
            .await
            .is_ok());
        drop(pconn);
        drop(plain);

        // Migrate: open_or_migrate_keystore detects the plaintext store, copies
        // the row into an encrypted one, swaps the file, and writes the marker.
        let sealed = open_or_migrate_keystore(&keystore_path, &TEST_DEVICE_KEY)
            .await
            .expect("migration");
        let sconn = connect(&sealed).await.unwrap();
        let mut rows = sconn
            .query("SELECT username FROM account", ())
            .await
            .expect("read migrated account");
        let row = rows.next().await.unwrap().expect("one account row");
        let username: String = row.get(0).unwrap();
        assert_eq!(username, "legacy-user", "account row survived migration");
        drop(rows);
        drop(sconn);
        drop(sealed);

        // The keystore is now sealed: unreadable in clear, and the marker exists.
        let plain2 = open_plain_db(&keystore_path).await.unwrap();
        let p2conn = plain2.connect().unwrap();
        let readable = match p2conn.query("SELECT count(*) FROM account", ()).await {
            Ok(mut r) => r.next().await.is_ok(),
            Err(_) => false,
        };
        assert!(!readable, "migrated keystore must be encrypted at rest");

        let mut marker = keystore_path.as_os_str().to_owned();
        marker.push(".sealed");
        assert!(std::path::PathBuf::from(marker).exists(), "marker written");

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn require_session_gates_data_access() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("require_session");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");

        // No session yet: the gate every data command goes through refuses.
        assert!(matches!(
            state.require_session.execute(),
            Err(DomainError::Unauthorized)
        ));

        let session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        assert!(state.require_session.execute().is_ok());

        // After logout the gate refuses again (and the vault is locked).
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state.require_session.execute(),
            Err(DomainError::Unauthorized)
        ));
        assert!(matches!(
            state.list_profiles.execute().await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn tampering_with_the_vault_is_detected_under_hard_fail() {
    tauri::async_runtime::block_on(async {
        let (mut config, dir) = temp_config("tamper");
        config.integrity = IntegrityPolicy::HardFail;
        let vault_path = config.vault_path.clone();
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
        let session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        // Write some data, then close cleanly: this writes the integrity baseline
        // and clears the dirty marker.
        state
            .create_profile
            .execute("Ada", "Lovelace", "Analytical Engine", None)
            .await
            .expect("create profile");
        state.logout.execute(&session.token).unwrap();

        // Tamper with the at-rest ciphertext.
        let mut bytes = std::fs::read(&vault_path).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        std::fs::write(&vault_path, &bytes).unwrap();

        // Re-login must be refused: a mismatch after a clean shutdown is treated
        // as tampering under HardFail.
        assert!(matches!(
            state.login.execute("alice", "password123").await,
            Err(DomainError::VaultTampered)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn presence_co2_is_computed_snapshotted_and_tied_to_presence() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("co2_presence");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");
        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
        let session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        let profile = state
            .create_profile
            .execute("Ada", "Lovelace", "Analytical Engine", None)
            .await
            .expect("profile");

        const DAY_1: i64 = 1_717_200_000_000;
        const DAY_2: i64 = DAY_1 + 86_400_000;

        let trip = |mode: &str, km: f64| TripInputDto {
            mode_id: mode.to_string(),
            distance_km: km,
            round_trip: true,
            occupants: None,
        };

        // Office day, 15 km round-trip petrol commute → 15*2*0.2388 = 7.164.
        let p = state
            .set_presence
            .execute(&profile.id, DAY_1, "office", vec![trip("car_petrol", 15.0)])
            .await
            .expect("set office");
        let co2 = p.co2_kg.expect("co2 present");
        assert!((co2 - 7.164).abs() < 1e-3, "got {co2}");
        assert!(!p.is_estimated);

        // The trip snapshot is persisted with the per-segment footprint.
        let trips = state
            .get_presence_trips
            .execute(&p.id)
            .await
            .expect("trips");
        assert_eq!(trips.len(), 1);
        assert_eq!(trips[0].mode_id, "car_petrol");
        assert!((trips[0].co2_kg - 7.164).abs() < 1e-3);

        // list_presences returns the day total.
        let d1 = state
            .list_presences
            .execute(&profile.id)
            .await
            .unwrap()
            .into_iter()
            .find(|p| p.day == DAY_1)
            .unwrap();
        assert!((d1.co2_kg.unwrap() - 7.164).abs() < 1e-3);

        // Re-encoding the office day REPLACES the snapshot (same presence row).
        let p2 = state
            .set_presence
            .execute(
                &profile.id,
                DAY_1,
                "office",
                vec![trip("train_hs_fr", 100.0)],
            )
            .await
            .expect("re-set office");
        assert_eq!(p2.id, p.id);
        let trips2 = state.get_presence_trips.execute(&p.id).await.unwrap();
        assert_eq!(trips2.len(), 1);
        assert_eq!(trips2[0].mode_id, "train_hs_fr");

        // Switching to a non-commute type clears trips and the footprint.
        let p3 = state
            .set_presence
            .execute(&profile.id, DAY_1, "vacation", vec![])
            .await
            .expect("vacation");
        assert!(p3.co2_kg.is_none());
        assert!(state
            .get_presence_trips
            .execute(&p.id)
            .await
            .unwrap()
            .is_empty());

        // Remote days also accept an optional trip (scope: office + remote).
        let r = state
            .set_presence
            .execute(&profile.id, DAY_2, "remote", vec![trip("ebike", 8.0)])
            .await
            .expect("remote");
        assert!((r.co2_kg.unwrap() - 0.1752).abs() < 1e-3);

        state.logout.execute(&session.token).unwrap();
        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn commute_crud_and_emission_factors() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("commute_crud");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");
        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");

        // CO2 operations require an unlocked vault.
        assert!(matches!(
            state.list_emission_factors.execute().await,
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
            .expect("profile");

        // The 2025 referential is seeded; building factors are hidden from the
        // picker, and car_ev carries its 4 grid variants.
        let factors = state
            .list_emission_factors
            .execute()
            .await
            .expect("factors");
        assert!(factors.len() >= 24, "got {}", factors.len());
        assert!(factors.iter().all(|f| f.category != "building"));
        let ev = factors
            .iter()
            .find(|f| f.mode_id == "car_ev")
            .expect("car_ev");
        assert_eq!(ev.grid_variants.len(), 4);

        // Create "30 km train + 5 km vélo".
        let segs = vec![
            CommuteSegmentInputDto {
                mode_id: "train_sncb".into(),
                distance_km: 30.0,
                occupants: None,
            },
            CommuteSegmentInputDto {
                mode_id: "bike".into(),
                distance_km: 5.0,
                occupants: None,
            },
        ];
        let c = state
            .create_commute
            .execute(&profile.id, "Train + vélo", true, segs)
            .await
            .expect("create commute");
        assert_eq!(c.segments.len(), 2);

        // list_commutes annotates an indicative CO2: 30*2*0.021 + 0 = 1.26.
        let list = state
            .list_commutes
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(list.len(), 1);
        assert!((list[0].co2_kg.unwrap() - 1.26).abs() < 1e-3);

        // Update: rename + replace segments (carpool, 2 occupants).
        let segs2 = vec![CommuteSegmentInputDto {
            mode_id: "car_petrol".into(),
            distance_km: 10.0,
            occupants: Some(2),
        }];
        let updated = state
            .update_commute
            .execute(&c.id, "Voiture", false, segs2)
            .await
            .expect("update");
        assert_eq!(updated.name, "Voiture");
        assert_eq!(updated.segments.len(), 1);
        assert_eq!(updated.segments[0].occupants, 2);

        // Delete removes it (and its segments cascade).
        state.delete_commute.execute(&c.id).await.expect("delete");
        assert!(state
            .list_commutes
            .execute(&profile.id)
            .await
            .unwrap()
            .is_empty());

        // Deleting the profile cascades to its commutes.
        let segs3 = vec![CommuteSegmentInputDto {
            mode_id: "walk".into(),
            distance_km: 1.0,
            occupants: None,
        }];
        state
            .create_commute
            .execute(&profile.id, "Marche", true, segs3)
            .await
            .expect("create2");
        state
            .delete_profile
            .execute(&profile.id)
            .await
            .expect("delete profile");
        assert!(state
            .list_commutes
            .execute(&profile.id)
            .await
            .unwrap()
            .is_empty());

        state.logout.execute(&session.token).unwrap();
        let _ = std::fs::remove_dir_all(dir);
    });
}

/// A day work entry with a default color, for the work-hours tests.
fn work_entry(title: &str, minutes: i64) -> WorkEntryInputDto {
    WorkEntryInputDto {
        title: title.to_string(),
        description: None,
        minutes,
        color: "#4B7F52".to_string(),
    }
}

#[test]
fn task_preset_crud_and_validation() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("task-preset");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        // Vault locked before login: preset operations are refused.
        assert!(matches!(
            state.list_task_presets.execute("nope").await,
            Err(DomainError::Unauthorized)
        ));

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
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

        // Create two presets out of alphabetical order.
        let beta = state
            .create_task_preset
            .execute(&profile.id, "Beta", None, 30, "#CBA53C")
            .await
            .expect("create beta");
        let alpha = state
            .create_task_preset
            .execute(
                &profile.id,
                "  Alpha  ",
                Some("  notes  ".to_string()),
                60,
                " #4b7f52 ",
            )
            .await
            .expect("create alpha");
        // Title/description trimmed, color preserved as given (trimmed).
        assert_eq!(alpha.title, "Alpha");
        assert_eq!(alpha.description.as_deref(), Some("notes"));

        // Listed alphabetically by title.
        let listed = state
            .list_task_presets
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(
            listed.iter().map(|p| p.title.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "Beta"]
        );

        // Field validation is rejected at the use-case boundary.
        assert!(matches!(
            state
                .create_task_preset
                .execute(&profile.id, "   ", None, 30, "#CBA53C")
                .await,
            Err(DomainError::Validation(_))
        ));
        assert!(matches!(
            state
                .create_task_preset
                .execute(&profile.id, "Bad", None, 7, "#CBA53C")
                .await,
            Err(DomainError::Validation(_))
        ));
        assert!(matches!(
            state
                .create_task_preset
                .execute(&profile.id, "Bad", None, 30, "not-a-color")
                .await,
            Err(DomainError::Validation(_))
        ));

        // Update keeps the id and created_at, refreshes updated_at.
        let updated = state
            .update_task_preset
            .execute(&alpha.id, "Zeta", None, 90, "#005377")
            .await
            .expect("update");
        assert_eq!(updated.id, alpha.id);
        assert_eq!(updated.created_at, alpha.created_at);
        assert!(updated.updated_at >= alpha.updated_at);
        assert_eq!(updated.title, "Zeta");
        assert_eq!(updated.description, None);

        // Updating an unknown preset is a validation error, not a silent no-op.
        assert!(matches!(
            state
                .update_task_preset
                .execute("does-not-exist", "X", None, 30, "#005377")
                .await,
            Err(DomainError::Validation(_))
        ));

        // Re-list reflects the rename (now [Beta, Zeta]).
        let listed = state
            .list_task_presets
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(
            listed.iter().map(|p| p.title.as_str()).collect::<Vec<_>>(),
            vec!["Beta", "Zeta"]
        );

        // Delete removes a single preset.
        state
            .delete_task_preset
            .execute(&beta.id)
            .await
            .expect("delete");
        let listed = state
            .list_task_presets
            .execute(&profile.id)
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, alpha.id);

        state.logout.execute(&session.token).unwrap();
        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn work_entries_schedule_recompute_and_type_flip_cleanup() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("work-entries");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
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

        const DAY_OFFICE: i64 = 1_717_200_000_000;
        const DAY_VACATION: i64 = DAY_OFFICE + 86_400_000;

        let office = state
            .set_presence
            .execute(&profile.id, DAY_OFFICE, "office", vec![])
            .await
            .expect("set office");
        let vacation = state
            .set_presence
            .execute(&profile.id, DAY_VACATION, "vacation", vec![])
            .await
            .expect("set vacation");

        // A fresh work day has no entries, but its schedule is seeded with the
        // profile's default start (08:30); end == start until entries are added.
        assert!(state
            .get_work_entries
            .execute(&office.id)
            .await
            .unwrap()
            .is_empty());
        let seeded = state
            .get_work_schedule
            .execute(&office.id)
            .await
            .unwrap()
            .expect("a fresh office day seeds a default schedule");
        assert_eq!(seeded.start_minutes, 510);
        assert_eq!(seeded.end_minutes, 510);
        // Non-work days never get a seeded schedule.
        assert!(state
            .get_work_schedule
            .execute(&vacation.id)
            .await
            .unwrap()
            .is_none());

        // Hours can only be encoded on office/remote days.
        assert!(matches!(
            state
                .set_work_entries
                .execute(&vacation.id, vec![work_entry("Nope", 60)])
                .await,
            Err(DomainError::Validation(_))
        ));
        assert!(matches!(
            state.set_work_schedule.execute(&vacation.id, 510).await,
            Err(DomainError::Validation(_))
        ));
        // An unknown presence is refused too.
        assert!(matches!(
            state
                .set_work_entries
                .execute("does-not-exist", vec![work_entry("Nope", 60)])
                .await,
            Err(DomainError::Validation(_))
        ));

        // Replace-all: positions follow array order; ids are generated server-side.
        let day = state
            .set_work_entries
            .execute(
                &office.id,
                vec![work_entry("Dev", 120), work_entry("Réunion", 60)],
            )
            .await
            .expect("set entries");
        assert_eq!(day.entries.len(), 2);
        assert_eq!(day.entries[0].position, 0);
        assert_eq!(day.entries[1].position, 1);
        assert!(day.entries.iter().all(|e| !e.id.is_empty()));
        // The seeded schedule's end was recomputed from the new entries (start + 180).
        let sched = day
            .schedule
            .expect("seeded schedule recomputed on entries save");
        assert_eq!(sched.start_minutes, 510);
        assert_eq!(sched.end_minutes, 510 + 180);

        let listed = state.get_work_entries.execute(&office.id).await.unwrap();
        assert_eq!(
            listed.iter().map(|e| e.title.as_str()).collect::<Vec<_>>(),
            vec!["Dev", "Réunion"]
        );

        // Setting the start derives the stored end (start + sum of durations).
        let sched = state
            .set_work_schedule
            .execute(&office.id, 510) // 08:30
            .await
            .expect("set schedule");
        assert_eq!(sched.start_minutes, 510);
        assert_eq!(sched.end_minutes, 510 + 180);

        // Editing the entries recomputes and returns the stored end.
        let first_ids: Vec<String> = listed.iter().map(|e| e.id.clone()).collect();
        let day = state
            .set_work_entries
            .execute(
                &office.id,
                vec![
                    work_entry("Dev", 120),
                    work_entry("Réunion", 60),
                    work_entry("Revue", 30),
                ],
            )
            .await
            .expect("set entries 2");
        let schedule = day.schedule.expect("schedule recomputed");
        assert_eq!(schedule.end_minutes, 510 + 210);
        assert_eq!(
            state
                .get_work_schedule
                .execute(&office.id)
                .await
                .unwrap()
                .unwrap()
                .end_minutes,
            510 + 210
        );
        // Replace-all regenerates ids (entries are not stable handles).
        assert!(day.entries.iter().all(|e| !first_ids.contains(&e.id)));

        // A single invalid entry aborts the whole save: nothing is written.
        assert!(matches!(
            state
                .set_work_entries
                .execute(&office.id, vec![work_entry("Ok", 60), work_entry("Bad", 7)])
                .await,
            Err(DomainError::Validation(_))
        ));
        let after = state.get_work_entries.execute(&office.id).await.unwrap();
        assert_eq!(after.len(), 3, "failed save must leave the day untouched");

        // Flipping the day off work clears its entries AND its schedule (the
        // upsert keeps the row id, so the cleanup runs via the type trigger).
        state
            .set_presence
            .execute(&profile.id, DAY_OFFICE, "vacation", vec![])
            .await
            .expect("flip to vacation");
        assert!(state
            .get_work_entries
            .execute(&office.id)
            .await
            .unwrap()
            .is_empty());
        assert!(state
            .get_work_schedule
            .execute(&office.id)
            .await
            .unwrap()
            .is_none());

        // Logout locks the vault: work-hours reads are refused again.
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state.get_work_entries.execute(&office.id).await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn office_day_seeds_default_start_time() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("seed-start");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");
        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
        let _session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        let profile = state
            .create_profile
            .execute("Ada", "Lovelace", "Analytical Engine", None)
            .await
            .expect("profile");

        const DAY: i64 = 1_717_200_000_000;
        const DAY_IMPORT: i64 = DAY + 86_400_000;

        // Set a custom default start (10:00) for this profile.
        let mut settings = state
            .get_profile_settings
            .execute(&profile.id)
            .await
            .expect("get settings");
        settings.default_start_minutes = 600;
        state
            .set_profile_settings
            .execute(&profile.id, settings)
            .await
            .expect("set settings");

        // Creating an office day seeds the schedule at the configured default.
        let office = state
            .set_presence
            .execute(&profile.id, DAY, "office", vec![])
            .await
            .expect("office");
        let sched = state
            .get_work_schedule
            .execute(&office.id)
            .await
            .unwrap()
            .expect("seeded schedule");
        assert_eq!(sched.start_minutes, 600);
        assert_eq!(sched.end_minutes, 600);

        // A user-set start is never clobbered by a later re-set (e.g. flipping
        // office <-> remote, both work days, keeps the same row id and schedule).
        state
            .set_work_schedule
            .execute(&office.id, 420) // user picks 07:00
            .await
            .expect("user sets start");
        let remote = state
            .set_presence
            .execute(&profile.id, DAY, "remote", vec![])
            .await
            .expect("flip to remote");
        assert_eq!(remote.id, office.id);
        assert_eq!(
            state
                .get_work_schedule
                .execute(&office.id)
                .await
                .unwrap()
                .unwrap()
                .start_minutes,
            420,
            "re-setting a work day must not overwrite the user's start"
        );

        // office -> vacation drops the schedule (type trigger); vacation -> office
        // re-creates it, so the default is re-seeded.
        state
            .set_presence
            .execute(&profile.id, DAY, "vacation", vec![])
            .await
            .expect("vacation");
        assert!(state
            .get_work_schedule
            .execute(&office.id)
            .await
            .unwrap()
            .is_none());
        let reoffice = state
            .set_presence
            .execute(&profile.id, DAY, "office", vec![])
            .await
            .expect("re-office");
        assert_eq!(
            state
                .get_work_schedule
                .execute(&reoffice.id)
                .await
                .unwrap()
                .unwrap()
                .start_minutes,
            600,
            "a re-created work day is re-seeded with the default"
        );

        // Bulk import never seeds a schedule (it does not go through set_presence).
        state
            .import_presences
            .execute(
                &profile.id,
                vec![ImportPresenceEntryDto {
                    day: DAY_IMPORT,
                    kind: "office".to_string(),
                    created_at: None,
                    updated_at: None,
                }],
                false,
            )
            .await
            .expect("import");
        let imported = state
            .list_presences
            .execute(&profile.id)
            .await
            .unwrap()
            .into_iter()
            .find(|p| p.day == DAY_IMPORT)
            .unwrap();
        assert!(
            state
                .get_work_schedule
                .execute(&imported.id)
                .await
                .unwrap()
                .is_none(),
            "import must not seed schedules"
        );

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn presence_co2_is_computed_per_profile() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("co2-per-profile");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");
        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
        let _session = state
            .login
            .execute("alice", "password123")
            .await
            .expect("login");
        let solo = state
            .create_profile
            .execute("Solo", "Driver", "Acme", None)
            .await
            .expect("profile solo");
        let pool = state
            .create_profile
            .execute("Car", "Pool", "Acme", None)
            .await
            .expect("profile pool");

        // The carpool profile rides two-up; the solo profile keeps the default 1.
        let mut pool_settings = state
            .get_profile_settings
            .execute(&pool.id)
            .await
            .expect("get pool settings");
        pool_settings.default_car_occupancy = 2;
        state
            .set_profile_settings
            .execute(&pool.id, pool_settings)
            .await
            .expect("set pool settings");

        const DAY: i64 = 1_717_200_000_000;
        // Falls back to the profile's default occupancy (occupants: None).
        let trip = || TripInputDto {
            mode_id: "car_petrol".to_string(),
            distance_km: 15.0,
            round_trip: true,
            occupants: None,
        };

        let solo_day = state
            .set_presence
            .execute(&solo.id, DAY, "office", vec![trip()])
            .await
            .expect("solo office");
        let pool_day = state
            .set_presence
            .execute(&pool.id, DAY, "office", vec![trip()])
            .await
            .expect("pool office");

        let solo_co2 = solo_day.co2_kg.expect("solo co2");
        let pool_co2 = pool_day.co2_kg.expect("pool co2");
        // Per-vehicle factor split across occupants: two-up is half the footprint.
        assert!((solo_co2 - 7.164).abs() < 1e-3, "solo {solo_co2}");
        assert!(
            (pool_co2 - solo_co2 / 2.0).abs() < 1e-3,
            "pool {pool_co2} should be half of solo {solo_co2}"
        );

        let _ = std::fs::remove_dir_all(dir);
    });
}

#[test]
fn day_note_persists_and_is_rendered_to_safe_html() {
    tauri::async_runtime::block_on(async {
        let (config, dir) = temp_config("day-note");
        let state = build_state(config, &TEST_DEVICE_KEY)
            .await
            .expect("build_state");

        state
            .register_account
            .execute("alice", "password123")
            .await
            .expect("register");
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

        const DAY: i64 = 1_717_200_000_000;
        let presence = state
            .set_presence
            .execute(&profile.id, DAY, "office", vec![])
            .await
            .expect("set office");

        // A fresh day has no note: empty Markdown and empty HTML.
        let fresh = state.get_day_note.execute(&presence.id).await.unwrap();
        assert!(fresh.markdown.is_empty());
        assert!(fresh.html.trim().is_empty());

        // Saving renders Markdown to sanitized HTML; the raw Markdown round-trips.
        let saved = state
            .set_day_note
            .execute(
                &presence.id,
                Some("# Titre\n\n**bold** <script>alert(1)</script>".to_string()),
            )
            .await
            .expect("set note");
        assert!(saved.markdown.contains("# Titre"));
        assert!(saved.html.contains("<h1>Titre</h1>"), "got: {}", saved.html);
        assert!(saved.html.contains("<strong>bold</strong>"));
        assert!(
            !saved.html.contains("<script"),
            "script survived: {}",
            saved.html
        );

        // The note is persisted and re-rendered identically on read.
        let read = state.get_day_note.execute(&presence.id).await.unwrap();
        assert_eq!(read.markdown, saved.markdown);
        assert_eq!(read.html, saved.html);

        // A blank note clears the column (stored as NULL).
        let cleared = state
            .set_day_note
            .execute(&presence.id, Some("   \n  ".to_string()))
            .await
            .expect("clear note");
        assert!(cleared.markdown.is_empty());
        assert!(state
            .get_day_note
            .execute(&presence.id)
            .await
            .unwrap()
            .markdown
            .is_empty());

        // Logout locks the vault: note reads are refused again.
        state.logout.execute(&session.token).unwrap();
        assert!(matches!(
            state.get_day_note.execute(&presence.id).await,
            Err(DomainError::Unauthorized)
        ));

        let _ = std::fs::remove_dir_all(dir);
    });
}
