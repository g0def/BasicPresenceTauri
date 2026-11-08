//! Point de composition : construit les implémentations d'infrastructure,
//! les injecte dans les use cases et expose l'ensemble via [`Services`].

use std::path::Path;
use std::sync::Arc;

use crate::application::services::credentials::CredentialGate;
use crate::application::use_cases::account_exists::AccountExistsUseCase;
use crate::application::use_cases::change_password::ChangePasswordUseCase;
use crate::application::use_cases::check_session::CheckSessionUseCase;
use crate::application::use_cases::copy_day::CopyDayUseCase;
use crate::application::use_cases::create_commute::CreateCommuteUseCase;
use crate::application::use_cases::create_profile::CreateProfileUseCase;
use crate::application::use_cases::create_task_preset::CreateTaskPresetUseCase;
use crate::application::use_cases::delete_commute::DeleteCommuteUseCase;
use crate::application::use_cases::delete_presence::DeletePresenceUseCase;
use crate::application::use_cases::delete_profile::DeleteProfileUseCase;
use crate::application::use_cases::delete_task_preset::DeleteTaskPresetUseCase;
use crate::application::use_cases::export_profile_bundle::ExportProfileBundleUseCase;
use crate::application::use_cases::export_profile_data::ExportProfileDataUseCase;
use crate::application::use_cases::get_day_note::GetDayNoteUseCase;
use crate::application::use_cases::get_licence::GetLicenceUseCase;
use crate::application::use_cases::get_presence_trips::GetPresenceTripsUseCase;
use crate::application::use_cases::get_profile_settings::GetProfileSettingsUseCase;
use crate::application::use_cases::get_work_entries::GetWorkEntriesUseCase;
use crate::application::use_cases::get_work_schedule::GetWorkScheduleUseCase;
use crate::application::use_cases::import_profile_bundle::ImportProfileBundleUseCase;
use crate::application::use_cases::inspect_profile_bundle::InspectProfileBundleUseCase;
use crate::application::use_cases::list_co2_referential::ListCo2ReferentialUseCase;
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
use crate::application::use_cases::set_profile_settings::SetProfileSettingsUseCase;
use crate::application::use_cases::set_work_entries::SetWorkEntriesUseCase;
use crate::application::use_cases::set_work_schedule::SetWorkScheduleUseCase;
use crate::application::use_cases::update_commute::UpdateCommuteUseCase;
use crate::application::use_cases::update_profile::UpdateProfileUseCase;
use crate::application::use_cases::update_task_preset::UpdateTaskPresetUseCase;
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::security_log::SecurityLog;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::markdown::MarkdownRenderer;
use crate::domain::services::password_hasher::PasswordHasher;
use crate::domain::services::profile_bundle::ProfileBundleCodec;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::spreadsheet_exporter::SpreadsheetExporter;
use crate::domain::services::token_generator::TokenGenerator;
use crate::domain::services::unit_of_work::UnitOfWork;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::clock::SystemClock;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::crypto::argon2_hasher::Argon2PasswordHasher;
use crate::infrastructure::crypto::key_service::Argon2KeyService;
use crate::infrastructure::crypto::token_generator::RandomTokenGenerator;
use crate::infrastructure::export::ods_writer::OdsSpreadsheetExporter;
use crate::infrastructure::markdown::comrak_renderer::ComrakMarkdownRenderer;
use crate::infrastructure::persistence::account_repository::LibsqlAccountRepository;
use crate::infrastructure::persistence::commute_repository::LibsqlCommuteRepository;
use crate::infrastructure::persistence::db::connect;
use crate::infrastructure::persistence::emission_factor_repository::LibsqlEmissionFactorRepository;
use crate::infrastructure::persistence::integrity_store::KeystoreIntegrityStore;
use crate::infrastructure::persistence::keystore_bootstrap::open_or_migrate_keystore;
use crate::infrastructure::persistence::migrations::{self, KEYSTORE_MIGRATIONS};
use crate::infrastructure::persistence::presence_repository::LibsqlPresenceRepository;
use crate::infrastructure::persistence::profile_repository::LibsqlProfileRepository;
use crate::infrastructure::persistence::profile_settings_repository::LibsqlProfileSettingsRepository;
use crate::infrastructure::persistence::security_log_repository::LibsqlSecurityLog;
use crate::infrastructure::persistence::task_preset_repository::LibsqlTaskPresetRepository;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;
use crate::infrastructure::persistence::work_entry_repository::LibsqlWorkEntryRepository;
use crate::infrastructure::session::in_memory_session_store::InMemorySessionStore;
use crate::infrastructure::transfer::json_bundle_codec::JsonProfileBundleCodec;

