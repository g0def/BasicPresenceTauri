use tauri::{AppHandle, State};

use crate::application::dto::export_dto::{ExportLabelsDto, ExportOptionsDto, ExportSummaryDto};
use crate::presentation::commands::error::AppError;
use crate::presentation::file_dialogs::pick_save_path;
use crate::presentation::state::AppState;

/// Export a profile's activity (presences + tasks + trips + notes) to a
/// multi-sheet OpenDocument (.ods) file. The backend shows the native "save as"
/// dialog itself (`default_file_name` is only a suggestion) — the WebView never
/// supplies a path. Returns `None` when the user cancels the dialog.
#[tauri::command]
pub async fn export_profile_data(
    profile_id: String,
    options: ExportOptionsDto,
    labels: ExportLabelsDto,
    default_file_name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ExportSummaryDto>, AppError> {
    state.require_session.execute().await?;
    let Some(path) = pick_save_path(&app, &default_file_name, "OpenDocument", "ods").await? else {
        return Ok(None);
    };
    // The dialog may have stayed open past the session's end: check again.
    state.require_session.execute().await?;
    Ok(Some(
        state
            .export_profile_data
            .execute(&profile_id, options, labels, &path)
            .await?,
    ))
}
