use crate::application::use_cases::account_exists::AccountExistsUseCase;
use crate::application::use_cases::check_session::CheckSessionUseCase;
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
use crate::domain::services::vault::VaultManager;
use std::sync::Arc;

/// Application state injected via Tauri `.manage()` and read by the commands.
pub struct AppState {
    pub register_account: RegisterAccountUseCase,
    pub login: LoginUseCase,
    pub logout: LogoutUseCase,
    pub check_session: CheckSessionUseCase,
    /// Backend-side gate: every vault-touching command calls it first, so the
    /// absolute session expiry holds even if the frontend never checks in.
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
    /// Vault handle, exposed so a window-close hook can lock it and refresh the
    /// integrity baseline on a clean app exit.
    pub vault: Arc<dyn VaultManager>,
    /// Keeps the keystore database alive for the lifetime of the app.
    #[allow(dead_code)]
    pub keystore_db: libsql::Database,
}
