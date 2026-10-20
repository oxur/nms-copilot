use std::fs::{self, File, FileTimes};
use std::time::{Duration, SystemTime};

use nms_cache::serialize::{deserialize, serialize};
use nms_cache::{
    CacheData, CacheError, extract_cache_data, is_cache_fresh, load_or_rebuild, read_cache,
    rebuild_model, write_cache,
};
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System, SystemId};
use nms_graph::GalaxyModel;

const HEADER_LEN: usize = 16;
const PREDECESSOR_SCHEMA1_SEMANTIC1: &[u8] =
    include_bytes!("fixtures/predecessor-schema1-semantic1.rkyv");
const PREDECESSOR_SCHEMA2_SEMANTIC2: &[u8] =
    include_bytes!("fixtures/predecessor-schema2-semantic2.rkyv");

fn rich_marker_addresses() -> (
    GalacticAddress,
    GalacticAddress,
    GalacticAddress,
    GalacticAddress,
) {
    (
        GalacticAddress::new(0, 0, 0, 1, 5, 0),
        GalacticAddress::new(0, 0, 0, 1, 3, 9),
        GalacticAddress::new(0, 0, 0, 2, 0, 0),
        GalacticAddress::new(3, 0, 0, 3, 0, 9),
    )
}

fn rich_player_state() -> PlayerState {
    let (u0, _, _, _) = rich_marker_addresses();
    PlayerState::new(
        u0,
        0,
        Some(GalacticAddress::new(3, 0, 0, 3, 1, 9)),
        Some(GalacticAddress::new(4, 5, 6, 4, 0, 9)),
        7,
        8,
        9,
    )
}

fn rich_base() -> PlayerBase {
    let (_, _, _, i9) = rich_marker_addresses();
    PlayerBase::new(
        "Base-9".into(),
        BaseType::HomePlanetBase,
        GalacticAddress::new(
            i9.voxel_x(),
            i9.voxel_y(),
            i9.voxel_z(),
            i9.solar_system_index(),
            1,
            9,
        ),
        [1.0, 2.0, 3.0],
        Some("AtlasUID".into()),
    )
}

fn rich_promoted_system() -> System {
    let (_, u9, _, _) = rich_marker_addresses();
    let id = SystemId::from_address(&u9);
    System::new(
        GalacticAddress::from_packed(id.system_address(), 9),
        Some("Promoted-9".into()),
        Some("Explorer".into()),
        Some(chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()),
        vec![Planet::new(
            2,
            Some(Biome::Toxic),
            None,
            true,
            Some("Leaf-9".into()),
            Some(171),
        )],
    )
}

fn expected_rich_cache(promoted: bool) -> CacheData {
    let (u0, u9, e0, i9) = rich_marker_addresses();
    CacheData {
        systems: vec![
            nms_cache::data::CachedSystem {
                address: GalacticAddress::from_packed(
                    SystemId::from_address(&u0).system_address(),
                    0,
                ),
                name: None,
                discoverer: None,
                timestamp_secs: None,
                planets: vec![],
                is_player_placeholder: true,
            },
            nms_cache::data::CachedSystem {
                address: GalacticAddress::from_packed(
                    SystemId::from_address(&u9).system_address(),
                    9,
                ),
                name: promoted.then(|| "Promoted-9".into()),
                discoverer: promoted.then(|| "Explorer".into()),
                timestamp_secs: promoted.then_some(1_700_000_000),
                planets: if promoted {
                    rich_promoted_system().planets
                } else {
                    vec![]
                },
                is_player_placeholder: !promoted,
            },
            nms_cache::data::CachedSystem {
                address: e0,
                name: None,
                discoverer: None,
                timestamp_secs: None,
                planets: vec![],
                is_player_placeholder: false,
            },
            nms_cache::data::CachedSystem {
                address: i9,
                name: Some("Imported-9".into()),
                discoverer: Some("AtlasOwner".into()),
                timestamp_secs: Some(1_700_000_010),
                planets: vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Import-leaf".into()),
                    Some(42),
                )],
                is_player_placeholder: false,
            },
        ],
        bases: vec![rich_base()],
        player_state: Some(rich_player_state()),
        active_galaxy: 7,
        save_version: 4725,
        cached_at: 0,
    }
}

fn expected_rich_model(promoted: bool) -> GalaxyModel {
    let (u0, u9, e0, i9) = rich_marker_addresses();
    let mut model = GalaxyModel::new();
    model.insert_player_placeholder(u0);
    model.insert_player_placeholder(u9);
    model.insert_system(System::new(e0, None, None, None, vec![]));
    model.insert_system(System::new(
        i9,
        Some("Imported-9".into()),
        Some("AtlasOwner".into()),
        Some(chrono::DateTime::from_timestamp(1_700_000_010, 0).unwrap()),
        vec![Planet::new(
            1,
            Some(Biome::Lush),
            None,
            false,
            Some("Import-leaf".into()),
            Some(42),
        )],
    ));
    if promoted {
        model.insert_system(rich_promoted_system());
    }
    model.insert_base(rich_base());
    model.player_state = Some(rich_player_state());
    model.set_active_galaxy(7);
    model.build_edges(nms_graph::EdgeStrategy::default());
    model
}

fn assert_rich_cache(actual: &CacheData, promoted: bool) {
    let expected = expected_rich_cache(promoted);
    let mut actual_systems = actual
        .systems
        .iter()
        .map(|row| format!("{row:?}"))
        .collect::<Vec<_>>();
    let mut expected_systems = expected
        .systems
        .iter()
        .map(|row| format!("{row:?}"))
        .collect::<Vec<_>>();
    actual_systems.sort();
    expected_systems.sort();
    assert_eq!(actual_systems, expected_systems);
    let mut actual_bases = actual
        .bases
        .iter()
        .map(|base| format!("{base:?}"))
        .collect::<Vec<_>>();
    let mut expected_bases = expected
        .bases
        .iter()
        .map(|base| format!("{base:?}"))
        .collect::<Vec<_>>();
    actual_bases.sort();
    expected_bases.sort();
    assert_eq!(actual_bases, expected_bases);
    assert_eq!(actual.player_state, expected.player_state);
    assert_eq!(actual.active_galaxy, 7);
    assert_eq!(actual.save_version, 4725);
}

