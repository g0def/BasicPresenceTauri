use tauri::{AppHandle, State};

use crate::application::dto::profile_bundle_dto::{
    BundleExportOptionsDto, BundleExportSummaryDto, BundleImportSelectionDto,
    BundleImportSummaryDto, BundleImportTargetDto, ConflictStrategy, InspectedBundleDto,
};
use crate::domain::error::DomainError;
use crate::presentation::commands::error::AppError;
use crate::presentation::file_dialogs::{pick_open_path, pick_save_path};
use crate::presentation::state::AppState;

/// Export a whole profile to a shareable, versioned JSON bundle. The backend
/// shows the native "save as" dialog (`default_file_name` is a suggestion
/// only); `None` when the user cancels. Session-gated like every
/// vault-touching command.
#[tauri::command]
pub async fn export_profile_bundle(
    profile_id: String,
    options: BundleExportOptionsDto,
    default_file_name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<BundleExportSummaryDto>, AppError> {
    state.require_session.execute().await?;
    let Some(path) = pick_save_path(&app, &default_file_name, "Profil", "json").await? else {
        return Ok(None);
    };
    state.require_session.execute().await?;
    Ok(Some(
        state
            .export_profile_bundle
            .execute(&profile_id, options, &path)
            .await?,
    ))
}

/// Let the user pick a bundle file (native dialog, backend side), parse it and
/// report what it contains so the import dialog can offer only the categories
/// that are present. The picked file is remembered under an opaque, single-use
/// handle for the import step. Reads, never writes. `None` when cancelled.
#[tauri::command]
pub async fn inspect_profile_bundle(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<InspectedBundleDto>, AppError> {
    state.require_session.execute().await?;
    let Some(path) = pick_open_path(&app, "Profil", "json").await? else {
        return Ok(None);
    };
    state.require_session.execute().await?;
    let manifest = state.inspect_profile_bundle.execute(&path).await?;
    let handle = state.pending_imports.insert(path)?;
    Ok(Some(InspectedBundleDto { handle, manifest }))
}

/// Import the chosen categories from the bundle inspected under `handle`,
/// either into a new profile or merged into an existing one.
/// `conflict_strategy` only affects days that already exist when merging.
#[tauri::command]
pub async fn import_profile_bundle(
    handle: String,
    selection: BundleImportSelectionDto,
    target: BundleImportTargetDto,
    conflict_strategy: ConflictStrategy,
    state: State<'_, AppState>,
) -> Result<BundleImportSummaryDto, AppError> {
    state.require_session.execute().await?;
    let path = state.pending_imports.take(&handle).ok_or_else(|| {
        DomainError::Validation("the selected file has expired, pick it again".into())
    })?;
    Ok(state
        .import_profile_bundle
        .execute(&path, selection, target, conflict_strategy)
        .await?)
}