/// Tous les use cases de l'application, prêts à être appelés par l'UI.
pub struct Services {
    pub register_account: RegisterAccountUseCase,
    pub login: LoginUseCase,
    pub logout: LogoutUseCase,
    pub change_password: ChangePasswordUseCase,
    pub check_session: CheckSessionUseCase,
    /// Garde côté backend : tout use case qui touche au vault est précédé
    /// de cet appel, pour que l'expiration absolue de session tienne même si
    /// l'UI ne se manifeste jamais.
    pub require_session: RequireSessionUseCase,
    pub account_exists: AccountExistsUseCase,
    pub create_profile: CreateProfileUseCase,
    pub list_profiles: ListProfilesUseCase,
    pub update_profile: UpdateProfileUseCase,
    pub delete_profile: DeleteProfileUseCase,
    pub set_active_profile: SetActiveProfileUseCase,
    pub set_presence: SetPresenceUseCase,
    pub list_presences: ListPresencesUseCase,
    pub delete_presence: DeletePresenceUseCase,
    /// Replays a whole day (type, trips, start time, tasks) onto another day.
    pub copy_day: CopyDayUseCase,
    pub list_emission_factors: ListEmissionFactorsUseCase,
    pub list_co2_referential: ListCo2ReferentialUseCase,
    pub create_commute: CreateCommuteUseCase,
    pub list_commutes: ListCommutesUseCase,
    pub update_commute: UpdateCommuteUseCase,
    pub delete_commute: DeleteCommuteUseCase,
    pub get_presence_trips: GetPresenceTripsUseCase,
    pub get_profile_settings: GetProfileSettingsUseCase,
    pub set_profile_settings: SetProfileSettingsUseCase,
    pub create_task_preset: CreateTaskPresetUseCase,
    pub list_task_presets: ListTaskPresetsUseCase,
    pub update_task_preset: UpdateTaskPresetUseCase,
    pub delete_task_preset: DeleteTaskPresetUseCase,
    pub get_work_entries: GetWorkEntriesUseCase,
    pub set_work_entries: SetWorkEntriesUseCase,
    pub get_work_schedule: GetWorkScheduleUseCase,
    pub set_work_schedule: SetWorkScheduleUseCase,
    pub get_day_note: GetDayNoteUseCase,
    pub set_day_note: SetDayNoteUseCase,
    pub get_licence: GetLicenceUseCase,
    pub export_profile_data: ExportProfileDataUseCase,
    pub export_profile_bundle: ExportProfileBundleUseCase,
    pub inspect_profile_bundle: InspectProfileBundleUseCase,
    pub import_profile_bundle: ImportProfileBundleUseCase,
    /// Handle du vault, pour le verrouiller (et rafraîchir la référence
    /// d'intégrité) à la fermeture propre de l'application.
    pub vault: Arc<dyn VaultManager>,
    /// Store de sessions, pour révoquer toutes les sessions à la sortie.
    pub sessions: Arc<dyn SessionStore>,
    /// Garde la base keystore ouverte pendant toute la vie de l'application.
    /// (Lu uniquement par les tests d'intégration.)
    #[allow(dead_code)]
    pub(crate) keystore_db: libsql::Database,
}

impl Services {
    /// Révoque toutes les sessions et verrouille le vault. À appeler à la
    /// fermeture de la fenêtre et à la sortie de l'application.
    pub async fn lock(&self) {
        self.sessions.clear();
        self.vault.close().await;
    }
}

/// Prépare le répertoire de données (créé en 0700 sous Unix), résout la clé
/// de l'appareil dans le trousseau OS, puis construit les services.
///
/// `data_dir` doit être le même répertoire que celui de la version Tauri
/// (identifiant `com.godef.basic-presence`) pour retrouver le vault existant.
pub async fn open_services(data_dir: &Path) -> Result<Services, DomainError> {
    crate::infrastructure::persistence::data_dir::prepare_private_data_dir(data_dir)
        .map_err(|e| DomainError::Storage(format!("cannot prepare app data dir: {e}")))?;

    let config = AppConfig::new(data_dir);
    // Per-device key that seals the keystore at rest (from the OS keychain).
    // Only minted when no sealed keystore exists yet.
    let allow_new_device_key =
        !crate::infrastructure::persistence::keystore_bootstrap::is_sealed(&config.keystore_path);
    let device_key =
        crate::infrastructure::crypto::device_key::resolve_device_key(allow_new_device_key)?;

    build_services(config, device_key.as_slice()).await
}

