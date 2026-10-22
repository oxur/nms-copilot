//! Checked cold names and same-parse live name observations.

pub mod support {
    pub mod live_mcp;
    pub mod name_observations;
}

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::Duration;

use nms_copilot::coordinator::{AppliedSaveUpdate, LiveSaveCoordinator};
use nms_core::delta::DiscoveryNames;
use nms_core::system::SystemId;
use nms_graph::GalaxyModel;
use nms_watch::WatchConfig;
use support::live_mcp::{assert_cache_oracle, assert_model_matches, compare_cache_model};
use support::name_observations::{
    ModelStage, SourcePhase, expected_cache, expected_counts, expected_model_with_units,
    source_bytes, warm_seed_cache,
};
use tempfile::TempDir;
use tokio::sync::RwLock;

const EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const QUIET_TIMEOUT: Duration = Duration::from_millis(900);

fn write_phase(path: &Path, phase: SourcePhase) {
    fs::write(path, source_bytes(phase)).expect("write literal save phase");
}

fn receive(receiver: &Receiver<AppliedSaveUpdate>) -> AppliedSaveUpdate {
    receiver
        .recv_timeout(EVENT_TIMEOUT)
        .expect("paired save observation arrives within bounded interval")
}

fn assert_quiet(receiver: &Receiver<AppliedSaveUpdate>) {
    assert!(matches!(
        receiver.recv_timeout(QUIET_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    ));
}

fn assert_stage(model: &RwLock<GalaxyModel>, cache_path: &Path, stage: ModelStage, units: u64) {
    let expected = expected_model_with_units(stage, units);
    let guard = model.try_read().expect("coordinator released model guard");
    assert_model_matches(&guard, &expected).expect("whole live model matches literal stage");
    let counts = expected_counts(stage);
    assert_eq!(guard.system_count(), counts.0, "system count");
    assert_eq!(guard.planet_count(), counts.1, "planet count");
    assert_eq!(
        guard
            .systems
            .values()
            .filter(|system| system.name.is_some())
            .count(),
        counts.2,
        "named system count"
    );
    assert_eq!(
        guard
            .planets
            .values()
            .filter(|planet| planet.name.is_some())
            .count(),
        counts.3,
        "named planet count"
    );
    let mut observed_edges = guard
        .graph
        .edge_indices()
        .map(|edge_index| {
            let (source_index, target_index) = guard
                .graph
                .edge_endpoints(edge_index)
                .expect("edge endpoints exist");
            let source = guard
                .graph
                .node_weight(source_index)
                .expect("edge source has a system identity");
            let target = guard
                .graph
                .node_weight(target_index)
                .expect("edge target has a system identity");
            let source_key = (source.reality_index(), source.system_address());
            let target_key = (target.reality_index(), target.system_address());
            let (first, second) = if source_key <= target_key {
                (source_key, target_key)
            } else {
                (target_key, source_key)
            };
            (
                first,
                second,
                guard
                    .graph
                    .edge_weight(edge_index)
                    .expect("edge weight exists")
                    .to_bits(),
            )
        })
        .collect::<Vec<_>>();
    observed_edges.sort_unstable();
    let mut expected_edges = vec![
        (
            (
                0,
                SystemId::from_address(&support::name_observations::a0()).system_address(),
            ),
            (
                0,
                SystemId::from_address(&support::name_observations::b0()).system_address(),
            ),
            400.0_f64.to_bits(),
        ),
        (
            (
                0,
                SystemId::from_address(&support::name_observations::a0()).system_address(),
            ),
            (
                0,
                SystemId::from_address(&support::name_observations::c0()).system_address(),
            ),
            800.0_f64.to_bits(),
        ),
        (
            (
                0,
                SystemId::from_address(&support::name_observations::b0()).system_address(),
            ),
            (
                0,
                SystemId::from_address(&support::name_observations::c0()).system_address(),
            ),
            400.0_f64.to_bits(),
        ),
    ];
    expected_edges.sort_unstable();
    assert_eq!(
        observed_edges, expected_edges,
        "literal live graph edge multiset"
    );
    drop(guard);

    let expected_data = expected_cache(stage, units);
    assert_cache_oracle(cache_path, &expected_data, &expected)
        .expect("complete raw cache and rebuilt model match literal stage");
    let raw = nms_cache::read_cache(cache_path).expect("read serialized cache after publication");
    let rebuilt = nms_cache::rebuild_model(&raw);
    compare_cache_model(&raw, &rebuilt, &expected_data, &expected)
        .expect("complete raw cache and rebuilt model match literal stage");
}

fn assert_a0_identity(
    model: &RwLock<GalaxyModel>,
    expected_index: usize,
    expected_weight: SystemId,
    expected_point: [f64; 3],
) {
    let guard = model
        .try_read()
        .expect("coordinator released A0 identity read");
    let id = SystemId::from_address(&support::name_observations::a0());
    let index = guard.node_map[&id];
    assert_eq!(
        index.index(),
        expected_index,
        "A0 graph node index survives update"
    );
    assert_eq!(guard.graph.node_weight(index), Some(&expected_weight));
    let point = guard.spatial[&0]
        .iter()
        .find(|point| point.id == id)
        .expect("A0 spatial entry survives update");
    assert_eq!(
        point.point, expected_point,
        "A0 spatial identity survives update"
    );
}

fn expect_event(
    receiver: &Receiver<AppliedSaveUpdate>,
    names_changed: bool,
    delta_empty: bool,
) -> AppliedSaveUpdate {
    let event = receive(receiver);
    assert_eq!(event.names_changed, names_changed);
    assert_eq!(event.update.delta.is_empty(), delta_empty);
    event
}

fn start_coordinator(
    temp: &TempDir,
    model: GalaxyModel,
) -> (
    Arc<RwLock<GalaxyModel>>,
    LiveSaveCoordinator,
    Receiver<AppliedSaveUpdate>,
) {
    let save_path = temp.path().join("save.json");
    let cache_path = temp.path().join("cache.rkyv");
    write_phase(&save_path, SourcePhase::Initial);
    nms_cache::write_cache(&warm_seed_cache(), &cache_path).expect("write opposed warm seed");
    let model = Arc::new(RwLock::new(model));
    let coordinator = LiveSaveCoordinator::start(
        Arc::clone(&model),
        WatchConfig {
            save_path,
            debounce: Duration::from_millis(100),
        },
        Some(cache_path),
    )
    .expect("start single coordinator and paired watcher");
    let receiver = coordinator.subscribe();
    assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));
    (model, coordinator, receiver)
}