fn assert_rich_model(actual: &GalaxyModel, promoted: bool) {
    let (u0, u9, e0, i9) = rich_marker_addresses();
    let u0_id = SystemId::from_address(&u0);
    let u9_id = SystemId::from_address(&u9);
    let e0_id = SystemId::from_address(&e0);
    let i9_id = SystemId::from_address(&i9);
    let expected = expected_rich_model(promoted);
    assert_eq!(actual.systems, expected.systems);
    assert_eq!(actual.planets, expected.planets);
    assert_eq!(actual.bases, expected.bases);
    assert_eq!(actual.player_state, expected.player_state);
    assert_eq!(actual.active_galaxy, 7);
    assert_eq!(actual.name_index, expected.name_index);
    assert_eq!(actual.biome_index, expected.biome_index);
    assert_eq!(actual.address_to_id, expected.address_to_id);
    assert_eq!(
        actual
            .spatial
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([0_u8, 9_u8])
    );
    let expected_markers = if promoted {
        std::collections::HashSet::from([SystemId::from_address(&rich_marker_addresses().0)])
    } else {
        std::collections::HashSet::from([
            SystemId::from_address(&rich_marker_addresses().0),
            SystemId::from_address(&rich_marker_addresses().1),
        ])
    };
    for (id, node) in &actual.node_map {
        assert_eq!(actual.graph.node_weight(*node), Some(id));
    }
    assert_eq!(
        actual
            .node_map
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([u0_id, u9_id, e0_id, i9_id])
    );
    assert_eq!(actual.graph.node_count(), 4);
    assert_eq!(actual.graph.edge_count(), 2);
    let mut edges = actual
        .graph
        .edge_indices()
        .map(|edge| {
            let (a, b) = actual.graph.edge_endpoints(edge).unwrap();
            let a = format!("{:?}", actual.graph.node_weight(a).unwrap());
            let b = format!("{:?}", actual.graph.node_weight(b).unwrap());
            let pair = if a <= b { (a, b) } else { (b, a) };
            (pair, *actual.graph.edge_weight(edge).unwrap())
        })
        .collect::<Vec<_>>();
    edges.sort_by(|a, b| a.0.cmp(&b.0));
    let mut expected_edges = vec![
        ((format!("{u0_id:?}"), format!("{e0_id:?}")), 264.682_872_8),
        ((format!("{u9_id:?}"), format!("{i9_id:?}")), 1200.0),
    ]
    .into_iter()
    .map(|((a, b), weight)| (if a <= b { (a, b) } else { (b, a) }, weight))
    .collect::<Vec<_>>();
    expected_edges.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(edges, expected_edges);
    for galaxy in [0, 9] {
        let mut actual_points = actual
            .spatial
            .get(&galaxy)
            .unwrap()
            .iter()
            .map(|point| (point.id, point.point))
            .collect::<Vec<_>>();
        let mut expected_points = expected
            .spatial
            .get(&galaxy)
            .unwrap()
            .iter()
            .map(|point| (point.id, point.point))
            .collect::<Vec<_>>();
        actual_points.sort_by_key(|(id, _)| (id.reality_index(), id.system_address()));
        expected_points.sort_by_key(|(id, _)| (id.reality_index(), id.system_address()));
        let expected_literal = if galaxy == 0 {
            vec![(u0_id, [0.0, 0.0, 0.0]), (e0_id, [0.0, 0.0, 0.0])]
        } else {
            vec![(u9_id, [0.0, 0.0, 0.0]), (i9_id, [3.0, 0.0, 0.0])]
        };
        assert_eq!(actual_points, expected_literal);
        assert_eq!(actual_points, expected_points);
    }
    let markers = actual
        .systems
        .keys()
        .copied()
        .filter(|id| actual.is_player_placeholder(id))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(markers, expected_markers);
}

fn save_mtime() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 5000000, "Nanites": 10000, "Specials": 500,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [1.0,2.0,3.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Semantic Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x250003AB8C07", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

fn source_model() -> GalaxyModel {
    source_model_from_json(test_save_json())
}

fn source_json_active_galaxy_9() -> String {
    let mut value: serde_json::Value = serde_json::from_str(test_save_json()).unwrap();
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["RealityIndex"] =
        serde_json::json!(9);
    serde_json::to_string(&value).unwrap()
}

fn source_model_from_json(json: &str) -> GalaxyModel {
    let save = nms_save::parse_save(json.as_bytes()).unwrap();
    GalaxyModel::from_save(&save)
}

fn assert_source_semantics(actual: &GalaxyModel) {
    let expected = source_model();
    assert_source_semantics_against(actual, &expected);
}

fn assert_source_semantics_against(actual: &GalaxyModel, expected: &GalaxyModel) {
    assert_eq!(actual.systems, expected.systems);
    assert_eq!(actual.planets, expected.planets);
    assert_eq!(actual.bases, expected.bases);
    assert_eq!(actual.player_state, expected.player_state);
    assert_eq!(actual.active_galaxy, expected.active_galaxy);
    assert_eq!(actual.systems.len(), 1);
    assert_eq!(actual.planets.len(), 2);
    let system = actual.systems.values().next().unwrap();
    assert_eq!(system.address.packed(), 0x050003AB8C07);
    let mut planet_indices = system
        .planets
        .iter()
        .map(|planet| planet.index)
        .collect::<Vec<_>>();
    planet_indices.sort_unstable();
    assert_eq!(planet_indices, [1, 2]);
    let base = actual.bases.get("semantic base").unwrap();
    assert_eq!(base.name, "Semantic Base");
    assert_eq!(base.address.packed(), 0x050003AB8C07);
    assert_eq!(base.position, [1.0, 2.0, 3.0]);
    let player = actual.player_state.as_ref().unwrap();
    assert_eq!(
        (player.units, player.nanites, player.quicksilver),
        (5_000_000, 10_000, 500)
    );
}

fn dual_galaxy_model(reverse: bool, with_player: bool) -> GalaxyModel {
    let mut model = GalaxyModel::new();
    if with_player {
        model.player_state = source_model().player_state;
    }
    let systems = [
        nms_core::system::System::new(
            nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 0),
            Some("Zero".into()),
            None,
            None,
            vec![nms_core::system::Planet::new(
                1,
                Some(nms_core::biome::Biome::Lush),
                None,
                false,
                Some("Zero marker".into()),
                None,
            )],
        ),
        nms_core::system::System::new(
            nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 9),
            Some("Nine".into()),
            None,
            None,
            vec![nms_core::system::Planet::new(
                1,
                Some(nms_core::biome::Biome::Toxic),
                None,
                false,
                Some("Nine marker".into()),
                None,
            )],
        ),
    ];
    for index in if reverse { [1, 0] } else { [0, 1] } {
        model.insert_system(systems[index].clone());
    }
    model.set_active_galaxy(9);
    model
}

fn assert_dual_galaxy_model(actual: &GalaxyModel, with_player: bool) {
    assert_dual_galaxy_model_with_selected(actual, with_player, 9);
    if with_player {
        assert_eq!(actual.player_position().unwrap().reality_index, 0);
    }
}

