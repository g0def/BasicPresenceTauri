use std::path::{Path, PathBuf};

/// Argon2id cost parameters (used for both the auth hash and the KEK derivation).
#[derive(Debug, Clone)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// Authentication / session policy.
#[derive(Debug, Clone)]
pub struct AuthPolicy {
    /// Absolute session lifetime in milliseconds.
    pub session_ttl_ms: i64,
    /// Number of consecutive failures before the account is locked.
    pub max_attempts: i64,
    /// Lockout duration in milliseconds once `max_attempts` is reached.
    pub lockout_ms: i64,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub keystore_path: PathBuf,
    pub vault_path: PathBuf,
    pub argon2: Argon2Params,
    pub auth: AuthPolicy,
}

impl AppConfig {
    /// Default configuration rooted at the given app-data directory.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            keystore_path: data_dir.join("keystore.db"),
            vault_path: data_dir.join("vault.db"),
            // OWASP Argon2id "46 MiB" profile (bank-grade). Tune to ~0.5-1s/hash
            // on the target hardware.
            argon2: Argon2Params {
                m_cost: 47104,
                t_cost: 1,
                p_cost: 1,
            },
            auth: AuthPolicy {
                session_ttl_ms: 15 * 60 * 1000, // 15 minutes, absolute
                max_attempts: 5,
                lockout_ms: 5 * 60 * 1000, // 5 minutes
            },
        }
    }
}
