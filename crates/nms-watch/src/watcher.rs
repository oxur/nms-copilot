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

/// Return whether an event path identifies the currently configured save file.
///
/// Notification backends may report paths through a different spelling of the
/// same directory. Canonicalization is a fallback only: both paths must still
/// exist for their canonical forms to identify the same file.
fn event_matches_save_path(event_path: &Path, save_path: &Path) -> bool {
    if event_path == save_path {
        return true;
    }

    match (event_path.canonicalize(), save_path.canonicalize()) {
        (Ok(event_path), Ok(save_path)) => event_path == save_path,
        _ => false,
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
    let watch_dir = save_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let watch_dir = watch_dir
        .canonicalize()
        .map_err(|e| WatchError::NotifyError(e.to_string()))?;

    debouncer
        .watcher()
        .watch(&watch_dir, notify::RecursiveMode::NonRecursive)
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
            let relevant = events.iter().any(|e| {
                e.kind == DebouncedEventKind::Any && event_matches_save_path(&e.path, &save_path_bg)
            });
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

    fn save_bytes(reality_index: u8, voxel_x: i16) -> Vec<u8> {
        format!(
            r#"{{
                "Version": 4720,
                "Platform": "Mac|Final",
                "ActiveContext": "Main",
                "CommonStateData": {{"SaveName": "LiveWatch", "TotalPlayTime": 100}},
                "BaseContext": {{
                    "GameMode": 1,
                    "PlayerStateData": {{
                        "UniverseAddress": {{
                            "RealityIndex": {reality_index},
                            "GalacticAddress": {{
                                "VoxelX": {voxel_x},
                                "VoxelY": 0,
                                "VoxelZ": 0,
                                "SolarSystemIndex": 1,
                                "PlanetIndex": 0
                            }}
                        }},
                        "Units": 7,
                        "Nanites": 8,
                        "Specials": 9,
                        "PersistentPlayerBases": []
                    }}
                }},
                "ExpeditionContext": {{
                    "GameMode": 6,
                    "PlayerStateData": {{
                        "UniverseAddress": {{
                            "RealityIndex": 0,
                            "GalacticAddress": {{
                                "VoxelX": 0,
                                "VoxelY": 0,
                                "VoxelZ": 0,
                                "SolarSystemIndex": 0,
                                "PlanetIndex": 0
                            }}
                        }},
                        "Units": 0,
                        "Nanites": 0,
                        "Specials": 0,
                        "PersistentPlayerBases": []
                    }}
                }},
                "DiscoveryManagerData": {{
                    "DiscoveryData-v1": {{
                        "ReserveStore": 0,
                        "ReserveManaged": 0,
                        "Store": {{"Record": []}}
                    }}
                }}
            }}"#
        )
        .into_bytes()
    }

    fn receive_movement(handle: &WatchHandle, reality_index: u8, voxel_x: i16) {
        let delta = handle
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher observes a real save-file change");
        let moved = delta.player_moved.expect("player movement is detected");
        assert_eq!(moved.to.reality_index, reality_index);
        assert_eq!(moved.to.voxel_x(), voxel_x);
    }

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
        const { assert!(MAX_CONSECUTIVE_FAILURES > 0) };
    }

    #[test]
    fn test_file_stability_check_ms_constant() {
        const { assert!(FILE_STABILITY_CHECK_MS > 0) };
        const { assert!(FILE_STABILITY_CHECK_MS <= 1000) };
    }

    #[test]
    fn test_event_matches_save_path_exact_and_canonical_identity() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let save_path = dir.path().join("save.json");
        std::fs::write(&save_path, b"save").expect("create save file");
        let canonical_path = save_path.canonicalize().expect("canonical save path");

        assert!(event_matches_save_path(&save_path, &save_path));
        assert!(event_matches_save_path(&canonical_path, &save_path));

        let other_dir = tempfile::tempdir().expect("other temporary directory");
        let other_save = other_dir.path().join("save.json");
        std::fs::write(&other_save, b"other save").expect("create other save file");
        assert!(!event_matches_save_path(&other_save, &save_path));

        let missing_a = dir.path().join("missing-a.json");
        let missing_b = other_dir.path().join("missing-a.json");
        assert!(!event_matches_save_path(&missing_a, &missing_b));
        assert!(event_matches_save_path(&missing_a, &missing_a));
    }

    #[test]
    fn test_start_watching_relative_directory_and_bare_filename() {
        let cwd = std::env::current_dir().expect("current directory");
        let relative_dir = tempfile::Builder::new()
            .prefix("nms-watch-relative-")
            .tempdir_in(&cwd)
            .expect("temporary directory under current directory");
        let relative_dir_path = relative_dir
            .path()
            .strip_prefix(&cwd)
            .expect("temporary directory is beneath current directory");
        let relative_save = relative_dir_path.join("save.json");
        std::fs::write(&relative_save, save_bytes(0, 0)).expect("write relative save");
        let relative_handle = start_watching(WatchConfig {
            save_path: relative_save.clone(),
            ..Default::default()
        })
        .expect("watch relative parent directory");
        std::fs::write(&relative_save, save_bytes(9, 0)).expect("update relative save");
        receive_movement(&relative_handle, 9, 0);

        let bare_file = tempfile::NamedTempFile::new_in(&cwd)
            .expect("temporary bare filename under current directory");
        let bare_name = bare_file
            .path()
            .file_name()
            .expect("temporary filename")
            .to_owned();
        let bare_save = PathBuf::from(bare_name);
        std::fs::write(&bare_save, save_bytes(0, 0)).expect("write bare save");
        let bare_handle = start_watching(WatchConfig {
            save_path: bare_save.clone(),
            ..Default::default()
        })
        .expect("watch bare filename through current directory");
        std::fs::write(&bare_save, save_bytes(0, 1)).expect("update bare save");
        receive_movement(&bare_handle, 0, 1);
    }

    #[cfg(unix)]
    #[test]
    fn test_start_watching_directory_symlink_alias_for_write_and_replacement() {
        use std::os::unix::fs::symlink;

        let parent = tempfile::tempdir().expect("temporary parent directory");
        let real_dir = parent.path().join("real");
        let alias_dir = parent.path().join("alias");
        std::fs::create_dir(&real_dir).expect("create real save directory");
        symlink(&real_dir, &alias_dir).expect("create directory symlink alias");

        let save_path = alias_dir.join("save.json");
        std::fs::write(&save_path, save_bytes(0, 0)).expect("write initial save");
        let handle = start_watching(WatchConfig {
            save_path: save_path.clone(),
            ..Default::default()
        })
        .expect("watch through caller's directory alias");

        std::fs::write(&save_path, save_bytes(9, 0)).expect("write through directory alias");
        receive_movement(&handle, 9, 0);

        let replacement = alias_dir.join("replacement.json");
        std::fs::write(&replacement, save_bytes(0, 1)).expect("write replacement save");
        std::fs::rename(&replacement, &save_path).expect("atomically replace through alias");
        receive_movement(&handle, 0, 1);
    }
}
