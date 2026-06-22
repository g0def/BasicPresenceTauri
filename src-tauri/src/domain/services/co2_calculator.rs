use std::collections::HashMap;

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::entities::emission_factor::{EmissionCategory, EmissionFactor, EmissionUnit};
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::trip::TripInput;

/// Radiative-forcing multiplier baked into the aviation factors, resolved per
/// referential year: DEFRA/DESNZ lowered it from 1.9 to 1.7 in the June-2023
/// update, so the 2025 referential's air factors are baked with 1.9 and the 2026
/// referential's with 1.7. When the user disables radiative forcing we DIVIDE by
/// the year-matching multiplier to recover the CO2-only value (never multiply —
/// that would double-count, R5).
pub(crate) fn radiative_forcing_factor(factor_year: i32) -> f64 {
    if factor_year >= 2026 {
        1.7
    } else {
        1.9
    }
}

/// Fallback mode used when a trip's `mode_id` is absent from the referential
/// (R6). The result is flagged `is_estimated`.
const FALLBACK_MODE_ID: &str = "car_average";

const OFFICE_DAY_MODE_ID: &str = "office_day";
const HOME_DAY_MODE_ID: &str = "home_day";

/// Result of computing one trip leg.
#[derive(Debug, Clone)]
pub struct ComputedTrip {
    pub mode_id: String,
    pub distance_km: f64,
    pub round_trip: bool,
    pub occupants: i64,
    pub co2_kg: f64,
    pub is_estimated: bool,
}

/// Result of computing a full presence day.
#[derive(Debug, Clone)]
pub struct DayEmission {
    pub trips: Vec<ComputedTrip>,
    pub total_kg: f64,
    /// `true` if any leg fell back to an estimated factor.
    pub is_estimated: bool,
}

/// Pure CO2 calculator. All I/O (loading factors, variants, settings) happens in
/// the caller; this service only does arithmetic so it is deterministic and
/// fully unit-testable against the acceptance criteria.
pub struct Co2Calculator;

