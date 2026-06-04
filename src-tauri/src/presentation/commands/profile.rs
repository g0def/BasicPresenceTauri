use tauri::State;

use crate::application::dto::profile_dto::{ProfileDto, ProfilesDto};
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Create a presence profile (requires an unlocked vault).
#[tauri::command]
pub async fn create_profile(
    first_name: String,
    last_name: String,
    enterprise: String,
    poste: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProfileDto, AppError> {
    Ok(state
        .create_profile
        .execute(&first_name, &last_name, &enterprise, poste.as_deref())
        .await?)
}

/// List every profile plus the active profile id.
#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> Result<ProfilesDto, AppError> {
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
    Ok(state
        .update_profile
        .execute(&id, &first_name, &last_name, &enterprise, poste.as_deref())
        .await?)
}

/// Delete a profile (reassigns the active selection if needed).
#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.delete_profile.execute(&id).await?;
    Ok(())
}

/// Mark a profile as the active one.
#[tauri::command]
pub async fn set_active_profile(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.set_active_profile.execute(&id).await?;
    Ok(())
}
