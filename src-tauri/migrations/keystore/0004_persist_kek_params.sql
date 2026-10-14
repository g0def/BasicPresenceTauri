-- Persist the Argon2id cost parameters used to derive this account's KEK.
--
-- The authentication hash carries its own parameters inside the PHC string, so
-- it survives a configuration change. The KEK derivation did not: the costs came
-- from `AppConfig` at call time, which means raising `m_cost` — the normal
-- response to faster hardware — would derive a *different* KEK from the same
-- password, fail to unwrap the DEK, and leave every existing vault permanently
-- unreadable. Nothing in the code said so.
--
-- Storing them per account makes the parameters a property of the stored key
-- material rather than of the running build, so the configuration can move
-- without stranding anyone (and a future re-derivation can upgrade accounts one
-- at a time).
--
-- Defaults are the values shipped up to now (OWASP 46 MiB profile), which is
-- what existing rows were necessarily created with.
ALTER TABLE account ADD COLUMN kek_m_cost INTEGER NOT NULL DEFAULT 47104;
ALTER TABLE account ADD COLUMN kek_t_cost INTEGER NOT NULL DEFAULT 1;
ALTER TABLE account ADD COLUMN kek_p_cost INTEGER NOT NULL DEFAULT 1;
