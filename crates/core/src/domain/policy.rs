//! Security policies: pure configuration values the use cases enforce. They
//! live in the domain (not in `infrastructure::config`) so the application layer
//! depends on nothing outside the domain.

use crate::domain::error::DomainError;

/// Argon2id cost parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost, KiB.
    pub m_cost: u32,
    /// Iterations.
    pub t_cost: u32,
    /// Lanes.
    pub p_cost: u32,
}

impl KdfParams {
    /// Upper bounds accepted for parameters read back from storage (PHC strings,
    /// stored KDF parameters). A tampered keystore must not be able to make the
    /// app allocate gigabytes or spin for minutes on every login attempt.
    pub const MAX_M_COST: u32 = 256 * 1024; // 256 MiB
    pub const MAX_T_COST: u32 = 10;
    pub const MAX_P_COST: u32 = 8;

    pub fn within_bounds(&self) -> bool {
        (1..=Self::MAX_M_COST).contains(&self.m_cost)
            && (1..=Self::MAX_T_COST).contains(&self.t_cost)
            && (1..=Self::MAX_P_COST).contains(&self.p_cost)
    }
}

/// Credential rules (NIST SP 800-63B: length only, no composition rules).
pub const MIN_USERNAME_LEN: usize = 3;
pub const MAX_USERNAME_LEN: usize = 64;
pub const MIN_PASSWORD_LEN: usize = 8;
pub const MAX_PASSWORD_LEN: usize = 1024;

pub fn validate_username(username: &str) -> Result<(), DomainError> {
    let len = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&len) {
        return Err(DomainError::Validation(format!(
            "username must be {MIN_USERNAME_LEN}-{MAX_USERNAME_LEN} characters"
        )));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), DomainError> {
    let len = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&len) {
        return Err(DomainError::Validation(format!(
            "password must be {MIN_PASSWORD_LEN}-{MAX_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

/// Authentication / session policy.
#[derive(Debug, Clone)]
pub struct AuthPolicy {
    /// Absolute session lifetime in milliseconds.
    pub session_ttl_ms: i64,
    /// A session with no backend activity for this long is revoked (and the
    /// vault locked), whatever the frontend does.
    pub idle_timeout_ms: i64,
    /// Number of consecutive failures before the account is locked.
    pub max_attempts: i64,
    /// Lockout duration once `max_attempts` is reached. Every further failure
    /// doubles it (progressive backoff), up to `max_lockout_ms`.
    pub lockout_ms: i64,
    /// Ceiling of the progressive lockout.
    pub max_lockout_ms: i64,
    /// Argon2id parameters used to derive the KEK that wraps the vault keys.
    /// Stronger than the auth-hash parameters: the KEK is what an offline
    /// attacker holding the keystore (and the device key) would brute-force.
    pub kek_kdf: KdfParams,
    /// Parameters the KEK of accounts created before `kek_kdf` existed was
    /// derived with (the auth-hash parameters at the time).
    pub legacy_kek_kdf: KdfParams,
}

impl AuthPolicy {
    /// Lockout to apply after `failed_attempts` consecutive failures, if any:
    /// `lockout_ms × 2^(failed_attempts − max_attempts)`, capped.
    pub fn lockout_for(&self, failed_attempts: i64) -> Option<i64> {
        if failed_attempts < self.max_attempts {
            return None;
        }
        let doublings = (failed_attempts - self.max_attempts).clamp(0, 30) as u32;
        let ms = self.lockout_ms.saturating_mul(1i64 << doublings);
        Some(ms.min(self.max_lockout_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> AuthPolicy {
        let kdf = KdfParams {
            m_cost: 64,
            t_cost: 1,
            p_cost: 1,
        };
        AuthPolicy {
            session_ttl_ms: 1,
            idle_timeout_ms: 1,
            max_attempts: 5,
            lockout_ms: 5 * 60_000,
            max_lockout_ms: 24 * 3_600_000,
            kek_kdf: kdf,
            legacy_kek_kdf: kdf,
        }
    }

    #[test]
    fn lockout_starts_at_the_threshold_and_doubles_up_to_the_cap() {
        let p = policy();
        assert_eq!(p.lockout_for(4), None);
        assert_eq!(p.lockout_for(5), Some(5 * 60_000));
        assert_eq!(p.lockout_for(6), Some(10 * 60_000));
        assert_eq!(p.lockout_for(7), Some(20 * 60_000));
        assert_eq!(p.lockout_for(40), Some(24 * 3_600_000));
        assert_eq!(p.lockout_for(i64::MAX), Some(24 * 3_600_000));
    }

    #[test]
    fn stored_parameters_are_bounded() {
        let ok = KdfParams {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        };
        assert!(ok.within_bounds());
        assert!(
            !KdfParams {
                m_cost: 4 * 1024 * 1024,
                ..ok
            }
            .within_bounds()
        );
        assert!(!KdfParams { t_cost: 0, ..ok }.within_bounds());
    }
}
