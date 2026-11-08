//! Owner-only access to the app-data directory.
//!
//! `keystore.db`, `vault.db` and their sidecars are encrypted/sealed, but they
//! are still created with the process umask (typically `0755`/`0644`), i.e.
//! readable by every local account. Restricting the directory to `0700` keeps
//! other accounts from even reading the ciphertext (defense in depth against
//! offline attacks), and covers files created later (WAL, `-shm`, `.hmac`)
//! without having to chmod each one. Already-present files are tightened to
//! `0600` too. No-op on non-Unix targets, where the per-user app-data location
//! is already ACL-protected.

use std::io;
use std::path::Path;

/// Create `dir` if needed and restrict it (and the files directly inside it)
/// to the current user.
pub fn prepare_private_data_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    restrict(dir)
}

#[cfg(unix)]
fn restrict(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        // `file_type` does not follow symlinks: never chmod a link target.
        if entry.file_type()?.is_file() {
            std::fs::set_permissions(entry.path(), std::fs::Permissions::from_mode(0o600))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn restrict(_dir: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn restricts_the_directory_and_existing_files_to_the_owner() {
        let dir = std::env::temp_dir().join(format!("bp_data_dir_{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        let file = dir.join("vault.db");
        std::fs::write(&file, b"x").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();

        prepare_private_data_dir(&dir).unwrap();

        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&file), 0o600);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn creates_a_missing_directory_as_owner_only() {
        let dir = std::env::temp_dir()
            .join(format!("bp_data_dir_{}", uuid::Uuid::now_v7()))
            .join("nested");

        prepare_private_data_dir(&dir).unwrap();

        assert_eq!(mode(&dir), 0o700);
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }
}
