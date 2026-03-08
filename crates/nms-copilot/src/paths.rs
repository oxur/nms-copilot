//! Path resolution for NMS Copilot data files.

use std::path::PathBuf;

/// Base directory for NMS Copilot data: `~/.nms-copilot/`.
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".nms-copilot")
}

/// Path to the history file: `~/.nms-copilot/history.txt`.
pub fn history_path() -> PathBuf {
    data_dir().join("history.txt")
}

/// Path to the config file: `~/.nms-copilot/config.toml`.
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// Default cache path (legacy): `~/.nms-copilot/galaxy.rkyv`.
pub fn cache_path() -> PathBuf {
    data_dir().join("galaxy.rkyv")
}

/// Per-save cache path: `~/.nms-copilot/<account_dir>/<save_stem>/galaxy.rkyv`.
///
/// Derives cache location from the save file path, using the parent directory
/// name (account) and file stem (no extension) as path components.
/// For example, `…/st_76561198025707979/save3.hg` becomes
/// `~/.nms-copilot/st_76561198025707979/save3/galaxy.rkyv`.
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

/// Ensure the data directory exists.
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
