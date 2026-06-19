use tauri::State;

use crate::application::dto::export_dto::{ExportLabelsDto, ExportOptionsDto, ExportSummaryDto};
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Export a profile's activity (presences + tasks + trips + notes) to a
/// multi-sheet OpenDocument (.ods) file at `path`. The frontend picks `path`
/// via the native save dialog, toggles sheets/columns with `options`, and
/// supplies the already-translated `labels`. Session-gated like every other
/// vault-touching command.
#[tauri::command]
pub async fn export_profile_data(
    profile_id: String,
    options: ExportOptionsDto,
    labels: ExportLabelsDto,
    path: String,
    state: State<'_, AppState>,
) -> Result<ExportSummaryDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .export_profile_data
        .execute(&profile_id, options, labels, &path)
        .await?)
}
