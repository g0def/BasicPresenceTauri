use tauri::State;
use zeroize::Zeroizing;

use crate::application::dto::login_result_dto::LoginResultDto;
use crate::application::dto::session_status_dto::SessionStatusDto;
use crate::application::dto::user_dto::UserDto;
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Whether an owner account already exists (decides Register vs Login screen).
#[tauri::command]
pub async fn account_exists(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state.account_exists.execute().await?)
}

/// Create the device-owner account (first run).
#[tauri::command]
pub async fn register(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<UserDto, AppError> {
    // Wrap the secret as soon as it crosses the IPC boundary so this copy is
    // wiped on drop (serde's transient deserialization buffers remain a known
    // residual limit of the Tauri IPC).
    let password = Zeroizing::new(password);
    Ok(state
        .register_account
        .execute(&username, password.as_str())
        .await?)
}

/// Authenticate, unlock the vault, and open a 15-minute session.
#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<LoginResultDto, AppError> {
    // Same zeroization-at-the-boundary as `register`.
    let password = Zeroizing::new(password);
    Ok(state.login.execute(&username, password.as_str()).await?)
}

/// Report whether the given session token is still valid.
#[tauri::command]
pub async fn check_session(
    token: String,
    state: State<'_, AppState>,
) -> Result<SessionStatusDto, AppError> {
    Ok(state.check_session.execute(&token)?)
}

/// Revoke the session and lock the vault.
#[tauri::command]
pub async fn logout(token: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.logout.execute(&token)?;
    Ok(())
}
