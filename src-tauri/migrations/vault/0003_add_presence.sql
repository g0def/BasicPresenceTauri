-- Daily presence records (in the ENCRYPTED vault). Each row attaches a presence
-- type to one day of a profile. A profile holds many presences (one-to-many),
-- with at most one presence per day (UNIQUE(profile_id, day)).
CREATE TABLE IF NOT EXISTS presence (
    id          TEXT PRIMARY KEY,           -- UUID v7
    profile_id  TEXT NOT NULL,              -- FK -> profile(id)
    day         INTEGER NOT NULL,           -- epoch ms, UTC midnight of the day
    type        TEXT NOT NULL,              -- 'office' | 'remote' | 'vacation' | 'holiday'
    created_at  INTEGER NOT NULL,           -- epoch ms
    updated_at  INTEGER NOT NULL,           -- epoch ms
    FOREIGN KEY (profile_id) REFERENCES profile(id) ON DELETE CASCADE,
    UNIQUE (profile_id, day)
);

CREATE INDEX IF NOT EXISTS idx_presence_profile_day ON presence(profile_id, day);
