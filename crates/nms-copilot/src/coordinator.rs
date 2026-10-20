//! One application-owned path from parsed save updates through the live model
//! to the optional cache and read-only observers.
//!
//! Observer channels are unbounded. Slow subscribers therefore do not delay
//! application or cache writes, but callers must drain their receiver to bound
//! their own memory use. A subscriber receives events published after its
//! registration; there is no initial snapshot or replay. An update already in
//! flight may be published after subscription because publication follows the
//! serialized apply/write operation.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use nms_cache::CacheData;
use nms_core::SaveUpdate;
use nms_graph::GalaxyModel;
use nms_watch::{WatchConfig, WatchUpdateHandle, start_watching_updates};
use tokio::sync::RwLock;

/// A complete parsed update after it has been applied to the shared model.
#[derive(Debug, Clone)]
pub struct AppliedSaveUpdate {
    /// Complete parsed player state, atlas delta, and source version.
    pub update: SaveUpdate,
    /// Existing human-readable discovery/movement notes, formatted pre-apply.
    pub notifications: Vec<String>,
}

/// Failure to start the owned live-update producer.
#[derive(Debug)]
pub enum CoordinatorStartError {
    Watch(nms_watch::WatchError),
    Worker(std::io::Error),
}

impl fmt::Display for CoordinatorStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Watch(error) => write!(
                formatter,
                "could not start save update coordinator: {error}"
            ),
            Self::Worker(error) => write!(formatter, "could not start coordinator worker: {error}"),
        }
    }
}

impl std::error::Error for CoordinatorStartError {}

/// Owns the single watcher producer and serialized live cache writer.
///
/// Drop signals shutdown and joins the worker, which in turn drops and joins
/// the watcher. An update already being applied or written is completed before
/// Drop returns. Queued updates are not promised to flush, and parser or disk
/// I/O has no hard completion deadline. Callers must release model guards
/// before dropping this coordinator.
pub struct LiveSaveCoordinator {
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    observers: Arc<Mutex<Vec<mpsc::Sender<AppliedSaveUpdate>>>>,
}

impl fmt::Debug for LiveSaveCoordinator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveSaveCoordinator")
            .field("worker_running", &self.worker.is_some())
            .finish_non_exhaustive()
    }
}

impl LiveSaveCoordinator {
    /// Start exactly one full-state watcher and its serialized app worker.
    pub fn start(
        model: Arc<RwLock<GalaxyModel>>,
        watch: WatchConfig,
        cache_path: Option<PathBuf>,
    ) -> Result<Self, CoordinatorStartError> {
        Self::start_with_writer(model, watch, cache_path, nms_cache::write_cache)
    }

    fn start_with_writer<W>(
        model: Arc<RwLock<GalaxyModel>>,
        watch: WatchConfig,
        cache_path: Option<PathBuf>,
        writer: W,
    ) -> Result<Self, CoordinatorStartError>
    where
        W: Fn(&CacheData, &Path) -> Result<(), nms_cache::CacheError> + Send + 'static,
    {
        let watcher = start_watching_updates(watch).map_err(CoordinatorStartError::Watch)?;
        let observers = Arc::new(Mutex::new(Vec::<mpsc::Sender<AppliedSaveUpdate>>::new()));
        let observers_for_worker = Arc::clone(&observers);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_for_worker = Arc::clone(&shutdown);
        let worker = thread::Builder::new()
            .name("nms-live-save-coordinator".into())
            .spawn(move || {
                run_worker(
                    model,
                    watcher,
                    cache_path,
                    observers_for_worker,
                    shutdown_for_worker,
                    writer,
                );
            })
            .map_err(CoordinatorStartError::Worker)?;
        Ok(Self {
            shutdown,
            worker: Some(worker),
            observers,
        })
    }

    /// Subscribe to future applied updates. The unbounded receiver must be
    /// drained by its owner; a slow subscriber never stalls cache writing.
    pub fn subscribe(&self) -> mpsc::Receiver<AppliedSaveUpdate> {
        let (sender, receiver) = mpsc::channel();
        lock_observers(&self.observers).push(sender);
        receiver
    }
}

impl Drop for LiveSaveCoordinator {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            eprintln!("Warning: live-save coordinator worker panicked during shutdown");
        }
    }
}

