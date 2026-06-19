use tauri::State;

use crate::application::dto::profile_bundle_dto::{
    BundleExportOptionsDto, BundleExportSummaryDto, BundleImportSelectionDto,
    BundleImportSummaryDto, BundleImportTargetDto, BundleManifestDto,
};
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

/// Export a whole profile to a shareable, versioned JSON bundle at `path`. The
/// frontend picks `path` via the native save dialog and toggles categories with
/// `options`. Session-gated like every vault-touching command.
#[tauri::command]
pub async fn export_profile_bundle(
    profile_id: String,
    options: BundleExportOptionsDto,
    path: String,
    state: State<'_, AppState>,
) -> Result<BundleExportSummaryDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .export_profile_bundle
        .execute(&profile_id, options, &path)
        .await?)
}

/// Parse a bundle file and report what it contains (per-category counts, profile
/// identity, compatibility, unknown transport modes) so the import dialog can
/// offer only the categories that are present. Reads, never writes.
#[tauri::command]
pub async fn inspect_profile_bundle(
    path: String,
    state: State<'_, AppState>,
) -> Result<BundleManifestDto, AppError> {
    state.require_session.execute()?;
    Ok(state.inspect_profile_bundle.execute(&path).await?)
}

/// Import the chosen categories from a bundle, either into a new profile or
/// merged into an existing one. `conflict_strategy` ("skip" | "replace") only
/// affects days that already exist when merging.
#[tauri::command]
pub async fn import_profile_bundle(
    path: String,
    selection: BundleImportSelectionDto,
    target: BundleImportTargetDto,
    conflict_strategy: String,
    state: State<'_, AppState>,
) -> Result<BundleImportSummaryDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .import_profile_bundle
        .execute(&path, selection, target, &conflict_strategy)
        .await?)
}