fn assert_dual_galaxy_model_with_selected(
    actual: &GalaxyModel,
    with_player: bool,
    selected_galaxy: u8,
) {
    let address0 = nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 0);
    let address9 = nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 9);
    let id0 = nms_core::system::SystemId::from_address(&address0);
    let id9 = nms_core::system::SystemId::from_address(&address9);
    assert_eq!(actual.active_galaxy, selected_galaxy);
    assert_eq!(actual.system_count(), 2);
    assert_eq!(actual.planet_count(), 2);
    assert_eq!(actual.spatial_for(0).unwrap().size(), 1);
    assert_eq!(actual.spatial_for(9).unwrap().size(), 1);
    assert_eq!(actual.address_to_id.get(&id0), Some(&id0));
    assert_eq!(actual.address_to_id.get(&id9), Some(&id9));
    assert_eq!(actual.system(&id0).unwrap().name.as_deref(), Some("Zero"));
    assert_eq!(actual.system(&id9).unwrap().name.as_deref(), Some("Nine"));
    assert_eq!(
        actual.planets.get(&(id0, 1)).unwrap().name.as_deref(),
        Some("Zero marker")
    );
    assert_eq!(
        actual.planets.get(&(id9, 1)).unwrap().name.as_deref(),
        Some("Nine marker")
    );
    assert_eq!(
        actual.planets.get(&(id0, 1)).unwrap().biome,
        Some(nms_core::biome::Biome::Lush)
    );
    assert_eq!(
        actual.planets.get(&(id9, 1)).unwrap().biome,
        Some(nms_core::biome::Biome::Toxic)
    );
    assert_eq!(actual.player_state.is_some(), with_player);
}

fn set_mtime(path: &std::path::Path, time: SystemTime) {
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(time))
        .unwrap();
}

fn set_fresh(cache_path: &std::path::Path, save_path: &std::path::Path) {
    set_mtime(save_path, save_mtime());
    set_mtime(cache_path, save_mtime() + Duration::from_secs(100));
    assert!(is_cache_fresh(cache_path, save_path));
}

fn paths(dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    (dir.path().join("save.json"), dir.path().join("galaxy.rkyv"))
}

fn legacy_cache() -> Vec<u8> {
    let mut data = extract_cache_data(&source_model(), 9999);
    data.save_version = 9999;
    rkyv::to_bytes::<rkyv::rancor::Error>(&data)
        .unwrap()
        .to_vec()
}

fn current_cache_with_revision(schema: u32, semantic: u32) -> Vec<u8> {
    let mut data = extract_cache_data(&source_model(), 9999);
    data.save_version = 9999;
    let mut bytes = serialize(&data).unwrap();
    bytes[8..12].copy_from_slice(&schema.to_le_bytes());
    bytes[12..16].copy_from_slice(&semantic.to_le_bytes());
    bytes
}

fn mismatched_cache_data(current_reality: u8, address_reality: u8) -> CacheData {
    let mut data = extract_cache_data(&source_model(), 9999);
    data.active_galaxy = 7;
    data.save_version = 9999;
    let player = data.player_state.as_mut().unwrap();
    player.current_address = nms_core::address::GalacticAddress::from_packed(
        player.current_address.packed(),
        address_reality,
    );
    player.current_reality = current_reality;
    player.units = 7;
    player.nanites = 8;
    player.quicksilver = 9;
    data
}

/// Build a valid rkyv payload inside the literal current envelope, bypassing
/// the public writer so reader tests can represent contradictory persisted data.
fn unchecked_current_envelope(data: &CacheData) -> Vec<u8> {
    let payload = rkyv::to_bytes::<rkyv::rancor::Error>(data).unwrap();
    let mut bytes = b"NMSCACHE\x03\0\0\0\x03\0\0\0".to_vec();
    bytes.extend_from_slice(&payload);
    bytes
}

fn assert_rebuilt_and_warm(cache_path: &std::path::Path, save_path: &std::path::Path) {
    let result = load_or_rebuild(cache_path, save_path, false).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
    set_fresh(cache_path, save_path);

    let warm = load_or_rebuild(cache_path, save_path, false).unwrap();
    assert!(warm.was_cached);
    assert_eq!(warm.save_version, 4720);
    assert_source_semantics(&warm.model);
}

#[test]
fn contradictory_player_reality_is_rejected_by_byte_file_and_write_boundaries() {
    let matching = mismatched_cache_data(9, 9);
    let matching_bytes = serialize(&matching).unwrap();
    let matching_decoded = deserialize(&matching_bytes).unwrap();
    let matching_player = matching_decoded.player_state.unwrap();
    assert_eq!(matching_player.current_reality, 9);
    assert_eq!(matching_player.current_address.reality_index, 9);

    for (current_reality, address_reality) in [(0, 9), (9, 0)] {
        let data = mismatched_cache_data(current_reality, address_reality);
        let bytes = unchecked_current_envelope(&data);
        assert_eq!(&bytes[..HEADER_LEN], b"NMSCACHE\x03\0\0\0\x03\0\0\0");

        let mut aligned = rkyv::util::AlignedVec::<16>::with_capacity(bytes.len() - HEADER_LEN);
        aligned.extend_from_slice(&bytes[HEADER_LEN..]);
        let decoded_payload = rkyv::from_bytes::<CacheData, rkyv::rancor::Error>(&aligned).unwrap();
        let contradictory_player = decoded_payload.player_state.unwrap();
        assert_eq!(contradictory_player.current_reality, current_reality);
        assert_eq!(
            contradictory_player.current_address.reality_index,
            address_reality
        );

        let error = deserialize(&bytes).unwrap_err();
        match error {
            CacheError::Deserialize(message) => {
                assert!(message.contains("current_reality"));
                assert!(message.contains("current_address.reality_index"));
                assert!(message.contains(&current_reality.to_string()));
                assert!(message.contains(&address_reality.to_string()));
            }
            other => panic!("expected player-state deserialize error, got {other:?}"),
        }

        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("bad.rkyv");
        fs::write(&cache_path, &bytes).unwrap();
        assert!(matches!(
            read_cache(&cache_path),
            Err(CacheError::Deserialize(_))
        ));

        let error = serialize(&data).unwrap_err();
        match error {
            CacheError::Serialize(message) => {
                assert!(message.contains("current_reality"));
                assert!(message.contains("current_address.reality_index"));
                assert!(message.contains(&current_reality.to_string()));
                assert!(message.contains(&address_reality.to_string()));
            }
            other => panic!("expected player-state serialize error, got {other:?}"),
        }

        let existing_path = dir.path().join("existing.rkyv");
        let existing_temp_path = existing_path.with_extension("rkyv.tmp");
        let sentinel = b"existing cache must remain byte-for-byte";
        fs::write(&existing_path, sentinel).unwrap();
        assert!(!existing_temp_path.exists());
        assert!(matches!(
            write_cache(&data, &existing_path),
            Err(CacheError::Serialize(_))
        ));
        assert_eq!(fs::read(&existing_path).unwrap(), sentinel);
        assert!(!existing_temp_path.exists());

        let missing_parent_path = dir.path().join("missing/nested/cache.rkyv");
        let missing_parent_temp_path = missing_parent_path.with_extension("rkyv.tmp");
        assert!(!missing_parent_path.parent().unwrap().exists());
        assert!(!missing_parent_path.exists());
        assert!(!missing_parent_temp_path.exists());
        assert!(matches!(
            write_cache(&data, &missing_parent_path),
            Err(CacheError::Serialize(_))
        ));
        assert!(!missing_parent_path.parent().unwrap().exists());
        assert!(!missing_parent_path.exists());
        assert!(!missing_parent_temp_path.exists());

        let mut wrong_schema = bytes.clone();
        wrong_schema[8..12].copy_from_slice(&2_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&wrong_schema),
            Err(CacheError::Incompatible(
                nms_cache::CacheCompatibilityError::SchemaMismatch {
                    found: 2,
                    expected: 3
                }
            ))
        ));
        let mut wrong_semantic = bytes.clone();
        wrong_semantic[12..16].copy_from_slice(&1_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&wrong_semantic),
            Err(CacheError::Incompatible(
                nms_cache::CacheCompatibilityError::SemanticMismatch {
                    found: 1,
                    expected: 3
                }
            ))
        ));
    }
}