fn lock_observers(
    observers: &Mutex<Vec<mpsc::Sender<AppliedSaveUpdate>>>,
) -> std::sync::MutexGuard<'_, Vec<mpsc::Sender<AppliedSaveUpdate>>> {
    // A panic while mutating the sender registry does not corrupt the senders
    // themselves; recovering the vector preserves delivery for later updates.
    observers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn run_worker<W>(
    model: Arc<RwLock<GalaxyModel>>,
    watcher: WatchUpdateHandle,
    cache_path: Option<PathBuf>,
    observers: Arc<Mutex<Vec<mpsc::Sender<AppliedSaveUpdate>>>>,
    shutdown: Arc<AtomicBool>,
    writer: W,
) where
    W: Fn(&CacheData, &Path) -> Result<(), nms_cache::CacheError>,
{
    while !shutdown.load(Ordering::Acquire) {
        match watcher.receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(update) => {
                let (notifications, cache_data) = {
                    let mut guard = model.blocking_write();
                    let notifications =
                        crate::watch::notifications_for_delta(&guard, &update.delta);
                    guard.apply_save_update(&update);
                    let cache_data = cache_path
                        .as_ref()
                        .map(|_| nms_cache::extract_cache_data(&guard, update.save_version));
                    (notifications, cache_data)
                };

                if let (Some(path), Some(data)) = (cache_path.as_deref(), cache_data.as_ref())
                    && let Err(error) = writer(data, path)
                {
                    eprintln!("Warning: could not update cache: {error}");
                }

                publish(
                    &observers,
                    AppliedSaveUpdate {
                        update,
                        notifications,
                    },
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    drop(watcher);
}

fn publish(observers: &Mutex<Vec<mpsc::Sender<AppliedSaveUpdate>>>, event: AppliedSaveUpdate) {
    lock_observers(observers).retain(|sender| sender.send(event.clone()).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::player::PlayerState;
    use std::sync::mpsc;

    pub(super) fn save_json(units: u64) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName":"Coordinator", "TotalPlayTime":1},
            "BaseContext": {"GameMode":1, "PlayerStateData": {
                "UniverseAddress":{"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":0}},
                "Units":units,"Nanites":8,"Specials":9,"PersistentPlayerBases":[]
            }},
            "ExpeditionContext": {"GameMode":6,"PlayerStateData": {
                "UniverseAddress":{"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":0,"PlanetIndex":0}},
                "Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]
            }},
            "DiscoveryManagerData":{"DiscoveryData-v1":{"ReserveStore":0,"ReserveManaged":0,"Store":{"Record":[]}}}
        })).expect("serialize test save")
    }

    pub(super) fn wait_for_event(
        receiver: &mpsc::Receiver<AppliedSaveUpdate>,
    ) -> AppliedSaveUpdate {
        receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("coordinator publishes watched update")
    }

    #[test]
    fn test_real_watcher_applies_complete_state_and_persists_matching_version() {
        let directory = tempfile::tempdir().expect("temporary save/cache directory");
        let save = directory.path().join("save.json");
        let cache = directory.path().join("cache.rkyv");
        std::fs::write(&save, save_json(7)).expect("write initial save");
        let parsed = nms_save::parse_save(&save_json(7)).expect("parse initial save");
        let model = Arc::new(RwLock::new(GalaxyModel::from_save(&parsed)));
        let coordinator = LiveSaveCoordinator::start(
            Arc::clone(&model),
            WatchConfig {
                save_path: save.clone(),
                debounce: Duration::from_millis(25),
            },
            Some(cache.clone()),
        )
        .expect("start live coordinator");
        let first = coordinator.subscribe();
        let second = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&save, save_json(70)).expect("write complete changed save");

        let event_a = wait_for_event(&first);
        let event_b = wait_for_event(&second);
        assert_eq!(event_a.update.save_version, 4720);
        assert_eq!(
            event_a.update.player_state,
            PlayerState::new(
                GalacticAddress::new(0, 0, 0, 1, 0, 0),
                0,
                None,
                None,
                70,
                8,
                9
            )
        );
        assert_eq!(event_a.update.player_state, event_b.update.player_state);
        assert_eq!(event_a.update.save_version, event_b.update.save_version);
        assert!(event_a.update.delta.is_empty());
        assert!(event_a.notifications.is_empty());

        let state = model.blocking_read().player_state.clone();
        assert_eq!(state, Some(event_a.update.player_state.clone()));
        let persisted = nms_cache::read_cache(&cache).expect("checked-read live cache");
        assert_eq!(persisted.save_version, event_a.update.save_version);
        assert_eq!(persisted.player_state, Some(event_a.update.player_state));
        assert_eq!(persisted.active_galaxy, 0);
        assert!(persisted.systems.is_empty());
        assert!(persisted.bases.is_empty());
        drop(coordinator);
    }

    #[test]
    fn test_blocked_writer_releases_model_and_preserves_update_order() {
        let directory = tempfile::tempdir().expect("temporary save/cache directory");
        let save = directory.path().join("save.json");
        let cache = directory.path().join("cache.rkyv");
        std::fs::write(&save, save_json(7)).expect("write initial save");
        let parsed = nms_save::parse_save(&save_json(7)).expect("parse initial save");
        let model = Arc::new(RwLock::new(GalaxyModel::from_save(&parsed)));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (versions_tx, versions_rx) = mpsc::channel();
        let writer = move |data: &CacheData, path: &Path| {
            started_tx
                .send(data.player_state.as_ref().unwrap().units)
                .unwrap();
            release_rx.recv().unwrap();
            versions_tx
                .send(data.player_state.as_ref().unwrap().units)
                .unwrap();
            nms_cache::write_cache(data, path)
        };
        let coordinator = LiveSaveCoordinator::start_with_writer(
            Arc::clone(&model),
            WatchConfig {
                save_path: save.clone(),
                debounce: Duration::from_millis(25),
            },
            Some(cache.clone()),
            writer,
        )
        .expect("start coordinator with test barrier writer");
        let events = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&save, save_json(70)).expect("write first update");
        assert_eq!(started_rx.recv_timeout(Duration::from_secs(5)).unwrap(), 70);

        let (lock_tx, lock_rx) = mpsc::channel();
        let model_for_probe = Arc::clone(&model);
        std::thread::spawn(move || {
            let state = model_for_probe.blocking_read().player_state.clone();
            lock_tx.send(state).unwrap();
        });
        let lock_result = lock_rx.recv_timeout(Duration::from_secs(1));

        std::fs::write(&save, save_json(90))
            .expect("write second update while first cache write blocks");
        std::thread::sleep(Duration::from_millis(400));
        let blocked_event = events.try_recv();
        // Always release the blocked writer before assertions, so a failed
        // oracle cannot hang coordinator Drop.
        release_tx.send(()).expect("release first writer");
        assert_eq!(lock_result.unwrap().unwrap().units, 70);
        assert!(
            matches!(blocked_event, Err(mpsc::TryRecvError::Empty)),
            "second update must wait behind first serialized write"
        );
        assert_eq!(
            versions_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            70
        );
        let first = events
            .recv_timeout(Duration::from_secs(5))
            .expect("publish first update");
        assert_eq!(first.update.player_state.units, 70);
        assert_eq!(started_rx.recv_timeout(Duration::from_secs(5)).unwrap(), 90);
        release_tx.send(()).expect("release second writer");
        let second = events
            .recv_timeout(Duration::from_secs(5))
            .expect("publish second update");
        assert_eq!(second.update.player_state.units, 90);
        assert_eq!(
            versions_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            90
        );
        let persisted = nms_cache::read_cache(&cache).expect("checked-read final cache");
        assert_eq!(persisted.player_state.unwrap().units, 90);
        drop(coordinator);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    pub(super) fn path_fixture(
        units: u64,
    ) -> (tempfile::TempDir, PathBuf, Arc<RwLock<GalaxyModel>>) {
        let directory = tempfile::tempdir().expect("temporary watcher directory");
        let path = directory.path().join("save.json");
        std::fs::write(&path, super::tests::save_json(units)).expect("write test save");
        let parsed =
            nms_save::parse_save(&super::tests::save_json(units)).expect("parse test save");
        (
            directory,
            path,
            Arc::new(RwLock::new(GalaxyModel::from_save(&parsed))),
        )
    }

    #[test]
    fn test_no_cache_path_never_invokes_writer() {
        let (_directory, path, model) = path_fixture(7);
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_writer = Arc::clone(&count);
        let coordinator = LiveSaveCoordinator::start_with_writer(
            Arc::clone(&model),
            WatchConfig {
                save_path: path.clone(),
                debounce: Duration::from_millis(25),
            },
            None,
            move |_, _| {
                count_for_writer.fetch_add(1, AtomicOrdering::SeqCst);
                Ok(())
            },
        )
        .expect("start no-cache coordinator");
        let events = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&path, super::tests::save_json(77)).unwrap();
        assert_eq!(
            super::tests::wait_for_event(&events)
                .update
                .player_state
                .units,
            77
        );
        assert_eq!(count.load(AtomicOrdering::SeqCst), 0);
        drop(coordinator);
    }

    #[test]
    fn test_failed_cache_write_does_not_stop_later_update_or_observers() {
        let (_directory, path, model) = path_fixture(7);
        let cache = path.with_extension("rkyv");
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_writer = Arc::clone(&calls);
        let writer_path = cache.clone();
        let coordinator = LiveSaveCoordinator::start_with_writer(
            Arc::clone(&model),
            WatchConfig {
                save_path: path.clone(),
                debounce: Duration::from_millis(25),
            },
            Some(cache.clone()),
            move |data, target| {
                if calls_for_writer.fetch_add(1, AtomicOrdering::SeqCst) == 0 {
                    return Err(nms_cache::CacheError::Io(std::io::Error::other(
                        "injected first write failure",
                    )));
                }
                assert_eq!(target, writer_path);
                nms_cache::write_cache(data, target)
            },
        )
        .expect("start writer-recovery coordinator");
        let events = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&path, super::tests::save_json(70)).unwrap();
        assert_eq!(
            super::tests::wait_for_event(&events)
                .update
                .player_state
                .units,
            70
        );
        std::fs::write(&path, super::tests::save_json(90)).unwrap();
        assert_eq!(
            super::tests::wait_for_event(&events)
                .update
                .player_state
                .units,
            90
        );
        assert_eq!(calls.load(AtomicOrdering::SeqCst), 2);
        assert_eq!(
            nms_cache::read_cache(&cache)
                .unwrap()
                .player_state
                .unwrap()
                .units,
            90
        );
        assert_eq!(
            model.blocking_read().player_state.as_ref().unwrap().units,
            90
        );
        drop(coordinator);
    }

    #[test]
    fn test_malformed_initial_save_fails_without_model_mutation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("invalid-save.json");
        std::fs::write(&path, b"not a parsed save").unwrap();
        let model = Arc::new(RwLock::new(GalaxyModel::new()));
        let result = LiveSaveCoordinator::start(
            Arc::clone(&model),
            WatchConfig {
                save_path: path,
                debounce: Duration::from_millis(20),
            },
            None,
        );
        assert!(result.is_err());
        assert!(model.blocking_read().player_state.is_none());
        assert!(model.blocking_read().systems.is_empty());
    }

    #[test]
    fn test_idle_drop_closes_observer_and_stops_owned_watcher() {
        let (_directory, path, model) = path_fixture(7);
        let coordinator = LiveSaveCoordinator::start(
            model,
            WatchConfig {
                save_path: path,
                debounce: Duration::from_millis(20),
            },
            None,
        )
        .unwrap();
        let events = coordinator.subscribe();
        drop(coordinator);
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }
}

