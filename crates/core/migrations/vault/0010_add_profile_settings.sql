-- Per-profile settings (in the encrypted vault). One optional row per profile;
-- the repository falls back to typed defaults when a profile has no row (lazy
-- default, mirroring co2_settings_repository::load). These columns supersede the
-- global `vault_meta.co2_config` blob for presence CO2 computation. The column
-- defaults equal Co2Settings::default() / the frontend's prior localStorage
-- defaults, so existing profiles keep their current behaviour.
CREATE TABLE IF NOT EXISTS profile_settings (
    profile_id                TEXT PRIMARY KEY,

    -- "heure de départ par défaut": seeded into work_day_schedule at presence
    -- creation for office/remote days. Same clock-time domain as the schedule.
    default_start_minutes     INTEGER NOT NULL DEFAULT 510   -- 08:30
                                CHECK (default_start_minutes BETWEEN 0 AND 1439),

    -- UI prefs that move from device-level localStorage to per-profile storage.
    note_font                 TEXT NOT NULL DEFAULT 'sans'
                                CHECK (note_font IN ('sans','serif','mono')),
    cell_display_mode         TEXT NOT NULL DEFAULT 'co2'
                                CHECK (cell_display_mode IN ('co2','hours')),

    -- CO2 config (typed columns mirroring the camelCase co2_config blob and the
    -- Co2Settings::default() values). grid_country was previously validated
    -- nowhere; the CHECK closes that silent-wrong-footprint gap.
    grid_country              TEXT NOT NULL DEFAULT 'BE'
                                CHECK (grid_country IN ('FR','BE','DE','EU')),
    default_car_occupancy     INTEGER NOT NULL DEFAULT 1   CHECK (default_car_occupancy >= 1),
    include_radiative_forcing INTEGER NOT NULL DEFAULT 1   CHECK (include_radiative_forcing IN (0,1)),
    count_building_energy     INTEGER NOT NULL DEFAULT 0   CHECK (count_building_energy IN (0,1)),
    working_days_per_year     INTEGER NOT NULL DEFAULT 220 CHECK (working_days_per_year >= 0),
    factor_year               INTEGER NOT NULL DEFAULT 2025,

    FOREIGN KEY (profile_id) REFERENCES profile(id) ON DELETE CASCADE
);

-- Backfill: every existing profile keeps the current behaviour. The column
-- defaults == Co2Settings::default(), which is exactly what the global CO2 path
-- used when the blob was absent (no UI ever wrote co2_config). No-op on a fresh
-- install (no profiles yet).
INSERT OR IGNORE INTO profile_settings (profile_id) SELECT id FROM profile;
