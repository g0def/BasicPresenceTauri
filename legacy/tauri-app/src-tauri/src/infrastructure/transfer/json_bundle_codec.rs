//! JSON implementation of the [`ProfileBundleCodec`] port: maps the domain
//! bundle data to/from a versioned, camelCase JSON document and reads/writes it
//! to disk. The serde wire structs are private to this adapter — they are the
//! file format, not a frontend contract (the commands return separate DTOs).

use std::io::Read;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::services::profile_bundle::{
    BundleOptions, ParsedBundle, ProfileBundleCodec, ProfileBundleData,
};
use crate::infrastructure::transfer::bundle_format::{BUNDLE_FORMAT, BundleFile};

/// Upper bound on a bundle file read into memory. A decade of daily presences,
/// trips, tasks and notes stays well under 10 MiB; anything far larger is not
/// a genuine export and would only exhaust memory.
const MAX_BUNDLE_BYTES: u64 = 50 * 1024 * 1024;

/// Reads/writes profile bundles as JSON files.
pub struct JsonProfileBundleCodec;

impl JsonProfileBundleCodec {
    pub fn new() -> Self {
        Self
    }
}

impl ProfileBundleCodec for JsonProfileBundleCodec {
    fn write(
        &self,
        data: &ProfileBundleData,
        options: &BundleOptions,
        path: &Path,
    ) -> Result<(), DomainError> {
        let file = BundleFile::from_domain(data, options);
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| DomainError::Export(format!("serialize bundle: {e}")))?;
        crate::infrastructure::fs::write_atomic(path, json.as_bytes())
            .map_err(|e| DomainError::Export(format!("write file: {e}")))
    }

    fn read(&self, path: &Path) -> Result<ParsedBundle, DomainError> {
        // Surface only the error *category* (not found, permission denied…):
        // Validation messages reach the UI verbatim, so no raw OS error here.
        let read_error =
            |e: std::io::Error| DomainError::Validation(format!("cannot read file: {}", e.kind()));
        // Read at most one byte past the cap (`take`, not a metadata check, so a
        // file growing after the check cannot slip through).
        let mut text = String::new();
        std::fs::File::open(path)
            .map_err(read_error)?
            .take(MAX_BUNDLE_BYTES + 1)
            .read_to_string(&mut text)
            .map_err(read_error)?;
        if text.len() as u64 > MAX_BUNDLE_BYTES {
            return Err(DomainError::Validation("file too large".to_string()));
        }
        // Strip a leading UTF-8 BOM if present (some editors add one).
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        // serde's message can quote file content: only the position goes out.
        let file: BundleFile = serde_json::from_str(text).map_err(|e| {
            DomainError::Validation(format!(
                "invalid profile file (line {}, column {})",
                e.line(),
                e.column()
            ))
        })?;
        if file.format != BUNDLE_FORMAT {
            return Err(DomainError::Validation(
                "unrecognized file format".to_string(),
            ));
        }
        file.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::co2_settings::Co2Settings;
    use crate::domain::entities::commute::{Commute, CommuteSegment};
    use crate::domain::entities::presence::{Presence, PresenceType};
    use crate::domain::entities::profile::Profile;
    use crate::domain::entities::profile_settings::ProfileSettings;
    use crate::domain::entities::task_preset::TaskPreset;
    use crate::domain::entities::trip::Trip;
    use crate::domain::entities::work_day_schedule::WorkDaySchedule;
    use crate::domain::entities::work_entry::WorkEntry;
    use crate::domain::services::profile_bundle::{BundleDay, SUPPORTED_BUNDLE_VERSION};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("bp-bundle-{}-{name}.json", std::process::id()));
        p
    }

    fn sample_data() -> ProfileBundleData {
        ProfileBundleData {
            exported_at: 1_700_000_000_000,
            profile: Profile {
                id: "p1".into(),
                first_name: "Ada".into(),
                last_name: "Lovelace".into(),
                enterprise: "Analytical".into(),
                poste: Some("Engineer".into()),
                created_at: 1,
                updated_at: 2,
            },
            days: vec![BundleDay {
                presence: Presence {
                    id: "d1".into(),
                    profile_id: "p1".into(),
                    day: 86_400_000,
                    kind: PresenceType::Office,
                    co2_kg: Some(3.5),
                    is_estimated: false,
                    created_at: 10,
                    updated_at: 20,
                    work_minutes: 0,
                },
                trips: vec![Trip {
                    id: "t1".into(),
                    mode_id: "car".into(),
                    distance_km: 12.5,
                    round_trip: true,
                    occupants: 1,
                    co2_kg: 3.5,
                    is_estimated: false,
                    factor_year: 2025,
                    position: 0,
                }],
                work_entries: vec![WorkEntry {
                    id: "w1".into(),
                    presence_id: "d1".into(),
                    title: "Coding".into(),
                    description: Some("feature".into()),
                    minutes: 120,
                    color: "#112233".into(),
                    position: 0,
                }],
                schedule: Some(WorkDaySchedule {
                    presence_id: "d1".into(),
                    start_minutes: 510,
                    end_minutes: 630,
                }),
                note: Some("a note".into()),
            }],
            task_presets: vec![TaskPreset {
                id: "tp1".into(),
                profile_id: "p1".into(),
                title: "Standup".into(),
                description: None,
                default_minutes: 15,
                color: "#abcdef".into(),
                created_at: 5,
                updated_at: 6,
            }],
            commutes: vec![Commute {
                id: "c1".into(),
                profile_id: "p1".into(),
                name: "Home".into(),
                round_trip: true,
                segments: vec![CommuteSegment {
                    id: "s1".into(),
                    commute_id: "c1".into(),
                    mode_id: "bike".into(),
                    distance_km: 4.0,
                    occupants: 1,
                    position: 0,
                }],
                created_at: 7,
                updated_at: 8,
            }],
            settings: Some(ProfileSettings {
                default_start_minutes: 480,
                note_font: "sans".into(),
                cell_display_mode: "co2".into(),
                co2: Co2Settings {
                    grid_country: "BE".into(),
                    default_car_occupancy: 1,
                    include_radiative_forcing: true,
                    count_building_energy: false,
                    working_days_per_year: 220,
                    factor_year: 2025,
                },
            }),
        }
    }

    fn all_options() -> BundleOptions {
        BundleOptions {
            include_days: true,
            include_trips: true,
            include_work_hours: true,
            include_notes: true,
            include_task_presets: true,
            include_commutes: true,
            include_settings: true,
        }
    }

    #[test]
    fn round_trips_all_categories() {
        let codec = JsonProfileBundleCodec::new();
        let path = temp_path("roundtrip");
        codec.write(&sample_data(), &all_options(), &path).unwrap();
        let parsed = codec.read(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(parsed.version, SUPPORTED_BUNDLE_VERSION);
        assert_eq!(parsed.profile.first_name, "Ada");
        assert_eq!(parsed.profile.poste.as_deref(), Some("Engineer"));
        assert_eq!(parsed.days.len(), 1);
        let day = &parsed.days[0];
        assert_eq!(day.presence.day, 86_400_000);
        assert!(matches!(day.presence.kind, PresenceType::Office));
        assert_eq!(day.presence.co2_kg, Some(3.5));
        assert_eq!(day.trips.len(), 1);
        assert_eq!(day.trips[0].mode_id, "car");
        assert_eq!(day.trips[0].distance_km, 12.5);
        assert_eq!(day.work_entries.len(), 1);
        assert_eq!(day.work_entries[0].title, "Coding");
        assert_eq!(day.work_entries[0].minutes, 120);
        assert_eq!(day.schedule.as_ref().unwrap().start_minutes, 510);
        assert_eq!(day.note.as_deref(), Some("a note"));
        assert_eq!(parsed.task_presets.len(), 1);
        assert_eq!(parsed.task_presets[0].title, "Standup");
        assert_eq!(parsed.commutes.len(), 1);
        assert_eq!(parsed.commutes[0].segments[0].mode_id, "bike");
        let s = parsed.settings.unwrap();
        assert_eq!(s.co2.grid_country, "BE");
        assert_eq!(s.default_start_minutes, 480);
    }

    #[test]
    fn options_off_omit_categories() {
        let codec = JsonProfileBundleCodec::new();
        let path = temp_path("opts");
        let opts = BundleOptions {
            include_days: false,
            include_trips: true,
            include_work_hours: true,
            include_notes: true,
            include_task_presets: false,
            include_commutes: false,
            include_settings: false,
        };
        codec.write(&sample_data(), &opts, &path).unwrap();
        let parsed = codec.read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(parsed.days.is_empty());
        assert!(parsed.task_presets.is_empty());
        assert!(parsed.commutes.is_empty());
        assert!(parsed.settings.is_none());
    }

    #[test]
    fn rejects_unknown_format_tag() {
        let path = temp_path("badformat");
        std::fs::write(
            &path,
            r#"{"format":"something-else","version":1,"exportedAt":0,"app":"x","profile":{"firstName":"a","lastName":"b","enterprise":"c"}}"#,
        )
        .unwrap();
        let res = JsonProfileBundleCodec::new().read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(res, Err(DomainError::Validation(_))));
    }

    #[test]
    fn rejects_invalid_json() {
        let path = temp_path("badjson");
        std::fs::write(&path, "{ not json").unwrap();
        let res = JsonProfileBundleCodec::new().read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(res, Err(DomainError::Validation(_))));
    }

    #[test]
    fn rejects_unknown_presence_kind() {
        let path = temp_path("badkind");
        std::fs::write(
            &path,
            r#"{"format":"basic-presence-profile","version":1,"exportedAt":0,"app":"x","profile":{"firstName":"a","lastName":"b","enterprise":"c"},"days":[{"day":0,"type":"teleport","isEstimated":false,"createdAt":0,"updatedAt":0}]}"#,
        )
        .unwrap();
        let res = JsonProfileBundleCodec::new().read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(res, Err(DomainError::Validation(_))));
    }

    #[test]
    fn rejects_a_file_over_the_size_cap() {
        let path = temp_path("too_large");
        let file = std::fs::File::create(&path).unwrap();
        // Sparse file: no real disk usage for the test.
        file.set_len(MAX_BUNDLE_BYTES + 1).unwrap();
        let res = JsonProfileBundleCodec::new().read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(res, Err(DomainError::Validation(m)) if m == "file too large"));
    }

    #[test]
    fn tolerates_utf8_bom() {
        let codec = JsonProfileBundleCodec::new();
        let path = temp_path("bom");
        codec.write(&sample_data(), &all_options(), &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("\u{feff}{content}")).unwrap();
        let parsed = codec.read(&path);
        let _ = std::fs::remove_file(&path);
        assert!(parsed.is_ok());
    }
}
