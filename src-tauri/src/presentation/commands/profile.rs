use tauri::State;

use crate::application::dto::profile_dto::{ProfileDto, ProfilesDto};
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

// Every command below is session-gated: `require_session` enforces the absolute
// expiry backend-side (revoking the session and locking the vault when it has
// passed), so a frontend that stops calling `check_session` gains nothing.

/// Create a presence profile (requires a valid session / unlocked vault).
#[tauri::command]
pub async fn create_profile(
    first_name: String,
    last_name: String,
    enterprise: String,
    poste: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProfileDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .create_profile
        .execute(&first_name, &last_name, &enterprise, poste.as_deref())
        .await?)
}

/// List every profile plus the active profile id.
#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> Result<ProfilesDto, AppError> {
    state.require_session.execute()?;
    Ok(state.list_profiles.execute().await?)
}

/// Update an existing profile.
#[tauri::command]
pub async fn update_profile(
    id: String,
    first_name: String,
    last_name: String,
    enterprise: String,
    poste: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProfileDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .update_profile
        .execute(&id, &first_name, &last_name, &enterprise, poste.as_deref())
        .await?)
}

/// Delete a profile (reassigns the active selection if needed).
#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.require_session.execute()?;
    state.delete_profile.execute(&id).await?;
    Ok(())
}

/// Mark a profile as the active one.
#[tauri::command]
pub async fn set_active_profile(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.require_session.execute()?;
    state.set_active_profile.execute(&id).await?;
    Ok(())
}
