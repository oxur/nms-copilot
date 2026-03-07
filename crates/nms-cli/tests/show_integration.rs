use nms_graph::GalaxyModel;
use nms_query::display::format_show_result;
use nms_query::show::{ShowQuery, ShowResult, execute_show};

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [100.0,200.0,300.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 1700000000, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"123","USN":"TestUser","PTK":"ST","TS":0}, "Name": "Sealab 2038", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn show_base_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_show(&model, &ShowQuery::Base("Sealab 2038".into())).unwrap();
    let output = format_show_result(&result);

    assert!(output.contains("Sealab 2038"));
    assert!(output.contains("HomePlanetBase"));
    assert!(output.contains("Euclid"));
}

#[test]
fn show_base_case_insensitive() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::Base("sealab 2038".into())).is_ok());
}

#[test]
fn show_system_with_planets() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    // Look up the system by hex address
    let sys_id = model.systems.keys().next().unwrap();
    let hex = format!("{:012X}", sys_id.0);
    let result = execute_show(&model, &ShowQuery::System(hex)).unwrap();

    match result {
        ShowResult::System(s) => {
            assert!(!s.system.planets.is_empty());
            let output = format_show_result(&ShowResult::System(s));
            assert!(output.contains("System Detail"));
        }
        _ => panic!("Expected system result"),
    }
}

#[test]
fn show_nonexistent_base_errors() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::Base("No Such Base".into())).is_err());
}

#[test]
fn show_nonexistent_system_errors() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::System("NoSystem".into())).is_err());
}
