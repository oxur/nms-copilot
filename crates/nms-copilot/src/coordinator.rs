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
                    apply_and_snapshot(&mut guard, &update, cache_path.is_some())
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

/// Apply one full parsed event and take its cache snapshot while holding the
/// model write lock. The worker uses this exact path before releasing the lock
/// and performing serialized disk I/O.
fn apply_and_snapshot(
    model: &mut GalaxyModel,
    update: &SaveUpdate,
    cache_enabled: bool,
) -> (Vec<String>, Option<CacheData>) {
    let notifications = crate::watch::notifications_for_delta(model, &update.delta);
    model.apply_save_update(update);
    let cache_data =
        cache_enabled.then(|| nms_cache::extract_cache_data(model, update.save_version));
    (notifications, cache_data)
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

    pub(super) fn save_json_full(
        version: u32,
        reality: u8,
        voxel_x: i16,
        units: u64,
        nanites: u64,
        quicksilver: u64,
        previous: Option<(u8, i16, i16, i16, u16, u8)>,
    ) -> Vec<u8> {
        let mut value: serde_json::Value =
            serde_json::from_slice(&save_json(units)).expect("parse fixture JSON");
        value["Version"] = serde_json::json!(version);
        value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["RealityIndex"] =
            serde_json::json!(reality);
        value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
            serde_json::json!(voxel_x);
        value["BaseContext"]["PlayerStateData"]["Nanites"] = serde_json::json!(nanites);
        value["BaseContext"]["PlayerStateData"]["Specials"] = serde_json::json!(quicksilver);
        let previous = previous.unwrap_or((0, 0, 0, 0, 0, 0));
        value["BaseContext"]["PlayerStateData"]["PreviousUniverseAddress"] = serde_json::json!({
            "RealityIndex": previous.0,
            "GalacticAddress": {
                "VoxelX": previous.1,
                "VoxelY": previous.2,
                "VoxelZ": previous.3,
                "SolarSystemIndex": previous.4,
                "PlanetIndex": previous.5
            }
        });
        serde_json::to_vec(&value).expect("serialize complete state fixture")
    }

    #[derive(Debug, Clone, PartialEq)]
    struct CacheSystemSnapshot {
        address: GalacticAddress,
        name: Option<String>,
        discoverer: Option<String>,
        timestamp_secs: Option<i64>,
        planets: Vec<nms_core::system::Planet>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct CacheSnapshot {
        version: u32,
        player: Option<PlayerState>,
        active_galaxy: u8,
        systems: Vec<CacheSystemSnapshot>,
        bases: Vec<nms_core::player::PlayerBase>,
    }

    fn cache_snapshot(data: &CacheData) -> CacheSnapshot {
        let mut systems: Vec<_> = data
            .systems
            .iter()
            .map(|system| CacheSystemSnapshot {
                address: system.address,
                name: system.name.clone(),
                discoverer: system.discoverer.clone(),
                timestamp_secs: system.timestamp_secs,
                planets: system.planets.clone(),
            })
            .collect();
        systems.sort_by_key(|system| {
            (
                system.address.reality_index,
                system.address.voxel_x(),
                system.address.voxel_y(),
                system.address.voxel_z(),
                system.address.solar_system_index(),
                system.address.planet_index(),
            )
        });
        let mut bases = data.bases.clone();
        bases.sort_by(|left, right| left.name.cmp(&right.name));
        CacheSnapshot {
            version: data.save_version,
            player: data.player_state.clone(),
            active_galaxy: data.active_galaxy,
            systems,
            bases,
        }
    }

    struct ReleaseBarrier(mpsc::Sender<()>);

    impl Drop for ReleaseBarrier {
        fn drop(&mut self) {
            let _ = self.0.send(());
            let _ = self.0.send(());
        }
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
        let (written_tx, written_rx) = mpsc::channel();
        let writer = move |data: &CacheData, path: &Path| {
            let snapshot = cache_snapshot(data);
            started_tx.send(snapshot.clone()).unwrap();
            release_rx
                .recv_timeout(Duration::from_secs(5))
                .map_err(|error| {
                    nms_cache::CacheError::Io(std::io::Error::other(format!(
                        "test writer barrier ended: {error}"
                    )))
                })?;
            written_tx.send(snapshot).unwrap();
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
        let release_guard = ReleaseBarrier(release_tx.clone());
        let events = coordinator.subscribe();
        std::thread::sleep(Duration::from_millis(100));
        let first_state = PlayerState::new(
            GalacticAddress::new(1, 0, 0, 1, 0, 0),
            0,
            Some(GalacticAddress::new(8, 7, 6, 2, 3, 9)),
            None,
            70,
            80,
            90,
        );
        let second_state = PlayerState::new(
            GalacticAddress::new(2, 0, 0, 1, 0, 9),
            9,
            Some(GalacticAddress::new(9, 8, 7, 2, 4, 0)),
            None,
            700,
            800,
            900,
        );
        let expected_first = CacheSnapshot {
            version: 4721,
            player: Some(first_state.clone()),
            active_galaxy: 0,
            systems: vec![],
            bases: vec![],
        };
        let expected_second = CacheSnapshot {
            version: 4722,
            player: Some(second_state.clone()),
            active_galaxy: 9,
            systems: vec![],
            bases: vec![],
        };
        std::fs::write(
            &save,
            save_json_full(4721, 0, 1, 70, 80, 90, Some((9, 8, 7, 6, 2, 3))),
        )
        .expect("write first update");
        assert_eq!(
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            expected_first
        );

        let (lock_tx, lock_rx) = mpsc::channel();
        let model_for_probe = Arc::clone(&model);
        std::thread::spawn(move || {
            let state = model_for_probe.blocking_read().player_state.clone();
            lock_tx.send(state).unwrap();
        });
        let lock_result = lock_rx.recv_timeout(Duration::from_secs(1));

        std::fs::write(
            &save,
            save_json_full(4722, 9, 2, 700, 800, 900, Some((0, 9, 8, 7, 2, 4))),
        )
        .expect("write second update while first cache write blocks");
        std::thread::sleep(Duration::from_millis(400));
        let blocked_event = events.try_recv();
        release_tx.send(()).expect("release first writer");
        assert_eq!(lock_result.unwrap(), Some(first_state.clone()));
        assert!(
            matches!(blocked_event, Err(mpsc::TryRecvError::Empty)),
            "second update must wait behind first serialized write"
        );
        assert_eq!(
            written_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            expected_first
        );
        let first = events
            .recv_timeout(Duration::from_secs(5))
            .expect("publish first update");
        assert_eq!(first.update.save_version, 4721);
        assert_eq!(first.update.player_state, first_state);
        assert_eq!(
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            expected_second
        );
        release_tx.send(()).expect("release second writer");
        let second = events
            .recv_timeout(Duration::from_secs(5))
            .expect("publish second update");
        assert_eq!(second.update.save_version, 4722);
        assert_eq!(second.update.player_state, second_state);
        assert_eq!(
            written_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            expected_second
        );
        let persisted = nms_cache::read_cache(&cache).expect("checked-read final cache");
        assert_eq!(cache_snapshot(&persisted), expected_second);
        assert_eq!(
            model.blocking_read().player_state,
            Some(second.update.player_state)
        );
        drop(release_guard);
        drop(coordinator);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::player::PlayerState;
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
        std::fs::write(
            &path,
            super::tests::save_json_full(4721, 0, 1, 70, 80, 90, Some((9, 8, 7, 6, 2, 3))),
        )
        .unwrap();
        let first = super::tests::wait_for_event(&events);
        assert_eq!(first.update.save_version, 4721);
        assert_eq!(
            first.update.player_state,
            PlayerState::new(
                GalacticAddress::new(1, 0, 0, 1, 0, 0),
                0,
                Some(GalacticAddress::new(8, 7, 6, 2, 3, 9)),
                None,
                70,
                80,
                90,
            )
        );
        std::fs::write(
            &path,
            super::tests::save_json_full(4722, 9, 2, 700, 800, 900, Some((0, 9, 8, 7, 2, 4))),
        )
        .unwrap();
        let second = super::tests::wait_for_event(&events);
        assert_eq!(second.update.save_version, 4722);
        let expected = PlayerState::new(
            GalacticAddress::new(2, 0, 0, 1, 0, 9),
            9,
            Some(GalacticAddress::new(9, 8, 7, 2, 4, 0)),
            None,
            700,
            800,
            900,
        );
        assert_eq!(second.update.player_state, expected.clone());
        assert_eq!(calls.load(AtomicOrdering::SeqCst), 2);
        let persisted = nms_cache::read_cache(&cache).unwrap();
        assert_eq!(persisted.save_version, 4722);
        assert_eq!(persisted.player_state, Some(expected.clone()));
        assert_eq!(persisted.active_galaxy, 9);
        assert_eq!(
            model.blocking_read().player_state.as_ref().unwrap().units,
            700
        );
        assert_eq!(model.blocking_read().player_state, Some(expected));
        drop(coordinator);
    }

    #[test]
    fn test_malformed_initial_save_fails_without_model_mutation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("invalid-save.json");
        std::fs::write(&path, b"not a parsed save").unwrap();
        let parsed = nms_save::parse_save(&super::tests::save_json(17)).unwrap();
        let mut initial = GalaxyModel::from_save(&parsed);
        initial.ensure_player_system();
        initial.set_active_galaxy(7);
        let initial_player = initial.player_state.clone();
        let initial_systems = initial.systems.clone();
        let initial_planets = initial.planets.clone();
        let initial_bases = initial.bases.clone();
        let initial_names = initial.name_index.clone();
        let initial_biomes = initial.biome_index.clone();
        let initial_addresses = initial.address_to_id.clone();
        let model = Arc::new(RwLock::new(initial));
        let cache = directory.path().join("existing-cache.rkyv");
        let cache_bytes = b"untouched cache sentinel";
        std::fs::write(&cache, cache_bytes).unwrap();
        let result = LiveSaveCoordinator::start(
            Arc::clone(&model),
            WatchConfig {
                save_path: path,
                debounce: Duration::from_millis(20),
            },
            Some(cache.clone()),
        );
        assert!(matches!(result, Err(CoordinatorStartError::Watch(_))));
        let after = model.blocking_read();
        assert_eq!(after.player_state, initial_player);
        assert_eq!(after.systems, initial_systems);
        assert_eq!(after.planets, initial_planets);
        assert_eq!(after.bases, initial_bases);
        assert_eq!(after.name_index, initial_names);
        assert_eq!(after.biome_index, initial_biomes);
        assert_eq!(after.address_to_id, initial_addresses);
        assert_eq!(after.active_galaxy, 7);
        assert_eq!(std::fs::read(cache).unwrap(), cache_bytes);
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
    use nms_core::address::GalacticAddress;
    use nms_core::player::PlayerState;

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
        let undrained = coordinator.subscribe();
        let disconnected = coordinator.subscribe();
        drop(disconnected);
        std::thread::sleep(Duration::from_millis(100));

        let fixtures = [
            (4721, 70, 80, 90, 0, 1),
            (4722, 700, 800, 900, 0, 2),
            (4723, 7_000, 8_000, 9_000, 9, 3),
        ];
        for (index, (version, units, nanites, quicksilver, reality, x)) in
            fixtures.into_iter().enumerate()
        {
            let previous = (0, 9, 8, 7, 2, 4);
            std::fs::write(
                &path,
                super::tests::save_json_full(
                    version,
                    reality,
                    x,
                    units,
                    nanites,
                    quicksilver,
                    Some(previous),
                ),
            )
            .unwrap();
            let expected = PlayerState::new(
                GalacticAddress::new(x, 0, 0, 1, 0, reality),
                reality,
                Some(GalacticAddress::new(9, 8, 7, 2, 4, 0)),
                None,
                units,
                nanites,
                quicksilver,
            );
            for receiver in [&first, &second] {
                let event = super::tests::wait_for_event(receiver);
                assert_eq!(event.update.save_version, version);
                assert_eq!(event.update.player_state, expected);
            }
            assert_eq!(index, version as usize - 4721);
        }
        for (version, units, nanites, quicksilver, reality, x) in fixtures {
            let event = undrained.recv_timeout(Duration::from_secs(1)).unwrap();
            assert_eq!(event.update.save_version, version);
            assert_eq!(
                event.update.player_state,
                PlayerState::new(
                    GalacticAddress::new(x, 0, 0, 1, 0, reality),
                    reality,
                    Some(GalacticAddress::new(9, 8, 7, 2, 4, 0)),
                    None,
                    units,
                    nanites,
                    quicksilver,
                )
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

#[cfg(test)]
mod full_update_component_tests {
    use super::*;
    use nms_core::delta::{SaveDelta, SaveUpdate};
    use nms_core::player::{BaseType, PlayerBase, PlayerState};
    use nms_core::system::{Planet, System};
    use nms_graph::spatial::SystemId;

    fn address(x: i16, reality: u8) -> nms_core::address::GalacticAddress {
        nms_core::address::GalacticAddress::new(x, 0, 0, 1, 0, reality)
    }

    fn atlas_system(name: &str, x: i16, reality: u8) -> System {
        System::new(
            address(x, reality),
            Some(name.into()),
            None,
            None,
            vec![Planet::new(
                0,
                None,
                None,
                false,
                Some(format!("{name} I")),
                None,
            )],
        )
    }

    fn base(name: &str, reality: u8) -> PlayerBase {
        PlayerBase::new(
            name.into(),
            BaseType::HomePlanetBase,
            address(10, reality),
            [1.0, 2.0, 3.0],
            Some("owner".into()),
        )
    }

    #[test]
    fn test_typed_full_update_adds_atlas_and_preserves_complete_state_and_selection() {
        let mut model = GalaxyModel::new();
        model.insert_system(atlas_system("Old Euclid", 1, 0));
        model.insert_system(atlas_system("Old Eissentam", 2, 9));
        model.insert_base(base("Euclid Base", 0));
        model.insert_base(base("Eissentam Base", 9));
        model.set_active_galaxy(7);
        model.player_state = Some(PlayerState::new(
            address(1, 0),
            0,
            Some(address(3, 0)),
            Some(address(4, 9)),
            10,
            20,
            30,
        ));

        let prior_systems = model.systems.clone();
        let prior_planets = model.planets.clone();
        let prior_bases = model.bases.clone();
        let prior_names = model.name_index.clone();
        let prior_addresses = model.address_to_id.clone();
        let prior_biomes = model.biome_index.clone();
        let prior_nodes = model.node_map.clone();
        let added_system = atlas_system("New Euclid", 5, 0);
        let existing_id = SystemId::from_address(&address(1, 0));
        let added_planet = Planet::new(1, None, None, true, Some("New moon".into()), Some(55));
        let added_base = base("New Base", 0);
        let full_state = PlayerState::new(
            address(1, 0),
            0,
            Some(address(6, 0)),
            Some(address(7, 9)),
            101,
            202,
            303,
        );
        let update = SaveUpdate {
            delta: SaveDelta {
                new_systems: vec![added_system.clone()],
                new_planets: vec![(existing_id, added_planet.clone())],
                new_bases: vec![added_base.clone()],
                ..SaveDelta::empty()
            },
            player_state: full_state.clone(),
            save_version: 5011,
        };
        let (notes, snapshot) = apply_and_snapshot(&mut model, &update, true);
        assert_eq!(
            notes,
            vec![
                "  New system: New Euclid (1 planet)",
                "  New scan: \"New moon\" (?) in Old Euclid",
                "  New base: New Base",
            ]
        );
        assert_eq!(
            model.active_galaxy, 7,
            "same physical galaxy retains selection"
        );
        assert_eq!(model.player_state, Some(full_state.clone()));
        let mut expected_existing_system = prior_systems[&existing_id].clone();
        expected_existing_system.planets.push(added_planet.clone());
        assert_eq!(
            model.systems.get(&existing_id),
            Some(&expected_existing_system)
        );
        assert!(
            prior_systems
                .iter()
                .filter(|(key, _)| **key != existing_id)
                .all(|(key, value)| model.systems.get(key) == Some(value))
        );
        assert!(
            prior_planets
                .iter()
                .all(|(key, value)| model.planets.get(key) == Some(value))
        );
        assert!(
            prior_bases
                .iter()
                .all(|(key, value)| model.bases.get(key) == Some(value))
        );
        assert!(
            prior_names
                .iter()
                .all(|(key, value)| model.name_index.get(key) == Some(value))
        );
        assert!(
            prior_addresses
                .iter()
                .all(|(key, value)| model.address_to_id.get(key) == Some(value))
        );
        assert!(
            prior_biomes
                .iter()
                .all(|(key, value)| model.biome_index.get(key) == Some(value))
        );
        assert!(
            prior_nodes
                .iter()
                .all(|(key, value)| model.node_map.get(key) == Some(value))
        );
        assert_eq!(model.graph.node_count(), 3);
        assert_eq!(model.node_map.len(), 3);
        assert_eq!(model.spatial[&0].size(), 2);
        assert_eq!(model.spatial[&9].size(), 1);
        assert_eq!(
            model
                .systems
                .get(&SystemId::from_address(&added_system.address)),
            Some(&added_system)
        );
        assert_eq!(model.planets.get(&(existing_id, 1)), Some(&added_planet));
        assert_eq!(model.bases.get("new base"), Some(&added_base));
        let snapshot = snapshot.expect("cache snapshot requested");
        assert_eq!(snapshot.save_version, 5011);
        assert_eq!(snapshot.player_state, Some(full_state));
        assert_eq!(snapshot.systems.len(), 3);
        assert_eq!(snapshot.bases.len(), 3);

        let cross_state = PlayerState::new(
            address(2, 9),
            9,
            Some(address(8, 0)),
            Some(address(9, 9)),
            404,
            505,
            606,
        );
        let cross = SaveUpdate {
            delta: SaveDelta::empty(),
            player_state: cross_state.clone(),
            save_version: 5012,
        };
        let (_, snapshot) = apply_and_snapshot(&mut model, &cross, true);
        assert_eq!(
            model.active_galaxy, 9,
            "cross-galaxy update selects incoming galaxy"
        );
        assert_eq!(model.player_state, Some(cross_state.clone()));
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.save_version, 5012);
        assert_eq!(snapshot.player_state, Some(cross_state));
        assert_eq!(snapshot.systems.len(), 3);
        assert_eq!(snapshot.bases.len(), 3);
    }
}
