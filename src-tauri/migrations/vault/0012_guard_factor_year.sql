-- Guard `profile_settings.factor_year` against vintages that do not exist in
-- the referential.
--
-- It was the only column of the table without a CHECK, and the only settings
-- field the application-side validator skipped — while that same validator is
-- what the (untrusted) bundle import relies on. An unknown year makes
-- `load_factor_maps` return empty maps, and `Co2Calculator` then resolves every
-- trip to 0.0 kgCO2e: a silently null carbon report, snapshotted per day and
-- therefore not repaired by fixing the setting afterwards.
--
-- A plain CHECK cannot be added to an existing SQLite table without rebuilding
-- it, and hard-coding the years would need editing on every new vintage. Two
-- triggers referencing `emission_factor` stay correct on their own.

-- Repair rows that are already poisoned, so nobody is left stuck on a vintage
-- the guard would now reject. Falls back to the most recent seeded year.
UPDATE profile_settings
   SET factor_year = (SELECT MAX(year) FROM emission_factor)
 WHERE factor_year NOT IN (SELECT DISTINCT year FROM emission_factor)
   AND EXISTS (SELECT 1 FROM emission_factor);

CREATE TRIGGER IF NOT EXISTS profile_settings_factor_year_insert
BEFORE INSERT ON profile_settings
FOR EACH ROW
WHEN NOT EXISTS (SELECT 1 FROM emission_factor WHERE year = NEW.factor_year)
BEGIN
    SELECT RAISE(ABORT, 'factor_year has no matching emission factors');
END;

CREATE TRIGGER IF NOT EXISTS profile_settings_factor_year_update
BEFORE UPDATE OF factor_year ON profile_settings
FOR EACH ROW
WHEN NOT EXISTS (SELECT 1 FROM emission_factor WHERE year = NEW.factor_year)
BEGIN
    SELECT RAISE(ABORT, 'factor_year has no matching emission factors');
END;
