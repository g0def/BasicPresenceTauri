// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKit on Wayland crashes without this when GPU DMA-BUF is unavailable.
    // SAFETY: first statement of `main`, before Tauri (or anything else) has
    // spawned a thread, so no other thread can read the environment concurrently.
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
    };

    basic_presence_lib::run()
}