#[test]
fn fresh_contradictory_cache_rebuilds_authoritative_source_then_warm_hits() {
    for (current_reality, address_reality, source_reality) in [(0, 9, 9), (9, 0, 0)] {
        let dir = tempfile::tempdir().unwrap();
        let (save_path, cache_path) = paths(&dir);
        let source_json = if source_reality == 9 {
            source_json_active_galaxy_9()
        } else {
            test_save_json().to_owned()
        };
        fs::write(&save_path, &source_json).unwrap();
        let data = mismatched_cache_data(current_reality, address_reality);
        let invalid_bytes = unchecked_current_envelope(&data);
        fs::write(&cache_path, &invalid_bytes).unwrap();
        set_fresh(&cache_path, &save_path);
        assert!(is_cache_fresh(&cache_path, &save_path));
        assert_eq!(data.save_version, 9999);
        assert_eq!(data.active_galaxy, 7);
        let cached_player = data.player_state.as_ref().unwrap();
        assert_eq!(cached_player.current_reality, current_reality);
        assert_eq!(cached_player.current_address.reality_index, address_reality);
        assert_eq!(
            (
                cached_player.units,
                cached_player.nanites,
                cached_player.quicksilver
            ),
            (7, 8, 9)
        );
        assert!(matches!(
            read_cache(&cache_path),
            Err(CacheError::Deserialize(_))
        ));

        let expected = source_model_from_json(&source_json);
        let rebuilt = load_or_rebuild(&cache_path, &save_path, false).unwrap();
        assert!(!rebuilt.was_cached);
        assert_eq!(rebuilt.save_version, 4720);
        assert_source_semantics_against(&rebuilt.model, &expected);
        let rebuilt_player = rebuilt.model.player_state.as_ref().unwrap();
        assert_eq!(
            rebuilt_player.current_address,
            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, source_reality)
        );
        assert_eq!(rebuilt_player.current_reality, source_reality);
        assert_eq!(rebuilt.model.active_galaxy, source_reality);
        assert_eq!(
            (
                rebuilt_player.units,
                rebuilt_player.nanites,
                rebuilt_player.quicksilver
            ),
            (5_000_000, 10_000, 500)
        );
        let rewritten_bytes = fs::read(&cache_path).unwrap();
        assert_eq!(
            &rewritten_bytes[..HEADER_LEN],
            b"NMSCACHE\x03\0\0\0\x03\0\0\0"
        );
        let rewritten = read_cache(&cache_path).unwrap();
        assert_eq!(rewritten.save_version, 4720);
        assert_eq!(rewritten.active_galaxy, source_reality);
        let rewritten_model = rebuild_model(&rewritten);
        assert_source_semantics_against(&rewritten_model, &expected);
        let rewritten_player = rewritten_model.player_state.as_ref().unwrap();
        assert_eq!(
            rewritten_player.current_address,
            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, source_reality)
        );
        assert_eq!(rewritten_player.current_reality, source_reality);
        assert_eq!(rewritten_model.active_galaxy, source_reality);
        assert_eq!(
            (
                rewritten_player.units,
                rewritten_player.nanites,
                rewritten_player.quicksilver
            ),
            (5_000_000, 10_000, 500)
        );

        set_fresh(&cache_path, &save_path);
        let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
        assert!(warm.was_cached);
        assert_eq!(warm.save_version, 4720);
        assert_source_semantics_against(&warm.model, &expected);
        let warm_player = warm.model.player_state.as_ref().unwrap();
        assert_eq!(
            warm_player.current_address,
            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, source_reality)
        );
        assert_eq!(warm_player.current_reality, source_reality);
        assert_eq!(warm.model.active_galaxy, source_reality);
        assert_eq!(
            (
                warm_player.units,
                warm_player.nanites,
                warm_player.quicksilver
            ),
            (5_000_000, 10_000, 500)
        );
    }
}

#[test]
fn malformed_source_and_no_cache_preserve_contradictory_cache_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    let bytes = unchecked_current_envelope(&mismatched_cache_data(0, 9));
    fs::write(&save_path, "not a save").unwrap();
    fs::write(&cache_path, &bytes).unwrap();
    set_fresh(&cache_path, &save_path);

    assert!(load_or_rebuild(&cache_path, &save_path, false).is_err());
    assert_eq!(fs::read(&cache_path).unwrap(), bytes);
    assert!(load_or_rebuild(&cache_path, &save_path, true).is_err());
    assert_eq!(fs::read(&cache_path).unwrap(), bytes);

    fs::write(&save_path, test_save_json()).unwrap();
    set_fresh(&cache_path, &save_path);
    let no_cache = load_or_rebuild(&cache_path, &save_path, true).unwrap();
    assert!(!no_cache.was_cached);
    assert_eq!(no_cache.save_version, 4720);
    assert_source_semantics(&no_cache.model);
    assert_eq!(fs::read(&cache_path).unwrap(), bytes);
}

