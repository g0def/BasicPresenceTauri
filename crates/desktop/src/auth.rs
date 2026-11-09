//! Authentication flow (port of `auth-provider.tsx`): first-run
//! registration, login with the security notice, logout and session end
//! (absolute expiry, inactivity, or a session the backend no longer knows),
//! change of password.

use basic_presence_core::application::dto::login_result_dto::LoginResultDto;
use basic_presence_core::application::view::format::{Lang, format_countdown, format_datetime};
use basic_presence_core::domain::policy::MIN_PASSWORD_LEN;
use chrono::TimeZone;
use slint::{ComponentHandle, ModelRc, VecModel};
use zeroize::Zeroizing;

use crate::context::Ctx;
use crate::errors::UiError;
use crate::session::{self, Tick, WARNING_MS};
use crate::{AppState, AppWindow, AuthState, Page, Screen, SecurityEventItem};

fn set_error(ui: &AppWindow, error: Option<UiError>) {
    let auth = ui.global::<AuthState>();
    let error = error.unwrap_or(UiError {
        code: "",
        arg: String::new(),
    });
    auth.set_error_code(error.code.into());
    auth.set_error_arg(error.arg.into());
}

/// UTC offset (minutes) of the local time zone at `epoch_ms`.
fn local_offset_minutes(epoch_ms: i64) -> i32 {
    chrono::Local
        .timestamp_millis_opt(epoch_ms)
        .single()
        .map(|d| d.offset().local_minus_utc() / 60)
        .unwrap_or(0)
}

/// Wire the auth callbacks and decide the first screen.
pub fn wire(ctx: &Ctx) {
    let Some(ui) = ctx.ui.upgrade() else { return };
    let auth = ui.global::<AuthState>();

    // First screen: registration on a fresh device, login otherwise.
    ctx.call(
        |s| async move { s.account_exists.execute().await },
        |ui, exists| {
            match exists {
                Ok(exists) => ui.global::<AuthState>().set_account_exists(exists),
                Err(e) => set_error(ui, Some(e.into())),
            }
            ui.global::<AppState>().set_screen(Screen::Login);
        },
    );

    let c = ctx.clone();
    auth.on_login(move |username, password| {
        let Some(ui) = c.ui.upgrade() else { return };
        // Zeroized once the attempt completes. (Slint's own copy of the field
        // is cleared by the screen right after submission.)
        let password = Zeroizing::new(password.to_string());
        let username = username.to_string();
        ui.global::<AuthState>().set_busy(true);
        set_error(&ui, None);
        let c2 = c.clone();
        c.call(
            move |s| async move { s.login.execute(&username, &password).await },
            move |ui, result| {
                ui.global::<AuthState>().set_busy(false);
                match result {
                    Ok(login) => start_session(&c2, ui, login),
                    Err(e) => set_error(ui, Some(e.into())),
                }
            },
        );
    });

    let c = ctx.clone();
    auth.on_register(move |username, password| {
        let Some(ui) = c.ui.upgrade() else { return };
        let password = Zeroizing::new(password.to_string());
        let username = username.to_string();
        ui.global::<AuthState>().set_busy(true);
        set_error(&ui, None);
        c.call(
            move |s| async move { s.register_account.execute(&username, &password).await },
            |ui, result| {
                let auth = ui.global::<AuthState>();
                auth.set_busy(false);
                match result {
                    // Account created: switch the gate to the login screen.
                    Ok(_) => auth.set_account_exists(true),
                    Err(e) => set_error(ui, Some(e.into())),
                }
            },
        );
    });

    let c = ctx.clone();
    auth.on_logout(move || {
        if let Some(ui) = c.ui.upgrade() {
            end_session(&c, &ui, false);
        }
    });

    let c = ctx.clone();
    auth.on_change_password(move |current, new, confirm| {
        let Some(ui) = c.ui.upgrade() else { return };
        let auth = ui.global::<AuthState>();
        auth.set_password_done(false);
        let (current, new, confirm) = (
            Zeroizing::new(current.to_string()),
            Zeroizing::new(new.to_string()),
            Zeroizing::new(confirm.to_string()),
        );
        let invalid = if new.chars().count() < MIN_PASSWORD_LEN {
            Some(("PASSWORD_TOO_SHORT", MIN_PASSWORD_LEN.to_string()))
        } else if *new != *confirm {
            Some(("PASSWORD_MISMATCH", String::new()))
        } else {
            None
        };
        if let Some((code, arg)) = invalid {
            auth.set_password_error_code(code.into());
            auth.set_password_error_arg(arg.into());
            return;
        }
        auth.set_password_error_code("".into());
        auth.set_password_busy(true);
        c.call_authed(
            move |s| async move { s.change_password.execute(&current, &new).await },
            |ui, result| {
                let auth = ui.global::<AuthState>();
                auth.set_password_busy(false);
                match result {
                    Ok(()) => auth.set_password_done(true),
                    Err(e) => {
                        auth.set_password_error_code(e.code.into());
                        auth.set_password_error_arg(e.arg.into());
                    }
                }
            },
        );
    });
}

