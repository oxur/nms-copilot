use nms_copilot::session::SessionState;
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

fn setup() -> (GalaxyModel, SessionState) {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);
    let session = SessionState::from_model(&model);
    (model, session)
}

#[test]
fn dispatch_find_returns_results() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("find").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn dispatch_stats_returns_output() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("stats").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Galaxy Statistics") || output.contains("system"));
}

#[test]
fn dispatch_info_returns_summary() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("info").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("systems"));
    assert!(output.contains("planets"));
}

#[test]
fn dispatch_help_returns_text() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("help").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Commands:"));
}

#[test]
fn dispatch_convert_glyphs() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("convert --glyphs 01717D8A4EA2")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("01717D8A4EA2"));
    assert!(output.contains("Signal Booster"));
}

#[test]
fn dispatch_show_base() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("show base \"Test Base\"")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Test Base"));
}

#[test]
fn dispatch_set_biome_filter() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("set biome Lush")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Lush"));
    assert!(session.biome_filter.is_some());
}

#[test]
fn dispatch_status_shows_state() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("status")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Euclid"));
    assert!(output.contains("Position:"));
}

#[test]
fn dispatch_reset_clears_state() {
    let (model, mut session) = setup();

    // Set a biome filter first
    let set_action = nms_copilot::commands::parse_line("set biome Toxic")
        .unwrap()
        .unwrap();
    nms_copilot::dispatch::dispatch(&set_action, &model, &mut session).unwrap();
    assert!(session.biome_filter.is_some());

    // Reset it
    let reset_action = nms_copilot::commands::parse_line("reset biome")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&reset_action, &model, &mut session).unwrap();
    assert!(output.contains("cleared"));
    assert!(session.biome_filter.is_none());
}

#[test]
fn dispatch_set_position_to_base() {
    let (model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("set position \"Test Base\"")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model, &mut session).unwrap();
    assert!(output.contains("Test Base"));
}
