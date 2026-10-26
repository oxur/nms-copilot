#![cfg(feature = "http")]

#[path = "support/live_mcp.rs"]
pub mod live_mcp;

use std::fs::File;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use nms_copilot::coordinator::{AppliedSaveUpdate, LiveSaveCoordinator};
use nms_copilot::mcp::{self, McpBackgroundError};
use nms_core::SaveUpdate;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{DiscoverySource, Planet, System, SystemId};
use nms_watch::WatchConfig;
use tokio::sync::RwLock;

const CHILD_MARKER: &str = "NMS_SLICE19_OWNED_CHILD";
const TEST_TIMEOUT: Duration = Duration::from_secs(8);

#[test]
fn test_literal_cache_validator_accepts_golden_and_rejects_discriminating_mutations() {
    use nms_cache::data::CachedSystem;
    let expected = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7);
    let expected_model = literal_expected_model();
    let rebuilt = nms_cache::rebuild_model(&expected);
    live_mcp::compare_cache_model(&expected, &rebuilt, &expected, &expected_model)
        .expect("literal whole-cache and rebuilt model golden passes");
    let mut reversed = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7);
    reversed.systems.reverse();
    let rebuilt_reversed = nms_cache::rebuild_model(&reversed);
    live_mcp::compare_cache_model(&reversed, &rebuilt_reversed, &expected, &expected_model)
        .expect("independent construction order preserves semantic graph identity");

    let mut wrong_currency = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7);
    wrong_currency.player_state.as_mut().unwrap().units = 70;
    let reject_raw = |actual: &nms_cache::CacheData| {
        live_mcp::compare_cache_model(actual, &expected_model, &expected, &expected_model).is_err()
    };
    assert!(reject_raw(&wrong_currency));
    let mut missing_planet = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7);
    missing_planet.systems[0].planets.clear();
    assert!(reject_raw(&missing_planet));
    let wrong_selection = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 9);
    assert!(reject_raw(&wrong_selection));
    let wrong_version = live_mcp::expected_cache(4721, 0, 7, 8, 9, None, 7);
    assert!(reject_raw(&wrong_version));
    let mut extra_named = live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7);
    extra_named.systems.push(CachedSystem {
        source: DiscoverySource::Personal,
        address: GalacticAddress::new(4, 4, 4, 4, 0, 9),
        name: Some("Unexpected".into()),
        discoverer: None,
        timestamp_secs: None,
        planets: vec![],
        is_player_placeholder: false,
    });
    assert!(reject_raw(&extra_named));

    let marker_data = live_mcp::expected_cold_cross_cache();
    let marker_model = live_mcp::expected_cold_cross_model();
    let mut wrong_marker_flag = live_mcp::expected_cold_cross_cache();
    wrong_marker_flag.systems[0].is_player_placeholder = false;
    assert!(
        live_mcp::compare_cache_model(
            &wrong_marker_flag,
            &marker_model,
            &marker_data,
            &marker_model
        )
        .is_err()
    );
    let genuine_empty = live_mcp::expected_genuine_empty_cross_model();
    assert!(
        live_mcp::compare_cache_model(&marker_data, &genuine_empty, &marker_data, &marker_model)
            .is_err()
    );

    let physical_nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let mut marker_cleared_raw = live_mcp::expected_cold_cross_cache();
    marker_cleared_raw
        .systems
        .iter_mut()
        .find(|system| system.address == physical_nine)
        .expect("literal current system exists")
        .is_player_placeholder = false;
    assert!(
        live_mcp::compare_cache_model(
            &marker_cleared_raw,
            &nms_cache::rebuild_model(&marker_cleared_raw),
            &marker_data,
            &marker_model,
        )
        .is_err()
    );

    let mut marker_cleared_derived = nms_cache::rebuild_model(&marker_data);
    marker_cleared_derived.insert_system(System::new(physical_nine, None, None, None, vec![]));
    assert!(
        live_mcp::compare_cache_model(
            &marker_data,
            &marker_cleared_derived,
            &marker_data,
            &marker_model,
        )
        .is_err()
    );

    let mut missing_current = live_mcp::expected_cold_cross_cache();
    missing_current
        .systems
        .retain(|system| system.address != physical_nine);
    assert!(
        live_mcp::compare_cache_model(
            &missing_current,
            &nms_cache::rebuild_model(&missing_current),
            &marker_data,
            &marker_model,
        )
        .is_err()
    );

    let location = serde_json::json!({
        "system": "(unknown)", "planets_in_system": 0, "galaxy": "Eissentam",
        "voxel_x": 0, "voxel_y": 0, "voxel_z": 0, "solar_system_index": 1,
        "portal_glyphs_hex": "000100000000",
        "portal_glyphs_emoji": nms_query::display::hex_to_emoji("000100000000"),
        "signal_booster": "07FF:007F:07FF:0001",
    });
    live_mcp::assert_location_at(
        &location,
        physical_nine,
        "(unknown)",
        0,
        "000100000000",
        "07FF:007F:07FF:0001",
    )
    .expect("literal exact physical unknown passes the complete location predicate");
    let mut decoy_location = location;
    decoy_location["system"] = serde_json::json!("Imported-9");
    assert!(
        live_mcp::assert_location_at(
            &decoy_location,
            physical_nine,
            "(unknown)",
            0,
            "000100000000",
            "07FF:007F:07FF:0001",
        )
        .is_err()
    );

    let lifecycle_stage = live_mcp::UnknownLifecycleStage::Startup;
    let lifecycle_data = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    let lifecycle_live = live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, false);
    let lifecycle_rebuilt = live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, true);
    let lifecycle_dir = tempfile::tempdir().expect("checked lifecycle codec fixtures");
    for (name, stage) in [
        ("startup", live_mcp::UnknownLifecycleStage::Startup),
        ("promoted", live_mcp::UnknownLifecycleStage::Promoted),
    ] {
        let expected_data = live_mcp::expected_unknown_lifecycle_cache(stage);
        let expected_rebuilt = live_mcp::expected_unknown_lifecycle_model(stage, true);
        let path = lifecycle_dir.path().join(format!("{name}.rkyv"));
        nms_cache::write_cache(&expected_data, &path).expect("write literal lifecycle cache");
        live_mcp::assert_cache_oracle(&path, &expected_data, &expected_rebuilt)
            .expect("checked codec read and rebuild match literal lifecycle graph");
        let mut reversed = live_mcp::expected_unknown_lifecycle_cache(stage);
        reversed.systems.reverse();
        let reversed_path = lifecycle_dir.path().join(format!("{name}-reversed.rkyv"));
        nms_cache::write_cache(&reversed, &reversed_path)
            .expect("write reversed literal lifecycle cache");
        live_mcp::assert_cache_oracle(&reversed_path, &expected_data, &expected_rebuilt)
            .expect("reversed system serialization preserves full lifecycle semantics");
    }
    live_mcp::compare_cache_model(
        &lifecycle_data,
        &lifecycle_rebuilt,
        &lifecycle_data,
        &lifecycle_rebuilt,
    )
    .expect("literal four-system cache/rebuilt graph baseline passes");
    let mut wrong_edge_endpoints =
        live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, true);
    let removed = wrong_edge_endpoints
        .graph
        .edge_indices()
        .next()
        .expect("literal same-galaxy edge exists");
    wrong_edge_endpoints.graph.remove_edge(removed);
    let d1 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let i9 = GalacticAddress::new(0, 0, 0, 3, 0, 9);
    let d1_id = nms_graph::SystemId::from_address(&d1);
    let i9_id = nms_graph::SystemId::from_address(&i9);
    wrong_edge_endpoints.graph.add_edge(
        wrong_edge_endpoints.node_map[&d1_id],
        wrong_edge_endpoints.node_map[&i9_id],
        264.682_872_8,
    );
    assert!(
        live_mcp::compare_cache_model(
            &lifecycle_data,
            &wrong_edge_endpoints,
            &lifecycle_data,
            &lifecycle_rebuilt,
        )
        .is_err()
    );
    let u2 = GalacticAddress::new(0, 0, 0, 2, 0, 0);

    let mut raw_marker_false = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    raw_marker_false
        .systems
        .iter_mut()
        .find(|system| system.address == u2)
        .expect("U2 raw row exists")
        .is_player_placeholder = false;
    let raw_marker_path = lifecycle_dir.path().join("raw-marker-false.rkyv");
    nms_cache::write_cache(&raw_marker_false, &raw_marker_path)
        .expect("ordinary U2 codec state remains valid when marker is false");
    let checked_raw_marker = nms_cache::read_cache(&raw_marker_path).unwrap();
    let raw_marker_rebuilt = nms_cache::rebuild_model(&checked_raw_marker);
    assert!(
        !checked_raw_marker
            .systems
            .iter()
            .find(|system| system.address == u2)
            .unwrap()
            .is_player_placeholder
    );
    assert!(
        live_mcp::compare_cache_model(
            &checked_raw_marker,
            &raw_marker_rebuilt,
            &lifecycle_data,
            &lifecycle_rebuilt,
        )
        .is_err()
    );

    let mut derived_marker_cleared =
        live_mcp::expected_unknown_lifecycle_model(lifecycle_stage, false);
    derived_marker_cleared.insert_system(System::new(u2, None, None, None, vec![]));
    assert!(
        live_mcp::compare_cache_model(
            &lifecycle_data,
            &derived_marker_cleared,
            &lifecycle_data,
            &lifecycle_live,
        )
        .is_err()
    );

    let mut missing_u2 = live_mcp::expected_unknown_lifecycle_cache(lifecycle_stage);
    missing_u2.systems.retain(|system| system.address != u2);
    let missing_u2_rebuilt = nms_cache::rebuild_model(&missing_u2);
    assert!(
        live_mcp::compare_cache_model(
            &missing_u2,
            &missing_u2_rebuilt,
            &lifecycle_data,
            &lifecycle_rebuilt,
        )
        .is_err()
    );

    let mut wrong_name_index = live_mcp::golden_model();
    wrong_name_index.name_index.clear();
    assert!(
        live_mcp::compare_cache_model(&expected, &wrong_name_index, &expected, &expected_model)
            .is_err()
    );
    let mut wrong_biome_index = live_mcp::golden_model();
    wrong_biome_index.biome_index.clear();
    assert!(
        live_mcp::compare_cache_model(&expected, &wrong_biome_index, &expected, &expected_model)
            .is_err()
    );
    let mut wrong_graph_weight = live_mcp::golden_model();
    let node = *wrong_graph_weight.node_map.values().next().unwrap();
    *wrong_graph_weight.graph.node_weight_mut(node).unwrap() =
        nms_graph::SystemId::from_address(&GalacticAddress::new(4, 4, 4, 4, 0, 9));
    assert!(
        live_mcp::compare_cache_model(&expected, &wrong_graph_weight, &expected, &expected_model)
            .is_err()
    );
    let mut wrong_spatial = live_mcp::golden_model();
    wrong_spatial.spatial.remove(&9);
    assert!(
        live_mcp::compare_cache_model(&expected, &wrong_spatial, &expected, &expected_model)
            .is_err()
    );
    let mut unexpected_empty_spatial = live_mcp::golden_model();
    unexpected_empty_spatial
        .spatial
        .insert(7, Default::default());
    assert!(
        live_mcp::compare_cache_model(
            &expected,
            &unexpected_empty_spatial,
            &expected,
            &expected_model
        )
        .is_err()
    );
}

