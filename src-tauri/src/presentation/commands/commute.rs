use tauri::State;

use crate::application::dto::commute_dto::{CommuteDto, CommuteSegmentInputDto};
use crate::application::dto::emission_factor_dto::{Co2ReferentialDto, EmissionFactorDto};
use crate::application::dto::trip_dto::TripDto;
use crate::presentation::commands::error::AppError;
use crate::presentation::state::AppState;

// Every command below is session-gated: `require_session` enforces the absolute
// expiry backend-side (revoking the session and locking the vault when it has
// passed), so a frontend that stops calling `check_session` gains nothing.

/// List the emission-factor referential (for the mode pickers).
#[tauri::command]
pub async fn list_emission_factors(
    state: State<'_, AppState>,
) -> Result<Vec<EmissionFactorDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.list_emission_factors.execute().await?)
}

/// List the full CO2 referential (every factor + scope/source + grid variants,
/// the active year and its radiative-forcing multiplier) for the methodology page.
#[tauri::command]
pub async fn list_co2_referential(
    state: State<'_, AppState>,
) -> Result<Co2ReferentialDto, AppError> {
    state.require_session.execute()?;
    Ok(state.list_co2_referential.execute().await?)
}

/// Create a saved commute template for a profile.
#[tauri::command]
pub async fn create_commute(
    profile_id: String,
    name: String,
    round_trip: bool,
    segments: Vec<CommuteSegmentInputDto>,
    state: State<'_, AppState>,
) -> Result<CommuteDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .create_commute
        .execute(&profile_id, &name, round_trip, segments)
        .await?)
}

/// List a profile's saved commutes (each with an indicative CO2 footprint).
#[tauri::command]
pub async fn list_commutes(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CommuteDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.list_commutes.execute(&profile_id).await?)
}

/// Update a saved commute (name, round-trip flag, full segment set).
#[tauri::command]
pub async fn update_commute(
    id: String,
    name: String,
    round_trip: bool,
    segments: Vec<CommuteSegmentInputDto>,
    state: State<'_, AppState>,
) -> Result<CommuteDto, AppError> {
    state.require_session.execute()?;
    Ok(state
        .update_commute
        .execute(&id, &name, round_trip, segments)
        .await?)
}

/// Delete a saved commute by id.
#[tauri::command]
pub async fn delete_commute(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.require_session.execute()?;
    state.delete_commute.execute(&id).await?;
    Ok(())
}

/// Fetch the trip snapshot of a presence day (to pre-fill the editor).
#[tauri::command]
pub async fn get_presence_trips(
    presence_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TripDto>, AppError> {
    state.require_session.execute()?;
    Ok(state.get_presence_trips.execute(&presence_id).await?)
}
