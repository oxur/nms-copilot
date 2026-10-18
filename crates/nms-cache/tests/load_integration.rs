use std::fs::{self, File, FileTimes};
use std::time::{Duration, SystemTime};

use nms_cache::serialize::{deserialize, serialize};
use nms_cache::{
    CacheData, CacheError, extract_cache_data, is_cache_fresh, load_or_rebuild, read_cache,
    rebuild_model, write_cache,
};
use nms_graph::GalaxyModel;

const HEADER_LEN: usize = 16;
const PREDECESSOR_SCHEMA1_SEMANTIC1: &[u8] =
    include_bytes!("fixtures/predecessor-schema1-semantic1.rkyv");

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
    let mut bytes = b"NMSCACHE\x02\0\0\0\x02\0\0\0".to_vec();
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
        assert_eq!(&bytes[..HEADER_LEN], b"NMSCACHE\x02\0\0\0\x02\0\0\0");

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
        let sentinel = b"existing cache must remain byte-for-byte";
        fs::write(&existing_path, sentinel).unwrap();
        assert!(matches!(
            write_cache(&data, &existing_path),
            Err(CacheError::Serialize(_))
        ));
        assert_eq!(fs::read(&existing_path).unwrap(), sentinel);

        let missing_parent_path = dir.path().join("missing/nested/cache.rkyv");
        assert!(matches!(
            write_cache(&data, &missing_parent_path),
            Err(CacheError::Serialize(_))
        ));
        assert!(!missing_parent_path.parent().unwrap().exists());

        let mut wrong_schema = bytes.clone();
        wrong_schema[8..12].copy_from_slice(&3_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&wrong_schema),
            Err(CacheError::Incompatible(
                nms_cache::CacheCompatibilityError::SchemaMismatch {
                    found: 3,
                    expected: 2
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
                    expected: 2
                }
            ))
        ));
    }
}

#[test]
fn fresh_contradictory_cache_rebuilds_authoritative_source_then_warm_hits() {
    for (current_reality, address_reality) in [(0, 9), (9, 0)] {
        let dir = tempfile::tempdir().unwrap();
        let (save_path, cache_path) = paths(&dir);
        fs::write(&save_path, test_save_json()).unwrap();
        let data = mismatched_cache_data(current_reality, address_reality);
        let invalid_bytes = unchecked_current_envelope(&data);
        fs::write(&cache_path, &invalid_bytes).unwrap();
        set_fresh(&cache_path, &save_path);
        assert!(is_cache_fresh(&cache_path, &save_path));
        assert_eq!(data.save_version, 9999);
        let cached_player = data.player_state.as_ref().unwrap();
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

        let expected = source_model();
        let rebuilt = load_or_rebuild(&cache_path, &save_path, false).unwrap();
        assert!(!rebuilt.was_cached);
        assert_eq!(rebuilt.save_version, 4720);
        assert_source_semantics_against(&rebuilt.model, &expected);
        let rewritten_bytes = fs::read(&cache_path).unwrap();
        assert_eq!(
            &rewritten_bytes[..HEADER_LEN],
            b"NMSCACHE\x02\0\0\0\x02\0\0\0"
        );
        let rewritten = read_cache(&cache_path).unwrap();
        assert_eq!(rewritten.save_version, 4720);
        assert_eq!(rebuild_model(&rewritten).systems, expected.systems);
        assert_eq!(rebuild_model(&rewritten).planets, expected.planets);
        assert_eq!(rebuild_model(&rewritten).bases, expected.bases);
        assert_eq!(
            rebuild_model(&rewritten).player_state,
            expected.player_state
        );
        assert_eq!(rewritten.active_galaxy, expected.active_galaxy);

        set_fresh(&cache_path, &save_path);
        let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
        assert!(warm.was_cached);
        assert_eq!(warm.save_version, 4720);
        assert_source_semantics_against(&warm.model, &expected);
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
        if with_player {
            let address = nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 9);
            let previous = nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 4);
            let freighter = nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 6);
            model.player_state = Some(nms_core::player::PlayerState::new(
                address,
                9,
                Some(previous),
                Some(freighter),
                7,
                8,
                9,
            ));
        }

        let data = extract_cache_data(&model, 4720);
        assert_eq!(data.active_galaxy, 7);
        let bytes = serialize(&data).unwrap();
        let decoded = deserialize(&bytes).unwrap();
        let rebuilt = rebuild_model(&decoded);
        assert_dual_galaxy_model_with_selected(&rebuilt, with_player, 7);
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
        assert_eq!(file_data.player_state, decoded.player_state);
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
        b"NMSCACHE\x02\0\0\0\x02\0\0\0"
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
    for (schema, semantic) in [(2, 1), (1, 2)] {
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
    let mut bytes = current_cache_with_revision(2, 2);
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
    let bytes = current_cache_with_revision(2, 1);
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
    let bytes = current_cache_with_revision(2, 1);
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
    assert_eq!(&bytes[..HEADER_LEN], b"NMSCACHE\x02\0\0\0\x02\0\0\0");
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
        b"NMSCACHE\x02\0\0\0\x02\0\0\0"
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
                expected: 2
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
                expected: 2
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
    assert_eq!(&rewritten[..HEADER_LEN], b"NMSCACHE\x02\0\0\0\x02\0\0\0");
    assert_eq!(deserialize(&rewritten).unwrap().active_galaxy, 9);
    assert!(is_cache_fresh(&cache_path, &save_path));

    let warm = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(warm.was_cached);
    assert_eq!(warm.save_version, 4720);
    assert_source_semantics_against(&warm.model, &expected);
    assert_eq!(warm.model.active_galaxy, 9);
}

#[test]
fn current_schema_semantic1_cache_rebuilds_and_dual_galaxy_transport_warm_hits() {
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
                expected: 2
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
        b"NMSCACHE\x02\0\0\0\x02\0\0\0"
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
