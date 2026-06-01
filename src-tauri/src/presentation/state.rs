use crate::application::use_cases::account_exists::AccountExistsUseCase;
use crate::application::use_cases::check_session::CheckSessionUseCase;
use crate::application::use_cases::login::LoginUseCase;
use crate::application::use_cases::logout::LogoutUseCase;
use crate::application::use_cases::register_account::RegisterAccountUseCase;

/// Application state injected via Tauri `.manage()` and read by the commands.
pub struct AppState {
    pub register_account: RegisterAccountUseCase,
    pub login: LoginUseCase,
    pub logout: LogoutUseCase,
    pub check_session: CheckSessionUseCase,
    pub account_exists: AccountExistsUseCase,
    /// Keeps the keystore database alive for the lifetime of the app.
    #[allow(dead_code)]
    pub keystore_db: libsql::Database,
}