#[test]
fn test_literal_cache_model_oracles_cover_empty_and_cold_cross_states() {
    let cases = [
        (
            live_mcp::expected_cache(4720, 0, 7, 8, 9, None, 7),
            live_mcp::expected_model(0, 7, 8, 9, None, 7),
        ),
        (
            live_mcp::expected_empty_cache(4720, 0, 7, 8, 9, 0),
            live_mcp::expected_empty_model(0, 7, 8, 9, 0),
        ),
        (
            live_mcp::expected_cold_cross_cache(),
            live_mcp::expected_cold_cross_model(),
        ),
        (
            live_mcp::expected_genuine_empty_cross_cache(),
            live_mcp::expected_genuine_empty_cross_model(),
        ),
    ];
    let directory = tempfile::tempdir().expect("temporary cache directory");
    for (index, (expected_data, expected_model)) in cases.iter().enumerate() {
        let path = directory.path().join(format!("literal-{index}.rkyv"));
        nms_cache::write_cache(expected_data, &path).expect("write literal cache fixture");
        live_mcp::assert_cache_oracle(&path, expected_data, expected_model)
            .expect("combined oracle accepts each independently authored fixture state");
    }
}

#[test]
fn test_owned_live_loader_coordinator_and_mcp_preserve_placeholder_identity() {
    use live_mcp::UnknownLifecycleStage as Stage;

    if std::env::var_os(CHILD_MARKER).is_none() {
        run_isolated(
            "test_owned_live_loader_coordinator_and_mcp_preserve_placeholder_identity",
            "unknown-live",
        );
        return;
    }
    let directory = tempfile::tempdir().expect("live lifecycle fixture directory");
    let save = directory.path().join("save.json");
    let cache = directory.path().join("cache.rkyv");
    let initial = live_mcp::unknown_lifecycle_source(
        4720,
        2,
        7,
        8,
        9,
        Some(GalacticAddress::new(2, 3, 4, 1, 1, 9)),
        false,
    );
    std::fs::write(&save, &initial).expect("write source fixture");
    let source_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    File::options()
        .write(true)
        .open(&save)
        .expect("open source timestamp")
        .set_times(std::fs::FileTimes::new().set_modified(source_time))
        .expect("set source timestamp");
    nms_cache::write_cache(&live_mcp::unknown_lifecycle_seed_cache(), &cache)
        .expect("write fresh atlas cache without current U2");
    File::options()
        .write(true)
        .open(&cache)
        .expect("open cache timestamp")
        .set_times(std::fs::FileTimes::new().set_modified(source_time + Duration::from_secs(100)))
        .expect("make seed cache fresh");

    let loaded = nms_cache::load_or_rebuild(&cache, &save, false)
        .expect("common loader repairs the missing warm U2 key");
    assert!(loaded.was_cached);
    let model = std::sync::Arc::new(RwLock::new(loaded.model));
    let coordinator = LiveSaveCoordinator::start(
        std::sync::Arc::clone(&model),
        WatchConfig {
            save_path: save.clone(),
            debounce: Duration::from_millis(137),
        },
        Some(cache.clone()),
    )
    .expect("start live file coordinator");
    let mcp_observer = coordinator.subscribe();
    let witness = coordinator.subscribe();
    let owner = mcp::spawn_mcp_background_with_updates(
        std::sync::Arc::clone(&model),
        "127.0.0.1:0".parse().unwrap(),
        Some(mcp_observer),
    )
    .expect("start observer MCP on the coordinator-owned model");
    let address = owner.local_addr();
    let (session, tools) = live_mcp::initialize_http(address).unwrap();
    assert!(tools.text().contains("where_am_i"));
    let mut stream = live_mcp::subscribe_sse(address, &session).unwrap();

    let d1 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    let u2 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 2, 0, 0));
    let u3 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 3, 0, 0));
    let e4 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 4, 0, 0));
    let i9 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 3, 0, 9));
    let initial_nodes = {
        let guard = model.blocking_read();
        live_mcp::assert_model_matches(
            &guard,
            &live_mcp::expected_unknown_lifecycle_model(Stage::Startup, false),
        )
        .unwrap();
        [d1, u2, e4, i9]
            .into_iter()
            .map(|id| (id, guard.node_map[&id]))
            .collect::<std::collections::HashMap<_, _>>()
    };
    live_mcp::assert_unknown_lifecycle_cache(&cache, Stage::Startup).unwrap();
    let (tool, resource) = live_mcp::location_rpc_pair(address, &session).unwrap();
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 2, 5, 0),
        "(unknown)",
        0,
        "500200000000",
        "07FF:007F:07FF:0002",
    )
    .unwrap();

    let moved = live_mcp::unknown_lifecycle_source(
        4721,
        3,
        70,
        80,
        90,
        Some(GalacticAddress::new(0, 0, 0, 2, 5, 0)),
        false,
    );
    let move_cursor = stream.cursor();
    atomic_replace(&save, &moved);
    let move_event = witness
        .recv_timeout(TEST_TIMEOUT)
        .expect("coordinator observer receives movement");
    assert_eq!(move_event.update.save_version, 4721);
    stream
        .wait_for_resource_updated_since(move_cursor, "nms://player/location", TEST_TIMEOUT)
        .unwrap();
    {
        let guard = model.blocking_read();
        live_mcp::assert_model_matches(
            &guard,
            &live_mcp::expected_unknown_lifecycle_model(Stage::Moved, false),
        )
        .unwrap();
        for (id, node) in &initial_nodes {
            assert_eq!(guard.node_map.get(id), Some(node));
        }
        assert_eq!(guard.graph.edge_count(), 6);
        for (left, right) in [(d1, u2), (d1, e4), (u2, e4), (d1, u3), (u2, u3), (u3, e4)] {
            let edge = guard
                .graph
                .find_edge(guard.node_map[&left], guard.node_map[&right])
                .expect("literal same-galaxy lifecycle pair exists");
            assert_eq!(*guard.graph.edge_weight(edge).unwrap(), 264.682_872_8);
        }
    }
    live_mcp::assert_unknown_lifecycle_cache(&cache, Stage::Moved).unwrap();
    let (tool, resource) = live_mcp::location_rpc_pair(address, &session).unwrap();
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 3, 5, 0),
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    )
    .unwrap();

    let moved_cache_bytes = std::fs::read(&cache).expect("read last-good cache before bad address");
    let moved_cache_time = std::fs::metadata(&cache).unwrap().modified().unwrap();
    let invalid_address = live_mcp::unknown_lifecycle_invalid_address_source();
    let source_before_invalid = std::fs::read(&save).expect("read actual last-good source");
    let source_time_before_invalid = std::fs::metadata(&save).unwrap().modified().unwrap();
    assert_eq!(source_before_invalid, moved);
    let invalid_cursor = stream.cursor();
    atomic_replace(&save, &invalid_address);
    let invalid_source_time = source_time_before_invalid + Duration::from_secs(100);
    assert!(invalid_source_time > source_time_before_invalid);
    File::options()
        .write(true)
        .open(&save)
        .expect("open invalid source timestamp")
        .set_times(std::fs::FileTimes::new().set_modified(invalid_source_time))
        .expect("advance invalid source timestamp deterministically");
    assert_eq!(std::fs::read(&save).unwrap(), invalid_address);
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        invalid_source_time
    );
    stream
        .wait_quiet_since(invalid_cursor, Duration::from_millis(650))
        .unwrap();
    assert!(matches!(
        witness.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    {
        let guard = model.blocking_read();
        live_mcp::assert_model_matches(
            &guard,
            &live_mcp::expected_unknown_lifecycle_model(Stage::Moved, false),
        )
        .unwrap();
        for (id, node) in &initial_nodes {
            assert_eq!(guard.node_map.get(id), Some(node));
        }
    }
    assert_eq!(std::fs::read(&cache).unwrap(), moved_cache_bytes);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        moved_cache_time
    );
    let (tool, resource) = live_mcp::location_rpc_pair(address, &session).unwrap();
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 3, 5, 0),
        "(unknown)",
        0,
        "500300000000",
        "07FF:007F:07FF:0003",
    )
    .unwrap();
    assert_eq!(std::fs::read(&save).unwrap(), invalid_address);
    assert_eq!(
        std::fs::metadata(&save).unwrap().modified().unwrap(),
        invalid_source_time
    );

    let promoted = live_mcp::unknown_lifecycle_source(
        4722,
        3,
        700,
        800,
        900,
        Some(GalacticAddress::new(0, 0, 0, 2, 5, 0)),
        true,
    );
    let promotion_cursor = stream.cursor();
    atomic_replace(&save, &promoted);
    let promotion_event = witness
        .recv_timeout(TEST_TIMEOUT)
        .expect("coordinator observer receives genuine discovery");
    assert_eq!(promotion_event.update.save_version, 4722);
    stream
        .wait_for_resource_updated_since(promotion_cursor, "nms://player/location", TEST_TIMEOUT)
        .unwrap();
    {
        let guard = model.blocking_read();
        live_mcp::assert_model_matches(
            &guard,
            &live_mcp::expected_unknown_lifecycle_model(Stage::Promoted, false),
        )
        .unwrap();
        for (id, node) in &initial_nodes {
            assert_eq!(guard.node_map.get(id), Some(node));
        }
        assert_eq!(guard.graph.edge_count(), 6);
    }
    live_mcp::assert_unknown_lifecycle_cache(&cache, Stage::Promoted).unwrap();
    let (tool, resource) = live_mcp::location_rpc_pair(address, &session).unwrap();
    live_mcp::assert_location_pair_at(
        &tool,
        &resource,
        GalacticAddress::new(0, 0, 0, 3, 5, 0),
        "-",
        1,
        "500300000000",
        "07FF:007F:07FF:0003",
    )
    .unwrap();

    owner.shutdown().expect("stop and join observer MCP");
    stream.wait_closed(TEST_TIMEOUT).unwrap();
    stream.close_and_join().unwrap();
    drop(coordinator);
}

