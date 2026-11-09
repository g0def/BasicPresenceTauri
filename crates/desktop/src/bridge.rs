//! Bridge between the UI thread (Slint event loop) and the core's use cases,
//! which run on a dedicated tokio runtime (libSQL, Argon2's `spawn_blocking`
//! and the vault's task-local transactions all require tokio).
//!
//! The UI never blocks on a use case: [`Bridge::run`] spawns the work on the
//! runtime and hands its result back to the event loop.

use std::future::Future;
use std::sync::Arc;

use basic_presence_core::services::Services;

#[derive(Clone)]
pub struct Bridge {
    runtime: tokio::runtime::Handle,
    services: Arc<Services>,
}

impl Bridge {
    pub fn new(runtime: tokio::runtime::Handle, services: Arc<Services>) -> Self {
        Self { runtime, services }
    }

    /// Run `work` on the runtime, then `done` with its output on the UI thread.
    pub fn run<T, Fut>(
        &self,
        work: impl FnOnce(Arc<Services>) -> Fut,
        done: impl FnOnce(T) + Send + 'static,
    ) where
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let future = work(self.services.clone());
        self.runtime.spawn(async move {
            let output = future.await;
            // Fails only once the event loop has quit: nothing left to update.
            let _ = slint::invoke_from_event_loop(move || done(output));
        });
    }

    /// Lock the vault and revoke every session (window close / app exit).
    /// Blocks: only call it from outside the runtime (UI thread, end of main).
    pub fn lock_blocking(&self) {
        let services = self.services.clone();
        self.runtime.block_on(async move { services.lock().await });
    }
}
