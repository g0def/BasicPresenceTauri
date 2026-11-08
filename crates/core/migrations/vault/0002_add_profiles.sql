-- Presence profiles (in the ENCRYPTED vault). One owner account may hold several
-- profiles; the active one is tracked in `vault_meta` under `active_profile_id`.
CREATE TABLE IF NOT EXISTS profile (
    id          TEXT PRIMARY KEY,          -- UUID v7
    first_name  TEXT NOT NULL,
    last_name   TEXT NOT NULL,
    enterprise  TEXT NOT NULL,
    poste       TEXT,                       -- optional
    created_at  INTEGER NOT NULL,           -- epoch ms
    updated_at  INTEGER NOT NULL            -- epoch ms
);

CREATE INDEX IF NOT EXISTS idx_profile_created_at ON profile(created_at);
