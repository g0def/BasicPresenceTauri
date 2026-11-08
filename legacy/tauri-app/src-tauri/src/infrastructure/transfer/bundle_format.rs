//! The on-disk bundle format: versioned, camelCase JSON wire structs and their
//! mapping to/from the domain bundle types. Private to the transfer adapter —
//! the file format is not a frontend contract (the commands return their own
//! DTOs) and no other layer sees these types.

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
    BundleDay, BundleOptions, ParsedBundle, ProfileBundleData, SUPPORTED_BUNDLE_VERSION,
};

/// Format tag stamped into every file; a file with a different tag is rejected.
pub(super) const BUNDLE_FORMAT: &str = "basic-presence-profile";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BundleFile {
    pub(super) format: String,
    pub(super) version: u32,
    pub(super) exported_at: i64,
    pub(super) app: String,
    pub(super) profile: FileProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) settings: Option<FileSettings>,
    #[serde(default)]
    pub(super) task_presets: Vec<FileTaskPreset>,
    #[serde(default)]
    pub(super) commutes: Vec<FileCommute>,
    #[serde(default)]
    pub(super) days: Vec<FileDay>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileProfile {
    pub(super) first_name: String,
    pub(super) last_name: String,
    pub(super) enterprise: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) poste: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileSettings {
    pub(super) default_start_minutes: i64,
    pub(super) note_font: String,
    pub(super) cell_display_mode: String,
    pub(super) grid_country: String,
    pub(super) default_car_occupancy: i64,
    pub(super) include_radiative_forcing: bool,
    pub(super) count_building_energy: bool,
    pub(super) working_days_per_year: i64,
    pub(super) factor_year: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileTaskPreset {
    pub(super) title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
    pub(super) default_minutes: i64,
    pub(super) color: String,
    pub(super) created_at: i64,
    pub(super) updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileCommuteSegment {
    pub(super) mode_id: String,
    pub(super) distance_km: f64,
    pub(super) occupants: i64,
    pub(super) position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileCommute {
    pub(super) name: String,
    pub(super) round_trip: bool,
    #[serde(default)]
    pub(super) segments: Vec<FileCommuteSegment>,
    pub(super) created_at: i64,
    pub(super) updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileTrip {
    pub(super) mode_id: String,
    pub(super) distance_km: f64,
    pub(super) round_trip: bool,
    pub(super) occupants: i64,
    pub(super) co2_kg: f64,
    pub(super) is_estimated: bool,
    pub(super) factor_year: i32,
    pub(super) position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileWorkEntry {
    pub(super) title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
    pub(super) minutes: i64,
    pub(super) color: String,
    pub(super) position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileSchedule {
    pub(super) start_minutes: i64,
    pub(super) end_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileDay {
    pub(super) day: i64,
    #[serde(rename = "type")]
    pub(super) kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) co2_kg: Option<f64>,
    pub(super) is_estimated: bool,
    pub(super) created_at: i64,
    pub(super) updated_at: i64,
    #[serde(default)]
    pub(super) trips: Vec<FileTrip>,
    #[serde(default)]
    pub(super) work_entries: Vec<FileWorkEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) schedule: Option<FileSchedule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) note: Option<String>,
}

// --- Domain → wire (write), applying the category options. ---

impl BundleFile {
    pub(super) fn from_domain(data: &ProfileBundleData, opts: &BundleOptions) -> Self {
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
    pub(super) fn into_domain(self) -> Result<ParsedBundle, DomainError> {
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
