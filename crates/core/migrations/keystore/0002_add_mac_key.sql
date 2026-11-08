-- MAC key for vault tamper-evidence, wrapped with the password-derived KEK
-- (same envelope as the DEK). Nullable so rows created before this migration
-- remain valid; they are backfilled on the next successful login.
ALTER TABLE account ADD COLUMN wrapped_mac_key BLOB;
ALTER TABLE account ADD COLUMN mac_key_nonce BLOB;
