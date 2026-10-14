-- Keystore: holds auth credentials + wrapped key material.
--
-- HISTORICAL NOTE: this file shipped when the keystore was a plaintext database,
-- on the grounds that every field inside is already protected (Argon2id hash,
-- wrapped DEK). It is no longer plaintext — the store is sealed at rest by a
-- per-device key from the OS keychain (see keystore_bootstrap.rs), which is what
-- stops an offline brute-force of a stolen file. Migrations are immutable
-- history, so the original claim is corrected here rather than rewritten.
CREATE TABLE IF NOT EXISTS account (
    id              TEXT PRIMARY KEY,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    wrapped_dek     BLOB NOT NULL,
    kek_salt        BLOB NOT NULL,
    dek_nonce       BLOB NOT NULL,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until    INTEGER,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);