#[test]
fn valid_independent_player_and_selected_galaxies_roundtrip_with_or_without_player() {
    for with_player in [true, false] {
        let mut model = dual_galaxy_model(false, false);
        model.set_active_galaxy(7);
        let expected_player = with_player.then(|| {
            nms_core::player::PlayerState::new(
                nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 9),
                9,
                Some(nms_core::address::GalacticAddress::from_packed(
                    0x150003AB8C07,
                    4,
                )),
                Some(nms_core::address::GalacticAddress::from_packed(
                    0x150003AB8C07,
                    6,
                )),
                7,
                8,
                9,
            )
        });
        model.player_state = expected_player.clone();

        let data = extract_cache_data(&model, 4720);
        assert_eq!(data.active_galaxy, 7);
        let bytes = serialize(&data).unwrap();
        let decoded = deserialize(&bytes).unwrap();
        assert_eq!(decoded.player_state, expected_player);
        let rebuilt = rebuild_model(&decoded);
        assert_dual_galaxy_model_with_selected(&rebuilt, with_player, 7);
        assert_eq!(rebuilt.systems, model.systems);
        assert_eq!(rebuilt.planets, model.planets);
        assert_eq!(rebuilt.bases, model.bases);
        assert_eq!(rebuilt.player_state, expected_player);
        if with_player {
            let player = rebuilt.player_state.as_ref().unwrap();
            assert_eq!(player.current_reality, 9);
            assert_eq!(player.current_address.reality_index, 9);
            assert_eq!(player.previous_address.unwrap().reality_index, 4);
            assert_eq!(player.freighter_address.unwrap().reality_index, 6);
            assert_eq!(
                (player.units, player.nanites, player.quicksilver),
                (7, 8, 9)
            );
        }

        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("valid.rkyv");
        write_cache(&data, &cache_path).unwrap();
        let file_data = read_cache(&cache_path).unwrap();
        let file_model = rebuild_model(&file_data);
        assert_dual_galaxy_model_with_selected(&file_model, with_player, 7);
        assert_eq!(file_data.player_state, expected_player);
        assert_eq!(file_model.player_state, expected_player);
        assert_eq!(file_model.systems, model.systems);
        assert_eq!(file_model.planets, model.planets);
        assert_eq!(file_model.bases, model.bases);
    }
}

#[test]
fn load_or_rebuild_from_save_when_no_cache() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();

    let result = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
    assert!(cache_path.exists());
    assert_eq!(
        &fs::read(&cache_path).unwrap()[..HEADER_LEN],
        b"NMSCACHE\x03\0\0\0\x03\0\0\0"
    );
}

#[test]
fn load_or_rebuild_uses_cache_when_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let data = extract_cache_data(&GalaxyModel::from_save(&save), 4720);
    write_cache(&data, &cache_path).unwrap();
    set_fresh(&cache_path, &save_path);

    let result = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
}

#[test]
fn load_or_rebuild_skips_cache_with_no_cache_flag() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();

    let result = load_or_rebuild(&cache_path, &save_path, true).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
    assert!(!cache_path.exists());
}

#[test]
fn headerless_legacy_archive_rebuilds_then_warm_hits() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    let legacy = legacy_cache();
    fs::write(&cache_path, &legacy).unwrap();
    assert!(matches!(
        read_cache(&cache_path),
        Err(CacheError::Incompatible(_))
    ));
    set_fresh(&cache_path, &save_path);

    assert_rebuilt_and_warm(&cache_path, &save_path);
    assert_eq!(read_cache(&cache_path).unwrap().save_version, 4720);
    assert!(fs::read(&cache_path).unwrap().starts_with(b"NMSCACHE"));
}

#[test]
fn schema_and_semantic_revision_mismatches_rebuild_then_warm_hit() {
    for (schema, semantic) in [(2, 3), (3, 2), (2, 2), (1, 3), (3, 1)] {
        let dir = tempfile::tempdir().unwrap();
        let (save_path, cache_path) = paths(&dir);
        fs::write(&save_path, test_save_json()).unwrap();
        fs::write(&cache_path, current_cache_with_revision(schema, semantic)).unwrap();
        set_fresh(&cache_path, &save_path);
        assert_rebuilt_and_warm(&cache_path, &save_path);
    }
}

#[test]
fn corrupt_current_payload_rebuilds_then_warm_hits() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    let mut bytes = current_cache_with_revision(3, 3);
    bytes.truncate(HEADER_LEN);
    fs::write(&cache_path, bytes).unwrap();
    set_fresh(&cache_path, &save_path);
    assert_rebuilt_and_warm(&cache_path, &save_path);
}

#[test]
fn incompatible_fresh_cache_and_malformed_save_preserve_cache_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, "not a save").unwrap();
    let bytes = current_cache_with_revision(2, 3);
    fs::write(&cache_path, &bytes).unwrap();
    set_fresh(&cache_path, &save_path);

    assert!(load_or_rebuild(&cache_path, &save_path, false).is_err());
    assert_eq!(fs::read(&cache_path).unwrap(), bytes);
}

#[test]
fn stale_compatible_sentinel_cache_rebuilds_from_source() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    let mut data = extract_cache_data(&source_model(), 9999);
    data.save_version = 9999;
    write_cache(&data, &cache_path).unwrap();
    set_mtime(&save_path, save_mtime() + Duration::from_secs(100));
    set_mtime(&cache_path, save_mtime());
    assert!(!is_cache_fresh(&cache_path, &save_path));

    let result = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
}

#[test]
fn no_cache_ignores_incompatible_existing_cache_and_preserves_it() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    let bytes = current_cache_with_revision(2, 3);
    fs::write(&cache_path, &bytes).unwrap();
    set_fresh(&cache_path, &save_path);

    let result = load_or_rebuild(&cache_path, &save_path, true).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
    assert_eq!(fs::read(&cache_path).unwrap(), bytes);
}

#[test]
fn cache_write_failure_returns_parsed_source_model() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let parent_file = dir.path().join("parent-file");
    fs::write(&save_path, test_save_json()).unwrap();
    fs::write(&parent_file, "regular file").unwrap();
    let cache_path = parent_file.join("galaxy.rkyv");

    let result = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!result.was_cached);
    assert_eq!(result.save_version, 4720);
    assert_source_semantics(&result.model);
}

#[test]
fn cache_freshness_uses_deterministic_file_times() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    fs::write(&cache_path, b"cache").unwrap();
    set_fresh(&cache_path, &save_path);
    assert!(is_cache_fresh(&cache_path, &save_path));
    set_mtime(&cache_path, save_mtime() - Duration::from_secs(1));
    assert!(!is_cache_fresh(&cache_path, &save_path));
}

#[test]
fn direct_file_roundtrip_has_header_and_source_version_is_not_header_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.rkyv");
    let data = extract_cache_data(&source_model(), 4720);
    write_cache(&data, &path).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(&bytes[..HEADER_LEN], b"NMSCACHE\x03\0\0\0\x03\0\0\0");
    assert_eq!(deserialize(&bytes).unwrap().save_version, 4720);
    assert_eq!(read_cache(&path).unwrap().save_version, 4720);
}

