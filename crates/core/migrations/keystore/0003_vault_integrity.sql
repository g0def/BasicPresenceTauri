-- Vault tamper-evidence state, moved out of the (unauthenticated) filesystem
-- sidecars `vault.db.hmac` / `vault.db.dirty` into the keystore, which is sealed
-- at rest by the per-device key. An attacker who can write next to the vault can
-- delete or create sidecar files at will, but cannot forge this row.
--
-- Singleton row (id = 1):
--   baseline_mac  HMAC-SHA256 (v2 framing) of the vault at its last clean close;
--                 NULL until the first clean close on this version.
--   session_open  1 while a vault session is in progress (set BEFORE the vault
--                 is opened and migrated), 0 after a clean close. A mismatch
--                 with session_open = 1 is a crash, not tampering.
CREATE TABLE vault_integrity (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    baseline_mac BLOB,
    session_open INTEGER NOT NULL DEFAULT 0 CHECK (session_open IN (0, 1)),
    updated_at   INTEGER NOT NULL DEFAULT 0
);

INSERT INTO vault_integrity (id) VALUES (1);
