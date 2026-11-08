use std::path::Path;
use std::sync::Arc;

use crate::application::dto::profile_bundle_dto::{BundleExportOptionsDto, BundleExportSummaryDto};
use crate::application::services::count;
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;
use crate::domain::services::profile_bundle::{
    BundleDay, BundleOptions, ProfileBundleCodec, ProfileBundleData,
};

/// Gather a profile's full dataset and hand it to the [`ProfileBundleCodec`],
/// which applies the chosen options and writes the file. Reuses the existing
/// read repository methods (no new SQL); like the ODS export, it gathers
/// everything and lets the codec filter (volume is modest for a personal tracker).
pub struct ExportProfileBundleUseCase {
    profiles: Arc<dyn ProfileRepository>,
    presences: Arc<dyn PresenceRepository>,
    work_entries: Arc<dyn WorkEntryRepository>,
    task_presets: Arc<dyn TaskPresetRepository>,
    commutes: Arc<dyn CommuteRepository>,
    profile_settings: Arc<dyn ProfileSettingsRepository>,
    codec: Arc<dyn ProfileBundleCodec>,
    clock: Arc<dyn Clock>,
}

impl ExportProfileBundleUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profiles: Arc<dyn ProfileRepository>,
        presences: Arc<dyn PresenceRepository>,
        work_entries: Arc<dyn WorkEntryRepository>,
        task_presets: Arc<dyn TaskPresetRepository>,
        commutes: Arc<dyn CommuteRepository>,
        profile_settings: Arc<dyn ProfileSettingsRepository>,
        codec: Arc<dyn ProfileBundleCodec>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            profiles,
            presences,
            work_entries,
            task_presets,
            commutes,
            profile_settings,
            codec,
            clock,
        }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        options: BundleExportOptionsDto,
        path: &Path,
    ) -> Result<BundleExportSummaryDto, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;

        let profile = self
            .profiles
            .find_by_id(profile_id)
            .await?
            .ok_or(DomainError::ProfileNotFound)?;

        // A handful of profile-wide queries, joined in memory (no per-day N+1).
        let presences = self.presences.list_by_profile(profile_id).await?;
        let mut trips = self.presences.list_trips_by_profile(profile_id).await?;
        let mut notes = self.presences.list_notes_by_profile(profile_id).await?;
        let mut entries = self.work_entries.list_by_profile(profile_id).await?;
        let mut schedules = self
            .work_entries
            .list_schedules_by_profile(profile_id)
            .await?;
        let days = presences
            .into_iter()
            .map(|presence| BundleDay {
                trips: trips.remove(&presence.id).unwrap_or_default(),
                work_entries: entries.remove(&presence.id).unwrap_or_default(),
                schedule: schedules.remove(&presence.id),
                note: notes.remove(&presence.id),
                presence,
            })
            .collect::<Vec<_>>();
        let task_presets = self.task_presets.list_by_profile(profile_id).await?;
        let commutes = self.commutes.list_by_profile(profile_id).await?;
        let settings = Some(self.profile_settings.load(profile_id).await?);

        let opts: BundleOptions = options.into();
        let data = ProfileBundleData {
            exported_at: self.clock.now_ms(),
            profile,
            days,
            task_presets,
            commutes,
            settings,
        };

        // Counts reflect what the codec writes under `opts`.
        let summary = BundleExportSummaryDto {
            days: if opts.include_days {
                count(data.days.len())
            } else {
                0
            },
            trips: if opts.include_days && opts.include_trips {
                data.days.iter().map(|d| count(d.trips.len())).sum()
            } else {
                0
            },
            work_entries: if opts.include_days && opts.include_work_hours {
                data.days.iter().map(|d| count(d.work_entries.len())).sum()
            } else {
                0
            },
            notes: if opts.include_days && opts.include_notes {
                // "Note present" = non-blank, matching inspect_profile_bundle so the
                // export summary and the import manifest can never disagree.
                data.days
                    .iter()
                    .filter(|d| {
                        d.note
                            .as_deref()
                            .map(|n| !n.trim().is_empty())
                            .unwrap_or(false)
                    })
                    .count() as u32
            } else {
                0
            },
            task_presets: if opts.include_task_presets {
                count(data.task_presets.len())
            } else {
                0
            },
            commutes: if opts.include_commutes {
                count(data.commutes.len())
            } else {
                0
            },
            settings: opts.include_settings && data.settings.is_some(),
            path: path.display().to_string(),
        };

        self.codec.write(&data, &opts, path)?;
        Ok(summary)
    }
}
