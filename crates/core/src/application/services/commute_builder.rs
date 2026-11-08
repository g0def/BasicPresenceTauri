//! Assembling a validated `Commute` from untrusted input — shared by create,
//! update and the bundle import, so all three enforce exactly the same rules.

use uuid::Uuid;

use crate::domain::entities::commute::{Commute, CommuteSegment};
use crate::domain::error::DomainError;
use crate::domain::rules;

/// One raw segment, whatever its source (command DTO or imported file).
pub struct SegmentInput<'a> {
    pub mode_id: &'a str,
    pub distance_km: f64,
    pub occupants: Option<i64>,
}

/// Validate inputs and assemble a `Commute` with fresh segment ids; positions
/// follow input order.
pub fn build_commute<'a>(
    id: String,
    profile_id: &str,
    name: &str,
    round_trip: bool,
    segments: impl ExactSizeIterator<Item = SegmentInput<'a>>,
    created_at: i64,
    updated_at: i64,
) -> Result<Commute, DomainError> {
    let profile_id = rules::required_id("profileId", profile_id)?;
    let name = rules::required_text("name", name, rules::MAX_TITLE_LEN)?;
    rules::segment_count(segments.len(), true)?;

    let mut domain_segments = Vec::with_capacity(segments.len());
    for (i, s) in segments.enumerate() {
        domain_segments.push(CommuteSegment {
            id: Uuid::now_v7().to_string(),
            commute_id: id.clone(),
            mode_id: rules::mode_id(i, s.mode_id)?,
            distance_km: rules::distance_km(i, s.distance_km)?,
            occupants: rules::occupants(i, s.occupants)?,
            position: i as i64,
        });
    }

    Ok(Commute {
        id,
        profile_id: profile_id.to_string(),
        name,
        round_trip,
        segments: domain_segments,
        created_at,
        updated_at,
    })
}
