-- Versioned key material (audit 2026-10, lot 1.3).
--   key_version  wrap format of wrapped_dek / wrapped_mac_key:
--                1 = XChaCha20-Poly1305 without associated data (legacy),
--                2 = with associated data binding role + account id.
--   kdf_*        Argon2id parameters the KEK was derived with. NULL for legacy
--                accounts, whose KEK used the auth-hash parameters of the time.
-- Legacy rows are upgraded transparently at the next successful login (the
-- password is needed to re-derive the KEK).
ALTER TABLE account ADD COLUMN key_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE account ADD COLUMN kdf_m_cost INTEGER;
ALTER TABLE account ADD COLUMN kdf_t_cost INTEGER;
ALTER TABLE account ADD COLUMN kdf_p_cost INTEGER;
