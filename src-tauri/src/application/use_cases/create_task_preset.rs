use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::services::clock::Clock;

/// Create a saved task preset (title, default duration, color) for a profile.
pub struct CreateTaskPresetUseCase {
    presets: Arc<dyn TaskPresetRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateTaskPresetUseCase {
    pub fn new(presets: Arc<dyn TaskPresetRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { presets, clock }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        title: &str,
        description: Option<String>,
        default_minutes: i64,
        color: &str,
    ) -> Result<TaskPresetDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        let now = self.clock.now_ms();
        let preset = TaskPreset {
            id: Uuid::now_v7().to_string(),
            profile_id: profile_id.to_string(),
            title: validate_title(title)?,
            description: normalize_description(description),
            default_minutes: validate_minutes(default_minutes)?,
            color: validate_color(color)?,
            created_at: now,
            updated_at: now,
        };
        self.presets.create(&preset).await?;
        Ok(TaskPresetDto::from(preset))
    }
}

// Shared field rules for presets and day entries (the slider's 5-min grid).
pub fn validate_title(title: &str) -> Result<String, DomainError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(DomainError::Validation("title is required".to_string()));
    }
    Ok(title.to_string())
}

pub fn normalize_description(description: Option<String>) -> Option<String> {
    description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
}

pub fn validate_minutes(minutes: i64) -> Result<i64, DomainError> {
    if !(5..=480).contains(&minutes) || minutes % 5 != 0 {
        return Err(DomainError::Validation(
            "minutes must be between 5 and 480 in steps of 5".to_string(),
        ));
    }
    Ok(minutes)
}

pub fn validate_color(color: &str) -> Result<String, DomainError> {
    let color = color.trim();
    let hex = color.strip_prefix('#').unwrap_or("");
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(DomainError::Validation(
            "color must be a #RRGGBB hex value".to_string(),
        ));
    }
    Ok(color.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minutes_must_stay_on_the_5_min_grid_within_bounds() {
        assert!(validate_minutes(5).is_ok());
        assert!(validate_minutes(480).is_ok());
        assert!(validate_minutes(125).is_ok());
        assert!(validate_minutes(0).is_err());
        assert!(validate_minutes(485).is_err());
        assert!(validate_minutes(7).is_err());
        assert!(validate_minutes(-5).is_err());
    }

    #[test]
    fn color_must_be_rrggbb_hex() {
        assert!(validate_color("#4B7F52").is_ok());
        assert!(validate_color(" #4b7f52 ").is_ok());
        assert!(validate_color("4B7F52").is_err());
        assert!(validate_color("#4B7F5").is_err());
        assert!(validate_color("#4B7F5G").is_err());
        assert!(validate_color("").is_err());
    }

    #[test]
    fn title_is_trimmed_and_required() {
        assert_eq!(validate_title("  Réunion  ").unwrap(), "Réunion");
        assert!(validate_title("   ").is_err());
    }

    #[test]
    fn blank_descriptions_collapse_to_none() {
        assert_eq!(normalize_description(Some("  ".to_string())), None);
        assert_eq!(
            normalize_description(Some(" notes ".to_string())),
            Some("notes".to_string())
        );
        assert_eq!(normalize_description(None), None);
    }
}
