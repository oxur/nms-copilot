//! Errors reported while preparing a file watcher.
//!
//! These errors cover path existence, initial checked parsing, and watcher
//! setup. Later parse failures are handled by the running worker and are not
//! sent as [`WatchError`] values.

use std::fmt;
use std::path::PathBuf;

/// An error encountered before a watcher starts.
///
/// The string payloads preserve the underlying setup or parser message, but
/// this type does not expose a typed [`std::error::Error::source`] chain.
#[derive(Debug)]
pub enum WatchError {
    /// The configured path did not exist when watcher setup began.
    SaveNotFound(
        /// Configured path that failed the existence check.
        PathBuf,
    ),
    /// Debouncer, parent-directory, watch-registration, or worker setup failed.
    NotifyError(
        /// Display message from the setup failure.
        String,
    ),
    /// The initial checked save parse failed.
    ParseError(
        /// Display message from the parser.
        String,
    ),
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
