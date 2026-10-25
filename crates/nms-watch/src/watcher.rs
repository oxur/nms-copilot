//! File watcher for a selected NMS save file.
//!
//! # Watch behavior
//!
//! The entry points synchronously check and parse the configured path, then
//! watch its parent directory nonrecursively. Only `Any`-kind events for the
//! configured path match: paths compare exactly first, with canonicalization
//! as a fallback only when both paths still exist.
//! Backend notification error batches are ignored by the worker.
//!
//! A file is considered stable after two successful equal-size reads 100 ms
//! apart; this is a heuristic, not a lock or complete-write guarantee.
//! Debouncing and backend coalescing can hide intermediate writes. Quiet
//! successful parses still advance the last-good baseline; failed parses do
//! not advance it or emit updates. The fifth consecutive parse failure warns
//! once on stderr, and a later successful parse resets the count.
//!
//! The legacy [`start_watching`] stream is unbounded and sends only nonempty
//! additive deltas. [`start_watching_updates`] and
//! [`start_watching_observations`] use capacity-one queues and can each have
//! one worker-held pending event. When full, these senders retry every 10 ms
//! and check cancellation; receiver disconnection is noticed only when an
//! attempted send observes it, not by a separate quiet-parse poll.
//!
//! Dropping a handle cancels and joins its worker. A queued event remains
//! readable before disconnection only if its receiver is retained separately;
//! dropping the whole handle also drops that receiver. A worker-held pending
//! event can be discarded. File I/O, parsing, and backend teardown have no
//! hard shutdown deadline. Worker panics are not sent as [`WatchError`] values;
//! owner drop warns best-effort to stderr if joining reports a panic. This
//! library does not write saves, models, or application caches, and emits no
//! normal update output to stdout.
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//! use nms_watch::{WatchConfig, WatchError};
//!
//! let config = WatchConfig::default();
//! assert!(config.save_path.as_os_str().is_empty());
//! assert_eq!(config.debounce, std::time::Duration::from_millis(500));
//! let error = WatchError::SaveNotFound(PathBuf::from("save.hg"));
//! assert_eq!(error.to_string(), "save file not found: save.hg");
//! ```
//!

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, Debouncer, new_debouncer};

use nms_core::delta::{DiscoveryNames, SaveDelta, SaveUpdate};
use nms_core::player::PlayerState;

use crate::delta::compute_delta;
use crate::error::WatchError;
use crate::snapshot::SaveSnapshot;

/// After this many consecutive parse failures, log a warning.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

/// Milliseconds to wait between file size checks for stability.
const FILE_STABILITY_CHECK_MS: u64 = 100;

/// Owner and receiver for the legacy additive-delta stream.
///
/// The channel is unbounded. It emits only nonempty [`SaveDelta`] values,
/// with no initial event; player-state, source-version, and explicit-name-only
/// changes may therefore produce no message. Dropping the handle cancels and
/// joins the worker. See the [module watch contract](crate::watcher#watch-behavior).
pub struct WatchHandle {
    /// Receive nonempty additive deltas from the watcher thread.
    pub receiver: mpsc::Receiver<SaveDelta>,
    /// Owns the debouncer and joined worker lifetime.
    _owner: WatchOwner,
}

/// Handle to a watcher that emits complete parsed save updates.
///
/// The receiver starts empty; starting a watcher does not emit an initial
/// update. Supported state/version changes arrive in source order, while an
/// unchanged parsed save is suppressed. The bounded queue holds one event and
/// the worker may retain one additional parsed event while waiting to enqueue
/// it. Dropping this handle stops and joins the worker; a worker-held pending
/// event may be discarded during shutdown. File I/O and parsing are
/// not subject to a hard shutdown deadline. See the
/// [module watch contract](crate::watcher#watch-behavior).
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use std::time::Duration;
/// use nms_watch::{WatchConfig, WatchError, start_watching_updates};
///
/// fn watch(path: PathBuf) -> Result<(), WatchError> {
///     let handle = start_watching_updates(WatchConfig {
///         save_path: path,
///         debounce: Duration::from_millis(500),
///     })?;
///     if let Ok(update) = handle.receiver.recv_timeout(Duration::from_secs(30)) {
///         let _complete_state = update.player_state;
///     }
///     Ok(())
/// }
/// ```
pub struct WatchUpdateHandle {
    /// Receive updates in source order; the capacity-one queue starts empty.
    pub receiver: mpsc::Receiver<SaveUpdate>,
    /// Owns the debouncer and joined worker lifetime.
    _owner: WatchOwner,
}

