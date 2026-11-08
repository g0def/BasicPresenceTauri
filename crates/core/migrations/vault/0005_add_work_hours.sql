-- Work-hours encoding (in the ENCRYPTED vault). Adds:
--   * reusable task presets per profile (title, description, default duration,
--     color) shown in the work-hours page's left panel,
--   * per-day ordered work entries attached to a presence day. Entries
--     snapshot the preset's title/description/color at encode time so editing
--     or deleting a preset never rewrites past days.

-- ── Saved task presets ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS task_preset (
    id              TEXT PRIMARY KEY,           -- UUID v7
    profile_id      TEXT NOT NULL,              -- FK -> profile(id)
    title           TEXT NOT NULL,
    description     TEXT,
    default_minutes INTEGER NOT NULL CHECK (default_minutes BETWEEN 5 AND 480),
    color           TEXT NOT NULL,              -- '#RRGGBB'
    created_at      INTEGER NOT NULL,           -- epoch ms
    updated_at      INTEGER NOT NULL,           -- epoch ms
    FOREIGN KEY (profile_id) REFERENCES profile(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_task_preset_profile ON task_preset(profile_id);

-- ── Per-day work entries ─────────────────────────────────────────────────────

-- Duration-stacked tasks of a work day (no explicit start times): `position`
-- is the chronological order, the day total is the sum of `minutes`.
CREATE TABLE IF NOT EXISTS work_entry (
    id           TEXT PRIMARY KEY,              -- UUID v7
    presence_id  TEXT NOT NULL,                 -- FK -> presence(id)
    title        TEXT NOT NULL,                 -- snapshotted from preset / custom
    description  TEXT,
    minutes      INTEGER NOT NULL CHECK (minutes BETWEEN 5 AND 480),
    color        TEXT NOT NULL,                 -- '#RRGGBB'
    position     INTEGER NOT NULL,              -- 0-based ordering within the day
    FOREIGN KEY (presence_id) REFERENCES presence(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_entry_presence ON work_entry(presence_id, position);

-- Presence upserts keep the same row id when only the type changes, so the FK
-- cascade alone never fires for office→vacation/holiday flips. Work hours only
-- make sense on work days: drop them whenever a day stops being one.
CREATE TRIGGER IF NOT EXISTS trg_work_entry_presence_type
AFTER UPDATE OF type ON presence
WHEN NEW.type NOT IN ('office','remote')
BEGIN
    DELETE FROM work_entry WHERE presence_id = NEW.id;
END;