fn start_session(ctx: &Ctx, ui: &AppWindow, login: LoginResultDto) {
    let now = session::now_ms();
    ctx.session.start(login.token, login.expires_at, now);

    let auth = ui.global::<AuthState>();
    auth.set_username(login.user.username.into());
    let lang = Lang::from_code(&ui.global::<AppState>().get_lang());
    let events: Vec<SecurityEventItem> = login
        .security_events
        .iter()
        .map(|e| SecurityEventItem {
            at: format_datetime(e.at, local_offset_minutes(e.at), lang).into(),
            kind: e.kind.clone().into(),
        })
        .collect();
    auth.set_notice_failed_attempts(login.failed_attempts_since_last_login as i32);
    auth.set_notice_open(login.failed_attempts_since_last_login > 0 || !events.is_empty());
    auth.set_notice_events(ModelRc::new(VecModel::from(events)));
    update_countdown(ui, login.expires_at - now);

    let app = ui.global::<AppState>();
    app.set_page(Page::Presence);
    app.set_screen(Screen::Main);
    crate::profiles::load(ctx);
}

/// End the session: drop it locally at once, reset the UI to the login
/// screen, then have the backend revoke it and lock the vault. `expired`
/// shows why the user was signed out.
pub fn end_session(ctx: &Ctx, ui: &AppWindow, expired: bool) {
    let token = ctx.session.clear();
    if token.is_none() && ui.global::<AppState>().get_screen() != Screen::Main {
        return; // already signed out
    }

    let auth = ui.global::<AuthState>();
    auth.set_username("".into());
    auth.set_notice_open(false);
    auth.set_password_error_code("".into());
    auth.set_password_done(false);
    set_error(
        ui,
        expired.then(|| UiError {
            code: "SESSION_EXPIRED",
            arg: String::new(),
        }),
    );
    crate::profiles::reset(ui);
    let app = ui.global::<AppState>();
    app.set_page(Page::Presence);
    app.set_screen(Screen::Login);

    // A failed backend logout is not fatal: its own expiry locks the vault.
    ctx.call(
        move |s| async move {
            match token {
                Some(token) => s.logout.execute(&token).await,
                None => Ok(()),
            }
        },
        |_, _| {},
    );
}

fn update_countdown(ui: &AppWindow, remaining_ms: i64) {
    let auth = ui.global::<AuthState>();
    auth.set_session_remaining(format_countdown(remaining_ms).into());
    auth.set_session_warning(remaining_ms <= WARNING_MS);
}

/// One-second tick: countdown, absolute expiry, inactivity.
pub fn tick(ctx: &Ctx, now: i64) {
    let Some(ui) = ctx.ui.upgrade() else { return };
    match ctx.session.tick(now) {
        None => {}
        Some(Tick::Running { remaining_ms }) => update_countdown(&ui, remaining_ms),
        Some(Tick::Expired | Tick::Idle) => end_session(ctx, &ui, true),
    }
}

/// User activity (any input on the window): postpones the inactivity logout
/// and, at most once a minute, tells the backend — it runs its own idle
/// timer. A session the backend no longer knows ends here too.
pub fn report_activity(ctx: &Ctx) {
    let Some(token) = ctx.session.activity(session::now_ms()) else {
        return;
    };
    let c = ctx.clone();
    ctx.call(
        move |s| async move { s.check_session.execute(&token).await },
        move |ui, status| {
            if matches!(status, Ok(ref s) if !s.valid) {
                end_session(&c, ui, true);
            }
        },
    );
}