/// A complete supported save update paired with its literal discovery names.
///
/// Both values come from the same checked parse. The complete explicit-name
/// observation map can be replaced as a whole: a present empty string means
/// clear, while a missing entry means preserve the model value.
#[derive(Debug, Clone)]
pub struct SaveObservation {
    /// Additive atlas delta, complete supported player state, and source version.
    pub update: SaveUpdate,
    /// Complete explicit system and planet names from the same checked parse.
    ///
    /// See [`nms_core::delta::DiscoveryNames`]: a present empty string means
    /// clear, while a missing map entry means preserve the model value.
    pub discovery_names: DiscoveryNames,
}

/// Handle to a watcher that emits complete updates and checked name maps.
///
/// The bounded receiver is empty at startup. It holds one observation while
/// the worker can retain one additional parsed observation; later events stay
/// in source order. See the [module watch contract](crate::watcher#watch-behavior).
/// Dropping the handle sets cancellation and joins the
/// worker. A worker-held pending event can be discarded during shutdown.
/// File I/O and parsing have no hard
/// shutdown deadline.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use std::time::Duration;
/// use nms_watch::{WatchConfig, WatchError, start_watching_observations};
///
/// fn observe(path: PathBuf) -> Result<(), WatchError> {
///     let handle = start_watching_observations(WatchConfig {
///         save_path: path,
///         debounce: Duration::from_millis(500),
///     })?;
///     if let Ok(observation) = handle.receiver.recv_timeout(Duration::from_secs(30)) {
///         let _complete_names = observation.discovery_names;
///         let _complete_player = observation.update.player_state;
///     }
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns [`WatchError::SaveNotFound`] for a missing path,
/// [`WatchError::ParseError`] when the initial save cannot be parsed, and
/// [`WatchError::NotifyError`] when watcher setup fails. Later parse failures
/// do not advance the baseline or enter the channel; the existing warning and
/// recovery policy is retained.
pub struct WatchObservationHandle {
    /// Receive complete observations in source order; starts empty.
    pub receiver: mpsc::Receiver<SaveObservation>,
    /// Owns the debouncer and joined worker lifetime.
    _owner: WatchOwner,
}

impl fmt::Debug for WatchObservationHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WatchObservationHandle")
            .field("receiver", &self.receiver)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for WatchUpdateHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WatchUpdateHandle")
            .field("receiver", &self.receiver)
            .finish_non_exhaustive()
    }
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

/// Path and debounce interval for a watcher.
///
/// The default path is empty and is not a discovered save location.
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Exact save-file path to parse and match against notifications.
    pub save_path: PathBuf,
    /// Backend debounce interval; the default is 500 milliseconds.
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

/// Check whether two file-size reads 100 ms apart both succeed and match.
///
/// This is only a stability heuristic: equal sizes do not prove an atomic
/// write, complete contents, or a lock against a later change.
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
    discovery_names: DiscoveryNames,
}

impl SourceSnapshot {
    fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let save = nms_save::parse_validated_save_file(path)?;
        Ok(Self {
            snapshot: SaveSnapshot::from_validated_save(&save),
            player_state: save.player_state().clone(),
            save_version: save.root().version,
            discovery_names: nms_graph::extract::extract_discovery_names_from_validated(&save),
        })
    }
}

