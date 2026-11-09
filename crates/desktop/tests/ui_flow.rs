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

use basic_presence::bridge::Bridge;
use basic_presence::session::{IDLE_TIMEOUT_MS, now_ms};
use basic_presence::{
    AppState, AppWindow, AuthState, DayCell, Page, PresenceState, ProfileState, Screen, Theme,
    auth, wire_features, wire_window,
};
use basic_presence_core::infrastructure::config::AppConfig;
use basic_presence_core::services::build_services;
use slint::Model;
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
fn app_flows_render_and_work_end_to_end() {
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
    let (ctx, _timer) = wire_features(&ui, bridge.clone());
    ui.show().unwrap();
    let app = ui.global::<AppState>();
    let auth_state = ui.global::<AuthState>();
    let profiles = ui.global::<ProfileState>();

    // ---------- Login / registration (S3) ----------
    h.pump_until("login screen", || app.get_screen() == Screen::Login);
    assert!(!auth_state.get_account_exists());
    h.snapshot("01-register-fr");

    // Translations (bundled .po), with placeholders.
    assert_eq!(
        app.invoke_error_text("INVALID_CREDENTIALS".into(), "".into()),
        "Identifiants invalides"
    );
    assert_eq!(
        app.invoke_error_text("ACCOUNT_LOCKED".into(), "30".into()),
        "Compte verrouillé. Réessayez dans 30 s."
    );
    assert_eq!(
        app.invoke_error_text("SOMETHING_ELSE".into(), "".into()),
        "Erreur interne"
    );

    auth_state.invoke_register("alice".into(), "password123".into());
    h.pump_until("account created", || {
        auth_state.get_account_exists() && !auth_state.get_busy()
    });
    assert_eq!(auth_state.get_error_code(), "");

    // A wrong password is reported with a localized error.
    auth_state.invoke_login("alice".into(), "wrong-password".into());
    h.pump_until("login refused", || {
        !auth_state.get_busy() && auth_state.get_error_code() != ""
    });
    assert_eq!(auth_state.get_error_code(), "INVALID_CREDENTIALS");
    h.snapshot("02-login-error-fr");

    // Live language switch + dark theme.
    app.invoke_set_lang("en".into());
    assert_eq!(app.get_lang(), "en");
    assert_eq!(
        app.invoke_error_text("INVALID_CREDENTIALS".into(), "".into()),
        "Invalid credentials"
    );
    ui.global::<Theme>().set_dark(true);
    h.snapshot("03-login-error-en-dark");
    app.invoke_set_lang("fr".into());
    ui.global::<Theme>().set_dark(false);

    // ---------- Signed in: security notice, empty profile state (S4) ----------
    auth_state.invoke_login("alice".into(), "password123".into());
    h.pump_until("main", || app.get_screen() == Screen::Main);
    h.pump_until("profiles loaded", || profiles.get_loaded());
    assert_eq!(auth_state.get_username(), "alice");
    assert!(auth_state.get_session_remaining().contains(':'));
    // The failed attempt above is reported once.
    assert!(auth_state.get_notice_open());
    assert_eq!(auth_state.get_notice_failed_attempts(), 1);
    h.snapshot("04-security-notice");
    auth_state.set_notice_open(false);
    assert_eq!(profiles.get_profiles().row_count(), 0);
    h.snapshot("05-empty-profile");

    // Create a profile through the dialog.
    profiles.set_form_open(true);
    h.snapshot("06-profile-form");
    profiles.invoke_save(
        "".into(),
        "Ada".into(),
        "Lovelace".into(),
        "Analytical Engine".into(),
        "".into(),
    );
    h.pump_until("profile created", || {
        !profiles.get_busy() && profiles.get_profiles().row_count() == 1
    });
    assert!(!profiles.get_form_open());
    assert_eq!(profiles.get_current_name(), "Ada Lovelace");
    let ada = profiles.get_profiles().row_data(0).unwrap();
    assert!(ada.active);

    // A second one, then switch the active profile.
    profiles.invoke_save(
        "".into(),
        "Grace".into(),
        "Hopper".into(),
        "Navy".into(),
        "Rear admiral".into(),
    );
    h.pump_until("second profile", || {
        profiles.get_profiles().row_count() == 2
    });
    let grace = profiles
        .get_profiles()
        .iter()
        .find(|p| p.first_name == "Grace")
        .unwrap();
    assert_eq!(grace.poste, "Rear admiral");
    profiles.invoke_set_active(grace.id.clone());
    h.pump_until("grace active", || {
        profiles.get_current_name() == "Grace Hopper"
    });

    // Validation errors surface in the dialog.
    profiles.invoke_save(
        grace.id.clone(),
        "".into(),
        "Hopper".into(),
        "Navy".into(),
        "".into(),
    );
    h.pump_until("validation error", || {
        !profiles.get_busy() && profiles.get_error_code() != ""
    });
    assert_eq!(profiles.get_error_code(), "VALIDATION");

    // Edit, then delete Ada.
    profiles.invoke_save(
        grace.id.clone(),
        "Grace".into(),
        "Hopper".into(),
        "US Navy".into(),
        "".into(),
    );
    h.pump_until("edited", || {
        profiles
            .get_profiles()
            .iter()
            .any(|p| p.enterprise == "US Navy")
    });
    app.set_page(Page::Settings);
    h.snapshot("07-settings");
    profiles.set_delete_name("Ada Lovelace".into());
    profiles.set_delete_open(true);
    h.snapshot("08-delete-confirm");
    profiles.invoke_delete(ada.id.clone());
    h.pump_until("deleted", || profiles.get_profiles().row_count() == 1);
    assert!(!profiles.get_delete_open());

    // ---------- Presence calendar (S5) ----------
    let presence = ui.global::<PresenceState>();
    app.set_page(Page::Presence);
    h.pump_until("calendar loaded", || {
        presence.get_cells().row_count() >= 28 && !presence.get_month_title().is_empty()
    });
    let cell = |day: i32| -> DayCell {
        presence
            .get_cells()
            .iter()
            .find(|c| c.in_month && c.day == day)
            .unwrap()
    };
    assert_eq!(presence.get_weekdays().row_data(0).unwrap(), "lu");
    assert_eq!(presence.get_month_co2(), "0 kg");
    h.snapshot("12-calendar-empty");

    // Office day with a custom car trip (two steps).
    presence.invoke_day_clicked(cell(10).key);
    assert!(presence.get_dialog_open());
    assert_eq!(presence.get_dialog_step(), "type");
    h.snapshot("13-day-dialog-type");
    presence.invoke_select_type("office".into());
    assert_eq!(presence.get_dialog_step(), "commute");
    assert!(!presence.get_can_confirm());
    presence.invoke_choose("custom".into());
    assert!(presence.get_modes().iter().any(|m| m.header));
    presence.invoke_set_segment_mode(0, "car_petrol".into());
    presence.invoke_set_segment_distance(0, "20".into());
    assert!(presence.get_can_confirm());
    assert!(presence.get_segments().row_data(0).unwrap().is_car);
    h.snapshot("14-day-dialog-commute");
    presence.invoke_confirm();
    h.pump_until("office saved", || {
        !presence.get_dialog_open() && cell(10).kind == "office"
    });
    assert!(
        cell(10).value.ends_with("kg"),
        "co2 shown: {}",
        cell(10).value
    );
    assert_ne!(presence.get_month_co2(), "0 kg");

    // Vacation is saved straight from the first step.
    presence.invoke_day_clicked(cell(11).key);
    presence.invoke_select_type("vacation".into());
    h.pump_until("vacation saved", || cell(11).kind == "vacation");
    assert_eq!(cell(11).value, "");

    // Copy day 10, paste onto an empty day, then over a filled one.
    presence.invoke_toggle_copy();
    assert_eq!(presence.get_clipboard(), "picking");
    presence.invoke_day_clicked(cell(13).key); // empty: nothing to copy
    assert_eq!(presence.get_clipboard(), "picking");
    presence.invoke_day_clicked(cell(10).key);
    assert_eq!(presence.get_clipboard(), "pasting");
    presence.invoke_day_clicked(cell(12).key);
    h.pump_until("pasted on empty day", || cell(12).kind == "office");
    presence.invoke_day_clicked(cell(11).key);
    assert!(presence.get_paste_confirm_open());
    h.snapshot("15-paste-confirm");
    presence.invoke_confirm_paste();
    h.pump_until("pasted over vacation", || cell(11).kind == "office");
    presence.invoke_cancel_copy();
    assert_eq!(presence.get_clipboard(), "idle");
    h.snapshot("16-calendar-filled");

    // Cell display: worked hours (persisted in the profile settings).
    presence.invoke_set_show_hours(true);
    h.pump_until("hours mode", || presence.get_show_hours());
    assert_eq!(cell(10).value, "");

    // Re-opening an office day pre-fills its trip; then delete it.
    presence.invoke_day_clicked(cell(12).key);
    h.pump_until("trips prefilled", || {
        presence.get_dialog_choice() == "custom"
    });
    assert_eq!(presence.get_segments().row_data(0).unwrap().distance, "20");
    assert_eq!(presence.get_dialog_kind(), "office");
    presence.invoke_delete();
    h.pump_until("deleted day", || cell(12).kind.is_empty());

    // Rust-computed texts follow the language.
    app.invoke_set_lang("en".into());
    assert!(
        presence
            .get_month_title()
            .chars()
            .next()
            .unwrap()
            .is_uppercase(),
        "{}",
        presence.get_month_title()
    );
    assert_eq!(presence.get_weekdays().row_data(0).unwrap(), "Mo");
    app.invoke_set_lang("fr".into());

    // ---------- Change password ----------
    auth_state.invoke_change_password("password123".into(), "short".into(), "short".into());
    assert_eq!(auth_state.get_password_error_code(), "PASSWORD_TOO_SHORT");
    assert_eq!(
        app.invoke_error_text(
            "PASSWORD_TOO_SHORT".into(),
            auth_state.get_password_error_arg()
        ),
        "Le mot de passe doit contenir au moins 8 caractères."
    );
    auth_state.invoke_change_password("password123".into(), "newpassword1".into(), "other".into());
    assert_eq!(auth_state.get_password_error_code(), "PASSWORD_MISMATCH");
    auth_state.invoke_change_password(
        "nope-nope".into(),
        "newpassword1".into(),
        "newpassword1".into(),
    );
    h.pump_until("wrong current password", || {
        !auth_state.get_password_busy() && auth_state.get_password_error_code() != ""
    });
    assert_eq!(auth_state.get_password_error_code(), "INVALID_CREDENTIALS");
    auth_state.invoke_change_password(
        "password123".into(),
        "newpassword1".into(),
        "newpassword1".into(),
    );
    h.pump_until("password changed", || auth_state.get_password_done());
    h.snapshot("09-password-changed");

    // ---------- About (Slint licence attribution) ----------
    app.set_page(Page::About);
    h.snapshot("10-about");

    // ---------- Session end: inactivity, then absolute expiry ----------
    auth::tick(&ctx, now_ms() + IDLE_TIMEOUT_MS + 1_000);
    h.pump_until("idle logout", || app.get_screen() == Screen::Login);
    assert_eq!(auth_state.get_error_code(), "SESSION_EXPIRED");
    assert!(!profiles.get_loaded());
    assert_eq!(app.get_page(), Page::Presence);
    h.snapshot("11-session-expired");

    // The new password works; logout goes back to the login screen.
    auth_state.invoke_login("alice".into(), "newpassword1".into());
    h.pump_until("main again", || app.get_screen() == Screen::Main);
    h.pump_until("profiles again", || profiles.get_loaded());
    assert_eq!(profiles.get_current_name(), "Grace Hopper");
    auth_state.invoke_logout();
    assert_eq!(app.get_screen(), Screen::Login);
    assert_eq!(auth_state.get_error_code(), "");

    bridge.lock_blocking();
    drop(ui);
    runtime.shutdown_timeout(Duration::from_secs(2));
    let _ = std::fs::remove_dir_all(dir);
}
