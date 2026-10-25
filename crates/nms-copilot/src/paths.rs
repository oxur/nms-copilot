//! Build paths for NMS Copilot data files.
//!
//! Path builders use the current user's home directory and return lexical
//! paths; they do not inspect save files or create the returned files. The
//! account and slot components in [`cache_path_for_save`] come from the
//! immediate parent directory name and file stem. This mapping is not
//! canonicalized, and distinct save paths can therefore map to the same cache.
//!
//! # Examples
//!
//! ```no_run
//! use nms_copilot::paths::{cache_path, cache_path_for_save, config_path, data_dir, history_path};
//! use std::path::Path;
//!
//! let root = data_dir();
//! assert_eq!(root.join("history.txt"), history_path());
//! assert_eq!(root.join("config.toml"), config_path());
//! assert_eq!(root.join("galaxy.rkyv"), cache_path());
//! assert_eq!(root.join("st_fixture/save3/galaxy.rkyv"), cache_path_for_save(Path::new("st_fixture/save3.hg")));
//! assert_eq!(root.join("default/save/galaxy.rkyv"), cache_path_for_save(Path::new("save.hg")));
//! ```
//!
//! [`ensure_data_dir`] is the explicit filesystem operation: it creates only
//! the data root directory and returns any creation error.

use std::path::PathBuf;

/// Return `~/.nms-copilot/`.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".nms-copilot")
}

/// Return the lexical path `~/.nms-copilot/history.txt` without creating it.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn history_path() -> PathBuf {
    data_dir().join("history.txt")
}

/// Return the lexical path `~/.nms-copilot/config.toml` without reading or creating it.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// Return the legacy lexical cache path `~/.nms-copilot/galaxy.rkyv`.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn cache_path() -> PathBuf {
    data_dir().join("galaxy.rkyv")
}

/// Per-save cache path: `~/.nms-copilot/<account_dir>/<save_stem>/galaxy.rkyv`.
///
/// Derive a lexical cache path from a save path without checking that it exists.
///
/// The immediate parent directory name supplies the account component (or
/// `default` when absent or not UTF-8); the file stem supplies the slot (or
/// `save` when absent or not UTF-8). The result is
/// `~/.nms-copilot/<account>/<slot>/galaxy.rkyv`. It is not canonicalized, so
/// different save paths with the same parent name and stem can collide.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn cache_path_for_save(save_path: &std::path::Path) -> PathBuf {
    let save_stem = save_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("save");
    let account_dir = save_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("default");
    data_dir()
        .join(account_dir)
        .join(save_stem)
        .join("galaxy.rkyv")
}

/// Create the data root directory recursively if needed.
///
/// This does not create the config, history, or cache files. Resolving the
/// root inherits [`data_dir`]’s panic if the home directory is unavailable.
///
/// # Errors
///
/// Returns the filesystem error if creating the data directory fails.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn ensure_data_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_ends_with_nms_copilot() {
        let dir = data_dir();
        assert!(dir.ends_with(".nms-copilot"));
    }

    #[test]
    fn test_history_path_under_data_dir() {
        let path = history_path();
        assert!(path.starts_with(data_dir()));
        assert_eq!(path.file_name().unwrap(), "history.txt");
    }

    #[test]
    fn test_config_path_under_data_dir() {
        let path = config_path();
        assert!(path.starts_with(data_dir()));
        assert_eq!(path.file_name().unwrap(), "config.toml");
    }

    #[test]
    fn test_cache_path_under_data_dir() {
        let path = cache_path();
        assert!(path.starts_with(data_dir()));
        assert_eq!(path.file_name().unwrap(), "galaxy.rkyv");
    }

    #[test]
    fn test_cache_path_for_save_uses_account_and_stem() {
        let save = std::path::Path::new("/nms/st_76561198025707979/save3.hg");
        let path = cache_path_for_save(save);
        assert!(path.starts_with(data_dir()));
        assert!(path.ends_with("st_76561198025707979/save3/galaxy.rkyv"));
    }

    #[test]
    fn test_cache_path_for_save_slot1_manual() {
        let save = std::path::Path::new("/nms/st_12345/save.hg");
        let path = cache_path_for_save(save);
        assert!(path.ends_with("st_12345/save/galaxy.rkyv"));
    }

    #[test]
    fn test_ensure_data_dir_creates_directory() {
        ensure_data_dir().unwrap();
        assert!(data_dir().is_dir());
    }
}
