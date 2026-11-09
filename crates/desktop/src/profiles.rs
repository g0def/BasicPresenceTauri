//! Profiles (port of `profile-provider.tsx` + the profile dialogs): list,
//! create / edit, delete, active profile.

use basic_presence_core::application::dto::profile_dto::ProfilesDto;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::context::Ctx;
use crate::errors::UiError;
use crate::{AppWindow, ProfileItem, ProfileState};

/// Show the profiles; the active one (else the first) names the header menu.
fn apply(ui: &AppWindow, dto: ProfilesDto) {
    let active = dto
        .active_profile_id
        .clone()
        .or_else(|| dto.profiles.first().map(|p| p.id.clone()));
    let items: Vec<ProfileItem> = dto
        .profiles
        .into_iter()
        .map(|p| ProfileItem {
            active: Some(&p.id) == active.as_ref(),
            id: p.id.into(),
            first_name: p.first_name.into(),
            last_name: p.last_name.into(),
            enterprise: p.enterprise.into(),
            poste: p.poste.unwrap_or_default().into(),
        })
        .collect();
    let current = items
        .iter()
        .find(|p| p.active)
        .map(|p| format!("{} {}", p.first_name, p.last_name))
        .unwrap_or_default();
    let state = ui.global::<ProfileState>();
    state.set_current_name(current.into());
    state.set_profiles(ModelRc::new(VecModel::from(items)));
    state.set_loaded(true);
}

fn show_error(ui: &AppWindow, e: UiError) {
    let state = ui.global::<ProfileState>();
    state.set_busy(false);
    state.set_error_code(e.code.into());
    state.set_error_arg(e.arg.into());
}

/// (Re)load the profile list.
pub fn load(ctx: &Ctx) {
    ctx.call_authed(
        |s| async move { s.list_profiles.execute().await },
        |ui, result| match result {
            Ok(dto) => apply(ui, dto),
            Err(e) => show_error(ui, e),
        },
    );
}

/// Forget the profiles (sign-out).
pub fn reset(ui: &AppWindow) {
    let state = ui.global::<ProfileState>();
    state.set_loaded(false);
    state.set_busy(false);
    state.set_form_open(false);
    state.set_delete_open(false);
    state.set_error_code("".into());
    state.set_current_name("".into());
    state.set_profiles(ModelRc::default());
}

/// Run a profile mutation, then reload the list and close the dialogs.
fn mutate<Fut>(
    ctx: &Ctx,
    work: impl FnOnce(std::sync::Arc<basic_presence_core::services::Services>) -> Fut + Send + 'static,
) where
    Fut: std::future::Future<Output = Result<(), basic_presence_core::domain::error::DomainError>>
        + Send
        + 'static,
{
    let Some(ui) = ctx.ui.upgrade() else { return };
    let state = ui.global::<ProfileState>();
    state.set_busy(true);
    state.set_error_code("".into());
    ctx.call_authed(
        move |s| async move {
            work(s.clone()).await?;
            s.list_profiles.execute().await
        },
        |ui, result| match result {
            Ok(dto) => {
                let state = ui.global::<ProfileState>();
                state.set_busy(false);
                state.set_form_open(false);
                state.set_delete_open(false);
                apply(ui, dto);
            }
            Err(e) => show_error(ui, e),
        },
    );
}

pub fn wire(ctx: &Ctx) {
    let Some(ui) = ctx.ui.upgrade() else { return };
    let state = ui.global::<ProfileState>();

    let c = ctx.clone();
    state.on_save(move |id, first, last, enterprise, poste| {
        let (id, first, last, enterprise) = (
            id.to_string(),
            first.to_string(),
            last.to_string(),
            enterprise.to_string(),
        );
        let poste = Some(poste.trim().to_string()).filter(|p| !p.is_empty());
        mutate(&c, move |s| async move {
            if id.is_empty() {
                s.create_profile
                    .execute(&first, &last, &enterprise, poste.as_deref())
                    .await
                    .map(|_| ())
            } else {
                s.update_profile
                    .execute(&id, &first, &last, &enterprise, poste.as_deref())
                    .await
                    .map(|_| ())
            }
        });
    });

    let c = ctx.clone();
    state.on_delete(move |id| {
        let id = id.to_string();
        mutate(
            &c,
            move |s| async move { s.delete_profile.execute(&id).await },
        );
    });

    let c = ctx.clone();
    state.on_set_active(move |id| {
        let id = id.to_string();
        mutate(&c, move |s| async move {
            s.set_active_profile.execute(&id).await
        });
    });
}