#[test]
fn compatible_cache_preserves_populated_model_with_independent_save_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.rkyv");
    let mut data = extract_cache_data(&source_model(), 9999);
    data.save_version = 9999;
    write_cache(&data, &path).unwrap();

    let restored = read_cache(&path).unwrap();
    assert_eq!(restored.save_version, 9999);
    assert_eq!(
        &fs::read(&path).unwrap()[..HEADER_LEN],
        b"NMSCACHE\x03\0\0\0\x03\0\0\0"
    );
    assert_source_semantics(&rebuild_model(&restored));
}

#[test]
fn actual_schema1_semantic1_predecessor_rebuilds_active_galaxy_source_then_warm_hits() {
    assert_eq!(
        &PREDECESSOR_SCHEMA1_SEMANTIC1[..HEADER_LEN],
        b"NMSCACHE\x01\0\0\0\x01\0\0\0"
    );
    assert!(matches!(
        deserialize(PREDECESSOR_SCHEMA1_SEMANTIC1),
        Err(CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 1,
                expected: 3
            }
        ))
    ));

    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    let active9_json = source_json_active_galaxy_9();
    fs::write(&save_path, &active9_json).unwrap();
    fs::write(&cache_path, PREDECESSOR_SCHEMA1_SEMANTIC1).unwrap();
    // A fresh incompatible cache must rebuild even when consecutive writes share an mtime.
    set_fresh(&cache_path, &save_path);
    assert!(matches!(
        read_cache(&cache_path),
        Err(CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 1,
                expected: 3
            }
        ))
    ));
    assert!(is_cache_fresh(&cache_path, &save_path));

    let rebuilt = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!rebuilt.was_cached);
    assert_eq!(rebuilt.save_version, 4720);
    let expected = source_model_from_json(&active9_json);
    assert_source_semantics_against(&rebuilt.model, &expected);
    assert_eq!(rebuilt.model.active_galaxy, 9);
    assert!(
        rebuilt
            .model
            .systems
            .keys()
            .all(|id| id.reality_index() == 0)
    );

    let rewritten = fs::read(&cache_path).unwrap();
    assert_eq!(&rewritten[..HEADER_LEN], b"NMSCACHE\x03\0\0\0\x03\0\0\0");
    assert_eq!(deserialize(&rewritten).unwrap().active_galaxy, 9);
    assert!(is_cache_fresh(&cache_path, &save_path));

    let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(warm.was_cached);
    assert_eq!(warm.save_version, 4720);
    assert_source_semantics_against(&warm.model, &expected);
    assert_eq!(warm.model.active_galaxy, 9);
}

#[test]
fn actual_schema2_semantic2_predecessor_rebuilds_source_then_current_and_marker_warm_hit() {
    assert_eq!(PREDECESSOR_SCHEMA2_SEMANTIC2.len(), 360);
    assert_eq!(
        &PREDECESSOR_SCHEMA2_SEMANTIC2[..HEADER_LEN],
        b"NMSCACHE\x02\0\0\0\x02\0\0\0"
    );
    assert!(matches!(
        deserialize(PREDECESSOR_SCHEMA2_SEMANTIC2),
        Err(CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 2,
                expected: 3
            }
        ))
    ));

    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    fs::write(&save_path, test_save_json()).unwrap();
    fs::write(&cache_path, PREDECESSOR_SCHEMA2_SEMANTIC2).unwrap();
    set_fresh(&cache_path, &save_path);
    assert!(is_cache_fresh(&cache_path, &save_path));

    let rebuilt = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!rebuilt.was_cached);
    assert_eq!(rebuilt.save_version, 4720);
    assert_source_semantics(&rebuilt.model);
    let rewritten = fs::read(&cache_path).unwrap();
    assert_eq!(&rewritten[..HEADER_LEN], b"NMSCACHE\x03\0\0\0\x03\0\0\0");
    let rewritten_mtime = fs::metadata(&cache_path).unwrap().modified().unwrap();
    let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(warm.was_cached);
    assert_source_semantics(&warm.model);
    assert_eq!(fs::read(&cache_path).unwrap(), rewritten);
    assert_eq!(
        fs::metadata(&cache_path).unwrap().modified().unwrap(),
        rewritten_mtime
    );

    let marker_dir = tempfile::tempdir().unwrap();
    let (marker_save, marker_cache) = paths(&marker_dir);
    fs::write(
        &marker_save,
        "malformed older source is not parsed for a fresh cache",
    )
    .unwrap();
    let mut marker_model = GalaxyModel::new();
    let marker_address = nms_core::address::GalacticAddress::new(5, 2, -3, 0x42, 0, 9);
    let marker_id = nms_core::system::SystemId::from_address(&marker_address);
    marker_model.insert_player_placeholder(marker_address);
    marker_model.player_state = Some(nms_core::player::PlayerState::new(
        marker_address,
        9,
        None,
        None,
        71,
        82,
        93,
    ));
    marker_model.set_active_galaxy(7);
    let marker_data = extract_cache_data(&marker_model, 4725);
    assert!(marker_data.systems[0].is_player_placeholder);
    write_cache(&marker_data, &marker_cache).unwrap();
    set_fresh(&marker_cache, &marker_save);
    let marker_bytes = fs::read(&marker_cache).unwrap();
    let marker_mtime = fs::metadata(&marker_cache).unwrap().modified().unwrap();
    let marker_warm = load_or_rebuild(&marker_cache, &marker_save, false).unwrap();
    assert!(marker_warm.was_cached);
    assert_eq!(marker_warm.save_version, 4725);
    assert!(marker_warm.model.is_player_placeholder(&marker_id));
    assert_eq!(marker_warm.model.player_state, marker_model.player_state);
    assert_eq!(marker_warm.model.active_galaxy, 7);
    assert_eq!(marker_warm.model.systems, marker_model.systems);
    assert_eq!(marker_warm.model.planets, marker_model.planets);
    assert_eq!(marker_warm.model.bases, marker_model.bases);
    assert_eq!(marker_warm.model.name_index, marker_model.name_index);
    assert_eq!(marker_warm.model.biome_index, marker_model.biome_index);
    assert_eq!(marker_warm.model.address_to_id, marker_model.address_to_id);
    assert_eq!(
        marker_warm
            .model
            .node_map
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([marker_id])
    );
    assert_eq!(marker_warm.model.graph.node_count(), 1);
    assert_eq!(
        marker_warm
            .model
            .graph
            .node_weight(marker_warm.model.node_map[&marker_id]),
        Some(&marker_id)
    );
    assert_eq!(
        marker_warm
            .model
            .spatial
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([9])
    );
    assert_eq!(
        marker_warm
            .model
            .spatial_for(9)
            .unwrap()
            .iter()
            .map(|point| (point.id, point.point))
            .collect::<Vec<_>>(),
        vec![(marker_id, [5.0, 2.0, -3.0])]
    );
    assert_eq!(fs::read(&marker_cache).unwrap(), marker_bytes);
    assert_eq!(
        fs::metadata(&marker_cache).unwrap().modified().unwrap(),
        marker_mtime
    );
}

