use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::display::format_find_results;
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::theme::Theme;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Home Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn find_pipeline_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery::default();
    let results = execute_find(&model, &query).unwrap();
    let output = format_find_results(&results, &Theme::none());

    assert!(!results.is_empty());
    assert!(output.contains("Portal Glyphs"));
}

#[test]
fn find_with_biome_filter() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        biome: Some(Biome::Lush),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    for r in &results {
        assert_eq!(r.planet.biome, Some(Biome::Lush));
    }
}

#[test]
fn find_from_base_reference() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        from: ReferencePoint::Base("Home Base".into()),
        nearest: Some(1),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 1);
}

#[test]
fn find_nearest_respects_limit() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        nearest: Some(1),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 1);
}
