//! File system watcher for NMS save files.
//!
//! Uses `notify` with debouncing to detect save file modifications.
//! Includes robustness features: file stability checks, consecutive
//! failure counting, and graceful error recovery.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};

use nms_core::delta::SaveDelta;

use crate::delta::compute_delta;
use crate::error::WatchError;
use crate::snapshot::SaveSnapshot;

/// After this many consecutive parse failures, log a warning.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

/// Milliseconds to wait between file size checks for stability.
const FILE_STABILITY_CHECK_MS: u64 = 100;

/// Handle to a running file watcher.
///
/// Dropping this stops the watcher thread.
pub struct WatchHandle {
    /// Receive deltas from the watcher thread.
    pub receiver: mpsc::Receiver<SaveDelta>,
    /// Keep the debouncer alive (dropping it stops watching).
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

/// Configuration for the file watcher.
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Path to the save file to watch.
    pub save_path: PathBuf,
    /// Debounce duration (default: 500ms).
    pub debounce: Duration,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            save_path: PathBuf::new(),
            debounce: Duration::from_millis(500),
        }
    }
}

/// Check that a file's size is stable (not being actively written).
///
/// Takes two size readings separated by `FILE_STABILITY_CHECK_MS` and
/// returns `true` only if both readings succeed and match.
fn is_file_stable(path: &Path) -> bool {
    let size1 = std::fs::metadata(path).map(|m| m.len()).ok();
    thread::sleep(Duration::from_millis(FILE_STABILITY_CHECK_MS));
    let size2 = std::fs::metadata(path).map(|m| m.len()).ok();
    size1 == size2 && size1.is_some()
}

/// Start watching a save file for changes.
///
/// Returns a `WatchHandle` whose `receiver` yields `SaveDelta` values
/// whenever the save file is modified.
///
/// # Errors
///
/// Returns `WatchError::SaveNotFound` if the save file does not exist,
/// `WatchError::ParseError` if the initial parse fails, or
/// `WatchError::NotifyError` if the file watcher cannot be created.
pub fn start_watching(config: WatchConfig) -> Result<WatchHandle, WatchError> {
    if !config.save_path.exists() {
        return Err(WatchError::SaveNotFound(config.save_path));
    }

    let (delta_tx, delta_rx) = mpsc::channel();
    let save_path = config.save_path.clone();

    // Take an initial snapshot for diffing
    let initial_snapshot =
        SaveSnapshot::from_file(&save_path).map_err(|e| WatchError::ParseError(e.to_string()))?;

    // Channel for notify events (internal)
    let (notify_tx, notify_rx) = mpsc::channel();

    let mut debouncer = new_debouncer(config.debounce, notify_tx)
        .map_err(|e| WatchError::NotifyError(e.to_string()))?;

    // Watch the parent directory (more reliable than watching the file directly,
    // since games often write to a temp file and rename).
    let watch_dir = save_path.parent().unwrap_or(&save_path);

    debouncer
        .watcher()
        .watch(watch_dir, notify::RecursiveMode::NonRecursive)
        .map_err(|e| WatchError::NotifyError(e.to_string()))?;

    // Background thread: receive notify events, re-parse, diff, send deltas
    let save_path_bg = save_path.clone();
    thread::spawn(move || {
        let mut snapshot = initial_snapshot;
        let mut consecutive_failures: usize = 0;

        for events in notify_rx {
            let events = match events {
                Ok(evts) => evts,
                Err(_) => continue,
            };

            // Only react if our save file was modified
            let relevant = events
                .iter()
                .any(|e| e.kind == DebouncedEventKind::Any && e.path == save_path_bg);
            if !relevant {
                continue;
            }

            // File stability check: ensure file size is stable before parsing
            if !is_file_stable(&save_path_bg) {
                continue;
            }

            // Re-parse save file
            match SaveSnapshot::from_file(&save_path_bg) {
                Ok(new_snapshot) => {
                    consecutive_failures = 0;
                    let delta = compute_delta(&snapshot, &new_snapshot);

                    if !delta.is_empty() && delta_tx.send(delta).is_err() {
                        break;
                    }

                    snapshot = new_snapshot;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    if consecutive_failures == MAX_CONSECUTIVE_FAILURES {
                        eprintln!(
                            "Warning: {MAX_CONSECUTIVE_FAILURES} consecutive parse failures. \
                             Save file may be corrupt or format changed: {e}"
                        );
                    }
                    // Keep watching -- the next save might succeed
                }
            }
        }
    });

    Ok(WatchHandle {
        receiver: delta_rx,
        _debouncer: debouncer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default_debounce() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce, Duration::from_millis(500));
    }

    #[test]
    fn test_start_watching_nonexistent_file_errors() {
        let config = WatchConfig {
            save_path: PathBuf::from("/tmp/nonexistent_nms_save_12345.json"),
            ..Default::default()
        };
        assert!(matches!(
            start_watching(config),
            Err(WatchError::SaveNotFound(_))
        ));
    }

    #[test]
    fn test_is_file_stable_for_static_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stable.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(is_file_stable(&path));
    }

    #[test]
    fn test_is_file_stable_nonexistent_returns_false() {
        assert!(!is_file_stable(Path::new(
            "/tmp/nonexistent_nms_stable_check_xyz"
        )));
    }

    #[test]
    fn test_max_consecutive_failures_constant() {
        assert!(MAX_CONSECUTIVE_FAILURES > 0);
    }

    #[test]
    fn test_file_stability_check_ms_constant() {
        assert!(FILE_STABILITY_CHECK_MS > 0);
        assert!(FILE_STABILITY_CHECK_MS <= 1000);
    }
}
