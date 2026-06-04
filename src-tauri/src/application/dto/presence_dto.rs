use serde::Serialize;

use crate::domain::entities::presence::Presence;

/// Presence shape returned to the frontend (serialized camelCase). The `kind`
/// field is exposed on the wire as `type`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceDto {
    pub id: String,
    pub profile_id: String,
    pub day: i64,
    #[serde(rename = "type")]
    pub kind: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Presence> for PresenceDto {
    fn from(p: Presence) -> Self {
        Self {
            id: p.id,
            profile_id: p.profile_id,
            day: p.day,
            kind: p.kind.as_str().to_string(),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
