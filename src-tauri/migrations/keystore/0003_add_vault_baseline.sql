-- Monotonic session counter for the vault's at-rest integrity baseline.
--
-- It lives in the keystore because the keystore is sealed by the OS-keychain
-- device key: an attacker holding only the data directory can rewrite
-- `vault.db` and its sidecar, but cannot forge a matching counter here. That is
-- what makes a wholesale rollback of the (vault, sidecar) pair detectable.
--
-- Single row by construction; 0 means "never baselined by this build", which is
-- the only state where a missing or legacy sidecar is accepted.
CREATE TABLE IF NOT EXISTS vault_baseline (
    id      INTEGER PRIMARY KEY CHECK (id = 1),
    counter INTEGER NOT NULL DEFAULT 0 CHECK (counter >= 0)
);

INSERT OR IGNORE INTO vault_baseline (id, counter) VALUES (1, 0);