enum UpdateSender {
    Delta(mpsc::Sender<SaveDelta>),
    Full(mpsc::SyncSender<SaveUpdate>),
    Observations(mpsc::SyncSender<SaveObservation>),
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

fn send_observation(
    sender: &mpsc::SyncSender<SaveObservation>,
    shutdown: &AtomicBool,
    mut observation: SaveObservation,
) -> bool {
    loop {
        if shutdown.load(Ordering::Acquire) {
            return false;
        }
        match sender.try_send(observation) {
            Ok(()) => return true,
            Err(mpsc::TrySendError::Disconnected(_)) => return false,
            Err(mpsc::TrySendError::Full(pending)) => {
                observation = pending;
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Start a watcher that sends nonempty additive atlas deltas.
///
/// The returned unbounded receiver starts empty. A successful parse sends a
/// delta only when it contains new keyed records, a base change, or player
/// movement; changes only to existing system/planet metadata, supported
/// player fields, source version, or explicit names can remain quiet. Backend
/// coalescing means this is not a lossless record of every file write.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::PathBuf;
/// use std::time::Duration;
/// use nms_watch::{start_watching, WatchConfig};
///
/// let handle = start_watching(WatchConfig {
///     save_path: PathBuf::from("save.hg"),
///     debounce: Duration::from_millis(500),
/// })?;
/// if let Ok(delta) = handle.receiver.recv_timeout(Duration::from_secs(30)) {
///     println!("{} changes", delta.change_count());
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`WatchError::SaveNotFound`] when the configured path is missing,
/// [`WatchError::ParseError`] when the initial checked parse fails, or
/// [`WatchError::NotifyError`] when debouncer, parent-directory, registration,
/// or worker setup fails. Later parser failures are not returned here.
///
/// # Panics
///
/// The initial parser can propagate its documented malformed compressed-size
/// offset panic on 32-bit targets or invalid bundled-mapping panic.
pub fn start_watching(config: WatchConfig) -> Result<WatchHandle, WatchError> {
    let (sender, receiver) = mpsc::channel();
    let owner = start_watching_inner(config, UpdateSender::Delta(sender))?;
    Ok(WatchHandle {
        receiver,
        _owner: owner,
    })
}

/// Start watching and emit complete supported player updates.
///
/// This includes all represented [`nms_core::player::PlayerState`] fields and source version
/// when either differs, even if the atlas delta is empty. It does not include
/// discovery-name observations; use [`start_watching_observations`] for that
/// stream.
///
/// Each event pairs the existing additive atlas delta with the complete
/// supported player state and source version from one parse. No initial event
/// is sent. Events are emitted when supported state, source version, or atlas
/// delta changes; unchanged parsed saves are suppressed. The capacity-one
/// queue preserves source order and the worker can hold one additional parsed
/// event while waiting for capacity. Dropping the returned handle stops and
/// joins the worker; a worker-held pending event may be discarded on shutdown.
/// File I/O and parsing have no hard shutdown deadline. This library API does
/// not coordinate application cache writers. See the
/// [module watch contract](crate::watcher#watch-behavior).
///
/// # Examples
///
/// See [`WatchUpdateHandle`] for a compiled filesystem example.
///
/// # Errors
///
/// Returns [`WatchError::SaveNotFound`] when the path is missing,
/// [`WatchError::ParseError`] when the initial save cannot be parsed, or
/// [`WatchError::NotifyError`] when notification or worker setup fails.
///
/// # Panics
///
/// The initial parser can propagate its documented malformed compressed-size
/// offset panic on 32-bit targets or invalid bundled-mapping panic.
pub fn start_watching_updates(config: WatchConfig) -> Result<WatchUpdateHandle, WatchError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let owner = start_watching_inner(config, UpdateSender::Full(sender))?;
    Ok(WatchUpdateHandle {
        receiver,
        _owner: owner,
    })
}

/// Start watching and emit complete supported updates with checked names.
///
/// The emitted name map is the complete set of explicit names from the same
/// parse as its update. It can report name-only changes, removed map entries,
/// and explicitly empty names. Absence from the map is not itself a request
/// for the consumer to clear a model value. The existing complete-update
/// predicate also emits when player state, source version, or atlas delta
/// changes. The update, source version, and complete explicit name map come
/// from the same validated parse. No initial event is sent. The capacity-one
/// queue and worker-held pending event use the complete-update stream's
/// ordering and shutdown behavior.
///
/// # Examples
///
/// See [`WatchObservationHandle`] for a compiled filesystem example showing
/// the complete names and player state received together.
///
/// # Errors
///
/// Returns [`WatchError::SaveNotFound`] for a missing path,
/// [`WatchError::ParseError`] for an initial parse failure, or
/// [`WatchError::NotifyError`] for watcher setup failures.
///
/// # Panics
///
/// The initial parser can propagate its documented malformed compressed-size
/// offset panic on 32-bit targets or invalid bundled-mapping panic.
pub fn start_watching_observations(
    config: WatchConfig,
) -> Result<WatchObservationHandle, WatchError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let owner = start_watching_inner(config, UpdateSender::Observations(sender))?;
    Ok(WatchObservationHandle {
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
                        let names_changed = source.discovery_names != new_source.discovery_names;

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
                            UpdateSender::Observations(update_tx) if changed || names_changed => {
                                send_observation(
                                    update_tx,
                                    &shutdown_worker,
                                    SaveObservation {
                                        update: SaveUpdate {
                                            delta,
                                            player_state: new_source.player_state.clone(),
                                            save_version: new_source.save_version,
                                        },
                                        discovery_names: new_source.discovery_names.clone(),
                                    },
                                )
                            }
                            UpdateSender::Observations(_) => true,
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

    fn save_bytes_with_system_name(name: &str) -> Vec<u8> {
        let name = name.replace('\\', "\\\\").replace('"', "\\\"");
        let row = format!(
            r#"{{"DD":{{"UA":"0x00070000000064","DT":"SolarSystem","VP":[]}},"DM":{{"CN":"{name}"}},"OWS":{{"LID":"","UID":"1","USN":"Author","PTK":"ST","TS":1700000000}},"FL":{{"U":1}}}}"#
        );
        let save = String::from_utf8(save_bytes(0, 0)).expect("watcher fixture is UTF-8");
        let replaced = save.replace("\"Record\": []", &format!("\"Record\": [{row}]"));
        assert_ne!(replaced, save, "watcher fixture record placeholder exists");
        replaced.into_bytes()
    }

    fn save_bytes_with_system_dm(dm: &str, address: &str, version: u32, units: u64) -> Vec<u8> {
        let row = format!(
            r#"{{"DD":{{"UA":"{address}","DT":"SolarSystem","VP":[]}},"DM":{dm},"OWS":{{"LID":"","UID":"1","USN":"Author","PTK":"ST","TS":1700000000}},"FL":{{"U":1}}}}"#
        );
        let mut save = String::from_utf8(save_bytes(0, 0)).expect("watcher fixture is UTF-8");
        save = save.replace("\"Version\": 4720", &format!("\"Version\": {version}"));
        save = save.replace("\"Units\": 7,", &format!("\"Units\": {units},"));
        let replaced = save.replace("\"Record\": []", &format!("\"Record\": [{row}]"));
        assert_ne!(replaced, save, "watcher fixture record placeholder exists");
        replaced.into_bytes()
    }

    fn literal_observation(version: u32, name: &str) -> SaveObservation {
        let address = nms_core::address::GalacticAddress::new(100, 0, 0, 7, 0, 0);
        let id = nms_core::system::SystemId::from_address(&address);
        SaveObservation {
            update: SaveUpdate {
                delta: SaveDelta::empty(),
                player_state: PlayerState::new(address, 0, None, None, 7, 8, 9),
                save_version: version,
            },
            discovery_names: DiscoveryNames {
                system_names: std::collections::HashMap::from([(id, name.to_owned())]),
                planet_names: std::collections::HashMap::new(),
            },
        }
    }

    struct ObservationWorkerGuard {
        shutdown: Arc<AtomicBool>,
        worker: Option<JoinHandle<()>>,
        receiver: Option<mpsc::Receiver<SaveObservation>>,
    }

    struct BoundedObservationHandle {
        handle: Option<WatchObservationHandle>,
        receiver: Option<mpsc::Receiver<SaveObservation>>,
        shutdown: Arc<AtomicBool>,
    }

    impl BoundedObservationHandle {
        fn new(mut handle: WatchObservationHandle) -> Self {
            let shutdown = Arc::clone(&handle._owner.shutdown);
            let receiver = std::mem::replace(&mut handle.receiver, mpsc::sync_channel(1).1);
            Self {
                handle: Some(handle),
                receiver: Some(receiver),
                shutdown,
            }
        }

        fn drop_and_retain_receiver(mut self) -> Result<mpsc::Receiver<SaveObservation>, String> {
            drop_observation_owner(
                self.handle
                    .take()
                    .expect("bounded owner retains its handle"),
                self.receiver
                    .take()
                    .expect("bounded owner retains its receiver"),
                Arc::clone(&self.shutdown),
            )
        }
    }

    impl Drop for BoundedObservationHandle {
        fn drop(&mut self) {
            if let (Some(handle), Some(receiver)) = (self.handle.take(), self.receiver.take()) {
                let _ = drop_observation_owner(handle, receiver, Arc::clone(&self.shutdown));
            }
        }
    }

    fn drop_observation_owner(
        handle: WatchObservationHandle,
        receiver: mpsc::Receiver<SaveObservation>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<mpsc::Receiver<SaveObservation>, String> {
        let (completed_tx, completed_rx) = mpsc::channel();
        let dropper = thread::spawn(move || {
            drop(handle);
            let _ = completed_tx.send(());
        });
        if completed_rx.recv_timeout(Duration::from_secs(1)).is_err() {
            shutdown.store(true, Ordering::Release);
            drop(receiver);
            dropper
                .join()
                .map_err(|_| "bounded owner cleanup thread panicked".to_owned())?;
            return Err("dropping observation owner exceeded one second".into());
        }
        dropper
            .join()
            .map_err(|_| "bounded owner cleanup thread panicked".to_owned())?;
        Ok(receiver)
    }

    impl Drop for ObservationWorkerGuard {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::Release);
            drop(self.receiver.take());
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
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
                "\"PreviousUniverseAddress\": {{\"RealityIndex\": {previous_reality}, \"GalacticAddress\": {{\"VoxelX\": {previous_x}, \"VoxelY\": 3, \"VoxelZ\": 4, \"SolarSystemIndex\": 1, \"PlanetIndex\": 1}}}},"
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
        assert!(format!("{handle:?}").starts_with("WatchUpdateHandle"));

        std::fs::write(&path, save_bytes_with_state(0, 0, 4720, 70, 80, 90, None))
            .expect("write currency-only save");
        let currency_update = receive_update(&handle);
        assert!(currency_update.delta.is_empty());
        assert_eq!(
            currency_update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                70,
                80,
                90,
            )
        );
        assert_eq!(currency_update.save_version, 4720);

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4720, 70, 80, 90, Some((9, 3))),
        )
        .expect("write previous-address-only save");
        let previous_update = receive_update(&handle);
        assert!(previous_update.delta.is_empty());
        assert_eq!(
            previous_update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                Some(nms_core::address::GalacticAddress::new(3, 3, 4, 1, 1, 9)),
                None,
                70,
                80,
                90,
            )
        );
        assert_eq!(previous_update.save_version, 4720);

        std::fs::write(
            &path,
            save_bytes_with_state(0, 0, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("write version-only save");
        let version_update = receive_update(&handle);
        assert!(version_update.delta.is_empty());
        assert_eq!(
            version_update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                Some(nms_core::address::GalacticAddress::new(3, 3, 4, 1, 1, 9)),
                None,
                70,
                80,
                90,
            )
        );
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
            save_bytes_with_state(9, 1, 4722, 700, 800, 900, Some((0, 2))),
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
        assert_eq!(
            moved.to,
            nms_core::address::GalacticAddress::new(1, 0, 0, 1, 0, 9)
        );
        assert_eq!(
            movement_update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(1, 0, 0, 1, 0, 9),
                9,
                Some(nms_core::address::GalacticAddress::new(2, 3, 4, 1, 1, 0)),
                None,
                700,
                800,
                900,
            )
        );
        assert_eq!(movement_update.save_version, 4722);
    }

