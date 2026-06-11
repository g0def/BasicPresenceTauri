-- Day start/end times for the work-hours page. One optional row per encoded
-- work day; with duration-stacked entries, the start time anchors each task
-- to a real clock time (entry N starts at day start + sum of previous
-- durations).

CREATE TABLE IF NOT EXISTS work_day_schedule (
    presence_id   TEXT PRIMARY KEY,            -- FK -> presence(id)
    start_minutes INTEGER NOT NULL CHECK (start_minutes BETWEEN 0 AND 1439),
    end_minutes   INTEGER NOT NULL CHECK (end_minutes BETWEEN 1 AND 1440),
    CHECK (end_minutes > start_minutes),
    FOREIGN KEY (presence_id) REFERENCES presence(id) ON DELETE CASCADE
);

-- Same cleanup rule as work_entry: a day that stops being a work day loses
-- its schedule (presence upserts keep the row id, so the cascade never fires
-- on a type flip).
CREATE TRIGGER IF NOT EXISTS trg_work_schedule_presence_type
AFTER UPDATE OF type ON presence
WHEN NEW.type NOT IN ('office','remote')
BEGIN
    DELETE FROM work_day_schedule WHERE presence_id = NEW.id;
END;
