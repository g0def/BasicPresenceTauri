use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use crate::application::dto::profile_bundle_dto::BundleManifestDto;
use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::services::profile_bundle::{
    ParsedBundle, ProfileBundleCodec, SUPPORTED_BUNDLE_VERSION,
};

/// Parse a bundle file (via the codec) and report what it contains, without
/// writing anything. Drives the import dialog's "intelligent" review step (only
/// offer categories that are present) and flags transport modes this device
/// doesn't know.
pub struct InspectProfileBundleUseCase {
    codec: Arc<dyn ProfileBundleCodec>,
    factors: Arc<dyn EmissionFactorRepository>,
}

impl InspectProfileBundleUseCase {
    pub fn new(
        codec: Arc<dyn ProfileBundleCodec>,
        factors: Arc<dyn EmissionFactorRepository>,
    ) -> Self {
        Self { codec, factors }
    }

    pub async fn execute(&self, path: &str) -> Result<BundleManifestDto, DomainError> {
        let path = path.trim();
        if path.is_empty() {
            return Err(DomainError::Validation("path is required".to_string()));
        }
        let bundle = self.codec.read(Path::new(path))?;

        let days = bundle.days.len() as u32;
        let trips: u32 = bundle.days.iter().map(|d| d.trips.len() as u32).sum();
        let work_entries: u32 = bundle
            .days
            .iter()
            .map(|d| d.work_entries.len() as u32)
            .sum();
        let notes: u32 = bundle
            .days
            .iter()
            .filter(|d| {
                d.note
                    .as_deref()
                    .map(|n| !n.trim().is_empty())
                    .unwrap_or(false)
            })
            .count() as u32;
        let task_presets = bundle.task_presets.len() as u32;
        let commutes = bundle.commutes.len() as u32;
        let has_settings = bundle.settings.is_some();

        // Resolve unknown modes while the whole bundle is still borrowable, before
        // moving the identity fields into the manifest below.
        let unknown_mode_ids = self.unknown_mode_ids(&bundle).await.unwrap_or_default();

        Ok(BundleManifestDto {
            version: bundle.version,
            compatible: bundle.version <= SUPPORTED_BUNDLE_VERSION,
            exported_at: bundle.exported_at,
            app: bundle.app,
            profile_first_name: bundle.profile.first_name,
            profile_last_name: bundle.profile.last_name,
            profile_enterprise: bundle.profile.enterprise,
            profile_poste: bundle.profile.poste,
            days,
            trips,
            work_entries,
            notes,
            task_presets,
            commutes,
            has_settings,
            unknown_mode_ids,
        })
    }

    /// Collect transport-mode ids referenced by the file (trips + commute legs)
    /// that don't resolve against the seeded referential. Best-effort: any read
    /// failure yields an empty list rather than blocking the inspection.
    async fn unknown_mode_ids(&self, bundle: &ParsedBundle) -> Result<Vec<String>, DomainError> {
        let mut referenced: HashSet<String> = HashSet::new();
        for d in &bundle.days {
            for t in &d.trips {
                referenced.insert(t.mode_id.clone());
            }
        }
        for c in &bundle.commutes {
            for s in &c.segments {
                referenced.insert(s.mode_id.clone());
            }
        }
        if referenced.is_empty() {
            return Ok(Vec::new());
        }

        // Resolve against the default referential year plus any year the file's
        // trips were computed with (factors are versioned by year).
        let mut years: HashSet<i32> = HashSet::new();
        years.insert(Co2Settings::default().factor_year);
        for d in &bundle.days {
            for t in &d.trips {
                years.insert(t.factor_year);
            }
        }
        if let Some(s) = &bundle.settings {
            years.insert(s.co2.factor_year);
        }

        let mut known: HashSet<String> = HashSet::new();
        for y in years {
            for f in self.factors.list(y).await? {
                known.insert(f.id);
            }
        }

        let mut unknown: Vec<String> = referenced
            .into_iter()
            .filter(|m| !known.contains(m))
            .collect();
        unknown.sort();
        Ok(unknown)
    }
}
