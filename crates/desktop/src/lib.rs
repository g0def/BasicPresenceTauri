//! BasicPresence desktop app (Slint UI over `basic-presence-core`).
//!
//! Composition: a tokio runtime for the use cases, the Slint window on the
//! main thread, and the [`bridge::Bridge`] between them. On exit the vault is
//! locked (integrity baseline refreshed, sessions revoked).

pub mod auth;
pub mod bridge;
pub mod context;
pub mod errors;
pub mod i18n;
pub mod profiles;
pub mod session;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use basic_presence_core::domain::error::DomainError;
use basic_presence_core::services::open_services;
use slint::ComponentHandle;

slint::include_modules!();

/// Same identifier as the Tauri app: data dir and keychain entry are shared,
/// so the existing vault opens as is.
const APP_ID: &str = "com.godef.basic-presence";

/// Per-user data directory: `$XDG_DATA_HOME/<id>` on Linux,
/// `~/Library/Application Support/<id>` on macOS, `%APPDATA%\<id>` on Windows
/// (the same as Tauri's `app_data_dir`). `BASIC_PRESENCE_DATA_DIR` overrides it
/// (tests, measurements: never touches the real vault).
fn data_dir() -> Result<PathBuf, DomainError> {
    if let Some(dir) = std::env::var_os("BASIC_PRESENCE_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_dir()
        .map(|d| d.join(APP_ID))
        .ok_or_else(|| DomainError::Storage("cannot resolve the user data directory".into()))
}

/// UI-only wiring shared by the app and the UI tests: version, language
/// selection (initial + switcher) and quit.
pub fn wire_window(ui: &AppWindow, lang: &str) {
    let state = ui.global::<AppState>();
    state.set_version(env!("CARGO_PKG_VERSION").into());
    let lang = i18n::normalize(lang);
    i18n::select(lang);
    state.set_lang(lang.into());
    let weak = ui.as_weak();
    state.on_set_lang(move |lang| {
        let lang = i18n::normalize(&lang);
        i18n::select(lang);
        if let Some(ui) = weak.upgrade() {
            ui.global::<AppState>().set_lang(lang.into());
        }
    });
    state.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
}

/// Wire every feature controller on the backend services. The returned timer
/// drives the session countdown / inactivity check: keep it alive.
pub fn wire_features(ui: &AppWindow, bridge: bridge::Bridge) -> (context::Ctx, slint::Timer) {
    let ctx = context::Ctx {
        ui: ui.as_weak(),
        bridge,
        session: session::Session::default(),
    };
    auth::wire(&ctx);
    profiles::wire(&ctx);

    let timer = slint::Timer::default();
    let c = ctx.clone();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(1),
        move || auth::tick(&c, session::now_ms()),
    );
    (ctx, timer)
}

/// Every keyboard / pointer input on the window counts as user activity.
fn watch_activity(ui: &AppWindow, ctx: context::Ctx) {
    use slint::winit_030::winit::event::WindowEvent;
    use slint::winit_030::{EventResult, WinitWindowAccessor};

    ui.window().on_winit_window_event(move |_, event| {
        if matches!(
            event,
            WindowEvent::KeyboardInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::Touch(_)
        ) {
            auth::report_activity(&ctx);
        }
        EventResult::Propagate
    });
}

/// Run the app until its window closes.
pub fn run() -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("bp-core")
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("BasicPresence could not start its runtime: {e}");
            return ExitCode::FAILURE;
        }
    };

    let ui = match AppWindow::new() {
        Ok(ui) => ui,
        Err(e) => {
            eprintln!("BasicPresence could not open its window: {e}");
            return ExitCode::FAILURE;
        }
    };
    let state = ui.global::<AppState>();
    wire_window(&ui, i18n::system_language());

    let opened = data_dir().and_then(|dir| runtime.block_on(open_services(&dir)));
    let mut _session_timer = None;
    let bridge = match opened {
        Ok(services) => {
            let bridge = bridge::Bridge::new(runtime.handle().clone(), Arc::new(services));
            let (ctx, timer) = wire_features(&ui, bridge.clone());
            watch_activity(&ui, ctx);
            _session_timer = Some(timer);
            Some(bridge)
        }
        // A startup failure (no OS keychain, lost device key, unreadable data
        // dir…) reaches the user as a readable screen, not as a panic.
        Err(e) => {
            state.set_startup_reason(errors::startup_reason(&e).into());
            state.set_startup_detail(e.to_string().into());
            state.set_screen(Screen::StartupError);
            None
        }
    };

    let result = ui.run();

    // Clean exit: lock the vault so its integrity baseline is recorded (the
    // next launch would otherwise see an unclean shutdown).
    if let Some(bridge) = bridge {
        bridge.lock_blocking();
    }
    drop(ui);
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("BasicPresence UI error: {e}");
            ExitCode::FAILURE
        }
    }
}
