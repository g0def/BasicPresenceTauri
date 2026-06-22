/// CO2 configuration consumed by the calculator. Stored per profile, embedded
/// in [`ProfileSettings`](crate::domain::entities::profile_settings::ProfileSettings);
/// these defaults apply when a profile has no stored row.
#[derive(Debug, Clone)]
pub struct Co2Settings {
    /// Charging country selecting the electric-grid preset (e.g. `"BE"`).
    pub grid_country: String,
    /// Default car occupancy when a trip omits it.
    pub default_car_occupancy: i64,
    /// Aviation: include the radiative-forcing effect (contrails).
    pub include_radiative_forcing: bool,
    /// Phase 2: add building energy for office/remote days.
    pub count_building_energy: bool,
    /// For annual projections (not used in Phase 1 calculations).
    #[allow(dead_code)] // read by the Phase 2 annual projections
    pub working_days_per_year: i64,
    /// Which referential year to resolve factors against.
    pub factor_year: i32,
}

impl Default for Co2Settings {
    fn default() -> Self {
        Self {
            grid_country: "BE".to_string(),
            default_car_occupancy: 1,
            include_radiative_forcing: true,
            count_building_energy: false,
            working_days_per_year: 220,
            factor_year: 2026,
        }
    }
}
