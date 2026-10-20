#![cfg(feature = "http")]

#[path = "support/live_mcp.rs"]
pub mod live_mcp;

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use nms_copilot::coordinator::AppliedSaveUpdate;
use nms_copilot::mcp::{self, McpBackgroundError};
use nms_core::SaveUpdate;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System};
use tokio::sync::RwLock;

const CHILD_MARKER: &str = "NMS_SLICE17_OWNED_CHILD";
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
        address: GalacticAddress::new(4, 4, 4, 4, 0, 9),
        name: Some("Unexpected".into()),
        discoverer: None,
        timestamp_secs: None,
        planets: vec![],
    });
    assert!(reject_raw(&extra_named));

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
    let mut stream = live_mcp::subscribe_sse(addr, &session).unwrap();
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

    let (tool_location, resource_location) = live_mcp::location_rpc_pair(addr, &session).unwrap();
    live_mcp::assert_location_pair(&tool_location, &resource_location, 0, "Origin-0", 1).unwrap();
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
