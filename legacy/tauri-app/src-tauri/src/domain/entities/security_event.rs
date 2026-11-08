/// Kinds of security-relevant events kept in the (sealed) keystore journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityEventKind {
    /// A successful login (marks the boundary of "since your last visit").
    Login,
    /// Too many wrong passwords: the account was locked.
    AccountLocked,
    /// The previous session ended without a clean close.
    UncleanShutdown,
    /// The vault failed its integrity check and was refused.
    TamperDetected,
    /// The vault failed its integrity check and the policy let it open.
    TamperTolerated,
    /// The password was changed.
    PasswordChanged,
}

impl SecurityEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::AccountLocked => "account_locked",
            Self::UncleanShutdown => "unclean_shutdown",
            Self::TamperDetected => "tamper_detected",
            Self::TamperTolerated => "tamper_tolerated",
            Self::PasswordChanged => "password_changed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Login,
            Self::AccountLocked,
            Self::UncleanShutdown,
            Self::TamperDetected,
            Self::TamperTolerated,
            Self::PasswordChanged,
        ]
        .into_iter()
        .find(|k| k.as_str() == s)
    }
}

/// One journal entry: what happened and when (epoch ms). No other data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityEvent {
    pub kind: SecurityEventKind,
    pub at_ms: i64,
}
