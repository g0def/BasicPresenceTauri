//! Command-boundary DTOs for the profile transfer feature (camelCase, the JS
//! frontier). The on-disk bundle *format* lives in the infrastructure codec
//! (`infrastructure/transfer`); these types are only the IPC request/response
//! shapes. Distinct from the ODS export (`export_dto`).

use serde::{Deserialize, Serialize};

use crate::domain::services::profile_bundle::BundleOptions;

/// Which categories to include in an export, received FROM the frontend.
/// `includeTrips`/`includeWorkHours`/`includeNotes` are children of
/// `includeDays` (ignored when it is off).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleExportOptionsDto {
    pub include_days: bool,
    pub include_trips: bool,
    pub include_work_hours: bool,
    pub include_notes: bool,
    pub include_task_presets: bool,
    pub include_commutes: bool,
    pub include_settings: bool,
}

impl From<BundleExportOptionsDto> for BundleOptions {
    fn from(d: BundleExportOptionsDto) -> Self {
        BundleOptions {
            include_days: d.include_days,
            include_trips: d.include_trips,
            include_work_hours: d.include_work_hours,
            include_notes: d.include_notes,
            include_task_presets: d.include_task_presets,
            include_commutes: d.include_commutes,
            include_settings: d.include_settings,
        }
    }
}

/// Outcome of an export, returned to the frontend (camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleExportSummaryDto {
    pub days: u32,
    pub trips: u32,
    pub work_entries: u32,
    pub notes: u32,
    pub task_presets: u32,
    pub commutes: u32,
    pub settings: bool,
    pub path: String,
}

/// What `inspect_profile_bundle` reports about a file before import: per-category
/// counts (so the UI only offers categories that are actually present) plus the
/// profile identity (to prefill the "new profile" form) and a compatibility flag.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifestDto {
    pub version: u32,
    /// `false` when the file was produced by a newer, unsupported schema.
    pub compatible: bool,
    pub exported_at: i64,
    pub app: String,
    pub profile_first_name: String,
    pub profile_last_name: String,
    pub profile_enterprise: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_poste: Option<String>,
    pub days: u32,
    pub trips: u32,
    pub work_entries: u32,
    pub notes: u32,
    pub task_presets: u32,
    pub commutes: u32,
    pub has_settings: bool,
    /// Transport-mode ids referenced by the file that this device's referential
    /// does not know (their footprint still imports, but the day can't be
    /// re-edited until the mode is known). Best-effort; empty when all resolve.
    pub unknown_mode_ids: Vec<String>,
}

/// Which categories the user chose to import (mirrors the export options).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportSelectionDto {
    pub include_days: bool,
    pub include_trips: bool,
    pub include_work_hours: bool,
    pub include_notes: bool,
    pub include_task_presets: bool,
    pub include_commutes: bool,
    pub include_settings: bool,
}

/// Import destination. `kind == "new"` creates a profile (identity defaults to
/// the bundle's, overridable field by field); `kind == "existing"` merges into
/// `profile_id`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportTargetDto {
    pub kind: String,
    #[serde(default)]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub enterprise: Option<String>,
    #[serde(default)]
    pub poste: Option<String>,
}

/// Per-category tallies of what an import actually wrote.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportSummaryDto {
    pub profile_id: String,
    pub days_imported: u32,
    pub days_replaced: u32,
    pub days_skipped: u32,
    pub trips: u32,
    pub work_entries: u32,
    pub notes: u32,
    pub task_presets: u32,
    pub commutes: u32,
    pub settings: bool,
}
