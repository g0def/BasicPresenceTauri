use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::import_presence_dto::{ImportPresenceEntryDto, ImportSummaryDto};
use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::services::clock::Clock;

/// Milliseconds in a UTC day. A valid `day` is a non-negative multiple of it
/// (epoch ms at UTC midnight), matching the key the frontend sends.
const MS_PER_DAY: i64 = 86_400_000;

/// Bulk-import presences for a profile from an exported file. Conflicts with
/// existing days are either skipped or replaced depending on `replace_existing`.
pub struct ImportPresencesUseCase {
    presences: Arc<dyn PresenceRepository>,
    clock: Arc<dyn Clock>,
}

impl ImportPresencesUseCase {
    pub fn new(presences: Arc<dyn PresenceRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { presences, clock }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        entries: Vec<ImportPresenceEntryDto>,
        replace_existing: bool,
    ) -> Result<ImportSummaryDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }

        let total = entries.len() as u32;
        if total == 0 {
            // Empty import is a no-op success (the frontend also guards this).
            return Ok(ImportSummaryDto {
                imported: 0,
                skipped: 0,
                replaced: 0,
                total: 0,
            });
        }

        // Validate + normalize every row before touching the vault (the import
        // is atomic, so one bad row must abort the whole batch). The frontend
        // already filters invalid rows; this is the defensive second line.
        let now = self.clock.now_ms();
        let mut domain_entries = Vec::with_capacity(entries.len());
        for (i, e) in entries.into_iter().enumerate() {
            if e.day < 0 || e.day % MS_PER_DAY != 0 {
                return Err(DomainError::Validation(format!(
                    "entry {i}: day must be epoch ms at UTC midnight"
                )));
            }
            let kind = PresenceType::parse(&e.kind)?;
            let created_at = e.created_at.unwrap_or(now);
            let updated_at = e.updated_at.unwrap_or(created_at);
            domain_entries.push(Presence {
                id: Uuid::now_v7().to_string(),
                profile_id: profile_id.to_string(),
                day: e.day,
                kind,
                // Imported days carry no commute footprint until re-encoded.
                co2_kg: None,
                is_estimated: false,
                created_at,
                updated_at,
                work_minutes: 0,
            });
        }

        let counts = self
            .presences
            .import_many(profile_id, &domain_entries, replace_existing)
            .await?;

        let imported = counts.inserted;
        let replaced = counts.updated;
        let skipped = total - imported - replaced;
        Ok(ImportSummaryDto {
            imported,
            skipped,
            replaced,
            total,
        })
    }
}