    #[test]
    fn test_observation_stream_emits_name_only_changes_and_legacy_streams_stay_quiet() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes_with_system_name("First name"))
            .expect("write initial named save");
        let config = || WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        };
        let delta = start_watching(config()).expect("start legacy delta watcher");
        let full = start_watching_updates(config()).expect("start legacy full watcher");
        let observations =
            start_watching_observations(config()).expect("start paired name observation watcher");
        assert!(matches!(
            observations
                .receiver
                .recv_timeout(Duration::from_millis(250)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(&path, save_bytes_with_system_name("Second name"))
            .expect("write name-only save change");
        let observation = observations
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("paired stream emits a name-only change");
        assert!(observation.update.delta.is_empty());
        assert_eq!(observation.update.save_version, 4720);
        assert_eq!(
            observation.update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                7,
                8,
                9,
            )
        );
        assert_eq!(observation.update.save_version, 4720);
        assert!(observation.update.delta.is_empty());
        let expected_id = nms_core::system::SystemId::from_address(
            &nms_core::address::GalacticAddress::new(100, 0, 0, 7, 0, 0),
        );
        assert_eq!(
            observation.discovery_names.system_names,
            std::collections::HashMap::from([(expected_id, "Second name".to_owned())])
        );
        assert!(observation.discovery_names.planet_names.is_empty());
        assert!(matches!(
            delta.receiver.recv_timeout(Duration::from_millis(700)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        assert!(matches!(
            full.receiver.recv_timeout(Duration::from_millis(700)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
    }

    #[test]
    fn test_observation_stream_emits_clear_and_omission_but_quiets_equivalent_dm() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(
            &path,
            save_bytes_with_system_dm(r#"{"CN":"Present"}"#, "0x00070000000064", 4720, 7),
        )
        .expect("write initial explicit name");
        let observations = start_watching_observations(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start paired watcher");
        assert!(matches!(
            observations
                .receiver
                .recv_timeout(Duration::from_millis(250)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(
            &path,
            save_bytes_with_system_dm(r#"{"CN":""}"#, "0x00070000000064", 4720, 7),
        )
        .expect("write explicit empty clear");
        let clear = observations
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("empty string is an emitted explicit clear");
        let id = nms_core::system::SystemId::from_address(
            &nms_core::address::GalacticAddress::new(100, 0, 0, 7, 0, 0),
        );
        assert!(clear.update.delta.is_empty());
        assert_eq!(clear.update.save_version, 4720);
        assert_eq!(
            clear.update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                7,
                8,
                9,
            )
        );
        assert_eq!(
            clear.discovery_names.system_names,
            std::collections::HashMap::from([(id, String::new())])
        );
        assert!(clear.discovery_names.planet_names.is_empty());

        std::fs::write(
            &path,
            save_bytes_with_system_dm("{}", "0x00070000000064", 4720, 7),
        )
        .expect("remove the name field");
        let omitted = observations
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("clear to absent emits the complete empty map");
        assert!(omitted.update.delta.is_empty());
        assert_eq!(omitted.update.save_version, 4720);
        assert_eq!(
            omitted.update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                7,
                8,
                9,
            )
        );
        assert!(omitted.discovery_names.system_names.is_empty());
        assert!(omitted.discovery_names.planet_names.is_empty());

        for dm in [r#"{"CN":null}"#, r#"{"CN":7}"#, "[]"] {
            std::fs::write(
                &path,
                save_bytes_with_system_dm(dm, "0x00070000000064", 4720, 7),
            )
            .expect("write equivalent absent/non-string name map");
            assert!(matches!(
                observations
                    .receiver
                    .recv_timeout(Duration::from_millis(450)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
        }
        std::fs::write(
            &path,
            save_bytes_with_system_dm("{}", "0x00070000000064", 4720, 7),
        )
        .expect("rewrite identical supported save");
        assert!(matches!(
            observations
                .receiver
                .recv_timeout(Duration::from_millis(450)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(
            &path,
            save_bytes_with_system_dm(
                r#"{"CN":"Must not advance baseline"}"#,
                "0x100000000000000",
                4720,
                7,
            ),
        )
        .expect("write invalid 57-bit address with attempted name");
        assert!(matches!(
            observations
                .receiver
                .recv_timeout(Duration::from_millis(450)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        std::fs::write(
            &path,
            save_bytes_with_system_dm(r#"{"CN":"Recovered"}"#, "0x00070000000064", 4721, 7),
        )
        .expect("recover from the rejected address against the last good baseline");
        let recovered = observations
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("recovery emits the accepted literal name");
        assert_eq!(recovered.update.save_version, 4721);
        assert!(recovered.update.delta.is_empty());
        assert_eq!(
            recovered.update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                7,
                8,
                9,
            )
        );
        assert_eq!(
            recovered.discovery_names.system_names,
            std::collections::HashMap::from([(id, "Recovered".to_owned())])
        );
    }

    #[test]
    fn test_observation_stream_forwards_complete_names_on_currency_only_update() {
        let _guard = native_watch_test_guard();
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        let initial =
            save_bytes_with_system_dm(r#"{"CN":"Stable name"}"#, "0x00070000000064", 4720, 7);
        std::fs::write(&path, &initial).expect("write named source");
        let observations = start_watching_observations(WatchConfig {
            save_path: path.clone(),
            debounce: Duration::from_millis(20),
        })
        .expect("start paired watcher");
        assert!(matches!(
            observations
                .receiver
                .recv_timeout(Duration::from_millis(250)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        std::fs::write(
            &path,
            save_bytes_with_system_dm(r#"{"CN":"Stable name"}"#, "0x00070000000064", 4720, 70),
        )
        .expect("write currency-only change with same explicit map");
        let observation = observations
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("full update predicate emits currency-only event");
        assert!(observation.update.delta.is_empty());
        assert_eq!(observation.update.save_version, 4720);
        assert_eq!(
            observation.update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                70,
                8,
                9,
            )
        );
        let id = nms_core::system::SystemId::from_address(
            &nms_core::address::GalacticAddress::new(100, 0, 0, 7, 0, 0),
        );
        assert_eq!(
            observation.discovery_names.system_names,
            std::collections::HashMap::from([(id, "Stable name".to_owned())])
        );
        assert!(observation.discovery_names.planet_names.is_empty());
    }

    #[test]
    fn test_send_observation_preserves_source_order_and_complete_maps() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        assert!(send_observation(
            &sender,
            &shutdown,
            literal_observation(4721, "First"),
        ));

        let worker_shutdown = Arc::clone(&shutdown);
        let worker_sender = sender.clone();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (completed_tx, completed_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            entered_tx.send(()).expect("test remains active");
            let sent = send_observation(
                &worker_sender,
                &worker_shutdown,
                literal_observation(4722, "Second"),
            );
            completed_tx.send(sent).expect("test observes completion");
        });
        let _worker_guard = ObservationWorkerGuard {
            shutdown: Arc::clone(&shutdown),
            worker: Some(worker),
            receiver: Some(receiver),
        };

        entered_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("second source update worker entered");
        let first = _worker_guard
            .receiver
            .as_ref()
            .expect("guard retains observation receiver")
            .recv_timeout(Duration::from_secs(1))
            .expect("first observation remains ahead of the pending second");
        assert_eq!(first.update.save_version, 4721);
        assert_eq!(
            first.discovery_names.system_names,
            literal_observation(4721, "First")
                .discovery_names
                .system_names
        );
        let second = _worker_guard
            .receiver
            .as_ref()
            .expect("guard retains observation receiver")
            .recv_timeout(Duration::from_secs(1))
            .expect("second observation follows the first");
        assert_eq!(second.update.save_version, 4722);
        assert_eq!(
            second.discovery_names.system_names,
            literal_observation(4722, "Second")
                .discovery_names
                .system_names
        );
        assert!(
            completed_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("second send completes")
        );
    }

    #[test]
    fn test_send_observation_cancellation_preserves_queued_event() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        assert!(send_observation(
            &sender,
            &shutdown,
            literal_observation(4721, "Queued"),
        ));

        let worker_shutdown = Arc::clone(&shutdown);
        let worker_sender = sender.clone();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (completed_tx, completed_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            entered_tx.send(()).expect("test remains active");
            let sent = send_observation(
                &worker_sender,
                &worker_shutdown,
                literal_observation(4722, "Pending"),
            );
            completed_tx.send(sent).expect("test observes completion");
        });
        let _worker_guard = ObservationWorkerGuard {
            shutdown: Arc::clone(&shutdown),
            worker: Some(worker),
            receiver: Some(receiver),
        };

        entered_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("pending sender entered");
        assert!(matches!(
            completed_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        shutdown.store(true, Ordering::Release);
        assert!(
            !completed_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("canceled pending send completes")
        );
        let queued = _worker_guard
            .receiver
            .as_ref()
            .expect("guard retains observation receiver")
            .recv_timeout(Duration::from_secs(1))
            .expect("cancellation preserves the already queued event");
        assert_eq!(queued.update.save_version, 4721);
        assert_eq!(
            queued.discovery_names.system_names,
            literal_observation(4721, "Queued")
                .discovery_names
                .system_names
        );
        drop(sender);
        assert!(matches!(
            _worker_guard
                .receiver
                .as_ref()
                .expect("guard retains observation receiver")
                .recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn test_observation_handle_drop_joins_idle_queued_and_blocked_workers() {
        let make_handle = |receiver, shutdown, worker| WatchObservationHandle {
            receiver,
            _owner: WatchOwner {
                debouncer: None,
                shutdown,
                worker: Some(worker),
            },
        };

        let (idle_sender, idle_receiver) = mpsc::sync_channel(1);
        let idle_shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&idle_shutdown);
        let (idle_ready_tx, idle_ready_rx) = mpsc::channel();
        let idle_worker = thread::spawn(move || {
            idle_ready_tx.send(()).expect("test remains active");
            while !worker_shutdown.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(1));
            }
            drop(idle_sender);
        });
        let idle_owner =
            BoundedObservationHandle::new(make_handle(idle_receiver, idle_shutdown, idle_worker));
        idle_ready_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("idle worker started");
        let idle_receiver = idle_owner
            .drop_and_retain_receiver()
            .expect("idle owner drops within bound");
        assert!(matches!(
            idle_receiver.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));

        let (queued_sender, queued_receiver) = mpsc::sync_channel(1);
        let queued_shutdown = Arc::new(AtomicBool::new(false));
        assert!(send_observation(
            &queued_sender,
            &queued_shutdown,
            literal_observation(4721, "Queued"),
        ));
        let worker_sender = queued_sender.clone();
        let queued_worker = thread::spawn(move || drop(worker_sender));
        let queued_owner = BoundedObservationHandle::new(make_handle(
            queued_receiver,
            queued_shutdown,
            queued_worker,
        ));
        let queued_receiver = queued_owner
            .drop_and_retain_receiver()
            .expect("queued owner drops within bound");
        let queued = queued_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("queued observation survives owner drop");
        assert_eq!(queued.update.save_version, 4721);
        assert_eq!(
            queued.discovery_names.system_names,
            literal_observation(4721, "Queued")
                .discovery_names
                .system_names
        );
        drop(queued_sender);
        assert!(matches!(
            queued_receiver.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));

        let (blocked_sender, blocked_receiver) = mpsc::sync_channel(1);
        let blocked_shutdown = Arc::new(AtomicBool::new(false));
        assert!(send_observation(
            &blocked_sender,
            &blocked_shutdown,
            literal_observation(4721, "Retained"),
        ));
        let worker_shutdown = Arc::clone(&blocked_shutdown);
        let worker_sender = blocked_sender.clone();
        let (blocked_entered_tx, blocked_entered_rx) = mpsc::channel();
        let (blocked_completed_tx, blocked_completed_rx) = mpsc::channel();
        let blocked_worker = thread::spawn(move || {
            blocked_entered_tx.send(()).expect("test remains active");
            let sent = send_observation(
                &worker_sender,
                &worker_shutdown,
                literal_observation(4722, "Discarded"),
            );
            blocked_completed_tx
                .send(sent)
                .expect("test observes blocked completion");
        });
        let blocked_owner = BoundedObservationHandle::new(make_handle(
            blocked_receiver,
            blocked_shutdown,
            blocked_worker,
        ));
        blocked_entered_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("blocked sender entered");
        assert!(matches!(
            blocked_completed_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        let blocked_receiver = blocked_owner
            .drop_and_retain_receiver()
            .expect("blocked owner drops within bound");
        assert!(
            !blocked_completed_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("owner shutdown releases blocked sender")
        );
        let retained = blocked_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("blocked pending event does not replace queued event");
        assert_eq!(retained.update.save_version, 4721);
        assert_eq!(
            retained.discovery_names.system_names,
            literal_observation(4721, "Retained")
                .discovery_names
                .system_names
        );
        drop(blocked_sender);
        assert!(matches!(
            blocked_receiver.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
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
            save_bytes_with_state(0, 0, 4721, 70, 80, 90, Some((9, 3))),
        )
        .expect("write version-only save");
        assert!(matches!(
            handle.receiver.recv_timeout(Duration::from_millis(400)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        std::fs::write(
            &path,
            save_bytes_with_state(9, 1, 4722, 70, 80, 90, Some((9, 3))),
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
        let initial_save = nms_save::parse_save_file(&path).expect("parse initial save");
        let mut model = nms_graph::GalaxyModel::from_save(&initial_save);
        let imported_address = nms_core::address::GalacticAddress::new(12, 4, 5, 3, 0, 4);
        let imported_system_id = nms_graph::SystemId::from_address(&imported_address);
        let imported_planet = nms_core::system::Planet::new(
            1,
            Some(nms_core::biome::Biome::Lush),
            None,
            false,
            Some("Imported leaf".into()),
            None,
        );
        model.insert_system(nms_core::system::System::new(
            imported_address,
            Some("Imported system".into()),
            None,
            None,
            vec![imported_planet],
        ));
        model.insert_base(nms_core::player::PlayerBase::new(
            "Imported base".into(),
            nms_core::player::BaseType::HomePlanetBase,
            imported_address,
            [1.0, 2.0, 3.0],
            Some("import-owner".into()),
        ));
        model.set_active_galaxy(7);
        let original_player_state = model.player_state.clone();
        let original_systems = model.systems.clone();
        let original_planets = model.planets.clone();
        let original_bases = model.bases.clone();
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
            assert_eq!(model.player_state, original_player_state);
            assert_eq!(model.systems, original_systems);
            assert_eq!(model.planets, original_planets);
            assert_eq!(model.bases, original_bases);
            assert_eq!(model.active_galaxy, 7);
        }

        let replacement = dir.path().join("replacement.json");
        std::fs::write(
            &replacement,
            save_bytes_with_state(9, 1, 4721, 70, 80, 90, None),
        )
        .expect("write valid replacement save");
        std::fs::rename(&replacement, &path).expect("atomically replace selected save");
        let update = receive_update(&handle);
        let moved = update
            .delta
            .player_moved
            .as_ref()
            .expect("movement from retained baseline");
        assert_eq!(
            moved.from,
            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0)
        );
        assert_eq!(
            moved.to,
            nms_core::address::GalacticAddress::new(1, 0, 0, 1, 0, 9)
        );
        assert_eq!(
            update.player_state,
            PlayerState::new(
                nms_core::address::GalacticAddress::new(1, 0, 0, 1, 0, 9),
                9,
                None,
                None,
                70,
                80,
                90,
            )
        );
        assert_eq!(update.save_version, 4721);
        model.apply_save_update(&update);
        assert_eq!(model.player_state, Some(update.player_state));
        assert_eq!(model.active_galaxy, 9);
        assert_eq!(model.systems, original_systems);
        assert_eq!(model.planets, original_planets);
        assert_eq!(model.bases, original_bases);
        assert_eq!(
            model.system(&imported_system_id).unwrap().name.as_deref(),
            Some("Imported system")
        );
        assert_eq!(
            model
                .planets
                .get(&(imported_system_id, 1))
                .unwrap()
                .name
                .as_deref(),
            Some("Imported leaf")
        );
        assert_eq!(
            model
                .bases
                .get("imported base")
                .unwrap()
                .owner_uid
                .as_deref(),
            Some("import-owner")
        );
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
