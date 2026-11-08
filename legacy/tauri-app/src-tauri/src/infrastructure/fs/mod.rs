//! Crash-safe file replacement.
//!
//! Writing straight onto the target leaves a truncated file if the process dies
//! (or the disk fills up) mid-write — and the previous good copy is already
//! gone. Instead, write a sibling temporary file, flush it to disk, then
//! atomically rename it over the target: readers see either the old file or the
//! complete new one, never a partial one.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// Sibling temporary path, in the same directory (rename is only atomic within
/// one filesystem), unique per process and call.
fn temp_sibling(target: &Path) -> io::Result<PathBuf> {
    let name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no file name"))?;
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).map_err(|e| io::Error::other(e.to_string()))?;
    let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let mut tmp = name.to_os_string();
    tmp.push(format!(".{suffix}.tmp"));
    Ok(target.with_file_name(tmp))
}

/// Flush a directory entry change (rename) to disk. No-op where directories
/// cannot be opened (Windows), where rename durability is handled by the OS.
pub fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}

/// Atomically replace `target` with what `write` produces at the temporary path
/// it is given. On any failure the temporary file is removed and `target` is
/// left untouched.
pub fn replace_with<E>(
    target: &Path,
    write: impl FnOnce(&Path) -> Result<(), E>,
    io_error: impl Fn(io::Error) -> E,
) -> Result<(), E> {
    let tmp = temp_sibling(target).map_err(&io_error)?;
    let result = write(&tmp).and_then(|()| {
        File::open(&tmp)
            .and_then(|f| f.sync_all())
            .map_err(&io_error)?;
        std::fs::rename(&tmp, target).map_err(&io_error)?;
        if let Some(dir) = target.parent() {
            sync_dir(dir).map_err(&io_error)?;
        }
        Ok(())
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Atomically replace `target` with `bytes`.
pub fn write_atomic(target: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(target, |tmp| std::fs::write(tmp, bytes), |e| e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bp-atomic-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn replaces_the_target_and_leaves_no_temp_file() {
        let dir = temp_dir("ok");
        let target = dir.join("out.json");
        std::fs::write(&target, b"old").unwrap();
        write_atomic(&target, b"new").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_failed_write_keeps_the_previous_file() {
        let dir = temp_dir("fail");
        let target = dir.join("out.ods");
        std::fs::write(&target, b"old").unwrap();
        let result = replace_with(
            &target,
            |tmp| {
                std::fs::write(tmp, b"partial").unwrap();
                Err::<(), io::Error>(io::Error::other("disk full"))
            },
            |e| e,
        );
        assert!(result.is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"old");
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            1,
            "temp file cleaned up"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
