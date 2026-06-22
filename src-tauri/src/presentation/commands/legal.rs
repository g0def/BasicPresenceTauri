use tauri::State;

use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Return the app licence rendered as sanitized, display-safe HTML.
///
/// Session-gated like every other command (the page is only reachable once
/// authenticated); the licence itself is static, trusted, embedded content.
#[tauri::command]
pub async fn get_licence(state: State<'_, AppState>) -> Result<String, AppError> {
    state.require_session.execute()?;
    Ok(state.get_licence.execute())
}
