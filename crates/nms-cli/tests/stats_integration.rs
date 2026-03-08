use nms_graph::GalaxyModel;
use nms_query::display::format_stats;
use nms_query::stats::{StatsQuery, execute_stats};
use nms_query::theme::Theme;

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
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Base Alpha", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""},
                    {"BaseVersion": 8, "GalacticAddress": "0x0A0002001234", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Base Beta", "BaseType": {"PersistentBaseTypes": "ExternalPlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x250003AB8C07", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x0A0002001234", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"B","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x1A0002001234", "DT": "Planet", "VP": ["0xEF", 4]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"B","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn stats_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = StatsQuery {
        biomes: true,
        discoveries: true,
    };
    let result = execute_stats(&model, &query);
    let output = format_stats(&result, &Theme::none());

    assert_eq!(result.system_count, 2);
    assert_eq!(result.planet_count, 3);
    assert_eq!(result.base_count, 2);
    assert!(output.contains("Galaxy Statistics"));
    assert!(output.contains("Biome Distribution"));
}

#[test]
fn stats_biome_distribution_consistent() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_stats(&model, &StatsQuery::default());
    let counted: usize = result.biome_counts.values().sum::<usize>() + result.unknown_biome_count;
    assert_eq!(counted, result.planet_count);
}

#[test]
fn stats_base_count_matches_save() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_stats(&model, &StatsQuery::default());
    assert_eq!(result.base_count, 2);
}