#[test]
fn marker_round_trip_preserves_full_model_and_genuine_planet_promotion() {
    let (u0, u9, e0, i9) = rich_marker_addresses();
    let mut source = GalaxyModel::new();
    source.insert_player_placeholder(u0);
    source.insert_player_placeholder(u9);
    source.insert_system(System::new(e0, None, None, None, vec![]));
    source.insert_system(System::new(
        i9,
        Some("Imported-9".into()),
        Some("AtlasOwner".into()),
        Some(chrono::DateTime::from_timestamp(1_700_000_010, 0).unwrap()),
        vec![Planet::new(
            1,
            Some(Biome::Lush),
            None,
            false,
            Some("Import-leaf".into()),
            Some(42),
        )],
    ));
    source.insert_base(rich_base());
    source.player_state = Some(rich_player_state());
    source.set_active_galaxy(7);
    source.build_edges(nms_graph::EdgeStrategy::default());

    let dir = tempfile::tempdir().unwrap();
    let cache_path = dir.path().join("rich-placeholder.rkyv");

    let raw = extract_cache_data(&source, 4725);
    assert_rich_cache(&raw, false);
    let encoded = serialize(&raw).unwrap();
    let decoded = deserialize(&encoded).unwrap();
    assert_rich_cache(&decoded, false);
    write_cache(&raw, &cache_path).unwrap();
    let checked_file = read_cache(&cache_path).unwrap();
    assert_rich_cache(&checked_file, false);
    let rebuilt = rebuild_model(&checked_file);
    assert_rich_model(&rebuilt, false);

    let mut reversed = extract_cache_data(&source, 4725);
    reversed.systems.reverse();
    assert_rich_cache(&reversed, false);
    write_cache(&reversed, &cache_path).unwrap();
    let reversed_file = read_cache(&cache_path).unwrap();
    assert_rich_cache(&reversed_file, false);
    assert_rich_model(&rebuild_model(&reversed_file), false);

    let mut promoted = rebuild_model(&checked_file);
    promoted.insert_system(rich_promoted_system());
    assert_rich_model(&promoted, true);
    let promoted_raw = extract_cache_data(&promoted, 4725);
    assert_rich_cache(&promoted_raw, true);
    let promoted_decoded = deserialize(&serialize(&promoted_raw).unwrap()).unwrap();
    assert_rich_cache(&promoted_decoded, true);
    write_cache(&promoted_raw, &cache_path).unwrap();
    let promoted_file = read_cache(&cache_path).unwrap();
    assert_rich_cache(&promoted_file, true);
    assert_rich_model(&rebuild_model(&promoted_file), true);
}

#[test]
fn current_schema3_semantic1_cache_rebuilds_and_dual_galaxy_transport_warm_hits() {
    let dir = tempfile::tempdir().unwrap();
    let (save_path, cache_path) = paths(&dir);
    let active9_json = source_json_active_galaxy_9();
    fs::write(&save_path, &active9_json).unwrap();
    let mut semantic1 = serialize(&extract_cache_data(&source_model(), 9999)).unwrap();
    semantic1[12..16].copy_from_slice(&1_u32.to_le_bytes());
    assert!(matches!(
        deserialize(&semantic1),
        Err(CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SemanticMismatch {
                found: 1,
                expected: 3
            }
        ))
    ));
    fs::write(&cache_path, &semantic1).unwrap();
    // Exercise semantic rejection, rather than relying on write timing to establish freshness.
    set_fresh(&cache_path, &save_path);
    assert!(is_cache_fresh(&cache_path, &save_path));

    let rebuilt = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!rebuilt.was_cached);
    assert_eq!(rebuilt.save_version, 4720);
    assert_eq!(rebuilt.model.active_galaxy, 9);
    assert_eq!(
        &fs::read(&cache_path).unwrap()[..HEADER_LEN],
        b"NMSCACHE\x03\0\0\0\x03\0\0\0"
    );
    set_fresh(&cache_path, &save_path);
    let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(warm.was_cached);
    assert_eq!(warm.save_version, 4720);
    assert_eq!(warm.model.active_galaxy, 9);

    for reverse in [false, true] {
        for with_player in [false, true] {
            let model = dual_galaxy_model(reverse, with_player);
            let data = extract_cache_data(&model, 9999);
            assert_eq!(data.active_galaxy, 9);
            let bytes = serialize(&data).unwrap();
            let direct = rebuild_model(&deserialize(&bytes).unwrap());
            assert_dual_galaxy_model(&direct, with_player);

            write_cache(&data, &cache_path).unwrap();
            set_fresh(&cache_path, &save_path);
            let cached = load_or_rebuild(&cache_path, &save_path, false).unwrap();
            assert!(cached.was_cached);
            assert_eq!(cached.save_version, 9999);
            assert_dual_galaxy_model(&cached.model, with_player);
        }
    }
}

