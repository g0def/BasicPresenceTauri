-- CO2 referential, millésime 2026 (corrigé) — voir documentation/calcul-impact-co2.md §8.
--
-- POURQUOI un nouveau millésime plutôt qu'un UPDATE du 2025 : le référentiel est
-- versionné par `year` exprès pour la reproductibilité (R9). Les jours déjà encodés
-- conservent leur `co2_kg` figé (table `presence_trip`, avec `factor_year`), donc
-- éditer les valeurs 2025 en place casserait la reproductibilité des footprints
-- passés. On seede donc un référentiel 2026 corrigé et on bascule le défaut.
--
-- CE QUI CHANGE vs 2025 (tout le reste est identique) :
--   * Aérien : valeurs DEFRA/DESNZ 2024-2025 (avec forçage radiatif 1,7, abaissé de
--     1,9 en juin 2023). domestic/court/long distingués ; medium = interpolation.
--     → le multiplicateur RF est résolu par année dans co2_calculator.rs (2026 ⇒ 1,7).
--   * Bus urbain 0,1515 → 0,113 (valeur ADEME représentative, périmètre usage).
--   * Métro/tram 0,005 → 0,0044 (ADEME Impact CO2).
--   * TER 0,0299 → 0,0238 et TGV 0,00343 → 0,0035 (SNCF, périmètre complet 2024).
--   * Eurostar 0,0055 → 0,006 (facteur officiel DEFRA cité par Eurostar).
--   * Thalys 0,0084 → 0,0069 (estimation SNCF 2023 / Eurostar Group).
--   * Voiture électrique : variante BE 0,1393 → 0,11 (cohérente avec le mix réseau
--     belge ~135 gCO2/kWh ; l'ancien 0,1393 surestimait la Belgique de ~20-25 %).
--     La valeur de base car_ev suit (alignée sur le défaut BE).
--   * Sources clarifiées : office_day = CIBSE TM46 / Circular Ecology (PAS DEFRA) ;
--     transport en commun (mix) = moyenne pondérée interne (pas un FE ADEME publié) ;
--     scooter_elec = aligné sur la trottinette (pas de FE ADEME dédié).
--
-- INCHANGÉ (valeurs verrouillées par les tests ou exactes vs source) : voitures
-- thermiques, vélo/marche/VAE/trottinette, autocar, train SNCB.

-- ── Référentiel 2026 ─────────────────────────────────────────────────────────

INSERT OR IGNORE INTO emission_factor (id, year, label, value, unit, category, scope, is_param, source) VALUES
 ('car_petrol',       2026, 'Voiture — essence',                  0.2388, 'kgCO2e/veh.km',       'car',              'usage(WtW)+fabrication',  0, 'ADEME Base Carbone'),
 ('car_diesel',       2026, 'Voiture — diesel',                   0.2275, 'kgCO2e/veh.km',       'car',              'usage(WtW)+fabrication',  0, 'ADEME Base Carbone'),
 ('car_average',      2026, 'Voiture — motorisation moyenne',     0.2311, 'kgCO2e/veh.km',       'car',              'usage(WtW)+fabrication',  0, 'ADEME Base Carbone'),
 ('car_phev',         2026, 'Voiture — hybride rechargeable',     0.1021, 'kgCO2e/veh.km',       'car',              'usage(hyp. élec. majoritaire)+fabrication', 0, 'ADEME (estimation usage électrique)'),
 ('car_ev',           2026, 'Voiture — électrique',               0.11,   'kgCO2e/veh.km',       'car',              'usage(mix réseau)+fabrication', 1, 'ADEME + mix réseau (Ember/OWID)'),
 ('taxi',             2026, 'Taxi / VTC',                         0.2311, 'kgCO2e/veh.km',       'car',              'usage(WtW)+fabrication',  0, 'ADEME (proxy voiture moyenne)'),
 ('walk',             2026, 'À pied',                             0.0,    'kgCO2e/km',           'active',           'convention',              0, 'ADEME Impact CO2'),
 ('bike',             2026, 'Vélo musculaire',                    0.0,    'kgCO2e/km',           'active',           'convention',              0, 'ADEME Impact CO2'),
 ('ebike',            2026, 'Vélo électrique',                    0.01095,'kgCO2e/km',           'active',           'usage+fabrication',       1, 'ADEME Impact CO2'),
 ('escooter',         2026, 'Trottinette électrique',             0.0249, 'kgCO2e/km',           'active',           'usage+fabrication',       1, 'ADEME Impact CO2'),
 ('scooter_elec',     2026, 'Scooter électrique',                 0.0249, 'kgCO2e/km',           'active',           'usage+fabrication',       1, 'aligné sur trottinette (pas de FE dédié)'),
 ('public_transport', 2026, 'Transport en commun (mix)',          0.04525,'kgCO2e/passenger.km', 'public_transport','usage (moyenne pondérée)', 0, 'moyenne pondérée interne (modes ADEME)'),
 ('bus',              2026, 'Bus urbain',                         0.113,  'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME / notre-environnement.gouv.fr'),
 ('coach',            2026, 'Autocar longue distance',            0.0295, 'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME / SNCF InfoGES 2024'),
 ('metro_tram',       2026, 'Métro / tram',                       0.0044, 'kgCO2e/passenger.km', 'public_transport','usage',                   0, 'ADEME Impact CO2'),
 ('train_sncb',       2026, 'Train Intercity (SNCB)',             0.021,  'kgCO2e/passenger.km', 'rail',            'usage(WtW traction)',     0, 'SNCB'),
 ('train_ter',        2026, 'Train régional (TER)',               0.0238, 'kgCO2e/passenger.km', 'rail',            'usage+fabrication+maintenance', 0, 'SNCF Open Data (périmètre complet 2024)'),
 ('train_hs_fr',      2026, 'TGV (France)',                       0.0035, 'kgCO2e/passenger.km', 'rail',            'usage+fabrication+maintenance', 0, 'SNCF Open Data (périmètre complet 2024)'),
 ('train_eurostar',   2026, 'Eurostar',                           0.006,  'kgCO2e/passenger.km', 'rail',            'usage',                   0, 'DEFRA (cité par Eurostar)'),
 ('train_thalys',     2026, 'Thalys',                             0.0069, 'kgCO2e/passenger.km', 'rail',            'usage (estimation)',      0, 'SNCF 2023 / Eurostar Group (estimation)'),
 ('plane_domestic',   2026, 'Avion — intérieur / très court',     0.2293, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif (1,7)', 1, 'DEFRA/DESNZ 2024'),
 ('plane_short',      2026, 'Avion — court-courrier',             0.1258, 'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif (1,7)', 1, 'DEFRA/DESNZ 2024'),
 ('plane_medium',     2026, 'Avion — moyen-courrier',             0.121,  'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif (1,7) — interpolé', 1, 'DEFRA/DESNZ 2024 (interpolation court/long)'),
 ('plane_long',       2026, 'Avion — long-courrier',              0.117,  'kgCO2e/passenger.km', 'air',             'usage+forçage radiatif (1,7)', 1, 'DEFRA/DESNZ 2024'),
 ('office_day',       2026, 'Jour au bureau (énergie bâtiment)',  3.5,    'kgCO2e/day',          'building',        'usage (bâtiment, hors trajet)', 0, 'CIBSE TM46 / Circular Ecology (Univ. Exeter)'),
 ('home_day',         2026, 'Jour télétravail (énergie domicile)',2.7,    'kgCO2e/day',          'building',        'usage (Homeworking, hors trajet)', 0, 'DEFRA/DESNZ (Homeworking)');

-- Variantes réseau électrique 2026 (kgCO2e/veh.km). BE abaissé à 0,11 pour refléter
-- le mix réseau belge réel (~135 gCO2/kWh) ; FR/DE/EU inchangés.
INSERT OR IGNORE INTO emission_factor_grid_variant (mode_id, year, country, value) VALUES
 ('car_ev', 2026, 'FR', 0.09),
 ('car_ev', 2026, 'BE', 0.11),
 ('car_ev', 2026, 'DE', 0.18),
 ('car_ev', 2026, 'EU', 0.13);

-- ── Bascule du défaut ────────────────────────────────────────────────────────

-- Tout profil existant passe au référentiel 2026 pour ses futurs jours. Les jours
-- déjà encodés conservent leur co2_kg / factor_year figés (presence_trip) — leurs
-- footprints restent reproductibles (R9) ; seuls les jours nouvellement encodés ou
-- ré-édités utilisent 2026. (No-op sur une install neuve : aucun profil encore.)
UPDATE profile_settings SET factor_year = 2026 WHERE factor_year = 2025;
