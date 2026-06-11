-- The day's end time is now derived (start + sum of task durations) instead
-- of user-entered, so the schedule row keeps only the start. Rebuild the
-- table without end_minutes (it carries CHECK constraints, so DROP COLUMN is
-- not an option), preserving existing start times. The cleanup trigger (on
-- `presence`) references this table by name, so drop it around the swap and
-- recreate it after — a dangling reference would break the schema reparse
-- that ALTER TABLE RENAME performs.

DROP TRIGGER IF EXISTS trg_work_schedule_presence_type;

CREATE TABLE IF NOT EXISTS work_day_schedule_new (
    presence_id   TEXT PRIMARY KEY,            -- FK -> presence(id)
    start_minutes INTEGER NOT NULL CHECK (start_minutes BETWEEN 0 AND 1439),
    FOREIGN KEY (presence_id) REFERENCES presence(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO work_day_schedule_new (presence_id, start_minutes)
  SELECT presence_id, start_minutes FROM work_day_schedule;

DROP TABLE work_day_schedule;
ALTER TABLE work_day_schedule_new RENAME TO work_day_schedule;

CREATE TRIGGER IF NOT EXISTS trg_work_schedule_presence_type
AFTER UPDATE OF type ON presence
WHEN NEW.type NOT IN ('office','remote')
BEGIN
    DELETE FROM work_day_schedule WHERE presence_id = NEW.id;
END;
