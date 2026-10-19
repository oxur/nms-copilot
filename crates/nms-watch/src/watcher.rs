//! File system watcher for NMS save files.
//!
//! Uses `notify` with debouncing to detect save file modifications.
//! Includes robustness features: file stability checks, consecutive
//! failure counting, and graceful error recovery.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, Debouncer, new_debouncer};

use nms_core::delta::{SaveDelta, SaveUpdate};
use nms_core::player::PlayerState;

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
    /// Owns the debouncer and joined worker lifetime.
    _owner: WatchOwner,
}

/// Handle to a watcher that emits complete parsed save updates.
pub struct WatchUpdateHandle {
    /// Receive complete parsed updates in source order.
    pub receiver: mpsc::Receiver<SaveUpdate>,
    /// Owns the debouncer and joined worker lifetime.
    _owner: WatchOwner,
}

struct WatchOwner {
    debouncer: Option<Debouncer<notify::RecommendedWatcher>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for WatchOwner {
    fn drop(&mut self) {
        drop(self.debouncer.take());
        self.shutdown.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            let _ = writeln!(
                std::io::stderr().lock(),
                "Warning: save watcher worker panicked during shutdown"
            );
        }
    }
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

/// Complete supported state derived from one parse of the selected save.
struct SourceSnapshot {
    snapshot: SaveSnapshot,
    player_state: PlayerState,
    save_version: u32,
}

impl SourceSnapshot {
    fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let save = nms_save::parse_save_file(path)?;
        Ok(Self {
            snapshot: SaveSnapshot::from_save(&save),
            player_state: save.to_core_player_state(),
            save_version: save.version,
        })
    }
}

enum UpdateSender {
    Delta(mpsc::Sender<SaveDelta>),
    Full(mpsc::SyncSender<SaveUpdate>),
}

