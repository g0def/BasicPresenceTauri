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

impl Trip {
    /// Distance actually travelled by this leg: the stored one-way `distance_km`
    /// doubled when the journey is a round trip.
    ///
    /// Read-side counterpart of the ×2 that [`crate::domain::services::co2_calculator`]
    /// applies to the *result* (R1) — the stored distance is deliberately never
    /// rewritten. Kept separate from the calculator on purpose: reassociating its
    /// multiplication would shift already-frozen `co2_kg` values (R9).
    pub fn counted_km(&self) -> f64 {
        if self.round_trip {
            self.distance_km * 2.0
        } else {
            self.distance_km
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trip(distance_km: f64, round_trip: bool) -> Trip {
        Trip {
            id: "t".to_string(),
            mode_id: "train_sncb".to_string(),
            distance_km,
            round_trip,
            occupants: 1,
            co2_kg: 0.0,
            is_estimated: false,
            factor_year: 2026,
            position: 0,
        }
    }

    #[test]
    fn counted_km_doubles_a_round_trip() {
        assert_eq!(trip(30.0, true).counted_km(), 60.0);
        assert_eq!(trip(30.0, false).counted_km(), 30.0);
        assert_eq!(trip(0.0, true).counted_km(), 0.0);
    }
}
