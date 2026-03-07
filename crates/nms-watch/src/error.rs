//! Error types for the file watcher.

use std::fmt;
use std::path::PathBuf;

/// Errors from the file watch system.
#[derive(Debug)]
pub enum WatchError {
    /// Save file not found at the expected path.
    SaveNotFound(PathBuf),
    /// File system notification error.
    NotifyError(String),
    /// Error parsing the save file during watch.
    ParseError(String),
}

impl fmt::Display for WatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SaveNotFound(p) => write!(f, "save file not found: {}", p.display()),
            Self::NotifyError(e) => write!(f, "file watcher error: {e}"),
            Self::ParseError(e) => write!(f, "save parse error during watch: {e}"),
        }
    }
}

impl std::error::Error for WatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_error_display_save_not_found() {
        let e = WatchError::SaveNotFound(PathBuf::from("/tmp/missing.json"));
        assert!(e.to_string().contains("save file not found"));
        assert!(e.to_string().contains("missing.json"));
    }

    #[test]
    fn test_watch_error_display_notify_error() {
        let e = WatchError::NotifyError("inotify limit".into());
        assert!(e.to_string().contains("file watcher error"));
    }

    #[test]
    fn test_watch_error_display_parse_error() {
        let e = WatchError::ParseError("invalid JSON".into());
        assert!(e.to_string().contains("save parse error"));
    }
}
