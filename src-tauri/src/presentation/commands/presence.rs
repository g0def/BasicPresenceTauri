use tauri::State;

use crate::application::dto::presence_dto::PresenceDto;
use crate::application::dto::trip_dto::TripInputDto;
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

// Every command below is session-gated: `require_session` enforces the absolute
// expiry backend-side (revoking the session and locking the vault when it has
// passed), so a frontend that stops calling `check_session` gains nothing.

/// Set (create or update) the presence type for a day of a profile. For
/// office/remote days, `trips` carries the commute legs whose footprint is
/// computed and snapshotted; it is absent/empty for other types.
#[tauri::command]
pub async fn set_presence(
    profile_id: String,
    day: i64,
    r#type: String,
    trips: Option<Vec<TripInputDto>>,
    state: State<'_, AppState>,
) -> Result<PresenceDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .set_presence
        .execute(&profile_id, day, &r#type, trips.unwrap_or_default())
        .await?)
}

/// List every presence recorded for a profile.
#[tauri::command]
pub async fn list_presences(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PresenceDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.list_presences.execute(&profile_id).await?)
}

/// Delete a single presence by id.
#[tauri::command]
pub async fn delete_presence(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.require_session.execute()?;
    state.delete_presence.execute(&id).await?;
    Ok(())
}