#[test]
fn explicit_placeholder_validation_is_shared_by_codec_and_preserves_write_targets() {
    use nms_cache::data::CachedSystem;
    use nms_core::address::GalacticAddress;
    use nms_core::system::{Planet, SystemId};

    let address = GalacticAddress::new(1, 2, 3, 4, 0, 9);
    let id = SystemId::from_address(&address);
    let expected_message = format!(
        "player placeholder {id} must have a planet-zero address and no discovery metadata or planets"
    );
    let data_for = |system: CachedSystem| CacheData {
        systems: vec![system],
        bases: vec![],
        player_state: None,
        active_galaxy: 7,
        save_version: 4720,
        cached_at: 0,
    };
    let valid = || CachedSystem {
        address,
        name: None,
        discoverer: None,
        timestamp_secs: None,
        planets: vec![],
        is_player_placeholder: true,
    };
    let assert_preserved = |model: &GalaxyModel, expected: &System| {
        assert_eq!(
            model.systems,
            std::collections::HashMap::from([(id, expected.clone())])
        );
        let expected_planets: std::collections::HashMap<(SystemId, u8), Planet> = expected
            .planets
            .iter()
            .map(|planet| ((id, planet.index), planet.clone()))
            .collect();
        assert_eq!(model.planets, expected_planets);
        let expected_names = expected
            .name
            .as_ref()
            .map(|name| {
                std::collections::HashMap::from([(
                    (expected.address.reality_index, name.to_lowercase()),
                    id,
                )])
            })
            .unwrap_or_default();
        assert_eq!(model.name_index, expected_names);
        let mut expected_biomes: std::collections::HashMap<Biome, Vec<(SystemId, u8)>> =
            std::collections::HashMap::new();
        for planet in &expected.planets {
            if let Some(biome) = planet.biome {
                expected_biomes
                    .entry(biome)
                    .or_default()
                    .push((id, planet.index));
            }
        }
        assert_eq!(model.biome_index, expected_biomes);
        assert_eq!(
            model.address_to_id,
            std::collections::HashMap::from([(id, id)])
        );
        assert_eq!(model.node_map.len(), 1);
        assert_eq!(model.graph.node_count(), 1);
        assert_eq!(model.graph.node_weight(model.node_map[&id]), Some(&id));
        assert_eq!(
            model
                .spatial
                .keys()
                .copied()
                .collect::<std::collections::HashSet<_>>(),
            std::collections::HashSet::from([expected.address.reality_index])
        );
        assert_eq!(
            model
                .spatial
                .get(&expected.address.reality_index)
                .unwrap()
                .iter()
                .map(|point| (point.id, point.point))
                .collect::<Vec<_>>(),
            vec![(
                id,
                [
                    expected.address.voxel_x() as f64,
                    expected.address.voxel_y() as f64,
                    expected.address.voxel_z() as f64
                ]
            )]
        );
        assert!(!model.is_player_placeholder(&id));
        assert_eq!(model.system_count(), 1);
        assert_eq!(model.planet_count(), expected.planets.len());
        assert!(model.bases.is_empty());
        assert_eq!(model.active_galaxy, 7);
    };
    let invalid_cases = [
        CachedSystem {
            address: GalacticAddress::new(1, 2, 3, 4, 1, 9),
            ..valid()
        },
        CachedSystem {
            name: Some(String::new()),
            ..valid()
        },
        CachedSystem {
            discoverer: Some(String::new()),
            ..valid()
        },
        CachedSystem {
            timestamp_secs: Some(0),
            ..valid()
        },
        CachedSystem {
            planets: vec![Planet::new(1, None, None, false, None, None)],
            ..valid()
        },
    ];

    for invalid in invalid_cases {
        let expected = System::new(
            invalid.address,
            invalid.name.clone(),
            invalid.discoverer.clone(),
            invalid
                .timestamp_secs
                .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0)),
            invalid.planets.clone(),
        );
        let data = data_for(invalid);
        match serialize(&data).unwrap_err() {
            CacheError::Serialize(message) => assert_eq!(message, expected_message),
            other => panic!("expected marker serialize error, got {other:?}"),
        }
        let payload = rkyv::to_bytes::<rkyv::rancor::Error>(&data).unwrap();
        let mut raw = b"NMSCACHE\x03\0\0\0\x03\0\0\0".to_vec();
        raw.extend_from_slice(&payload);
        match deserialize(&raw).unwrap_err() {
            CacheError::Deserialize(message) => assert_eq!(message, expected_message),
            other => panic!("expected marker deserialize error, got {other:?}"),
        }
        let rebuilt = rebuild_model(&data);
        assert_preserved(&rebuilt, &expected);
    }

    let rich_invalid = CachedSystem {
        address: GalacticAddress::new(1, 2, 3, 4, 1, 9),
        name: Some("Rich-malformed".into()),
        discoverer: Some("Trusted-owner".into()),
        timestamp_secs: Some(1_700_000_123),
        planets: vec![Planet::new(
            1,
            Some(Biome::Toxic),
            None,
            true,
            Some("Retained-world".into()),
            Some(0xBEEF),
        )],
        is_player_placeholder: true,
    };
    let rich_expected = System::new(
        rich_invalid.address,
        rich_invalid.name.clone(),
        rich_invalid.discoverer.clone(),
        Some(chrono::DateTime::from_timestamp(1_700_000_123, 0).unwrap()),
        rich_invalid.planets.clone(),
    );
    let rich_rebuilt = rebuild_model(&data_for(rich_invalid));
    assert_preserved(&rich_rebuilt, &rich_expected);

    let mut genuine_empty = valid();
    genuine_empty.is_player_placeholder = false;
    let genuine_data = data_for(genuine_empty);
    let genuine_bytes = serialize(&genuine_data).unwrap();
    let genuine_decoded = deserialize(&genuine_bytes).unwrap();
    let genuine_model = rebuild_model(&genuine_decoded);
    assert!(!genuine_decoded.systems[0].is_player_placeholder);
    assert!(!genuine_model.is_player_placeholder(&id));
    assert!(genuine_model.system(&id).unwrap().planets.is_empty());

    let genuine = || CachedSystem {
        name: Some("Genuine".into()),
        planets: vec![Planet::new(
            2,
            None,
            None,
            false,
            Some("World".into()),
            None,
        )],
        is_player_placeholder: false,
        ..valid()
    };
    for systems in [vec![valid(), genuine()], vec![genuine(), valid()]] {
        let mut mixed = data_for(valid());
        mixed.systems = systems;
        let rebuilt = rebuild_model(&mixed);
        assert!(!rebuilt.is_player_placeholder(&id));
        let expected = System::new(
            address,
            Some("Genuine".into()),
            None,
            None,
            vec![Planet::new(
                2,
                None,
                None,
                false,
                Some("World".into()),
                None,
            )],
        );
        assert_preserved(&rebuilt, &expected);
    }

    let mut invalid_write = valid();
    invalid_write.name = Some(String::new());
    let invalid_write = data_for(invalid_write);
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("existing.rkyv");
    let temp = existing.with_extension("rkyv.tmp");
    fs::write(&existing, b"preserve existing sentinel").unwrap();
    set_mtime(&existing, save_mtime() + Duration::from_secs(23));
    let before_bytes = fs::read(&existing).unwrap();
    let before_mtime = fs::metadata(&existing).unwrap().modified().unwrap();
    assert!(matches!(
        write_cache(&invalid_write, &existing),
        Err(CacheError::Serialize(_))
    ));
    assert_eq!(fs::read(&existing).unwrap(), before_bytes);
    assert_eq!(
        fs::metadata(&existing).unwrap().modified().unwrap(),
        before_mtime
    );
    assert!(!temp.exists());

    let absent_parent = dir.path().join("absent/nested/cache.rkyv");
    assert!(matches!(
        write_cache(&invalid_write, &absent_parent),
        Err(CacheError::Serialize(_))
    ));
    assert!(!absent_parent.parent().unwrap().exists());
    assert!(!absent_parent.exists());
    assert!(!absent_parent.with_extension("rkyv.tmp").exists());
}
