-- Minimal security journal (audit finding I4). Lives in the keystore, which is
-- sealed at rest by the per-device key. Records WHAT happened and WHEN — never
-- any business content — and is capped to the most recent rows by the app.
--   kind  one of: login, account_locked, unclean_shutdown, tamper_detected,
--         tamper_tolerated, password_changed
CREATE TABLE security_event (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    kind  TEXT NOT NULL,
    at_ms INTEGER NOT NULL
);
CREATE INDEX idx_security_event_at ON security_event(at_ms);