fn send_full_update(
    sender: &mpsc::SyncSender<SaveUpdate>,
    shutdown: &AtomicBool,
    mut update: SaveUpdate,
) -> bool {
    loop {
        if shutdown.load(Ordering::Acquire) {
            return false;
        }
        match sender.try_send(update) {
            Ok(()) => return true,
            Err(mpsc::TrySendError::Disconnected(_)) => return false,
            Err(mpsc::TrySendError::Full(pending)) => {
                update = pending;
                thread::sleep(Duration::from_millis(10));
            }
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
    let (sender, receiver) = mpsc::channel();
    let owner = start_watching_inner(config, UpdateSender::Delta(sender))?;
    Ok(WatchHandle {
        receiver,
        _owner: owner,
    })
}

/// Start watching a save and emit complete supported player updates.
///
/// Each event pairs the existing additive atlas delta with the complete
/// supported player state and source version from one parse. The capacity-one
/// queue preserves source order during normal operation and retains a pending
/// event while the receiver catches up. Dropping this handle stops and joins
/// the worker. This library API does not coordinate application cache writers.
///
/// # Errors
///
/// Returns [`WatchError::SaveNotFound`] when the path is missing,
/// [`WatchError::ParseError`] when the initial save cannot be parsed, or
/// [`WatchError::NotifyError`] when notification or worker setup fails.
pub fn start_watching_updates(config: WatchConfig) -> Result<WatchUpdateHandle, WatchError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let owner = start_watching_inner(config, UpdateSender::Full(sender))?;
    Ok(WatchUpdateHandle {
        receiver,
        _owner: owner,
    })
}

fn start_watching_inner(
    config: WatchConfig,
    sender: UpdateSender,
) -> Result<WatchOwner, WatchError> {
    if !config.save_path.exists() {
        return Err(WatchError::SaveNotFound(config.save_path));
    }

    let save_path = config.save_path.clone();

    let initial_source =
        SourceSnapshot::from_file(&save_path).map_err(|e| WatchError::ParseError(e.to_string()))?;

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

    // Background worker owns the parse baseline and all event processing.
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_worker = Arc::clone(&shutdown);
    let save_path_bg = save_path.clone();
    let worker = thread::Builder::new()
        .name("nms-save-watch".into())
        .spawn(move || {
            let mut source = initial_source;
            let mut consecutive_failures: usize = 0;

            while !shutdown_worker.load(Ordering::Acquire) {
                let events = match notify_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(events) => events,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                let events = match events {
                    Ok(evts) => evts,
                    Err(_) => continue,
                };

                // Only react if our save file was modified
                let relevant = events.iter().any(|e| {
                    e.kind == DebouncedEventKind::Any
                        && event_matches_save_path(&e.path, &save_path_bg)
                });
                if !relevant {
                    continue;
                }

                // File stability check: ensure file size is stable before parsing
                if !is_file_stable(&save_path_bg) {
                    continue;
                }

                // Re-parse save file
                match SourceSnapshot::from_file(&save_path_bg) {
                    Ok(new_source) => {
                        consecutive_failures = 0;
                        let delta = compute_delta(&source.snapshot, &new_source.snapshot);
                        let changed = !delta.is_empty()
                            || source.player_state != new_source.player_state
                            || source.save_version != new_source.save_version;

                        let sent = match &sender {
                            UpdateSender::Delta(delta_tx) if !delta.is_empty() => {
                                delta_tx.send(delta).is_ok()
                            }
                            UpdateSender::Delta(_) => true,
                            UpdateSender::Full(update_tx) if changed => send_full_update(
                                update_tx,
                                &shutdown_worker,
                                SaveUpdate {
                                    delta,
                                    player_state: new_source.player_state.clone(),
                                    save_version: new_source.save_version,
                                },
                            ),
                            UpdateSender::Full(_) => true,
                        };
                        if !sent {
                            break;
                        }

                        source = new_source;
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
        })
        .map_err(|e| WatchError::NotifyError(format!("failed to start watcher worker: {e}")))?;

    Ok(WatchOwner {
        debouncer: Some(debouncer),
        shutdown,
        worker: Some(worker),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static NATIVE_WATCH_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn native_watch_test_guard() -> std::sync::MutexGuard<'static, ()> {
        NATIVE_WATCH_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

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

    fn save_bytes_with_state(
        reality_index: u8,
        voxel_x: i16,
        version: u32,
        units: u64,
        nanites: u64,
        quicksilver: u64,
        previous: Option<(u8, i16)>,
    ) -> Vec<u8> {
        let mut json =
            String::from_utf8(save_bytes(reality_index, voxel_x)).expect("fixture is UTF-8 JSON");
        json = json.replace("\"Version\": 4720", &format!("\"Version\": {version}"));
        json = json.replace("\"Units\": 7,", &format!("\"Units\": {units},"));
        json = json.replace("\"Nanites\": 8,", &format!("\"Nanites\": {nanites},"));
        json = json.replace("\"Specials\": 9,", &format!("\"Specials\": {quicksilver},"));
        if let Some((previous_reality, previous_x)) = previous {
            let previous_json = format!(
                "\"PreviousUniverseAddress\": {{\"RealityIndex\": {previous_reality}, \"GalacticAddress\": {{\"VoxelX\": {previous_x}, \"VoxelY\": 3, \"VoxelZ\": 4, \"SolarSystemIndex\": 5, \"PlanetIndex\": 1}}}},"
            );
            json = json.replacen(
                "\"Units\":",
                &format!("{previous_json}\n                        \"Units\":"),
                1,
            );
        }
        json.into_bytes()
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

    fn receive_update(handle: &WatchUpdateHandle) -> SaveUpdate {
        handle
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher observes a real save-file update")
    }

    #[test]
    fn test_start_watching_updates_observes_state_and_version_only_changes() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 7, 8, 9, None))
            .expect("write initial save");
        let handle = start_watching_updates(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start full update watcher");

        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 70, 80, 90, None))
            .expect("write currency-only save");
        let currency_update = receive_update(&handle);
        assert!(currency_update.delta.is_empty());
        assert_eq!(currency_update.player_state.units, 70);
        assert_eq!(currency_update.player_state.nanites, 80);
        assert_eq!(currency_update.player_state.quicksilver, 90);
        assert_eq!(currency_update.player_state.freighter_address, None);
        assert_eq!(currency_update.save_version, 4720);

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4720, 70, 80, 90, Some((9, 3))),
        )
        .expect("write previous-address-only save");
        let previous_update = receive_update(&handle);
        assert!(previous_update.delta.is_empty());
        assert_eq!(previous_update.player_state.units, 70);
        assert_eq!(
            previous_update
                .player_state
                .previous_address
                .unwrap()
                .voxel_x(),
            3
        );
        assert_eq!(previous_update.save_version, 4720);

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("write version-only save");
        let version_update = receive_update(&handle);
        assert!(version_update.delta.is_empty());
        assert_eq!(version_update.player_state.units, 70);
        assert_eq!(version_update.save_version, 4721);

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("rewrite identical supported state");
        assert!(matches!(
            handle.receiver.recv_timeout(Duration::from_millis(400)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        let other_path = dir.path().join("different.json");
        std::fs::write(
            &other_path,
            save_bytes_with_state(9, 9, 4721, 700, 800, 900, Some((9, 3))),
        )
        .expect("write another file beside selected save");
        assert!(matches!(
            handle.receiver.recv_timeout(Duration::from_millis(400)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(
            &path,
            save_bytes_with_state(9, 1, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("write movement save");
        let movement_update = receive_update(&handle);
        let moved = movement_update
            .delta
            .player_moved
            .expect("movement is included");
        assert_eq!(
            moved.from,
            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0)
        );
        assert_eq!(moved.to.reality_index, 9);
        assert_eq!(moved.to.voxel_x(), 1);
        assert_eq!(movement_update.player_state.units, 70);
        assert_eq!(movement_update.player_state.nanites, 80);
        assert_eq!(movement_update.player_state.quicksilver, 90);
        assert_eq!(movement_update.save_version, 4721);
    }

    #[test]
    fn test_start_watching_keeps_state_only_changes_out_of_legacy_stream() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 7, 8, 9, None))
            .expect("write initial save");
        let handle = start_watching(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start legacy watcher");

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4720, 70, 80, 90, Some((9, 3))),
        )
        .expect("write state-only save");
        assert!(matches!(
            handle.receiver.recv_timeout(Duration::from_millis(400)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(
            &path,
            save_bytes_with_state(9, 1, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("write movement save");
        let delta = handle
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("legacy stream retains movement");
        let moved = delta.player_moved.expect("movement");
        assert_eq!(moved.to.reality_index, 9);
        assert_eq!(moved.to.voxel_x(), 1);
    }

    #[test]
    fn test_send_full_update_cancellation_releases_full_queue_worker() {
        let address = nms_core::address::GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let update = move |save_version| SaveUpdate {
            delta: SaveDelta::empty(),
            player_state: PlayerState::new(address, 0, None, None, 7, 8, 9),
            save_version,
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(update(4721))
            .expect("fill capacity-one channel");
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_sender = sender.clone();
        let worker =
            thread::spawn(move || send_full_update(&worker_sender, &worker_shutdown, update(4722)));
        thread::sleep(Duration::from_millis(30));
        shutdown.store(true, Ordering::Release);
        assert!(!worker.join().expect("worker exits after cancellation"));
        assert_eq!(
            receiver.recv().expect("queued event remains").save_version,
            4721
        );
    }

    #[test]
    fn test_send_full_update_preserves_source_order() {
        let address = nms_core::address::GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let update = move |save_version| SaveUpdate {
            delta: SaveDelta::empty(),
            player_state: PlayerState::new(address, 0, None, None, 7, 8, 9),
            save_version,
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        let shutdown = AtomicBool::new(false);
        assert!(send_full_update(&sender, &shutdown, update(4721)));
        let second_sender = sender.clone();
        let second =
            thread::spawn(move || send_full_update(&second_sender, &shutdown, update(4722)));
        assert_eq!(
            receiver.recv().expect("first queued event").save_version,
            4721
        );
        assert!(second.join().expect("second event is enqueued"));
        assert_eq!(
            receiver.recv().expect("second queued event").save_version,
            4722
        );
    }

    #[test]
    fn test_full_watcher_malformed_input_does_not_advance_baseline() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 7, 8, 9, None))
            .expect("write initial valid save");
        let handle = start_watching_updates(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start full update watcher");

        for attempt in 0..5 {
            let malformed = format!("{{{}", " ".repeat(attempt + 1));
            std::fs::write(&path, malformed).expect("write malformed JSON attempt");
            assert!(nms_save::parse_save_file(&path).is_err());
            assert!(matches!(
                handle.receiver.recv_timeout(Duration::from_millis(400)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
        }

        std::fs::write(&path, save_bytes_with_state(9, 1, 4721, 70, 80, 90, None))
            .expect("restore valid save");
        let update = receive_update(&handle);
        let moved = update
            .delta
            .player_moved
            .expect("movement from retained baseline");
        assert_eq!(moved.from.reality_index, 0);
        assert_eq!(moved.from.voxel_x(), 0);
        assert_eq!(moved.to.reality_index, 9);
        assert_eq!(moved.to.voxel_x(), 1);
        assert_eq!(update.player_state.units, 70);
        assert_eq!(update.save_version, 4721);
    }

    #[test]
    fn test_full_watcher_drop_joins_idle_and_after_update_workers() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 7, 8, 9, None))
            .expect("write initial save");
        let idle = start_watching_updates(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start idle full watcher");
        drop(idle);

        std::fs::write(&path, save_bytes_with_state(0, 0, 4721, 70, 80, 90, None))
            .expect("write state-only update");
        let mut handle = start_watching_updates(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start watcher for real update");
        std::fs::write(&path, save_bytes_with_state(9, 1, 4722, 70, 80, 90, None))
            .expect("write movement update");
        let queued = receive_update(&handle);
        assert_eq!(queued.save_version, 4722);
        let receiver = std::mem::replace(&mut handle.receiver, mpsc::channel().1);
        drop(handle);
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
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
        let _guard = native_watch_test_guard();
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
        let _guard = native_watch_test_guard();
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
