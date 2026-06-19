use std::path::Path;
use std::sync::Arc;

use crate::application::dto::profile_bundle_dto::{BundleExportOptionsDto, BundleExportSummaryDto};
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
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
        path: &str,
    ) -> Result<BundleExportSummaryDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        let path = path.trim();
        if path.is_empty() {
            return Err(DomainError::Validation("path is required".to_string()));
        }

        let profile = self
            .profiles
            .find_by_id(profile_id)
            .await?
            .ok_or(DomainError::ProfileNotFound)?;

        let presences = self.presences.list_by_profile(profile_id).await?;
        let mut days = Vec::with_capacity(presences.len());
        for presence in presences {
            let trips = self.presences.list_trips(&presence.id).await?;
            let work_entries = self.work_entries.list_by_presence(&presence.id).await?;
            let schedule = self.work_entries.get_schedule(&presence.id).await?;
            let note = self.presences.get_note(&presence.id).await?;
            days.push(BundleDay {
                presence,
                trips,
                work_entries,
                schedule,
                note,
            });
        }
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
                data.days.len() as u32
            } else {
                0
            },
            trips: if opts.include_days && opts.include_trips {
                data.days.iter().map(|d| d.trips.len() as u32).sum()
            } else {
                0
            },
            work_entries: if opts.include_days && opts.include_work_hours {
                data.days.iter().map(|d| d.work_entries.len() as u32).sum()
            } else {
                0
            },
            notes: if opts.include_days && opts.include_notes {
                data.days.iter().filter(|d| d.note.is_some()).count() as u32
            } else {
                0
            },
            task_presets: if opts.include_task_presets {
                data.task_presets.len() as u32
            } else {
                0
            },
            commutes: if opts.include_commutes {
                data.commutes.len() as u32
            } else {
                0
            },
            settings: opts.include_settings && data.settings.is_some(),
            path: path.to_string(),
        };

        self.codec.write(&data, &opts, Path::new(path))?;
        Ok(summary)
    }
}