/// Composition root: build the infrastructure implementations, inject them into
/// the use cases, and assemble the [`Services`].
pub async fn build_services(config: AppConfig, device_key: &[u8]) -> Result<Services, DomainError> {
    // Keystore: auth credentials + wrapped key material. Sealed at rest behind
    // the per-device key (creates a fresh encrypted store, or migrates a legacy
    // plaintext one). Every field inside is already cryptographically protected;
    // this layer additionally defeats offline brute-force of a stolen file.
    let keystore_db = open_or_migrate_keystore(&config.keystore_path, device_key).await?;
    let keystore_conn = connect(&keystore_db).await?;
    migrations::run(&keystore_conn, KEYSTORE_MIGRATIONS).await?;
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    // Infrastructure implementations (behind domain ports).
    let accounts: Arc<dyn AccountRepository> =
        Arc::new(LibsqlAccountRepository::new(keystore_conn.clone()));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::new(config.argon2)?);
    let keys: Arc<dyn KeyService> = Arc::new(Argon2KeyService);
    let credential_gate: CredentialGate = Arc::new(tokio::sync::Mutex::new(()));
    let security_log: Arc<dyn SecurityLog> =
        Arc::new(LibsqlSecurityLog::new(keystore_conn.clone()));
    let tokens: Arc<dyn TokenGenerator> = Arc::new(RandomTokenGenerator);
    let sessions: Arc<dyn SessionStore> = Arc::new(InMemorySessionStore::new());
    // Keep a concrete handle so the profile repository can read the live vault
    // connection, while the auth use cases depend on the `VaultManager` port.
    let vault_impl = Arc::new(LibsqlVaultManager::new(
        config.vault_path.clone(),
        config.integrity,
        KeystoreIntegrityStore::new(keystore_conn),
        clock.clone(),
    ));
    let vault: Arc<dyn VaultManager> = vault_impl.clone();
    let uow: Arc<dyn UnitOfWork> = vault_impl.clone();
    let profiles: Arc<dyn ProfileRepository> =
        Arc::new(LibsqlProfileRepository::new(vault_impl.clone()));
    let presences: Arc<dyn PresenceRepository> =
        Arc::new(LibsqlPresenceRepository::new(vault_impl.clone()));
    let factors: Arc<dyn EmissionFactorRepository> =
        Arc::new(LibsqlEmissionFactorRepository::new(vault_impl.clone()));
    let commutes: Arc<dyn CommuteRepository> =
        Arc::new(LibsqlCommuteRepository::new(vault_impl.clone()));
    let profile_settings: Arc<dyn ProfileSettingsRepository> =
        Arc::new(LibsqlProfileSettingsRepository::new(vault_impl.clone()));
    let task_presets: Arc<dyn TaskPresetRepository> =
        Arc::new(LibsqlTaskPresetRepository::new(vault_impl.clone()));
    let work_entries: Arc<dyn WorkEntryRepository> =
        Arc::new(LibsqlWorkEntryRepository::new(vault_impl.clone()));
    let markdown: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());
    // App licence: static text embedded at compile time. The path is relative to
    // this file (crates/core/src/services.rs), so `../../../LICENCE` is the repo-root file —
    // the single source of truth, shown in-app rendered to sanitized HTML.
    const LICENCE_MD: &str = include_str!("../../../LICENCE");
    let exporter: Arc<dyn SpreadsheetExporter> = Arc::new(OdsSpreadsheetExporter::new());
    let bundle_codec: Arc<dyn ProfileBundleCodec> = Arc::new(JsonProfileBundleCodec::new());

    Ok(Services {
        register_account: RegisterAccountUseCase::new(
            accounts.clone(),
            hasher.clone(),
            keys.clone(),
            clock.clone(),
            config.auth.clone(),
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
            credential_gate.clone(),
            security_log.clone(),
        ),
        change_password: ChangePasswordUseCase::new(
            accounts.clone(),
            hasher.clone(),
            keys.clone(),
            clock.clone(),
            config.auth.clone(),
            credential_gate.clone(),
            security_log.clone(),
        ),
        logout: LogoutUseCase::new(sessions.clone(), vault.clone()),
        check_session: CheckSessionUseCase::new(sessions.clone(), vault.clone(), clock.clone()),
        require_session: RequireSessionUseCase::new(sessions.clone(), vault.clone(), clock.clone()),
        account_exists: AccountExistsUseCase::new(accounts.clone()),
        create_profile: CreateProfileUseCase::new(profiles.clone(), clock.clone(), uow.clone()),
        list_profiles: ListProfilesUseCase::new(profiles.clone()),
        update_profile: UpdateProfileUseCase::new(profiles.clone(), clock.clone()),
        delete_profile: DeleteProfileUseCase::new(profiles.clone()),
        set_active_profile: SetActiveProfileUseCase::new(profiles.clone()),
        set_presence: SetPresenceUseCase::new(
            presences.clone(),
            factors.clone(),
            profile_settings.clone(),
            work_entries.clone(),
            clock.clone(),
            uow.clone(),
        ),
        list_presences: ListPresencesUseCase::new(presences.clone()),
        delete_presence: DeletePresenceUseCase::new(presences.clone()),
        copy_day: CopyDayUseCase::new(
            presences.clone(),
            work_entries.clone(),
            SetPresenceUseCase::new(
                presences.clone(),
                factors.clone(),
                profile_settings.clone(),
                work_entries.clone(),
                clock.clone(),
                uow.clone(),
            ),
            SetWorkScheduleUseCase::new(work_entries.clone(), presences.clone()),
            SetWorkEntriesUseCase::new(work_entries.clone(), presences.clone(), uow.clone()),
            uow.clone(),
        ),
        list_emission_factors: ListEmissionFactorsUseCase::new(factors.clone()),
        list_co2_referential: ListCo2ReferentialUseCase::new(factors.clone()),
        create_commute: CreateCommuteUseCase::new(commutes.clone(), clock.clone()),
        list_commutes: ListCommutesUseCase::new(
            commutes.clone(),
            factors.clone(),
            profile_settings.clone(),
        ),
        update_commute: UpdateCommuteUseCase::new(commutes.clone(), clock.clone()),
        delete_commute: DeleteCommuteUseCase::new(commutes.clone()),
        get_presence_trips: GetPresenceTripsUseCase::new(presences.clone()),
        get_profile_settings: GetProfileSettingsUseCase::new(profile_settings.clone()),
        set_profile_settings: SetProfileSettingsUseCase::new(profile_settings.clone()),
        create_task_preset: CreateTaskPresetUseCase::new(task_presets.clone(), clock.clone()),
        list_task_presets: ListTaskPresetsUseCase::new(task_presets.clone()),
        update_task_preset: UpdateTaskPresetUseCase::new(task_presets.clone(), clock.clone()),
        delete_task_preset: DeleteTaskPresetUseCase::new(task_presets.clone()),
        get_work_entries: GetWorkEntriesUseCase::new(work_entries.clone()),
        set_work_entries: SetWorkEntriesUseCase::new(
            work_entries.clone(),
            presences.clone(),
            uow.clone(),
        ),
        get_work_schedule: GetWorkScheduleUseCase::new(work_entries.clone()),
        set_work_schedule: SetWorkScheduleUseCase::new(work_entries.clone(), presences.clone()),
        get_day_note: GetDayNoteUseCase::new(presences.clone(), markdown.clone()),
        set_day_note: SetDayNoteUseCase::new(presences.clone(), markdown.clone(), clock.clone()),
        get_licence: GetLicenceUseCase::new(LICENCE_MD, markdown.clone()),
        export_profile_data: ExportProfileDataUseCase::new(
            presences.clone(),
            work_entries.clone(),
            exporter.clone(),
        ),
        export_profile_bundle: ExportProfileBundleUseCase::new(
            profiles.clone(),
            presences.clone(),
            work_entries.clone(),
            task_presets.clone(),
            commutes.clone(),
            profile_settings.clone(),
            bundle_codec.clone(),
            clock.clone(),
        ),
        inspect_profile_bundle: InspectProfileBundleUseCase::new(
            bundle_codec.clone(),
            factors.clone(),
        ),
        import_profile_bundle: ImportProfileBundleUseCase::new(
            bundle_codec.clone(),
            profiles.clone(),
            presences.clone(),
            work_entries.clone(),
            task_presets.clone(),
            commutes.clone(),
            profile_settings.clone(),
            clock.clone(),
            uow.clone(),
        ),
        vault: vault.clone(),
        sessions: sessions.clone(),
        keystore_db,
    })
}
