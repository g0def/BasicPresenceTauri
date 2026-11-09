//! Backend errors as shown by the UI: a stable code (the same codes as the
//! former IPC `AppError`) plus an optional argument, localized in Slint by
//! `AppState.error-text`. Internal details never reach the UI.

use basic_presence_core::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiError {
    pub code: &'static str,
    pub arg: String,
}

impl From<DomainError> for UiError {
    fn from(e: DomainError) -> Self {
        let (code, arg) = match e {
            DomainError::InvalidCredentials => ("INVALID_CREDENTIALS", String::new()),
            DomainError::AccountLocked { retry_after_ms } => {
                ("ACCOUNT_LOCKED", (retry_after_ms / 1000).max(1).to_string())
            }
            DomainError::AccountAlreadyExists => ("ACCOUNT_EXISTS", String::new()),
            DomainError::Unauthorized => ("SESSION_EXPIRED", String::new()),
            DomainError::ProfileNotFound | DomainError::NotFound => ("NOT_FOUND", String::new()),
            DomainError::Validation(m) => ("VALIDATION", m),
            DomainError::KeystoreUnrecoverable => ("KEYSTORE_UNRECOVERABLE", String::new()),
            DomainError::VaultTampered => ("VAULT_TAMPERED", String::new()),
            DomainError::Export(_) => ("EXPORT_FAILED", String::new()),
            other => {
                // Storage / crypto / keychain details stay out of the UI.
                eprintln!("BasicPresence: internal error: {other}");
                ("INTERNAL", String::new())
            }
        };
        Self { code, arg }
    }
}

/// Startup failure category, matching `AppState.startup-text`.
pub fn startup_reason(e: &DomainError) -> &'static str {
    match e {
        DomainError::Keychain(_) => "keychain",
        DomainError::KeystoreUnrecoverable => "keystore-lost",
        _ => "generic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_the_former_ipc_contract() {
        assert_eq!(
            UiError::from(DomainError::InvalidCredentials).code,
            "INVALID_CREDENTIALS"
        );
        assert_eq!(
            UiError::from(DomainError::Unauthorized).code,
            "SESSION_EXPIRED"
        );
        let locked = UiError::from(DomainError::AccountLocked {
            retry_after_ms: 1500,
        });
        assert_eq!((locked.code, locked.arg.as_str()), ("ACCOUNT_LOCKED", "1"));
        let locked = UiError::from(DomainError::AccountLocked { retry_after_ms: 10 });
        assert_eq!(locked.arg, "1");
        let v = UiError::from(DomainError::Validation("bad".into()));
        assert_eq!((v.code, v.arg.as_str()), ("VALIDATION", "bad"));
        let internal = UiError::from(DomainError::Storage("secret path".into()));
        assert_eq!((internal.code, internal.arg.as_str()), ("INTERNAL", ""));
    }

    #[test]
    fn startup_reasons() {
        assert_eq!(
            startup_reason(&DomainError::Keychain("x".into())),
            "keychain"
        );
        assert_eq!(
            startup_reason(&DomainError::KeystoreUnrecoverable),
            "keystore-lost"
        );
        assert_eq!(startup_reason(&DomainError::Storage("x".into())), "generic");
    }
}
