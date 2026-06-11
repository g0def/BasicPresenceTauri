use serde::Serialize;

use crate::domain::entities::task_preset::TaskPreset;

/// A saved task preset returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPresetDto {
    pub id: String,
    pub profile_id: String,
    pub title: String,
    pub description: Option<String>,
    pub default_minutes: i64,
    pub color: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<TaskPreset> for TaskPresetDto {
    fn from(p: TaskPreset) -> Self {
        Self {
            id: p.id,
            profile_id: p.profile_id,
            title: p.title,
            description: p.description,
            default_minutes: p.default_minutes,
            color: p.color,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
