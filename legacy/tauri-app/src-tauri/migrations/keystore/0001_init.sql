-- Keystore (NOT encrypted): holds auth credentials + wrapped key material.
-- Safe to store in clear: the password is Argon2id-hashed and the DEK is wrapped.
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
