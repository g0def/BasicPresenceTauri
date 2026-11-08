use serde::Serialize;

use crate::domain::entities::profile::Profile;

/// Profile shape returned to the frontend (serialized camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub enterprise: String,
    pub poste: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Profile> for ProfileDto {
    fn from(p: Profile) -> Self {
        Self {
            id: p.id,
            first_name: p.first_name,
            last_name: p.last_name,
            enterprise: p.enterprise,
            poste: p.poste,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

/// The full profile list plus the active profile id (for the header switcher).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesDto {
    pub profiles: Vec<ProfileDto>,
    pub active_profile_id: Option<String>,
}
