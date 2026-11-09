//! Authentication flow: first-run registration, login, logout (port of the
//! former `auth-provider.tsx`).
//!
//! The session token stays in Rust memory only (never in the UI, never on
//! disk); the backend's `require_session` remains the gate for every
//! vault-touching use case.

use std::sync::{Arc, Mutex};

use slint::ComponentHandle;
use zeroize::Zeroizing;

use crate::bridge::Bridge;
use crate::errors::UiError;
use crate::{AppState, AppWindow, Screen};

/// The current session token, if signed in.
#[derive(Clone, Default)]
pub struct Session(Arc<Mutex<Option<Zeroizing<String>>>>);

impl Session {
    fn set(&self, token: Option<String>) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = token.map(Zeroizing::new);
    }

    fn take(&self) -> Option<Zeroizing<String>> {
        self.0.lock().unwrap_or_else(|p| p.into_inner()).take()
    }
}

fn show_error(ui: &AppWindow, error: UiError) {
    let state = ui.global::<AppState>();
    state.set_error_code(error.code.into());
    state.set_error_arg(error.arg.into());
}

fn clear_error(ui: &AppWindow) {
    show_error(
        ui,
        UiError {
            code: "",
            arg: String::new(),
        },
    );
}

/// Wire the auth callbacks and decide the first screen.
pub fn wire(ui: &AppWindow, bridge: Bridge, session: Session) {
    let state = ui.global::<AppState>();

    // First screen: registration on a fresh device, login otherwise.
    let weak = ui.as_weak();
    bridge.run(
        |s| async move { s.account_exists.execute().await },
        move |exists| {
            let Some(ui) = weak.upgrade() else { return };
            let state = ui.global::<AppState>();
            match exists {
                Ok(exists) => state.set_account_exists(exists),
                Err(e) => show_error(&ui, e.into()),
            }
            state.set_screen(Screen::Login);
        },
    );

    let (weak, b, sess) = (ui.as_weak(), bridge.clone(), session.clone());
    state.on_login(move |username, password| {
        let Some(ui) = weak.upgrade() else { return };
        // Zeroized once the attempt completes. (Slint's own copy of the field
        // is cleared by the screen right after submission.)
        let password = Zeroizing::new(password.to_string());
        let username = username.to_string();
        ui.global::<AppState>().set_busy(true);
        clear_error(&ui);
        let (weak, b2, sess) = (weak.clone(), b.clone(), sess.clone());
        b.run(
            move |s| async move { s.login.execute(&username, &password).await },
            move |result| {
                let Some(ui) = weak.upgrade() else { return };
                let state = ui.global::<AppState>();
                state.set_busy(false);
                match result {
                    Ok(login) => {
                        sess.set(Some(login.token));
                        state.set_username(login.user.username.into());
                        state.set_screen(Screen::Home);
                        load_profile_count(&ui, &b2);
                    }
                    Err(e) => show_error(&ui, e.into()),
                }
            },
        );
    });

    let (weak, b) = (ui.as_weak(), bridge.clone());
    state.on_register(move |username, password| {
        let Some(ui) = weak.upgrade() else { return };
        let password = Zeroizing::new(password.to_string());
        let username = username.to_string();
        ui.global::<AppState>().set_busy(true);
        clear_error(&ui);
        let weak = weak.clone();
        b.run(
            move |s| async move { s.register_account.execute(&username, &password).await },
            move |result| {
                let Some(ui) = weak.upgrade() else { return };
                let state = ui.global::<AppState>();
                state.set_busy(false);
                match result {
                    // Account created: switch the gate to the login screen.
                    Ok(_) => state.set_account_exists(true),
                    Err(e) => show_error(&ui, e.into()),
                }
            },
        );
    });

    let (weak, b) = (ui.as_weak(), bridge);
    state.on_logout(move || {
        let token = session.take();
        let weak = weak.clone();
        b.run(
            move |s| async move {
                match token {
                    Some(token) => s.logout.execute(&token).await,
                    None => Ok(()),
                }
            },
            move |_| {
                let Some(ui) = weak.upgrade() else { return };
                let state = ui.global::<AppState>();
                state.set_username("".into());
                state.set_profile_count(0);
                state.set_screen(Screen::Login);
            },
        );
    });
}

fn load_profile_count(ui: &AppWindow, bridge: &Bridge) {
    let weak = ui.as_weak();
    bridge.run(
        |s| async move {
            s.require_session.execute().await?;
            s.list_profiles.execute().await
        },
        move |result| {
            let Some(ui) = weak.upgrade() else { return };
            match result {
                Ok(profiles) => ui
                    .global::<AppState>()
                    .set_profile_count(profiles.profiles.len() as i32),
                Err(e) => show_error(&ui, e.into()),
            }
        },
    );
}
