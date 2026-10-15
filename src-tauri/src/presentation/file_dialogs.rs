//! Native file dialogs, opened by the backend.
//!
//! File paths never cross the IPC boundary: the WebView asks for an export or
//! an import, the Rust side shows the dialog and uses the path the *user*
//! picked. A compromised WebView therefore cannot turn the export/import
//! commands into "write this file anywhere" / "probe that file" primitives
//! (audit finding M1).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::domain::error::DomainError;

/// Keep only a plain file name from a suggestion sent by the WebView: no
/// directory part, no traversal, bounded length.
fn sanitize_file_name(suggested: &str, extension: &str) -> String {
    let base: String = suggested
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .take(120)
        .collect();
    let base = base.trim().trim_start_matches('.');
    let base = if base.is_empty() { "export" } else { base };
    if base.to_lowercase().ends_with(&format!(".{extension}")) {
        base.to_string()
    } else {
        format!("{base}.{extension}")
    }
}

/// Make sure the chosen path carries the expected extension.
fn with_extension(path: PathBuf, extension: &str) -> PathBuf {
    let has_it = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(extension));
    if has_it {
        path
    } else {
        let mut os = path.into_os_string();
        os.push(format!(".{extension}"));
        PathBuf::from(os)
    }
}

fn dialog_error(e: impl std::fmt::Display) -> DomainError {
    DomainError::Export(format!("file dialog: {e}"))
}

/// Show a "save as" dialog; `None` when the user cancels.
pub async fn pick_save_path(
    app: &AppHandle,
    suggested_name: &str,
    filter_name: &str,
    extension: &'static str,
) -> Result<Option<PathBuf>, DomainError> {
    let app = app.clone();
    let file_name = sanitize_file_name(suggested_name, extension);
    let filter_name = filter_name.to_string();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut builder = app
            .dialog()
            .file()
            .add_filter(filter_name, &[extension])
            .set_file_name(file_name);
        if let Some(window) = app.get_webview_window("main") {
            builder = builder.set_parent(&window);
        }
        builder.blocking_save_file()
    })
    .await
    .map_err(dialog_error)?;
    picked
        .map(|p| p.into_path().map(|p| with_extension(p, extension)))
        .transpose()
        .map_err(dialog_error)
}

/// Show an "open file" dialog; `None` when the user cancels.
pub async fn pick_open_path(
    app: &AppHandle,
    filter_name: &str,
    extension: &'static str,
) -> Result<Option<PathBuf>, DomainError> {
    let app = app.clone();
    let filter_name = filter_name.to_string();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut builder = app.dialog().file().add_filter(filter_name, &[extension]);
        if let Some(window) = app.get_webview_window("main") {
            builder = builder.set_parent(&window);
        }
        builder.blocking_pick_file()
    })
    .await
    .map_err(dialog_error)?;
    picked
        .map(|p| p.into_path())
        .transpose()
        .map_err(dialog_error)
}

/// How long an inspected bundle stays importable.
const PENDING_IMPORT_TTL: Duration = Duration::from_secs(15 * 60);

/// Bundles the user picked and inspected, awaiting the import step, keyed by
/// an unguessable single-use handle. The WebView only ever holds the handle.
#[derive(Default)]
pub struct PendingImports {
    entries: Mutex<HashMap<String, (PathBuf, Instant)>>,
}

impl PendingImports {
    pub fn insert(&self, path: PathBuf) -> Result<String, DomainError> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|e| DomainError::Crypto(e.to_string()))?;
        let handle: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        entries.retain(|_, (_, at)| at.elapsed() < PENDING_IMPORT_TTL);
        entries.insert(handle.clone(), (path, Instant::now()));
        Ok(handle)
    }

    /// Consume a handle (single use).
    pub fn take(&self, handle: &str) -> Option<PathBuf> {
        let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        match entries.remove(handle) {
            Some((path, at)) if at.elapsed() < PENDING_IMPORT_TTL => Some(path),
            _ => None,
        }
    }

    /// Forget everything (logout / exit).
    pub fn clear(&self) {
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggested_names_cannot_carry_a_directory() {
        assert_eq!(sanitize_file_name("../../.bashrc", "ods"), "bashrc.ods");
        assert_eq!(sanitize_file_name("/etc/passwd", "json"), "passwd.json");
        assert_eq!(sanitize_file_name("C:\\x\\y.json", "json"), "y.json");
        assert_eq!(sanitize_file_name("Ada 2026.ods", "ods"), "Ada 2026.ods");
        assert_eq!(sanitize_file_name("", "ods"), "export.ods");
    }

    #[test]
    fn chosen_paths_get_the_expected_extension() {
        assert_eq!(
            with_extension(PathBuf::from("/tmp/a"), "ods"),
            PathBuf::from("/tmp/a.ods")
        );
        assert_eq!(
            with_extension(PathBuf::from("/tmp/a.ODS"), "ods"),
            PathBuf::from("/tmp/a.ODS")
        );
    }

    #[test]
    fn import_handles_are_single_use() {
        let pending = PendingImports::default();
        let handle = pending.insert(PathBuf::from("/tmp/b.json")).unwrap();
        assert_eq!(handle.len(), 32);
        assert_eq!(pending.take(&handle), Some(PathBuf::from("/tmp/b.json")));
        assert_eq!(pending.take(&handle), None);
        assert_eq!(pending.take("forged"), None);
    }
}
