/// One leg of a saved commute template. `distance_km` is one-way; `occupants`
/// only affects car-category (per-vehicle) modes.
#[derive(Debug, Clone)]
pub struct CommuteSegment {
    pub id: String,
    pub commute_id: String,
    pub mode_id: String,
    pub distance_km: f64,
    pub occupants: i64,
    pub position: i64,
}

/// A reusable, named commute owned by a profile (e.g. "30 km train + 5 km vélo").
/// `round_trip` applies to the whole journey.
#[derive(Debug, Clone)]
pub struct Commute {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub round_trip: bool,
    pub segments: Vec<CommuteSegment>,
    pub created_at: i64,
    pub updated_at: i64,
}
