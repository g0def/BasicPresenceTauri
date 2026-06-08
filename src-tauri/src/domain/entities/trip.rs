/// A single trip leg fed to the CO2 calculator. `distance_km` is one-way and the
/// `round_trip` flag is the journey-level value applied to every leg.
#[derive(Debug, Clone)]
pub struct TripInput {
    pub mode_id: String,
    pub distance_km: f64,
    pub round_trip: bool,
    pub occupants: i64,
}

/// A per-day snapshot row (the ticket's "Trip"): the raw inputs plus the
/// computed `co2_kg` frozen at encode time, attached to a presence day. Makes a
/// past day's footprint reproducible (R9) regardless of later referential edits.
/// The owning `presence_id` is assigned by the repository at persist time.
#[derive(Debug, Clone)]
pub struct Trip {
    pub id: String,
    pub mode_id: String,
    pub distance_km: f64,
    pub round_trip: bool,
    pub occupants: i64,
    pub co2_kg: f64,
    pub is_estimated: bool,
    /// Referential year used to compute `co2_kg` (audit / reproducibility).
    pub factor_year: i32,
    pub position: i64,
}