#[cfg(test)]
mod observer_and_drop_tests {
    use super::*;

    #[test]
    fn test_observers_receive_order_without_backpressure_or_history() {
        let (_directory, path, model) = super::additional_tests::path_fixture(7);
        let coordinator = LiveSaveCoordinator::start(
            Arc::clone(&model),
            WatchConfig {
                save_path: path.clone(),
                debounce: Duration::from_millis(25),
            },
            None,
        )
        .expect("start observer coordinator");
        let first = coordinator.subscribe();
        let second = coordinator.subscribe();
        let _undrained = coordinator.subscribe();
        let disconnected = coordinator.subscribe();
        drop(disconnected);
        std::thread::sleep(Duration::from_millis(100));

        for units in [70, 80, 90] {
            std::fs::write(&path, super::tests::save_json(units)).unwrap();
            assert_eq!(
                super::tests::wait_for_event(&first)
                    .update
                    .player_state
                    .units,
                units
            );
            assert_eq!(
                super::tests::wait_for_event(&second)
                    .update
                    .player_state
                    .units,
                units
            );
        }
        let late = coordinator.subscribe();
        assert!(
            matches!(late.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "late subscribers receive no history"
        );
        let registered = lock_observers(&coordinator.observers);
        assert_eq!(
            registered.len(),
            4,
            "only active subscribers remain registered"
        );
        drop(registered);
        drop(coordinator);
        assert!(matches!(
            first.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert!(matches!(
            second.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert!(matches!(
            late.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn test_drop_waits_for_inflight_cache_writer_and_update_publication() {
        let (_directory, path, model) = super::additional_tests::path_fixture(7);
        let cache = path.with_extension("rkyv");
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let coordinator = LiveSaveCoordinator::start_with_writer(
            Arc::clone(&model),
            WatchConfig {
                save_path: path.clone(),
                debounce: Duration::from_millis(25),
            },
            Some(cache.clone()),
            move |data, target| {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                nms_cache::write_cache(data, target)
            },
        )
        .expect("start lifecycle coordinator");
        let events = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&path, super::tests::save_json(70)).unwrap();
        started_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("first writer entered barrier");

        let (drop_tx, drop_rx) = mpsc::channel();
        std::thread::spawn(move || {
            drop(coordinator);
            drop_tx.send(()).unwrap();
        });
        let drop_before_release = drop_rx.recv_timeout(Duration::from_millis(100));
        // Release before any assertions, so this test cannot strand Drop.
        release_tx.send(()).unwrap();
        assert!(
            matches!(drop_before_release, Err(mpsc::RecvTimeoutError::Timeout)),
            "coordinator drop joins the in-flight write"
        );
        drop_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("coordinator drop completes after writer release");
        assert_eq!(
            events
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .update
                .player_state
                .units,
            70
        );
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert_eq!(
            nms_cache::read_cache(&cache)
                .unwrap()
                .player_state
                .unwrap()
                .units,
            70
        );
        assert_eq!(
            model.blocking_read().player_state.as_ref().unwrap().units,
            70
        );
    }
}
