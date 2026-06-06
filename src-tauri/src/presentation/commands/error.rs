use serde::Serialize;

use crate::domain::error::DomainError;

/// Error shape returned to the WebView. Internal details (SQL, crypto) are
/// collapsed to a generic `INTERNAL` so they never leak to the frontend.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InvalidCredentials => {
                AppError::new("INVALID_CREDENTIALS", "Identifiants invalides")
            }
            DomainError::AccountLocked { retry_after_ms } => AppError::new(
                "ACCOUNT_LOCKED",
                format!(
                    "Compte verrouillé. Réessayez dans {} s.",
                    (retry_after_ms / 1000).max(1)
                ),
            ),
            DomainError::AccountAlreadyExists => {
                AppError::new("ACCOUNT_EXISTS", "Un compte existe déjà sur cet appareil")
            }
            DomainError::Unauthorized => {
                AppError::new("SESSION_EXPIRED", "Session expirée. Reconnectez-vous.")
            }
            DomainError::ProfileNotFound => AppError::new("NOT_FOUND", "Profil introuvable"),
            DomainError::Validation(m) => AppError::new("VALIDATION", m),
            DomainError::KeystoreUnrecoverable => AppError::new(
                "KEYSTORE_UNRECOVERABLE",
                "La clé sécurisée de cet appareil est manquante ou a changé : \
                 le stockage local chiffré ne peut pas être ouvert.",
            ),
            DomainError::VaultTampered => AppError::new(
                "VAULT_TAMPERED",
                "L'intégrité du stockage chiffré a échoué : le fichier a été modifié.",
            ),
            DomainError::Storage(_)
            | DomainError::Hashing(_)
            | DomainError::Crypto(_)
            | DomainError::Token(_)
            | DomainError::Keychain(_) => AppError::new("INTERNAL", "Erreur interne"),
        }
    }
}
