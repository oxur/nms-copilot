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

/// Path to the cache file: `~/.nms-copilot/galaxy.rkyv`.
pub fn cache_path() -> PathBuf {
    data_dir().join("galaxy.rkyv")
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
    fn test_ensure_data_dir_creates_directory() {
        ensure_data_dir().unwrap();
        assert!(data_dir().is_dir());
    }
}
