use std::fs;
use std::thread;
use std::time::Duration;

use nms_cache::{extract_cache_data, load_or_rebuild, write_cache};
use nms_graph::GalaxyModel;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": []
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn load_or_rebuild_from_save_when_no_cache() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    fs::write(&save_path, test_save_json()).unwrap();

    let (model, was_cached) = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!was_cached);
    assert!(!model.systems.is_empty());

    // Cache file should now exist
    assert!(cache_path.exists());
}

#[test]
fn load_or_rebuild_uses_cache_when_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    // Write save and build cache
    fs::write(&save_path, test_save_json()).unwrap();
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);
    let data = extract_cache_data(&model, 4720);

    thread::sleep(Duration::from_millis(50));
    write_cache(&data, &cache_path).unwrap();

    let (_, was_cached) = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(was_cached);
}

#[test]
fn load_or_rebuild_skips_cache_with_no_cache_flag() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    fs::write(&save_path, test_save_json()).unwrap();

    // Even with no_cache=true, should work
    let (_, was_cached) = load_or_rebuild(&cache_path, &save_path, true).unwrap();
    assert!(!was_cached);
    // And no cache file should be written
    assert!(!cache_path.exists());
}
