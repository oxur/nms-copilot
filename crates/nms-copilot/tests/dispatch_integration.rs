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
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn dispatch_find_returns_results() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("find").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn dispatch_stats_returns_output() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("stats").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("Galaxy Statistics") || output.contains("system"));
}

#[test]
fn dispatch_info_returns_summary() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("info").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("systems"));
    assert!(output.contains("planets"));
}

#[test]
fn dispatch_help_returns_text() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("help").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("Commands:"));
}

#[test]
fn dispatch_convert_glyphs() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("convert --glyphs 01717D8A4EA2")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("01717D8A4EA2"));
    assert!(output.contains("Signal Booster"));
}

#[test]
fn dispatch_show_base() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("show base \"Test Base\"")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("Test Base"));
}
