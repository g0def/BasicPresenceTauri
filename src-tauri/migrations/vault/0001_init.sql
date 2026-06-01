-- Vault (ENCRYPTED with the DEK). Holds the confidential data.
-- Presence / transport-mode / CO2 tables will be added here in later iterations.
-- For now a tiny meta table proves the encrypted DB opens and is writable.
CREATE TABLE IF NOT EXISTS vault_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
