//! Profile transfer bundle — a portable, versioned document capturing a whole
//! profile so it can be shared and re-imported losslessly. This module is the
//! domain view: the data gathered for a write, the data parsed from a read, and
//! the [`ProfileBundleCodec`] port. The on-disk encoding (JSON today) and all
//! file I/O live behind the port in the infrastructure layer, mirroring
//! [`SpreadsheetExporter`](crate::domain::services::spreadsheet_exporter).

use std::path::Path;

use crate::domain::entities::commute::Commute;
use crate::domain::entities::presence::Presence;
use crate::domain::entities::profile::Profile;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::entities::trip::Trip;
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;

/// Highest bundle schema version this build can read and write. A file with a
/// greater version is rejected on import (and flagged incompatible on inspect).
pub const SUPPORTED_BUNDLE_VERSION: u32 = 1;

/// Which categories the codec includes when writing a bundle. The three day
/// children are only written when `include_days` is set.
#[derive(Debug, Clone, Copy)]
pub struct BundleOptions {
    pub include_days: bool,
    pub include_trips: bool,
    pub include_work_hours: bool,
    pub include_notes: bool,
    pub include_task_presets: bool,
    pub include_commutes: bool,
    pub include_settings: bool,
}

/// One presence day with everything attached, already gathered from the vault.
pub struct BundleDay {
    pub presence: Presence,
    pub trips: Vec<Trip>,
    pub work_entries: Vec<WorkEntry>,
    pub schedule: Option<WorkDaySchedule>,
    /// Raw Markdown of the day's note, when set and non-blank.
    pub note: Option<String>,
}

/// The full dataset of a profile to write. The codec applies [`BundleOptions`].
/// `exported_at` is supplied by the caller (from the injected clock) so the
/// codec stays free of any time source.
pub struct ProfileBundleData {
    pub exported_at: i64,
    pub profile: Profile,
    pub days: Vec<BundleDay>,
    pub task_presets: Vec<TaskPreset>,
    pub commutes: Vec<Commute>,
    pub settings: Option<ProfileSettings>,
}

/// A bundle parsed back from a file. The contained entities reuse the domain
/// types but their `id`/owner-id/timestamp fields are placeholders — a bundle is
/// profile-agnostic, so the import use case assigns fresh ids and a target.
pub struct ParsedBundle {
    pub version: u32,
    pub exported_at: i64,
    /// App version that produced the file.
    pub app: String,
    pub profile: Profile,
    pub days: Vec<BundleDay>,
    pub task_presets: Vec<TaskPreset>,
    pub commutes: Vec<Commute>,
    pub settings: Option<ProfileSettings>,
}

/// Port: read/write a profile transfer bundle as a file. Implemented in the
/// infrastructure layer (JSON). Keeps file I/O and (de)serialization out of the
/// use cases, so the orchestration stays unit-testable behind a mock.
pub trait ProfileBundleCodec: Send + Sync {
    /// Serialize the chosen categories of `data` and write the file at `path`.
    fn write(
        &self,
        data: &ProfileBundleData,
        options: &BundleOptions,
        path: &Path,
    ) -> Result<(), DomainError>;

    /// Read and parse the bundle file at `path`. A read/parse failure or an
    /// unrecognized format surfaces as [`DomainError::Validation`]; a too-new
    /// `version` is returned as-is (the caller decides compatibility).
    fn read(&self, path: &Path) -> Result<ParsedBundle, DomainError>;
}
