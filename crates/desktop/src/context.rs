//! What every feature controller needs: the window, the bridge to the core
//! and the session. Cheap to clone and `Send`, so it can travel into the
//! `done` callbacks of [`Bridge::run`].

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};

use basic_presence_core::domain::error::DomainError;
use basic_presence_core::services::Services;

use crate::AppWindow;
use crate::bridge::Bridge;
use crate::errors::UiError;
use crate::session::Session;

#[derive(Clone)]
pub struct Ctx {
    pub ui: slint::Weak<AppWindow>,
    pub bridge: Bridge,
    pub session: Session,
    /// Data the controllers share (active profile, loaded month, …).
    pub shared: Arc<Mutex<Shared>>,
}

/// State shared between the feature controllers. Only touched on the UI
/// thread in practice; the mutex makes [`Ctx`] `Send`.
#[derive(Default)]
pub struct Shared {
    pub lang: crate::i18n::Lang,
    /// Active profile (the one whose data is shown).
    pub profile_id: Option<String>,
    pub presence: crate::presence::PresenceData,
}

impl Ctx {
    pub fn new(ui: slint::Weak<AppWindow>, bridge: Bridge, session: Session) -> Self {
        Self {
            ui,
            bridge,
            session,
            shared: Arc::default(),
        }
    }

    pub fn shared(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Run a use case, then `done` on the UI thread (skipped if the window
    /// is gone).
    pub fn call<T, Fut>(
        &self,
        work: impl FnOnce(Arc<Services>) -> Fut,
        done: impl FnOnce(&AppWindow, T) + Send + 'static,
    ) where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let ui = self.ui.clone();
        self.bridge.run(work, move |output| {
            if let Some(ui) = ui.upgrade() {
                done(&ui, output);
            }
        });
    }

    /// Run a vault-touching use case behind the backend session gate
    /// (`require_session`). An expired session ends the session in the UI
    /// (back to the login screen) instead of reaching `done`.
    pub fn call_authed<T, Fut>(
        &self,
        work: impl FnOnce(Arc<Services>) -> Fut + Send + 'static,
        done: impl FnOnce(&AppWindow, Result<T, UiError>) + Send + 'static,
    ) where
        Fut: Future<Output = Result<T, DomainError>> + Send + 'static,
        T: Send + 'static,
    {
        let ctx = self.clone();
        self.call(
            move |s| async move {
                s.require_session.execute().await?;
                work(s).await
            },
            move |ui, result: Result<T, DomainError>| {
                let result = result.map_err(UiError::from);
                if matches!(&result, Err(e) if e.code == "SESSION_EXPIRED") {
                    crate::auth::end_session(&ctx, ui, true);
                    return;
                }
                done(ui, result);
            },
        );
    }
}
