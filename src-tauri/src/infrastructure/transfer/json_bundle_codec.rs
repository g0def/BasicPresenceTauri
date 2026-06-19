//! JSON implementation of the [`ProfileBundleCodec`] port: maps the domain
//! bundle data to/from a versioned, camelCase JSON document and reads/writes it
//! to disk. The serde wire structs are private to this adapter — they are the
//! file format, not a frontend contract (the commands return separate DTOs).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::entities::commute::{Commute, CommuteSegment};
use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::profile::Profile;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::entities::trip::Trip;
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::services::profile_bundle::{
    BundleDay, BundleOptions, ParsedBundle, ProfileBundleCodec, ProfileBundleData,
    SUPPORTED_BUNDLE_VERSION,
};

/// Format tag stamped into every file; a file with a different tag is rejected.
const BUNDLE_FORMAT: &str = "basic-presence-profile";

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
        std::fs::write(path, json).map_err(|e| DomainError::Export(format!("write file: {e}")))?;
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<ParsedBundle, DomainError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| DomainError::Validation(format!("cannot read file: {e}")))?;
        // Strip a leading UTF-8 BOM if present (some editors add one).
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let file: BundleFile = serde_json::from_str(text)
            .map_err(|e| DomainError::Validation(format!("invalid profile file: {e}")))?;
        if file.format != BUNDLE_FORMAT {
            return Err(DomainError::Validation(
                "unrecognized file format".to_string(),
            ));
        }
        file.into_domain()
    }
}

