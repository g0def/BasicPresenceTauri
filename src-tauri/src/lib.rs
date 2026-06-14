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
use crate::application::use_cases::create_commute::CreateCommuteUseCase;
use crate::application::use_cases::create_profile::CreateProfileUseCase;
use crate::application::use_cases::create_task_preset::CreateTaskPresetUseCase;
use crate::application::use_cases::delete_commute::DeleteCommuteUseCase;
use crate::application::use_cases::delete_presence::DeletePresenceUseCase;
use crate::application::use_cases::delete_profile::DeleteProfileUseCase;
use crate::application::use_cases::delete_task_preset::DeleteTaskPresetUseCase;
use crate::application::use_cases::get_day_note::GetDayNoteUseCase;
use crate::application::use_cases::get_presence_trips::GetPresenceTripsUseCase;
use crate::application::use_cases::get_work_entries::GetWorkEntriesUseCase;
use crate::application::use_cases::get_work_schedule::GetWorkScheduleUseCase;
use crate::application::use_cases::import_presences::ImportPresencesUseCase;
use crate::application::use_cases::list_commutes::ListCommutesUseCase;
use crate::application::use_cases::list_emission_factors::ListEmissionFactorsUseCase;
use crate::application::use_cases::list_presences::ListPresencesUseCase;
use crate::application::use_cases::list_profiles::ListProfilesUseCase;
use crate::application::use_cases::list_task_presets::ListTaskPresetsUseCase;
use crate::application::use_cases::login::LoginUseCase;
use crate::application::use_cases::logout::LogoutUseCase;
use crate::application::use_cases::register_account::RegisterAccountUseCase;
use crate::application::use_cases::require_session::RequireSessionUseCase;
use crate::application::use_cases::set_active_profile::SetActiveProfileUseCase;
use crate::application::use_cases::set_day_note::SetDayNoteUseCase;
use crate::application::use_cases::set_presence::SetPresenceUseCase;
use crate::application::use_cases::set_work_entries::SetWorkEntriesUseCase;
use crate::application::use_cases::set_work_schedule::SetWorkScheduleUseCase;
use crate::application::use_cases::update_commute::UpdateCommuteUseCase;
use crate::application::use_cases::update_profile::UpdateProfileUseCase;
use crate::application::use_cases::update_task_preset::UpdateTaskPresetUseCase;
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::co2_settings_repository::Co2SettingsRepository;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::markdown::MarkdownRenderer;
use crate::domain::services::password_hasher::PasswordHasher;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::token_generator::TokenGenerator;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::clock::SystemClock;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::crypto::argon2_hasher::Argon2PasswordHasher;
use crate::infrastructure::crypto::key_service::Argon2KeyService;
use crate::infrastructure::crypto::token_generator::RandomTokenGenerator;
use crate::infrastructure::markdown::comrak_renderer::ComrakMarkdownRenderer;
use crate::infrastructure::persistence::account_repository::LibsqlAccountRepository;
use crate::infrastructure::persistence::co2_settings_repository::LibsqlCo2SettingsRepository;
use crate::infrastructure::persistence::commute_repository::LibsqlCommuteRepository;
use crate::infrastructure::persistence::db::connect;
use crate::infrastructure::persistence::emission_factor_repository::LibsqlEmissionFactorRepository;
use crate::infrastructure::persistence::keystore_bootstrap::open_or_migrate_keystore;
use crate::infrastructure::persistence::migrations::{self, KEYSTORE_MIGRATIONS};
use crate::infrastructure::persistence::presence_repository::LibsqlPresenceRepository;
use crate::infrastructure::persistence::profile_repository::LibsqlProfileRepository;
use crate::infrastructure::persistence::task_preset_repository::LibsqlTaskPresetRepository;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;
use crate::infrastructure::persistence::work_entry_repository::LibsqlWorkEntryRepository;
use crate::infrastructure::session::in_memory_session_store::InMemorySessionStore;
use crate::presentation::commands::{auth, commute, presence, profile, work_hours};
use crate::presentation::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .on_window_event(|window, event| {
            // On a clean window close, lock the vault so its integrity baseline
            // is refreshed (otherwise the next launch sees an "unclean" state).
            if matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) {
                window.state::<AppState>().vault.close();
            }
        })
        .setup(|app| {
            // Resolve (and create) the per-app data directory for the DB files.
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("cannot create app data dir: {e}"))?;

            // Per-device key that seals the keystore at rest (from the OS keychain).
            let device_key = crate::infrastructure::crypto::device_key::resolve_device_key()
                .map_err(|e| format!("cannot access OS secure storage: {e:?}"))?;

            let config = AppConfig::new(&data_dir);
            let state = tauri::async_runtime::block_on(build_state(config, device_key.as_slice()))
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
            profile::set_active_profile,
            presence::set_presence,
            presence::list_presences,
            presence::delete_presence,
            presence::import_presences,
            commute::list_emission_factors,
            commute::create_commute,
            commute::list_commutes,
            commute::update_commute,
            commute::delete_commute,
            commute::get_presence_trips,
            work_hours::create_task_preset,
            work_hours::list_task_presets,
            work_hours::update_task_preset,
            work_hours::delete_task_preset,
            work_hours::get_work_entries,
            work_hours::set_work_entries,
            work_hours::get_work_schedule,
            work_hours::set_work_schedule,
            work_hours::get_day_note,
            work_hours::set_day_note
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Composition root: build the infrastructure implementations, inject them into
/// the use cases, and assemble the managed `AppState`.
async fn build_state(config: AppConfig, device_key: &[u8]) -> Result<AppState, DomainError> {
    // Keystore: auth credentials + wrapped key material. Sealed at rest behind
    // the per-device key (creates a fresh encrypted store, or migrates a legacy
    // plaintext one). Every field inside is already cryptographically protected;
    // this layer additionally defeats offline brute-force of a stolen file.
    let keystore_db = open_or_migrate_keystore(&config.keystore_path, device_key).await?;
    let keystore_conn = connect(&keystore_db).await?;
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
    let vault_impl = Arc::new(LibsqlVaultManager::new(
        config.vault_path.clone(),
        config.integrity,
    ));
    let vault: Arc<dyn VaultManager> = vault_impl.clone();
    let profiles: Arc<dyn ProfileRepository> =
        Arc::new(LibsqlProfileRepository::new(vault_impl.clone()));
    let presences: Arc<dyn PresenceRepository> =
        Arc::new(LibsqlPresenceRepository::new(vault_impl.clone()));
    let factors: Arc<dyn EmissionFactorRepository> =
        Arc::new(LibsqlEmissionFactorRepository::new(vault_impl.clone()));
    let commutes: Arc<dyn CommuteRepository> =
        Arc::new(LibsqlCommuteRepository::new(vault_impl.clone()));
    let co2_settings: Arc<dyn Co2SettingsRepository> =
        Arc::new(LibsqlCo2SettingsRepository::new(vault_impl.clone()));
    let task_presets: Arc<dyn TaskPresetRepository> =
        Arc::new(LibsqlTaskPresetRepository::new(vault_impl.clone()));
    let work_entries: Arc<dyn WorkEntryRepository> =
        Arc::new(LibsqlWorkEntryRepository::new(vault_impl.clone()));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let markdown: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());

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
        require_session: RequireSessionUseCase::new(sessions.clone(), vault.clone(), clock.clone()),
        account_exists: AccountExistsUseCase::new(accounts.clone()),
        create_profile: CreateProfileUseCase::new(profiles.clone(), clock.clone()),
        list_profiles: ListProfilesUseCase::new(profiles.clone()),
        update_profile: UpdateProfileUseCase::new(profiles.clone(), clock.clone()),
        delete_profile: DeleteProfileUseCase::new(profiles.clone()),
        set_active_profile: SetActiveProfileUseCase::new(profiles.clone()),
        set_presence: SetPresenceUseCase::new(
            presences.clone(),
            factors.clone(),
            co2_settings.clone(),
            clock.clone(),
        ),
        list_presences: ListPresencesUseCase::new(presences.clone()),
        delete_presence: DeletePresenceUseCase::new(presences.clone()),
        import_presences: ImportPresencesUseCase::new(presences.clone(), clock.clone()),
        list_emission_factors: ListEmissionFactorsUseCase::new(
            factors.clone(),
            co2_settings.clone(),
        ),
        create_commute: CreateCommuteUseCase::new(commutes.clone(), clock.clone()),
        list_commutes: ListCommutesUseCase::new(
            commutes.clone(),
            factors.clone(),
            co2_settings.clone(),
        ),
        update_commute: UpdateCommuteUseCase::new(commutes.clone(), clock.clone()),
        delete_commute: DeleteCommuteUseCase::new(commutes.clone()),
        get_presence_trips: GetPresenceTripsUseCase::new(presences.clone()),
        create_task_preset: CreateTaskPresetUseCase::new(task_presets.clone(), clock.clone()),
        list_task_presets: ListTaskPresetsUseCase::new(task_presets.clone()),
        update_task_preset: UpdateTaskPresetUseCase::new(task_presets.clone(), clock.clone()),
        delete_task_preset: DeleteTaskPresetUseCase::new(task_presets.clone()),
        get_work_entries: GetWorkEntriesUseCase::new(work_entries.clone()),
        set_work_entries: SetWorkEntriesUseCase::new(work_entries.clone(), presences.clone()),
        get_work_schedule: GetWorkScheduleUseCase::new(work_entries.clone()),
        set_work_schedule: SetWorkScheduleUseCase::new(work_entries.clone(), presences.clone()),
        get_day_note: GetDayNoteUseCase::new(presences.clone(), markdown.clone()),
        set_day_note: SetDayNoteUseCase::new(presences.clone(), markdown.clone(), clock.clone()),
        vault: vault.clone(),
        keystore_db,
    })
}
