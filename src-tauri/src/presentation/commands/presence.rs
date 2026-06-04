use tauri::State;

use crate::application::dto::presence_dto::PresenceDto;
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Set (create or update) the presence type for a day of a profile.
#[tauri::command]
pub async fn set_presence(
    profile_id: String,
    day: i64,
    r#type: String,
    state: State<'_, AppState>,
) -> Result<PresenceDto, AppError> {
    Ok(state
        .set_presence
        .execute(&profile_id, day, &r#type)
        .await?)
}

/// List every presence recorded for a profile.
#[tauri::command]
pub async fn list_presences(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PresenceDto>, AppError> {
    Ok(state.list_presences.execute(&profile_id).await?)
}

/// Delete a single presence by id.
#[tauri::command]
pub async fn delete_presence(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.delete_presence.execute(&id).await?;
    Ok(())
}
