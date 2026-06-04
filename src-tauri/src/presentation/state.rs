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

/// Application state injected via Tauri `.manage()` and read by the commands.
pub struct AppState {
    pub register_account: RegisterAccountUseCase,
    pub login: LoginUseCase,
    pub logout: LogoutUseCase,
    pub check_session: CheckSessionUseCase,
    pub account_exists: AccountExistsUseCase,
    pub create_profile: CreateProfileUseCase,
    pub list_profiles: ListProfilesUseCase,
    pub update_profile: UpdateProfileUseCase,
    pub delete_profile: DeleteProfileUseCase,
    pub set_active_profile: SetActiveProfileUseCase,
    /// Keeps the keystore database alive for the lifetime of the app.
    #[allow(dead_code)]
    pub keystore_db: libsql::Database,
}
