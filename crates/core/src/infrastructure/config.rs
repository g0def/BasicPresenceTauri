use std::path::{Path, PathBuf};

pub use crate::domain::policy::{AuthPolicy, KdfParams};

/// What to do when the vault's at-rest integrity check fails on a *clean*
/// previous shutdown (an unclean shutdown is always treated as expected).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityPolicy {
    /// Open anyway and re-baseline (for lenient deployments). A crash is NOT
    /// what this covers — an unclean shutdown is always tolerated regardless of
    /// the policy — so allowing a clean-shutdown mismatch means accepting a
    /// genuinely modified vault.
    #[allow(dead_code)] // kept as a configuration escape hatch (used in tests)
    WarnAndAllow,
    /// Refuse to open a tampered vault (default — bank-grade requirement).
    HardFail,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub keystore_path: PathBuf,
    pub vault_path: PathBuf,
    /// Argon2id parameters of the authentication hash (PHC string).
    pub argon2: KdfParams,
    pub auth: AuthPolicy,
    pub integrity: IntegrityPolicy,
}

/// OWASP Argon2id "46 MiB" profile: the authentication hash, and the KEK of
/// accounts created before the KEK got its own, stronger, parameters.
const AUTH_HASH_KDF: KdfParams = KdfParams {
    m_cost: 47104,
    t_cost: 1,
    p_cost: 1,
};

impl AppConfig {
    /// Default configuration rooted at the given app-data directory.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            keystore_path: data_dir.join("keystore.db"),
            vault_path: data_dir.join("vault.db"),
            argon2: AUTH_HASH_KDF,
            auth: AuthPolicy {
                session_ttl_ms: 15 * 60 * 1000, // 15 minutes, absolute
                // Backend-enforced inactivity limit. The UI logs out after 5
                // minutes without user input and pings the backend while the
                // user is active; this is the safety net if the UI does not.
                idle_timeout_ms: 6 * 60 * 1000,
                max_attempts: 5,
                lockout_ms: 5 * 60 * 1000, // 5 minutes, then doubling
                max_lockout_ms: 24 * 60 * 60 * 1000,
                // KEK derivation: RFC 9106 "second recommended" memory size
                // (64 MiB) with 3 passes. The KEK is what an offline attacker
                // holding the keystore (and the device key) would brute-force,
                // so it is worth ~4× the cost of the auth hash. Paid once per
                // login. Tune to ~0.5-1 s on the target hardware.
                kek_kdf: KdfParams {
                    m_cost: 65536,
                    t_cost: 3,
                    p_cost: 1,
                },
                legacy_kek_kdf: AUTH_HASH_KDF,
            },
            // Tamper-evidence must block: a MAC mismatch after a *clean*
            // shutdown has no innocent explanation (crashes go through the
            // session-open path and are tolerated).
            integrity: IntegrityPolicy::HardFail,
        }
    }
}