#[test]
fn test_owned_mcp_http_lifecycle_isolation() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        run_owned_server_child();
    } else {
        run_isolated("test_owned_mcp_http_lifecycle_isolation", "lifecycle");
    }
}

#[test]
fn test_owned_mcp_bind_error_is_typed_and_address_recovers() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        run_bind_error_child();
    } else {
        run_isolated(
            "test_owned_mcp_bind_error_is_typed_and_address_recovers",
            "bind",
        );
    }
}

fn run_isolated(test_name: &str, marker: &str) {
    let stdout_file = tempfile::NamedTempFile::new().expect("create child stdout capture");
    let stderr_file = tempfile::NamedTempFile::new().expect("create child stderr capture");
    let stdout = stdout_file.reopen().expect("clone child stdout capture");
    let stderr = stderr_file.reopen().expect("clone child stderr capture");
    let child = Command::new(std::env::current_exe().expect("test executable path"))
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_MARKER, marker)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated MCP integration child");
    let mut guard = ChildGuard::new(child);
    let deadline = Instant::now() + TEST_TIMEOUT;
    loop {
        if let Some(status) = guard.child_mut().try_wait().expect("poll child") {
            guard.take().wait().expect("reap isolated child");
            let stdout = base64(&std::fs::read(stdout_file.path()).expect("read full stdout file"));
            let stderr = base64(&std::fs::read(stderr_file.path()).expect("read full stderr file"));
            eprintln!("{marker} child stdout base64: {stdout}");
            eprintln!("{marker} child stderr base64: {stderr}");
            assert!(
                status.success(),
                "isolated child failed; stdout={stdout}; stderr={stderr}"
            );
            return;
        }
        if Instant::now() >= deadline {
            guard.kill_and_wait();
            let stdout =
                base64(&std::fs::read(stdout_file.path()).expect("read timed-out child stdout"));
            let stderr =
                base64(&std::fs::read(stderr_file.path()).expect("read timed-out child stderr"));
            eprintln!("{marker} timed-out child stdout base64: {stdout}");
            eprintln!("{marker} timed-out child stderr base64: {stderr}");
            panic!(
                "isolated {marker} child exceeded eight seconds; stdout={stdout}; stderr={stderr}"
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn atomic_replace(path: &std::path::Path, bytes: &[u8]) {
    let replacement = path.with_extension("replacement");
    std::fs::write(&replacement, bytes).expect("write replacement save beside source");
    std::fs::rename(&replacement, path).expect("atomically replace source on same filesystem");
}

fn run_owned_server_child() {
    let model = std::sync::Arc::new(RwLock::new(live_mcp::golden_model()));
    let expected = literal_expected_model();
    let (updates_tx, updates_rx) = std::sync::mpsc::channel::<AppliedSaveUpdate>();
    let owner = mcp::spawn_mcp_background_with_updates(
        std::sync::Arc::clone(&model),
        "127.0.0.1:0".parse().unwrap(),
        Some(updates_rx),
    )
    .expect("owned server binds before returning");
    let addr = owner.local_addr();
    assert_ne!(addr.port(), 0);
    let (session, tools) = live_mcp::initialize_http(addr).unwrap();
    assert!(tools.text().contains("where_am_i"));
    let mut stream = live_mcp::subscribe_sse_uris(
        addr,
        &session,
        &["nms://player/location", "nms://galaxy/stats"],
    )
    .unwrap();
    live_mcp::assert_model_matches(&model.blocking_read(), &expected).unwrap();

    let mut contradictory = SaveUpdate {
        delta: nms_core::SaveDelta::empty(),
        player_state: PlayerState::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
            9,
            None,
            None,
            999,
            998,
            997,
        ),
        save_version: 9999,
    };
    contradictory.delta.new_systems.push(System::new(
        GalacticAddress::new(4, 4, 4, 4, 0, 9),
        Some("observer-only-fake".into()),
        None,
        None,
        vec![],
    ));
    let first_cursor = stream.cursor();
    updates_tx
        .send(AppliedSaveUpdate {
            update: contradictory.clone(),
            names_changed: false,
            notifications: vec![],
        })
        .unwrap();
    let first_end = stream
        .wait_for_resource_updated_since(first_cursor, "nms://player/location", TEST_TIMEOUT)
        .unwrap();
    stream
        .wait_quiet_since(first_end, Duration::from_millis(650))
        .unwrap();

    let second_cursor = stream.cursor();
    updates_tx
        .send(AppliedSaveUpdate {
            update: contradictory,
            names_changed: false,
            notifications: vec![],
        })
        .unwrap();
    let quiet_error = stream
        .wait_quiet_since(second_cursor, Duration::from_millis(650))
        .expect_err("new player-location event must fail the quiet window");
    assert_eq!(
        quiet_error,
        "unexpected player-location notification during negative window"
    );
    let second_end = stream
        .wait_for_resource_updated_since(second_cursor, "nms://player/location", TEST_TIMEOUT)
        .unwrap();
    assert!(second_end > second_cursor);
    stream
        .wait_quiet_since(second_end, Duration::from_millis(650))
        .unwrap();

    let names_cursor = stream.cursor();
    updates_tx
        .send(AppliedSaveUpdate {
            update: SaveUpdate {
                delta: nms_core::SaveDelta::empty(),
                player_state: contradictory_state(),
                save_version: 4720,
            },
            names_changed: true,
            notifications: vec![],
        })
        .unwrap();
    let location_end = stream
        .wait_for_resource_updated_since(names_cursor, "nms://player/location", TEST_TIMEOUT)
        .unwrap();
    let stats_end = stream
        .wait_for_resource_updated_since(location_end, "nms://galaxy/stats", TEST_TIMEOUT)
        .unwrap();
    assert!(stats_end > location_end);
    stream
        .wait_quiet_uris_since(
            stats_end,
            &["nms://player/location", "nms://galaxy/stats"],
            Duration::from_millis(650),
        )
        .unwrap();

    let (tool_location, resource_location) = live_mcp::location_rpc_pair(addr, &session).unwrap();
    live_mcp::assert_location_pair_at(
        &tool_location,
        &resource_location,
        GalacticAddress::new(0, 0, 0, 1, 0, 0),
        "Origin-0",
        1,
        "000100000000",
        "07FF:007F:07FF:0001",
    )
    .unwrap();
    assert!(live_mcp::assert_model_matches(&model.blocking_read(), &expected).is_ok());

    owner.shutdown().expect("explicit stop and worker join");
    stream.wait_closed(TEST_TIMEOUT).unwrap();
    stream.close_and_join().unwrap();

    let second = mcp::spawn_mcp_background_with_updates(std::sync::Arc::clone(&model), addr, None)
        .expect("former port is reusable");
    let (second_session, _) = live_mcp::initialize_http(addr).unwrap();
    let mut second_stream = live_mcp::subscribe_sse(addr, &second_session).unwrap();
    drop(second);
    second_stream.wait_closed(TEST_TIMEOUT).unwrap();
    second_stream.close_and_join().unwrap();

    let third = mcp::spawn_mcp_background_with_updates(std::sync::Arc::clone(&model), addr, None)
        .expect("port remains reusable after defensive Drop");
    let (third_session, _) = live_mcp::initialize_http(addr).unwrap();
    let mut third_stream = live_mcp::subscribe_sse(addr, &third_session).unwrap();
    drop(third);
    third_stream.wait_closed(TEST_TIMEOUT).unwrap();
    third_stream.close_and_join().unwrap();
}

fn contradictory_state() -> PlayerState {
    PlayerState::new(
        GalacticAddress::new(0, 0, 0, 1, 0, 9),
        9,
        None,
        None,
        999,
        998,
        997,
    )
}

fn run_bind_error_child() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let address = listener.local_addr().unwrap();
    let model = std::sync::Arc::new(RwLock::new(live_mcp::golden_model()));
    let error =
        mcp::spawn_mcp_background_with_updates(std::sync::Arc::clone(&model), address, None)
            .expect_err("occupied address fails startup");
    assert!(
        matches!(error, McpBackgroundError::Bind { address: got, source }
        if got == address && source.kind() == std::io::ErrorKind::AddrInUse)
    );
    drop(listener);
    let owner = mcp::spawn_mcp_background_with_updates(model, address, None)
        .expect("address binds after blocker is released");
    assert_eq!(owner.local_addr(), address);
    owner.shutdown().expect("join recovered owner");
}

fn literal_expected_model() -> nms_graph::GalaxyModel {
    let mut expected = nms_graph::GalaxyModel::new();
    for (reality, name, biome, planet) in [
        (0, "Origin-0", Biome::Lush, "Leaf-0"),
        (9, "Imported-9", Biome::Toxic, "Leaf-9"),
    ] {
        let address = GalacticAddress::new(0, 0, 0, 1, 0, reality);
        expected.insert_system(System::new(
            address,
            Some(name.into()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(biome),
                None,
                false,
                Some(planet.into()),
                None,
            )],
        ));
        expected.insert_base(PlayerBase::new(
            format!("Base-{reality}"),
            BaseType::HomePlanetBase,
            address,
            [0.0; 3],
            Some("fixture-owner".into()),
        ));
    }
    expected.player_state = Some(PlayerState::new(
        GalacticAddress::new(0, 0, 0, 1, 0, 0),
        0,
        None,
        None,
        7,
        8,
        9,
    ));
    expected.set_active_galaxy(7);
    expected
}

struct ChildGuard(Option<Child>);
impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }
    fn child_mut(&mut self) -> &mut Child {
        self.0.as_mut().expect("child guard owns child")
    }
    fn take(&mut self) -> Child {
        self.0.take().expect("child guard owns child")
    }
    fn kill_and_wait(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
