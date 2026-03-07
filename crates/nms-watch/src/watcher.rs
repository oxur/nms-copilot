//! File system watcher for NMS save files.
//!
//! Uses `notify` with debouncing to detect save file modifications.

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};

use nms_core::delta::SaveDelta;

use crate::delta::compute_delta;
use crate::error::WatchError;
use crate::snapshot::SaveSnapshot;

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

        for events in notify_rx {
            let events = match events {
                Ok(evts) => evts,
                Err(_) => continue,
            };

            // Only react if our save file was modified
            let dominated_event = events
                .iter()
                .any(|e| e.kind == DebouncedEventKind::Any && e.path == save_path_bg);
            if !dominated_event {
                continue;
            }

            // Re-parse save file
            let new_snapshot = match SaveSnapshot::from_file(&save_path_bg) {
                Ok(s) => s,
                Err(_) => continue, // Partial write or permission error -- skip
            };

            // Compute delta
            let delta = compute_delta(&snapshot, &new_snapshot);

            if !delta.is_empty() {
                // Send delta; if receiver is dropped, stop
                if delta_tx.send(delta).is_err() {
                    break;
                }
            }

            snapshot = new_snapshot;
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
}
