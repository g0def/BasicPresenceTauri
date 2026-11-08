-- CO2 commute footprint (in the ENCRYPTED vault). Adds:
--   * a year-versioned emission-factor referential (never hardcode factors in
--     logic — they live here),
--   * per-country grid overrides for electric modes,
--   * reusable named commute templates (with ordered segments) per profile,
--   * a per-day snapshot of the trips that produced a presence day's CO2,
--   * denormalized co2_kg / is_estimated columns on `presence`.

-- ── Referential ──────────────────────────────────────────────────────────────

-- Emission factors, versioned by reference year. `value`'s meaning depends on
-- `unit`. `category` drives both UI grouping and the radiative-forcing gate
-- (RF applies only to `air`). `is_param` flags modes whose effective value can
-- be overridden by a parameter (the electric grid country).
CREATE TABLE IF NOT EXISTS emission_factor (
    id          TEXT NOT NULL,              -- mode_id, e.g. 'car_petrol'
    year        INTEGER NOT NULL,           -- reference year, e.g. 2025
    label       TEXT NOT NULL,              -- displayed label
    value       REAL NOT NULL,              -- factor value (see unit)
    unit        TEXT NOT NULL CHECK (unit IN
                  ('kgCO2e/veh.km','kgCO2e/passenger.km','kgCO2e/km','kgCO2e/day')),
    category    TEXT NOT NULL CHECK (category IN
                  ('car','active','public_transport','rail','air','building')),
    scope       TEXT,                       -- traceability (usage/amont/fabrication…)
    is_param    INTEGER NOT NULL DEFAULT 0, -- 0/1: value depends on a parameter
    source      TEXT,                       -- ADEME / DEFRA / SNCB …
    PRIMARY KEY (id, year)
);

-- Per-(mode, country) override for electric modes (replaces the base value when
-- the configured grid_country matches). Year-versioned like the factors.
CREATE TABLE IF NOT EXISTS emission_factor_grid_variant (
    mode_id  TEXT NOT NULL,
    year     INTEGER NOT NULL,
    country  TEXT NOT NULL,                 -- 'FR' | 'BE' | 'DE' | 'EU'
    value    REAL NOT NULL,
    PRIMARY KEY (mode_id, country, year),
    FOREIGN KEY (mode_id, year) REFERENCES emission_factor(id, year) ON DELETE CASCADE
);

-- ── Saved commute templates ──────────────────────────────────────────────────

