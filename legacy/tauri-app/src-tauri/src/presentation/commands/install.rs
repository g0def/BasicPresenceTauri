use std::path::Path;

/// Marker file shipped by the Arch package (`basicpresence-bin/PKGBUILD`).
const PACMAN_MARKER: &str = "/usr/lib/basic-presence/managed-by-pacman";

/// Whether this installation is owned by a system package manager (pacman).
///
/// The Arch package repackages the `.deb`, so the binary reports the `Deb`
/// bundle type: the updater would then try `pkexec dpkg -i`, which does not
/// exist on Arch and would bypass pacman anyway. The frontend uses this to
/// point the user to `pacman -Syu` instead of installing the update itself.
///
/// Not session-gated: it reveals nothing about the vault.
#[tauri::command]
pub fn is_package_managed() -> bool {
    cfg!(target_os = "linux") && is_marked(Path::new(PACMAN_MARKER))
}

fn is_marked(marker: &Path) -> bool {
    marker.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_marker_file_only_when_present() {
        let marker =
            std::env::temp_dir().join(format!("bp_pacman_marker_{}", uuid::Uuid::now_v7()));
        assert!(!is_marked(&marker));
        std::fs::write(&marker, "").unwrap();
        assert!(is_marked(&marker));
        std::fs::remove_file(&marker).unwrap();
    }
}