// --- Wire structs (the on-disk JSON format). Private to this adapter. ---

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundleFile {
    format: String,
    version: u32,
    exported_at: i64,
    app: String,
    profile: FileProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    settings: Option<FileSettings>,
    #[serde(default)]
    task_presets: Vec<FileTaskPreset>,
    #[serde(default)]
    commutes: Vec<FileCommute>,
    #[serde(default)]
    days: Vec<FileDay>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileProfile {
    first_name: String,
    last_name: String,
    enterprise: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    poste: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileSettings {
    default_start_minutes: i64,
    note_font: String,
    cell_display_mode: String,
    grid_country: String,
    default_car_occupancy: i64,
    include_radiative_forcing: bool,
    count_building_energy: bool,
    working_days_per_year: i64,
    factor_year: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileTaskPreset {
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    default_minutes: i64,
    color: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileCommuteSegment {
    mode_id: String,
    distance_km: f64,
    occupants: i64,
    position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileCommute {
    name: String,
    round_trip: bool,
    #[serde(default)]
    segments: Vec<FileCommuteSegment>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileTrip {
    mode_id: String,
    distance_km: f64,
    round_trip: bool,
    occupants: i64,
    co2_kg: f64,
    is_estimated: bool,
    factor_year: i32,
    position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileWorkEntry {
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    minutes: i64,
    color: String,
    position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileSchedule {
    start_minutes: i64,
    end_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDay {
    day: i64,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    co2_kg: Option<f64>,
    is_estimated: bool,
    created_at: i64,
    updated_at: i64,
    #[serde(default)]
    trips: Vec<FileTrip>,
    #[serde(default)]
    work_entries: Vec<FileWorkEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schedule: Option<FileSchedule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

// --- Domain → wire (write), applying the category options. ---

impl BundleFile {
    fn from_domain(data: &ProfileBundleData, opts: &BundleOptions) -> Self {
        let days = if opts.include_days {
            data.days
                .iter()
                .map(|d| FileDay {
                    day: d.presence.day,
                    kind: d.presence.kind.as_str().to_string(),
                    co2_kg: d.presence.co2_kg,
                    is_estimated: d.presence.is_estimated,
                    created_at: d.presence.created_at,
                    updated_at: d.presence.updated_at,
                    trips: if opts.include_trips {
                        d.trips
                            .iter()
                            .map(|t| FileTrip {
                                mode_id: t.mode_id.clone(),
                                distance_km: t.distance_km,
                                round_trip: t.round_trip,
                                occupants: t.occupants,
                                co2_kg: t.co2_kg,
                                is_estimated: t.is_estimated,
                                factor_year: t.factor_year,
                                position: t.position,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    },
                    work_entries: if opts.include_work_hours {
                        d.work_entries
                            .iter()
                            .map(|e| FileWorkEntry {
                                title: e.title.clone(),
                                description: e.description.clone(),
                                minutes: e.minutes,
                                color: e.color.clone(),
                                position: e.position,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    },
                    schedule: if opts.include_work_hours {
                        d.schedule.as_ref().map(|s| FileSchedule {
                            start_minutes: s.start_minutes,
                            end_minutes: s.end_minutes,
                        })
                    } else {
                        None
                    },
                    note: if opts.include_notes {
                        d.note.clone()
                    } else {
                        None
                    },
                })
                .collect()
        } else {
            Vec::new()
        };

        let task_presets = if opts.include_task_presets {
            data.task_presets
                .iter()
                .map(|p| FileTaskPreset {
                    title: p.title.clone(),
                    description: p.description.clone(),
                    default_minutes: p.default_minutes,
                    color: p.color.clone(),
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                })
                .collect()
        } else {
            Vec::new()
        };

        let commutes = if opts.include_commutes {
            data.commutes
                .iter()
                .map(|c| FileCommute {
                    name: c.name.clone(),
                    round_trip: c.round_trip,
                    segments: c
                        .segments
                        .iter()
                        .map(|s| FileCommuteSegment {
                            mode_id: s.mode_id.clone(),
                            distance_km: s.distance_km,
                            occupants: s.occupants,
                            position: s.position,
                        })
                        .collect(),
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                })
                .collect()
        } else {
            Vec::new()
        };

        let settings = if opts.include_settings {
            data.settings.as_ref().map(|s| FileSettings {
                default_start_minutes: s.default_start_minutes,
                note_font: s.note_font.clone(),
                cell_display_mode: s.cell_display_mode.clone(),
                grid_country: s.co2.grid_country.clone(),
                default_car_occupancy: s.co2.default_car_occupancy,
                include_radiative_forcing: s.co2.include_radiative_forcing,
                count_building_energy: s.co2.count_building_energy,
                working_days_per_year: s.co2.working_days_per_year,
                factor_year: s.co2.factor_year,
            })
        } else {
            None
        };

        Self {
            format: BUNDLE_FORMAT.to_string(),
            version: SUPPORTED_BUNDLE_VERSION,
            exported_at: data.exported_at,
            app: env!("CARGO_PKG_VERSION").to_string(),
            profile: FileProfile {
                first_name: data.profile.first_name.clone(),
                last_name: data.profile.last_name.clone(),
                enterprise: data.profile.enterprise.clone(),
                poste: data.profile.poste.clone(),
            },
            settings,
            task_presets,
            commutes,
            days,
        }
    }

    // --- Wire → domain (read). Entity ids/owner-ids are left as placeholders
    // (the import use case assigns fresh ones); `kind` is validated here. ---
    fn into_domain(self) -> Result<ParsedBundle, DomainError> {
        let mut days = Vec::with_capacity(self.days.len());
        for d in self.days {
            let kind = PresenceType::parse(&d.kind)?;
            let trips = d
                .trips
                .into_iter()
                .map(|t| Trip {
                    id: String::new(),
                    mode_id: t.mode_id,
                    distance_km: t.distance_km,
                    round_trip: t.round_trip,
                    occupants: t.occupants,
                    co2_kg: t.co2_kg,
                    is_estimated: t.is_estimated,
                    factor_year: t.factor_year,
                    position: t.position,
                })
                .collect();
            let work_entries = d
                .work_entries
                .into_iter()
                .map(|e| WorkEntry {
                    id: String::new(),
                    presence_id: String::new(),
                    title: e.title,
                    description: e.description,
                    minutes: e.minutes,
                    color: e.color,
                    position: e.position,
                })
                .collect();
            let schedule = d.schedule.map(|s| WorkDaySchedule {
                presence_id: String::new(),
                start_minutes: s.start_minutes,
                end_minutes: s.end_minutes,
            });
            days.push(BundleDay {
                presence: Presence {
                    id: String::new(),
                    profile_id: String::new(),
                    day: d.day,
                    kind,
                    co2_kg: d.co2_kg,
                    is_estimated: d.is_estimated,
                    created_at: d.created_at,
                    updated_at: d.updated_at,
                    work_minutes: 0,
                },
                trips,
                work_entries,
                schedule,
                note: d.note,
            });
        }

        let task_presets = self
            .task_presets
            .into_iter()
            .map(|p| TaskPreset {
                id: String::new(),
                profile_id: String::new(),
                title: p.title,
                description: p.description,
                default_minutes: p.default_minutes,
                color: p.color,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect();

        let commutes = self
            .commutes
            .into_iter()
            .map(|c| Commute {
                id: String::new(),
                profile_id: String::new(),
                name: c.name,
                round_trip: c.round_trip,
                segments: c
                    .segments
                    .into_iter()
                    .map(|s| CommuteSegment {
                        id: String::new(),
                        commute_id: String::new(),
                        mode_id: s.mode_id,
                        distance_km: s.distance_km,
                        occupants: s.occupants,
                        position: s.position,
                    })
                    .collect(),
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();

        let settings = self.settings.map(|s| ProfileSettings {
            default_start_minutes: s.default_start_minutes,
            note_font: s.note_font,
            cell_display_mode: s.cell_display_mode,
            co2: Co2Settings {
                grid_country: s.grid_country,
                default_car_occupancy: s.default_car_occupancy,
                include_radiative_forcing: s.include_radiative_forcing,
                count_building_energy: s.count_building_energy,
                working_days_per_year: s.working_days_per_year,
                factor_year: s.factor_year,
            },
        });

        Ok(ParsedBundle {
            version: self.version,
            exported_at: self.exported_at,
            app: self.app,
            profile: Profile {
                id: String::new(),
                first_name: self.profile.first_name,
                last_name: self.profile.last_name,
                enterprise: self.profile.enterprise,
                poste: self.profile.poste,
                created_at: 0,
                updated_at: 0,
            },
            days,
            task_presets,
            commutes,
            settings,
        })
    }
}