-- A reusable, named commute (e.g. "30 km train + 5 km vélo") owned by a profile.
CREATE TABLE IF NOT EXISTS commute (
    id          TEXT PRIMARY KEY,           -- UUID v7
    profile_id  TEXT NOT NULL,              -- FK -> profile(id)
    name        TEXT NOT NULL,
    round_trip  INTEGER NOT NULL DEFAULT 1, -- 0/1: applies to the whole journey
    created_at  INTEGER NOT NULL,           -- epoch ms
    updated_at  INTEGER NOT NULL,           -- epoch ms
    FOREIGN KEY (profile_id) REFERENCES profile(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_commute_profile ON commute(profile_id);

-- Ordered legs of a saved commute. `distance_km` is one-way; `occupants` only
-- matters for car-category modes (carpool).
CREATE TABLE IF NOT EXISTS commute_segment (
    id           TEXT PRIMARY KEY,          -- UUID v7
    commute_id   TEXT NOT NULL,
    mode_id      TEXT NOT NULL,             -- references an emission_factor.id
    distance_km  REAL NOT NULL CHECK (distance_km >= 0),
    occupants    INTEGER NOT NULL DEFAULT 1 CHECK (occupants >= 1),
    position     INTEGER NOT NULL,          -- 0-based ordering within the commute
    FOREIGN KEY (commute_id) REFERENCES commute(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_commute_segment_commute ON commute_segment(commute_id, position);

-- ── Per-day snapshot (the ticket's "Trip") ───────────────────────────────────

-- Frozen at encode time so a past day's CO2 stays reproducible (R9) even if the
-- referential or a saved commute later changes. Independent of `commute` so a
-- "custom" (un-saved) trip can also be recorded.
CREATE TABLE IF NOT EXISTS presence_trip (
    id           TEXT PRIMARY KEY,          -- UUID v7
    presence_id  TEXT NOT NULL,             -- FK -> presence(id)
    mode_id      TEXT NOT NULL,
    distance_km  REAL NOT NULL CHECK (distance_km >= 0),
    round_trip   INTEGER NOT NULL DEFAULT 0,
    occupants    INTEGER NOT NULL DEFAULT 1 CHECK (occupants >= 1),
    co2_kg       REAL NOT NULL,             -- snapshotted per-segment emission
    is_estimated INTEGER NOT NULL DEFAULT 0,
    factor_year  INTEGER NOT NULL,          -- referential year used (audit/R9)
    position     INTEGER NOT NULL,
    FOREIGN KEY (presence_id) REFERENCES presence(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_presence_trip_presence ON presence_trip(presence_id, position);

-- Denormalized day total on presence (NULL for pre-existing / non-commute days).
ALTER TABLE presence ADD COLUMN co2_kg REAL;
ALTER TABLE presence ADD COLUMN is_estimated INTEGER NOT NULL DEFAULT 0;

-- ── Seed: 2025 referential (ADEME / DEFRA / SNCF / SNCB) ──────────────────────

INSERT OR IGNORE INTO emission_factor (id, year, label, value, unit, category, scope, is_param, source) VALUES
 ('car_petrol',       2025, 'Voiture — essence',                  0.2388, 'kgCO2e/veh.km',       'car',              'usage+amont+fabrication', 0, 'ADEME 2024-2025'),
 ('car_diesel',       2025, 'Voiture — diesel',                   0.2275, 'kgCO2e/veh.km',       'car',              'usage+amont+fabrication', 0, 'ADEME 2024-2025'),
 ('car_average',      2025, 'Voiture — motorisation moyenne',     0.2311, 'kgCO2e/veh.km',       'car',              'usage+amont+fabrication', 0, 'ADEME 2024-2025'),
 ('car_phev',         2025, 'Voiture — hybride rechargeable',     0.1021, 'kgCO2e/veh.km',       'car',              'usage+amont+fabrication', 0, 'ADEME 2024-2025'),
 ('car_ev',           2025, 'Voiture — électrique',               0.1393, 'kgCO2e/veh.km',       'car',              'usage+fabrication',       1, 'ADEME 2024-2025'),
 ('taxi',             2025, 'Taxi / VTC',                         0.2311, 'kgCO2e/veh.km',       'car',              'usage+amont+fabrication', 0, 'ADEME 2024-2025'),
 ('walk',             2025, 'À pied',                             0.0,    'kgCO2e/km',           'active',           'convention',              0, 'ADEME 2024-2025'),
 ('bike',             2025, 'Vélo musculaire',                    0.0,    'kgCO2e/km',           'active',           'convention',              0, 'ADEME 2024-2025'),
 ('ebike',            2025, 'Vélo électrique',                    0.01095,'kgCO2e/km',           'active',           'usage+fabrication',       1, 'ADEME 2024-2025'),
 ('escooter',         2025, 'Trottinette électrique',             0.0249, 'kgCO2e/km',           'active',           'usage+fabrication',       1, 'ADEME 2024-2025'),
 ('scooter_elec',     2025, 'Scooter électrique',                 0.0249, 'kgCO2e/km',           'active',           'usage+fabrication',       1, 'ADEME 2024-2025'),
 ('public_transport', 2025, 'Transport en commun (mix)',          0.04525,'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME 2024-2025'),
 ('bus',              2025, 'Bus urbain',                         0.1515, 'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME 2024-2025'),
 ('coach',            2025, 'Autocar longue distance',            0.0295, 'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME 2024-2025'),
 ('metro_tram',       2025, 'Métro / tram',                       0.005,  'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME 2024-2025'),
 ('train_sncb',       2025, 'Train Intercity (SNCB)',             0.021,  'kgCO2e/passenger.km', 'rail',            'usage+fabrication',       0, 'SNCB'),
 ('train_ter',        2025, 'Train régional (TER)',               0.0299, 'kgCO2e/passenger.km', 'rail',            'usage+fabrication',       0, 'SNCF Open Data'),
 ('train_hs_fr',      2025, 'TGV (France)',                       0.00343,'kgCO2e/passenger.km', 'rail',            'usage+fabrication',       0, 'SNCF Open Data'),
 ('train_eurostar',   2025, 'Eurostar',                           0.0055, 'kgCO2e/passenger.km', 'rail',            'usage+fabrication',       0, 'opérateur'),
 ('train_thalys',     2025, 'Thalys',                             0.0084, 'kgCO2e/passenger.km', 'rail',            'usage+fabrication',       0, 'opérateur'),
 ('plane_domestic',   2025, 'Avion — intérieur / très court',     0.2582, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif',  1, 'DEFRA 2024-2025'),
 ('plane_short',      2025, 'Avion — court-courrier',             0.2582, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif',  1, 'DEFRA 2024-2025'),
 ('plane_medium',     2025, 'Avion — moyen-courrier',             0.1872, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif',  1, 'DEFRA 2024-2025'),
 ('plane_long',       2025, 'Avion — long-courrier',              0.1517, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif',  1, 'DEFRA 2024-2025'),
 ('office_day',       2025, 'Jour au bureau (énergie bâtiment)',  3.5,    'kgCO2e/day',          'building',        'usage',                   0, 'DEFRA 2024-2025'),
 ('home_day',         2025, 'Jour télétravail (énergie domicile)',2.7,    'kgCO2e/day',          'building',        'usage',                   0, 'DEFRA 2024-2025');

-- Electric grid presets for car_ev (kgCO2e/veh.km). BE uses the precise table
-- value 0.1393 (the §3.3 "≈0.14" is an indicative rounding) so the BE default
-- and the BE grid variant agree.
INSERT OR IGNORE INTO emission_factor_grid_variant (mode_id, year, country, value) VALUES
 ('car_ev', 2025, 'FR', 0.09),
 ('car_ev', 2025, 'BE', 0.1393),
 ('car_ev', 2025, 'DE', 0.18),
 ('car_ev', 2025, 'EU', 0.13);