#[test]
fn test_checked_name_observations_apply_order_clear_ignore_and_recover() {
    let temp = TempDir::new().expect("create isolated observation fixture");
    let cache_path = temp.path().join("cache.rkyv");
    let model = nms_cache::rebuild_model(&warm_seed_cache());
    let (model, coordinator, receiver) = start_coordinator(&temp, model);
    assert_stage(&model, &cache_path, ModelStage::Initial, 7);
    let (a0_index, a0_weight, a0_point) = {
        let guard = model.try_read().expect("read initial A0 identity");
        let id = SystemId::from_address(&support::name_observations::a0());
        let index = guard.node_map[&id];
        let point = guard.spatial[&0]
            .iter()
            .find(|point| point.id == id)
            .expect("initial A0 spatial entry")
            .point;
        (
            index.index(),
            *guard.graph.node_weight(index).unwrap(),
            point,
        )
    };

    let save_path = temp.path().join("save.json");
    write_phase(&save_path, SourcePhase::Overwrite);
    let overwrite = expect_event(&receiver, true, true);
    assert!(overwrite.notifications.is_empty());
    assert_stage(&model, &cache_path, ModelStage::Overwrite, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    write_phase(&save_path, SourcePhase::Promote);
    let promote = expect_event(&receiver, false, false);
    assert!(!promote.update.delta.new_systems.is_empty());
    assert!(!promote.update.delta.new_planets.is_empty());
    assert_stage(&model, &cache_path, ModelStage::Promote, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    write_phase(&save_path, SourcePhase::Rename);
    let rename = expect_event(&receiver, true, true);
    assert!(rename.notifications.is_empty());
    assert_stage(&model, &cache_path, ModelStage::Rename, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    write_phase(&save_path, SourcePhase::Clear);
    let clear = expect_event(&receiver, true, true);
    assert!(clear.notifications.is_empty());
    assert_stage(&model, &cache_path, ModelStage::Clear, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    write_phase(&save_path, SourcePhase::Omit);
    let _omitted = expect_event(&receiver, false, true);
    assert_stage(&model, &cache_path, ModelStage::Ignore, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    for phase in [
        SourcePhase::Null,
        SourcePhase::NonString,
        SourcePhase::NonObject,
        SourcePhase::NonObjectDm,
    ] {
        if phase == SourcePhase::NonObjectDm {
            let raw: serde_json::Value =
                serde_json::from_slice(&source_bytes(phase)).expect("literal non-object DM save");
            let records = raw
                .pointer("/DiscoveryManagerData/DiscoveryData-v1/Store/Record")
                .and_then(serde_json::Value::as_array)
                .expect("literal source records");
            let system = records
                .iter()
                .find(|row| {
                    row.pointer("/DD/DT").and_then(serde_json::Value::as_str) == Some("SolarSystem")
                        && row.pointer("/DD/UA").and_then(serde_json::Value::as_str)
                            == Some("0x00070000000064")
                })
                .expect("literal A0 system row");
            let planet = records
                .iter()
                .find(|row| {
                    row.pointer("/DD/DT").and_then(serde_json::Value::as_str) == Some("Planet")
                        && row.pointer("/DD/UA").and_then(serde_json::Value::as_str)
                            == Some("0x10070000000064")
                })
                .expect("literal A0 planet row");
            assert_eq!(
                system["DM"],
                serde_json::json!(["discovery metadata is not an object"])
            );
            assert_eq!(planet["DM"], serde_json::json!(17));
        }
        write_phase(&save_path, phase);
        assert_quiet(&receiver);
        assert_stage(&model, &cache_path, ModelStage::Ignore, 7);
    }

    let last_good_cache = fs::read(&cache_path).expect("read last-good cache");
    let last_good_cache_mtime = fs::metadata(&cache_path)
        .expect("metadata for last-good cache")
        .modified()
        .expect("last-good cache mtime");
    for attempt in 1..=5 {
        fs::write(
            &save_path,
            support::name_observations::invalid_address_attempt(attempt),
        )
        .expect("write distinct invalid checked address");
        assert_quiet(&receiver);
        assert_eq!(
            fs::read(&cache_path).expect("cache remains readable"),
            last_good_cache
        );
        assert_eq!(
            fs::metadata(&cache_path)
                .expect("cache metadata remains available")
                .modified()
                .expect("cache mtime remains available"),
            last_good_cache_mtime,
            "rejected candidate must not refresh cache stamp"
        );
        assert_stage(&model, &cache_path, ModelStage::Ignore, 7);
    }

    write_phase(&save_path, SourcePhase::Recover);
    let recovery = expect_event(&receiver, true, true);
    assert!(recovery.notifications.is_empty());
    assert_stage(&model, &cache_path, ModelStage::Recover, 7);
    assert_a0_identity(&model, a0_index, a0_weight, a0_point);

    write_phase(&save_path, SourcePhase::Recover);
    assert_quiet(&receiver);
    assert_stage(&model, &cache_path, ModelStage::Recover, 7);

    drop(receiver);
    drop(coordinator);
}

#[test]
fn test_counter_only_update_applies_complete_explicit_name_map() {
    let temp = TempDir::new().expect("create counter-only fixture");
    let save_path = temp.path().join("save.json");
    let cache_path = temp.path().join("cache.rkyv");
    write_phase(&save_path, SourcePhase::Overwrite);
    nms_cache::write_cache(&warm_seed_cache(), &cache_path).expect("write opposed warm cache");
    let model = Arc::new(RwLock::new(nms_cache::rebuild_model(&warm_seed_cache())));
    let coordinator = LiveSaveCoordinator::start(
        Arc::clone(&model),
        WatchConfig {
            save_path: save_path.clone(),
            debounce: Duration::from_millis(100),
        },
        Some(cache_path.clone()),
    )
    .expect("start counter-only coordinator");
    let receiver = coordinator.subscribe();

    let before_names = nms_graph::extract::extract_discovery_names_from_validated(
        &nms_save::parse_validated_save_file(&save_path).expect("parse initial Overwrite save"),
    );
    let b0 = SystemId::from_address(&support::name_observations::b0());
    let b0_p2 = (b0, 2);
    let literal_names = DiscoveryNames {
        system_names: std::collections::HashMap::from([(b0, "Kepler, \"α\"\nNorth".to_owned())]),
        planet_names: std::collections::HashMap::from([(b0_p2, " Lush, \"β\"\nMoon ".to_owned())]),
    };
    assert_eq!(before_names, literal_names);

    fs::write(&save_path, source_bytes(SourcePhase::CounterOnly)).expect("write currency update");
    let event = expect_event(&receiver, true, true);
    assert!(event.notifications.is_empty());
    let after_names = nms_graph::extract::extract_discovery_names_from_validated(
        &nms_save::parse_validated_save_file(&save_path).expect("parse CounterOnly save"),
    );
    assert_eq!(after_names, literal_names);
    assert_eq!(before_names, after_names);
    assert_eq!(event.update.save_version, 4720);
    assert_eq!(
        event.update.player_state,
        support::name_observations::player_state(70)
    );
    assert!(event.update.delta.is_empty());
    assert!(event.names_changed);
    assert_stage(&model, &cache_path, ModelStage::Overwrite, 70);

    drop(receiver);
    drop(coordinator);
}

#[cfg(feature = "http")]
#[test]
fn test_coordinator_subscription_invalidates_location_and_stats_for_name_only_update() {
    let temp = TempDir::new().expect("create MCP observer fixture");
    let save_path = temp.path().join("save.json");
    let cache_path = temp.path().join("cache.rkyv");
    write_phase(&save_path, SourcePhase::Initial);
    nms_cache::write_cache(&warm_seed_cache(), &cache_path).expect("write imported cache seed");
    let model = Arc::new(RwLock::new(nms_cache::rebuild_model(&warm_seed_cache())));
    let coordinator = LiveSaveCoordinator::start(
        Arc::clone(&model),
        WatchConfig {
            save_path: save_path.clone(),
            debounce: Duration::from_millis(100),
        },
        Some(cache_path.clone()),
    )
    .expect("start paired save coordinator");
    let updates = coordinator.subscribe();
    let witness = coordinator.subscribe();
    let owner = nms_copilot::mcp::spawn_mcp_background_with_updates(
        Arc::clone(&model),
        "127.0.0.1:0".parse().unwrap(),
        Some(updates),
    )
    .expect("start MCP with the actual coordinator subscription");
    let address = owner.local_addr();
    let (session, _) = support::live_mcp::initialize_http(address).unwrap();
    let mut sse = support::live_mcp::subscribe_sse_uris(
        address,
        &session,
        &["nms://player/location", "nms://galaxy/stats"],
    )
    .unwrap();
    let cursor = sse.cursor();

    write_phase(&save_path, SourcePhase::Overwrite);
    let event = receive(&witness);
    assert_eq!(event.update.save_version, 4720);
    assert_eq!(
        event.update.player_state,
        support::name_observations::player_state(7)
    );
    assert!(event.update.delta.is_empty());
    assert!(event.names_changed);
    assert!(event.notifications.is_empty());
    let location_end = sse
        .wait_for_resource_updated_since(cursor, "nms://player/location", EVENT_TIMEOUT)
        .unwrap();
    let stats_end = sse
        .wait_for_resource_updated_since(location_end, "nms://galaxy/stats", EVENT_TIMEOUT)
        .unwrap();
    sse.wait_quiet_uris_since(
        stats_end,
        &["nms://player/location", "nms://galaxy/stats"],
        Duration::from_millis(650),
    )
    .unwrap();

    let (tool, resource) = support::live_mcp::location_rpc_pair(address, &session).unwrap();
    let expected_location =
        support::name_observations::expected_location(ModelStage::Overwrite, false);
    assert_eq!(tool, expected_location);
    assert_eq!(resource, expected_location);
    let stats = support::live_mcp::resource_json(address, &session, "nms://galaxy/stats").unwrap();
    assert_eq!(
        support::name_observations::normalize_stats_biomes(stats),
        support::name_observations::normalize_stats_biomes(
            support::name_observations::expected_stats(ModelStage::Overwrite, false)
        )
    );
    assert_stage(&model, &cache_path, ModelStage::Overwrite, 7);

    sse.close_and_join().unwrap();
    owner.shutdown().expect("stop and join MCP server");
    drop(coordinator);
}

#[test]
fn test_checked_cold_extraction_preserves_literal_names_and_empty_clears() {
    let parsed = nms_save::parse_validated_save(&source_bytes(SourcePhase::Recover))
        .expect("parse fixed literal save");
    let model = GalaxyModel::from_validated_save(&parsed);
    let names = nms_graph::extract::extract_discovery_names_from_validated(&parsed);
    assert_eq!(names.system_names.len(), 2);
    assert_eq!(names.planet_names.len(), 2);
    assert_eq!(model.system_count(), 3);
    assert_eq!(model.planet_count(), 3);
    assert_eq!(
        model.player_state,
        Some(support::name_observations::player_state(7))
    );
    assert_eq!(model.active_galaxy, 0);
    let a0 = model
        .system(&SystemId::from_address(&support::name_observations::a0()))
        .expect("literal A0 discovery exists in cold source");
    assert_eq!(a0.name.as_deref(), Some("Current Recovered"));
    assert_eq!(a0.discoverer.as_deref(), Some("Explorer"));
    assert_eq!(
        a0.timestamp.map(|timestamp| timestamp.timestamp()),
        Some(1_700_000_000)
    );
    assert_eq!(a0.planets.len(), 1);
    assert_eq!(a0.planets[0].name.as_deref(), Some("Recovered Moon"));
    assert_eq!(a0.planets[0].biome, Some(nms_core::biome::Biome::Toxic));
    assert!(a0.planets[0].infested);
    assert_eq!(a0.planets[0].seed_hash, Some(0xAB));
    let b0 = model
        .system(&SystemId::from_address(&support::name_observations::b0()))
        .expect("literal B0 discovery exists in cold source");
    assert_eq!(b0.name.as_deref(), Some("Kepler, \"α\"\nNorth"));
    assert_eq!(b0.planets[0].name.as_deref(), Some(" Lush, \"β\"\nMoon "));
    assert!(
        model
            .system(&SystemId::from_address(&support::name_observations::c0()))
            .is_none()
    );

    let clear = nms_save::parse_validated_save(&source_bytes(SourcePhase::Clear))
        .expect("parse explicit empty-name source");
    let clear_model = GalaxyModel::from_validated_save(&clear);
    assert_eq!(clear_model.system_count(), 3);
    assert_eq!(clear_model.planet_count(), 3);
    let cleared_system = clear_model
        .system(&nms_core::system::SystemId::from_address(
            &support::name_observations::a0(),
        ))
        .expect("literal A0 remains a real system");
    assert_eq!(cleared_system.name, None);
    assert_eq!(cleared_system.planets[0].name, None);
}
