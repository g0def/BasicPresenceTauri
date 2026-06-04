mod application;
mod domain;
mod infrastructure;
mod presentation;

#[cfg(test)]
mod integration_tests;

use std::sync::Arc;

use tauri::Manager;

use crate::application::use_cases::account_exists::AccountExistsUseCase;
use crate::application::use_cases::check_session::CheckSessionUseCase;
use crate::application::use_cases::create_profile::CreateProfileUseCase;
use crate::application::use_cases::delete_profile::DeleteProfileUseCase;
use crate::application::use_cases::list_profiles::ListProfilesUseCase;
use crate::application::use_cases::login::LoginUseCase;
use crate::application::use_cases::logout::LogoutUseCase;
use crate::application::use_cases::register_account::RegisterAccountUseCase;
use crate::application::use_cases::set_active_profile::SetActiveProfileUseCase;
use crate::application::use_cases::update_profile::UpdateProfileUseCase;
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::password_hasher::PasswordHasher;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::token_generator::TokenGenerator;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::clock::SystemClock;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::crypto::argon2_hasher::Argon2PasswordHasher;
use crate::infrastructure::crypto::key_service::Argon2KeyService;
use crate::infrastructure::crypto::token_generator::RandomTokenGenerator;
use crate::infrastructure::persistence::account_repository::LibsqlAccountRepository;
use crate::infrastructure::persistence::db::{connect, open_plain_db};
use crate::infrastructure::persistence::migrations::{self, KEYSTORE_MIGRATIONS};
use crate::infrastructure::persistence::profile_repository::LibsqlProfileRepository;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;
use crate::infrastructure::session::in_memory_session_store::InMemorySessionStore;
use crate::presentation::commands::{auth, profile};
use crate::presentation::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve (and create) the per-app data directory for the DB files.
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("cannot create app data dir: {e}"))?;

            let config = AppConfig::new(&data_dir);
            let state = tauri::async_runtime::block_on(build_state(config))
                .map_err(|e| format!("failed to initialize backend: {e:?}"))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::account_exists,
            auth::register,
            auth::login,
            auth::check_session,
            auth::logout,
            profile::create_profile,
            profile::list_profiles,
            profile::update_profile,
            profile::delete_profile,
            profile::set_active_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Composition root: build the infrastructure implementations, inject them into
/// the use cases, and assemble the managed `AppState`.
async fn build_state(config: AppConfig) -> Result<AppState, DomainError> {
    // Keystore (plaintext): auth credentials + wrapped key material.
    let keystore_db = open_plain_db(&config.keystore_path).await?;
    let keystore_conn = connect(&keystore_db)?;
    migrations::run(&keystore_conn, KEYSTORE_MIGRATIONS).await?;

    // Infrastructure implementations (behind domain ports).
    let accounts: Arc<dyn AccountRepository> =
        Arc::new(LibsqlAccountRepository::new(keystore_conn));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::new(
        config.argon2.m_cost,
        config.argon2.t_cost,
        config.argon2.p_cost,
    )?);
    let keys: Arc<dyn KeyService> = Arc::new(Argon2KeyService::new(
        config.argon2.m_cost,
        config.argon2.t_cost,
        config.argon2.p_cost,
    )?);
    let tokens: Arc<dyn TokenGenerator> = Arc::new(RandomTokenGenerator);
    let sessions: Arc<dyn SessionStore> = Arc::new(InMemorySessionStore::new());
    // Keep a concrete handle so the profile repository can read the live vault
    // connection, while the auth use cases depend on the `VaultManager` port.
    let vault_impl = Arc::new(LibsqlVaultManager::new(config.vault_path.clone()));
    let vault: Arc<dyn VaultManager> = vault_impl.clone();
    let profiles: Arc<dyn ProfileRepository> =
        Arc::new(LibsqlProfileRepository::new(vault_impl.clone()));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    Ok(AppState {
        register_account: RegisterAccountUseCase::new(
            accounts.clone(),
            hasher.clone(),
            keys.clone(),
            clock.clone(),
        ),
        login: LoginUseCase::new(
            accounts.clone(),
            hasher.clone(),
            keys.clone(),
            tokens.clone(),
            sessions.clone(),
            vault.clone(),
            clock.clone(),
            config.auth.clone(),
        ),
        logout: LogoutUseCase::new(sessions.clone(), vault.clone()),
        check_session: CheckSessionUseCase::new(sessions.clone(), vault.clone(), clock.clone()),
        account_exists: AccountExistsUseCase::new(accounts.clone()),
        create_profile: CreateProfileUseCase::new(profiles.clone(), clock.clone()),
        list_profiles: ListProfilesUseCase::new(profiles.clone()),
        update_profile: UpdateProfileUseCase::new(profiles.clone(), clock.clone()),
        delete_profile: DeleteProfileUseCase::new(profiles.clone()),
        set_active_profile: SetActiveProfileUseCase::new(profiles.clone()),
        keystore_db,
    })
}
