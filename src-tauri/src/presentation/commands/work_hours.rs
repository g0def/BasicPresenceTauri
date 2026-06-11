use tauri::State;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::application::dto::work_entry_dto::{
    WorkDayDto, WorkDayScheduleDto, WorkEntryDto, WorkEntryInputDto,
};
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

// Every command below is session-gated: `require_session` enforces the absolute
// expiry backend-side (revoking the session and locking the vault when it has
// passed), so a frontend that stops calling `check_session` gains nothing.

/// Create a saved task preset for a profile.
#[tauri::command]
pub async fn create_task_preset(
    profile_id: String,
    title: String,
    description: Option<String>,
    default_minutes: i64,
    color: String,
    state: State<'_, AppState>,
) -> Result<TaskPresetDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .create_task_preset
        .execute(&profile_id, &title, description, default_minutes, &color)
        .await?)
}

/// List a profile's saved task presets.
#[tauri::command]
pub async fn list_task_presets(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TaskPresetDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.list_task_presets.execute(&profile_id).await?)
}

/// Update a saved task preset (past day entries keep their snapshot).
#[tauri::command]
pub async fn update_task_preset(
    id: String,
    title: String,
    description: Option<String>,
    default_minutes: i64,
    color: String,
    state: State<'_, AppState>,
) -> Result<TaskPresetDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .update_task_preset
        .execute(&id, &title, description, default_minutes, &color)
        .await?)
}

/// Delete a saved task preset by id.
#[tauri::command]
pub async fn delete_task_preset(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.require_session.execute()?;
    state.delete_task_preset.execute(&id).await?;
    Ok(())
}

/// Fetch the work entries of a presence day (to fill the hours page).
#[tauri::command]
pub async fn get_work_entries(
    presence_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<WorkEntryDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.get_work_entries.execute(&presence_id).await?)
}

/// Atomically replace the work entries of an office/remote presence day.
/// Returns the canonical entries plus the schedule (its end recomputed).
#[tauri::command]
pub async fn set_work_entries(
    presence_id: String,
    entries: Vec<WorkEntryInputDto>,
    state: State<'_, AppState>,
) -> Result<WorkDayDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .set_work_entries
        .execute(&presence_id, entries)
        .await?)
}

/// Fetch the start time of a presence day (None when never set).
#[tauri::command]
pub async fn get_work_schedule(
    presence_id: String,
    state: State<'_, AppState>,
) -> Result<Option<WorkDayScheduleDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.get_work_schedule.execute(&presence_id).await?)
}

/// Set the start time of an office/remote presence day (the end is derived).
#[tauri::command]
pub async fn set_work_schedule(
    presence_id: String,
    start_minutes: i64,
    state: State<'_, AppState>,
) -> Result<WorkDayScheduleDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .set_work_schedule
        .execute(&presence_id, start_minutes)
        .await?)
}