impl Co2Calculator {
    /// Compute the footprint of a presence day.
    ///
    /// * `factors`        — mode_id -> factor (for `settings.factor_year`).
    /// * `grid_variants`  — (mode_id, country) -> overriding value (electric modes).
    ///
    /// Vacation/holiday days are always 0 (R7); office/remote sum their trips and
    /// optionally add building energy when `count_building_energy` is on.
    pub fn compute_day(
        trips: &[TripInput],
        presence: PresenceType,
        factors: &HashMap<String, EmissionFactor>,
        grid_variants: &HashMap<(String, String), f64>,
        settings: &Co2Settings,
    ) -> DayEmission {
        // R7: "off" days (vacation/holiday) never carry a footprint, even if
        // stray trips were passed.
        if matches!(presence, PresenceType::Vacation | PresenceType::Holiday) {
            return DayEmission {
                trips: Vec::new(),
                total_kg: 0.0,
                is_estimated: false,
            };
        }

        let computed: Vec<ComputedTrip> = trips
            .iter()
            .map(|t| Self::compute_trip(t, factors, grid_variants, settings))
            .collect();

        let trips_total: f64 = computed.iter().map(|c| c.co2_kg).sum();
        let is_estimated = computed.iter().any(|c| c.is_estimated);

        let building = if settings.count_building_energy {
            let mode = match presence {
                PresenceType::Office => Some(OFFICE_DAY_MODE_ID),
                PresenceType::Remote => Some(HOME_DAY_MODE_ID),
                _ => None,
            };
            mode.and_then(|id| factors.get(id))
                .map(|f| f.value)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        DayEmission {
            trips: computed,
            total_kg: (trips_total + building).max(0.0), // R8
            is_estimated,
        }
    }

    /// Compute one trip leg, following the fixed formula precedence:
    /// resolve factor (R6 fallback) → electric grid variant XOR aviation RF →
    /// × distance × round-trip → ÷ occupants (veh.km only) → clamp ≥ 0.
    fn compute_trip(
        trip: &TripInput,
        factors: &HashMap<String, EmissionFactor>,
        grid_variants: &HashMap<(String, String), f64>,
        settings: &Co2Settings,
    ) -> ComputedTrip {
        // 1. Resolve the factor; fall back to a known average if missing (R6).
        let (factor, is_estimated) = match factors.get(&trip.mode_id) {
            Some(f) => (Some(f), false),
            None => (factors.get(FALLBACK_MODE_ID), true),
        };

        let co2 = match factor {
            None => 0.0, // unknown mode AND no fallback available: 0, but flagged.
            Some(factor) => {
                // 2. Effective factor: an electric grid variant REPLACES the base
                //    value; otherwise aviation may drop radiative forcing. These
                //    are mutually exclusive (no air mode has a grid variant), so
                //    RF is applied at most once (R5).
                let mut ef = factor.value;
                if let Some(v) =
                    grid_variants.get(&(trip.mode_id.clone(), settings.grid_country.clone()))
                {
                    ef = *v; // R4: charging-country grid preset.
                } else if factor.category == EmissionCategory::Air
                    && !settings.include_radiative_forcing
                {
                    ef /= radiative_forcing_factor(settings.factor_year);
                }

                // 3. Distance × factor × round-trip (R1).
                let raw = trip.distance_km * ef * if trip.round_trip { 2.0 } else { 1.0 };

                // 4. Per-vehicle factors are split across occupants (carpool, R2);
                //    passenger.km / km factors are already per person.
                let co2 = if factor.unit == EmissionUnit::VehKm {
                    raw / (trip.occupants.max(1) as f64)
                } else {
                    raw
                };

                co2.max(0.0) // 5. R8: never negative.
            }
        };

        ComputedTrip {
            mode_id: trip.mode_id.clone(),
            distance_km: trip.distance_km,
            round_trip: trip.round_trip,
            occupants: trip.occupants,
            co2_kg: co2,
            is_estimated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ef(id: &str, value: f64, unit: EmissionUnit, category: EmissionCategory) -> EmissionFactor {
        EmissionFactor {
            id: id.to_string(),
            label: id.to_string(),
            value,
            unit,
            category,
            is_param: matches!(category, EmissionCategory::Air)
                || matches!(id, "car_ev" | "ebike" | "escooter" | "scooter_elec"),
            scope: None,
            source: None,
        }
    }

    /// Build the (subset of the) seeded referential needed by the tests.
    fn seed_factors() -> HashMap<String, EmissionFactor> {
        use EmissionCategory::*;
        use EmissionUnit::*;
        let list = [
            ef("car_petrol", 0.2388, VehKm, Car),
            ef("car_average", 0.2311, VehKm, Car),
            ef("car_ev", 0.1393, VehKm, Car),
            ef("taxi", 0.2311, VehKm, Car),
            ef("walk", 0.0, Km, Active),
            ef("bike", 0.0, Km, Active),
            ef("ebike", 0.01095, Km, Active),
            ef("train_hs_fr", 0.00343, PassengerKm, Rail),
            ef("plane_short", 0.2582, PassengerKm, Air),
            ef("office_day", 3.5, Day, Building),
            ef("home_day", 2.7, Day, Building),
        ];
        list.into_iter().map(|f| (f.id.clone(), f)).collect()
    }

    fn grid_variants() -> HashMap<(String, String), f64> {
        let mut m = HashMap::new();
        m.insert(("car_ev".to_string(), "FR".to_string()), 0.09);
        m.insert(("car_ev".to_string(), "BE".to_string()), 0.1393);
        m.insert(("car_ev".to_string(), "DE".to_string()), 0.18);
        m.insert(("car_ev".to_string(), "EU".to_string()), 0.13);
        m
    }

    fn trip(mode_id: &str, distance_km: f64, round_trip: bool, occupants: i64) -> TripInput {
        TripInput {
            mode_id: mode_id.to_string(),
            distance_km,
            round_trip,
            occupants,
        }
    }

    /// Day total for a single trip with default (BE, RF on, building off) settings.
    fn day_total(trips: &[TripInput], presence: PresenceType) -> DayEmission {
        Co2Calculator::compute_day(
            trips,
            presence,
            &seed_factors(),
            &grid_variants(),
            &Co2Settings::default(),
        )
    }

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-3, "expected {b}, got {a}");
    }

    #[test]
    fn ac1_car_petrol_oneway_and_roundtrip() {
        approx(
            day_total(&[trip("car_petrol", 20.0, false, 1)], PresenceType::Office).total_kg,
            4.776,
        );
        approx(
            day_total(&[trip("car_petrol", 20.0, true, 1)], PresenceType::Office).total_kg,
            9.552,
        );
    }

    #[test]
    fn ac2_car_average_roundtrip_carpool() {
        approx(
            day_total(&[trip("car_average", 30.0, true, 3)], PresenceType::Office).total_kg,
            4.622,
        );
    }

    #[test]
    fn ac3_train_hs_fr_roundtrip() {
        approx(
            day_total(&[trip("train_hs_fr", 400.0, true, 1)], PresenceType::Office).total_kg,
            2.744,
        );
    }

    #[test]
    fn ac4_ebike_roundtrip() {
        approx(
            day_total(&[trip("ebike", 8.0, true, 1)], PresenceType::Office).total_kg,
            0.1752,
        );
    }

    #[test]
    fn ac5_walk_and_bike_are_zero() {
        approx(
            day_total(&[trip("walk", 3.0, false, 1)], PresenceType::Office).total_kg,
            0.0,
        );
        approx(
            day_total(&[trip("bike", 12.0, true, 1)], PresenceType::Office).total_kg,
            0.0,
        );
    }

    #[test]
    fn ac6_plane_short_radiative_forcing_toggle() {
        // RF off divides the (RF-inclusive) factor by the year-matching multiplier:
        // 1.9 for the 2025 referential (legacy DEFRA), 1.7 for 2026 (DEFRA lowered
        // it in June 2023). The fixture factor is the same; only the divisor moves.
        let off_2025 = Co2Settings {
            include_radiative_forcing: false,
            factor_year: 2025,
            ..Default::default()
        };
        let res_2025 = Co2Calculator::compute_day(
            &[trip("plane_short", 500.0, false, 1)],
            PresenceType::Office,
            &seed_factors(),
            &grid_variants(),
            &off_2025,
        );
        approx(res_2025.total_kg, 500.0 * 0.2582 / 1.9);

        let off_2026 = Co2Settings {
            include_radiative_forcing: false,
            factor_year: 2026,
            ..Default::default()
        };
        let res_2026 = Co2Calculator::compute_day(
            &[trip("plane_short", 500.0, false, 1)],
            PresenceType::Office,
            &seed_factors(),
            &grid_variants(),
            &off_2026,
        );
        approx(res_2026.total_kg, 500.0 * 0.2582 / 1.7);

        // RF on: factor used as-is (independent of the year).
        approx(
            day_total(
                &[trip("plane_short", 500.0, false, 1)],
                PresenceType::Office,
            )
            .total_kg,
            129.1,
        );
    }

    #[test]
    fn ac7_week_commute_only() {
        // 3 office days of a 15 km round-trip petrol commute, building energy off.
        let office = day_total(&[trip("car_petrol", 15.0, true, 1)], PresenceType::Office);
        let remote = day_total(&[], PresenceType::Remote);
        approx(office.total_kg * 3.0 + remote.total_kg * 2.0, 21.492);
    }

    #[test]
    fn ac8_week_with_building_energy() {
        let on = Co2Settings {
            count_building_energy: true,
            ..Default::default()
        };
        let factors = seed_factors();
        let variants = grid_variants();
        let office = Co2Calculator::compute_day(
            &[trip("car_petrol", 15.0, true, 1)],
            PresenceType::Office,
            &factors,
            &variants,
            &on,
        );
        let remote =
            Co2Calculator::compute_day(&[], PresenceType::Remote, &factors, &variants, &on);
        approx(office.total_kg * 3.0 + remote.total_kg * 2.0, 37.392);
    }

    #[test]
    fn ac9_org_three_employees() {
        let per_employee =
            day_total(&[trip("car_petrol", 15.0, true, 1)], PresenceType::Office).total_kg * 3.0; // weekly commute-only total from AC-7
        approx(per_employee * 3.0, 64.476);
    }

    #[test]
    fn ac10_breakdown_sums_to_total() {
        let day = day_total(
            &[
                trip("train_hs_fr", 30.0, true, 1),
                trip("ebike", 5.0, true, 1),
            ],
            PresenceType::Office,
        );
        let sum: f64 = day.trips.iter().map(|t| t.co2_kg).sum();
        approx(sum, day.total_kg);
    }

    #[test]
    fn occupant_division_only_for_veh_km() {
        // Train (passenger.km) ignores occupants; car (veh.km) divides by them.
        let train = day_total(
            &[trip("train_hs_fr", 100.0, false, 4)],
            PresenceType::Office,
        );
        approx(train.total_kg, 100.0 * 0.00343);
        let car = day_total(&[trip("car_petrol", 100.0, false, 4)], PresenceType::Office);
        approx(car.total_kg, 100.0 * 0.2388 / 4.0);
    }

    #[test]
    fn electric_grid_variant_replaces_default() {
        let factors = seed_factors();
        let variants = grid_variants();
        // BE (default) uses the BE variant 0.1393 — matches the base value.
        let be = Co2Calculator::compute_day(
            &[trip("car_ev", 100.0, false, 1)],
            PresenceType::Office,
            &factors,
            &variants,
            &Co2Settings::default(),
        );
        approx(be.total_kg, 100.0 * 0.1393);
        // FR uses 0.09.
        let fr = Co2Settings {
            grid_country: "FR".to_string(),
            ..Default::default()
        };
        let fr_res = Co2Calculator::compute_day(
            &[trip("car_ev", 100.0, false, 1)],
            PresenceType::Office,
            &factors,
            &variants,
            &fr,
        );
        approx(fr_res.total_kg, 100.0 * 0.09);
    }

    #[test]
    fn missing_factor_falls_back_and_flags_estimated() {
        let day = day_total(
            &[trip("unknown_mode", 10.0, false, 1)],
            PresenceType::Office,
        );
        assert!(day.is_estimated);
        // Fallback to car_average (veh.km), occupants 1.
        approx(day.total_kg, 10.0 * 0.2311);
    }

    #[test]
    fn off_days_are_zero_even_with_trips() {
        approx(
            day_total(&[trip("car_petrol", 50.0, true, 1)], PresenceType::Vacation).total_kg,
            0.0,
        );
        approx(
            day_total(&[trip("car_petrol", 50.0, true, 1)], PresenceType::Holiday).total_kg,
            0.0,
        );
    }

    #[test]
    fn building_energy_only_when_enabled_and_office_or_remote() {
        let on = Co2Settings {
            count_building_energy: true,
            ..Default::default()
        };
        let factors = seed_factors();
        let variants = grid_variants();
        // Office with no trips → office_day (3.5).
        approx(
            Co2Calculator::compute_day(&[], PresenceType::Office, &factors, &variants, &on)
                .total_kg,
            3.5,
        );
        // Remote with no trips → home_day (2.7).
        approx(
            Co2Calculator::compute_day(&[], PresenceType::Remote, &factors, &variants, &on)
                .total_kg,
            2.7,
        );
        // Vacation stays 0 regardless.
        approx(
            Co2Calculator::compute_day(&[], PresenceType::Vacation, &factors, &variants, &on)
                .total_kg,
            0.0,
        );
    }

    #[test]
    fn multimodal_segments_sum() {
        let day = day_total(
            &[
                trip("train_hs_fr", 30.0, true, 1),
                trip("ebike", 5.0, true, 1),
            ],
            PresenceType::Office,
        );
        approx(day.total_kg, 30.0 * 2.0 * 0.00343 + 5.0 * 2.0 * 0.01095);
    }
}
