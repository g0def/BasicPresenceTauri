use tauri::State;

use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

// Session-gated like every other vault-touching command.

/// Load a profile's settings (lazy defaults when never saved).
#[tauri::command]
pub async fn get_profile_settings(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<ProfileSettingsDto, AppError> {
    state.require_session.execute()?;
    Ok(state.get_profile_settings.execute(&profile_id).await?)
}

/// Persist a profile's full settings row, returning the canonical stored value.
#[tauri::command]
pub async fn set_profile_settings(
    profile_id: String,
    settings: ProfileSettingsDto,
    state: State<'_, AppState>,
) -> Result<ProfileSettingsDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .set_profile_settings
        .execute(&profile_id, settings)
        .await?)
}
