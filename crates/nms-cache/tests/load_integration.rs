use std::fs::{self, File, FileTimes};
use std::time::{Duration, SystemTime};

use nms_cache::serialize::{deserialize, serialize};
use nms_cache::{
    CacheError, extract_cache_data, is_cache_fresh, load_or_rebuild, read_cache, rebuild_model,
    write_cache,
};
use nms_graph::GalaxyModel;

const HEADER_LEN: usize = 16;

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
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    GalaxyModel::from_save(&save)
}

fn assert_source_semantics(actual: &GalaxyModel) {
    let expected = source_model();
    assert_eq!(actual.systems, expected.systems);
    assert_eq!(actual.planets, expected.planets);
    assert_eq!(actual.bases, expected.bases);
    assert_eq!(actual.player_state, expected.player_state);
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
        b"NMSCACHE\x01\0\0\0\x01\0\0\0"
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
    let mut bytes = current_cache_with_revision(1, 1);
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
    let bytes = current_cache_with_revision(1, 2);
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
    assert_eq!(&bytes[..HEADER_LEN], b"NMSCACHE\x01\0\0\0\x01\0\0\0");
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
        b"NMSCACHE\x01\0\0\0\x01\0\0\0"
    );
    assert_source_semantics(&rebuild_model(&restored));
}
