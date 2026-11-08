//! Methodology page helpers (ported from
//! `features/methodology/domain/worked-examples.ts`).

use crate::application::dto::emission_factor_dto::{Co2ReferentialDto, EmissionFactorDto};

/// Group factors by category, preserving the referential's order.
pub fn factors_by_category(
    factors: &[EmissionFactorDto],
) -> Vec<(String, Vec<&EmissionFactorDto>)> {
    let mut groups: Vec<(String, Vec<&EmissionFactorDto>)> = Vec::new();
    for f in factors {
        match groups.iter_mut().find(|(c, _)| *c == f.category) {
            Some((_, list)) => list.push(f),
            None => groups.push((f.category.clone(), vec![f])),
        }
    }
    groups
}

/// Which illustrative calculation an example is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkedExampleKind {
    /// 20 km by petrol car, alone (round trip).
    Car,
    /// The same car trip shared by 3.
    Carpool,
    /// 400 km by high-speed train (round trip).
    Train,
    /// 500 km one-way short-haul flight.
    Plane,
}

/// One illustrative calculation shown on the methodology page.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkedExample {
    pub kind: WorkedExampleKind,
    /// One-way distance, km.
    pub km: f64,
    /// Emission factor used, kgCO₂e per km.
    pub factor: f64,
    /// Resulting footprint, kgCO₂e.
    pub total: f64,
}

/// Worked examples computed from the LIVE factors (never hard-coded results).
/// Examples whose factor is missing from the referential are skipped.
pub fn worked_examples(referential: &Co2ReferentialDto) -> Vec<WorkedExample> {
    let factor = |id: &str| {
        referential
            .factors
            .iter()
            .find(|f| f.mode_id == id)
            .map(|f| f.value)
    };
    let mut out = Vec::new();
    if let Some(car) = factor("car_petrol") {
        let round_trip = 20.0 * 2.0 * car;
        out.push(WorkedExample {
            kind: WorkedExampleKind::Car,
            km: 20.0,
            factor: car,
            total: round_trip,
        });
        out.push(WorkedExample {
            kind: WorkedExampleKind::Carpool,
            km: 20.0,
            factor: car,
            total: round_trip / 3.0,
        });
    }
    if let Some(tgv) = factor("train_hs_fr") {
        out.push(WorkedExample {
            kind: WorkedExampleKind::Train,
            km: 400.0,
            factor: tgv,
            total: 400.0 * 2.0 * tgv,
        });
    }
    if let Some(plane) = factor("plane_short") {
        out.push(WorkedExample {
            kind: WorkedExampleKind::Plane,
            km: 500.0,
            factor: plane,
            total: 500.0 * plane,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factor(mode_id: &str, category: &str, value: f64) -> EmissionFactorDto {
        EmissionFactorDto {
            mode_id: mode_id.into(),
            label: mode_id.into(),
            value,
            unit: "kgCO2e/km".into(),
            category: category.into(),
            is_param: false,
            grid_variants: vec![],
            scope: None,
            source: None,
        }
    }

    fn referential(factors: Vec<EmissionFactorDto>) -> Co2ReferentialDto {
        Co2ReferentialDto {
            factor_year: 2024,
            radiative_forcing: 2.0,
            factors,
        }
    }

    #[test]
    fn groups_preserve_order() {
        let fs = [
            factor("car_petrol", "car", 0.2),
            factor("bike", "active", 0.0),
            factor("car_diesel", "car", 0.19),
        ];
        let groups = factors_by_category(&fs);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "car");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, "active");
    }

    #[test]
    fn worked_examples_use_live_factors() {
        let ex = worked_examples(&referential(vec![
            factor("car_petrol", "car", 0.25),
            factor("train_hs_fr", "rail", 0.003),
            factor("plane_short", "air", 0.2),
        ]));
        let kinds: Vec<_> = ex.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            [
                WorkedExampleKind::Car,
                WorkedExampleKind::Carpool,
                WorkedExampleKind::Train,
                WorkedExampleKind::Plane
            ]
        );
        assert!((ex[0].total - 10.0).abs() < 1e-9);
        assert!((ex[1].total - 10.0 / 3.0).abs() < 1e-9);
        assert!((ex[2].total - 2.4).abs() < 1e-9);
        assert!((ex[3].total - 100.0).abs() < 1e-9);
    }

    #[test]
    fn missing_factors_are_skipped() {
        let ex = worked_examples(&referential(vec![factor("plane_short", "air", 0.2)]));
        assert_eq!(ex.len(), 1);
        assert_eq!(ex[0].kind, WorkedExampleKind::Plane);
        assert!(worked_examples(&referential(vec![])).is_empty());
    }
}
