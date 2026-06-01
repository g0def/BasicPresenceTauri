use serde::Serialize;

use crate::domain::entities::user::User;

/// User shape returned to the frontend (serialized camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}
