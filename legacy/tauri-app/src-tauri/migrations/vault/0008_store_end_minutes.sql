-- Store the day's end time alongside the start. It stays DERIVED — the
-- backend recomputes it (start + sum of the day's entry durations) on every
-- entries/start save — but persisting it makes the stored schedule complete
-- and lets the frontend display exactly what the vault holds. Values may
-- exceed 1440 when a day's tasks run past midnight (display wraps).

ALTER TABLE work_day_schedule ADD COLUMN end_minutes INTEGER NOT NULL DEFAULT 0;

UPDATE work_day_schedule
SET end_minutes = start_minutes + (
    SELECT COALESCE(SUM(minutes), 0)
    FROM work_entry
    WHERE work_entry.presence_id = work_day_schedule.presence_id
);
