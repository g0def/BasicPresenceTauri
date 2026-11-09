//! Headless end-to-end test of the UI: the real Slint components, wired to the
//! real core services (temporary vault, fixed device key — never the OS
//! keychain), rendered off-screen by Slint's software renderer.
//!
//! Set `BP_SNAPSHOT_DIR=<dir>` to also write PNG screenshots of each step:
//! `BP_SNAPSHOT_DIR=/tmp/shots cargo test -p basic-presence --test ui_flow`
//!
//! Slint's platform is process-global and its components are not `Send`, so
//! the whole scenario lives in ONE test function.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use basic_presence::auth::{self, Session};
use basic_presence::bridge::Bridge;
use basic_presence::{AppState, AppWindow, Screen, Theme, wire_window};
use basic_presence_core::infrastructure::config::AppConfig;
use basic_presence_core::services::build_services;
use slint::Rgb8Pixel;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{EventLoopProxy, Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize};

const WIDTH: u32 = 1100;
const HEIGHT: u32 = 800;

type Queue = Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>;

/// `invoke_from_event_loop` lands in a queue the test drains by hand.
struct Proxy(Queue);

impl EventLoopProxy for Proxy {
    fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
        Ok(())
    }

    fn invoke_from_event_loop(
        &self,
        event: Box<dyn FnOnce() + Send>,
    ) -> Result<(), slint::EventLoopError> {
        self.0.lock().unwrap().push(event);
        Ok(())
    }
}

struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    queue: Queue,
}

impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(Proxy(self.queue.clone())))
    }
}

struct Harness {
    window: Rc<MinimalSoftwareWindow>,
    queue: Queue,
    shots: Option<PathBuf>,
}

impl Harness {
    /// Run queued UI callbacks and timers until `done` holds (10 s max).
    fn pump_until(&self, what: &str, done: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let jobs = std::mem::take(&mut *self.queue.lock().unwrap());
            for job in jobs {
                job();
            }
            slint::platform::update_timers_and_animations();
            if done() {
                return;
            }
            assert!(Instant::now() < deadline, "timed out waiting for: {what}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Render the window; write `<name>.png` when snapshots are enabled.
    fn snapshot(&self, name: &str) {
        slint::platform::update_timers_and_animations();
        self.window.request_redraw();
        let mut buffer = vec![Rgb8Pixel::default(); (WIDTH * HEIGHT) as usize];
        let drawn = self.window.draw_if_needed(|renderer| {
            renderer.render(&mut buffer, WIDTH as usize);
        });
        assert!(drawn, "{name}: nothing was rendered");
        // Something other than the page background must have been drawn.
        assert!(
            buffer.iter().any(|p| *p != buffer[0]),
            "{name}: blank render"
        );
        let Some(dir) = &self.shots else { return };
        std::fs::create_dir_all(dir).unwrap();
        let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let bytes: Vec<u8> = buffer.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&bytes)
            .unwrap();
    }
}

#[test]
fn login_flow_renders_and_works_end_to_end() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    window.set_size(PhysicalSize::new(WIDTH, HEIGHT));
    let queue: Queue = Arc::default();
    slint::platform::set_platform(Box::new(Headless {
        window: window.clone(),
        queue: queue.clone(),
    }))
    .expect("platform set once");
    let h = Harness {
        window,
        queue,
        shots: std::env::var_os("BP_SNAPSHOT_DIR").map(PathBuf::from),
    };

    // Real services on a throwaway data dir, with a fixed device key.
    let dir = std::env::temp_dir().join(format!("bp-ui-flow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let services = runtime
        .block_on(build_services(AppConfig::new(&dir), &[7u8; 32]))
        .expect("services");
    let bridge = Bridge::new(runtime.handle().clone(), Arc::new(services));

    let ui = AppWindow::new().expect("window");
    wire_window(&ui, "fr");
    auth::wire(&ui, bridge.clone(), Session::default());
    ui.show().unwrap();
    let state = ui.global::<AppState>();

    // Fresh device → registration screen.
    h.pump_until("login screen", || state.get_screen() == Screen::Login);
    assert!(!state.get_account_exists());
    h.snapshot("01-register-fr");

    // Translations (bundled .po), with placeholders.
    assert_eq!(
        state.invoke_error_text("INVALID_CREDENTIALS".into(), "".into()),
        "Identifiants invalides"
    );
    assert_eq!(
        state.invoke_error_text("ACCOUNT_LOCKED".into(), "30".into()),
        "Compte verrouillé. Réessayez dans 30 s."
    );
    assert_eq!(
        state.invoke_error_text("SOMETHING_ELSE".into(), "".into()),
        "Erreur interne"
    );

    // Register, then the gate switches to login.
    state.invoke_register("alice".into(), "password123".into());
    h.pump_until("account created", || {
        state.get_account_exists() && !state.get_busy()
    });
    assert_eq!(state.get_error_code(), "");

    // A wrong password is reported with a localized error.
    state.invoke_login("alice".into(), "wrong-password".into());
    h.pump_until("login refused", || {
        !state.get_busy() && state.get_error_code() != ""
    });
    assert_eq!(state.get_error_code(), "INVALID_CREDENTIALS");
    h.snapshot("02-login-error-fr");

    // Live language switch.
    state.invoke_set_lang("en".into());
    assert_eq!(state.get_lang(), "en");
    assert_eq!(
        state.invoke_error_text("INVALID_CREDENTIALS".into(), "".into()),
        "Invalid credentials"
    );
    ui.global::<Theme>().set_dark(true);
    h.snapshot("03-login-error-en-dark");

    // Successful login → home, with data read through the session gate.
    state.invoke_login("alice".into(), "password123".into());
    h.pump_until("home", || state.get_screen() == Screen::Home);
    assert_eq!(state.get_username(), "alice");
    h.pump_until("profiles loaded", || !state.get_busy());
    assert_eq!(state.get_profile_count(), 0);
    h.snapshot("04-home-en-dark");

    // Logout → back to the login screen.
    state.invoke_logout();
    h.pump_until("logged out", || state.get_screen() == Screen::Login);
    assert_eq!(state.get_username(), "");

    bridge.lock_blocking();
    drop(ui);
    runtime.shutdown_timeout(Duration::from_secs(2));
    let _ = std::fs::remove_dir_all(dir);
}
