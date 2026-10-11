use assert_cmd::cargo::cargo_bin_cmd;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::System;
use nms_graph::{GalaxyModel, SystemId};
use nms_query::display::{format_find_results, format_show_system};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::show::{ShowQuery, ShowResult, execute_show};
use nms_query::theme::Theme;
use predicates::prelude::*;
use std::collections::HashSet;

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

    assert_eq!(results.len(), 2);
    let expected = HashSet::from([
        (
            SystemId::from_address(&GalacticAddress::from_packed(0x001000000064, 0)),
            1,
        ),
        (
            SystemId::from_address(&GalacticAddress::from_packed(0x002000000C80, 0)),
            1,
        ),
    ]);
    let actual: HashSet<_> = results
        .iter()
        .map(|result| {
            let system_id = SystemId::from_address(&result.system.address);
            let associated_planet = result
                .system
                .planets
                .iter()
                .find(|planet| planet.index == result.planet.index)
                .unwrap();
            assert_eq!(associated_planet, &result.planet);
            (system_id, result.planet.index)
        })
        .collect();
    assert_eq!(actual, expected);
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
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].planet.biome, Some(Biome::Lush));
    assert_eq!(
        SystemId::from_address(&results[0].system.address),
        SystemId::from_address(&GalacticAddress::from_packed(0x001000000064, 0))
    );
    assert_eq!(results[0].planet.index, 1);
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

#[test]
fn find_infested_flag_filters_the_real_cli_process_result() {
    let source = test_save_json();
    let original = r#""VP": ["0xCD", 1]"#;
    let modified = r#""VP": ["0xCD", 65536]"#;
    assert!(source.contains(original));
    let json = source.replace(original, modified);
    let fixture = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(fixture.path(), json).unwrap();

    cargo_bin_cmd!("nms")
        .args([
            "find",
            "--save",
            fixture.path().to_str().unwrap(),
            "--infested",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("002000000C80"))
        .stdout(predicates::str::contains("001000000064").not());
}

#[test]
fn show_formatter_omits_distance_when_player_and_system_galaxies_differ() {
    let player = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let remote = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let mut model = GalaxyModel::new();
    model.player_state = Some(PlayerState::new(player, 0, None, None, 0, 0, 0));
    model.set_active_galaxy(9);
    model.insert_system(System::new(
        remote,
        Some("Remote".into()),
        None,
        None,
        vec![],
    ));

    let ShowResult::System(result) =
        execute_show(&model, &ShowQuery::System("Remote".into())).unwrap()
    else {
        panic!("expected system result");
    };
    assert_eq!(result.distance_from_player, None);
    assert!(!format_show_system(&result, &Theme::none()).contains("Distance"));
}
