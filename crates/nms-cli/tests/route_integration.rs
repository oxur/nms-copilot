//! CLI and typed route formatting regressions for galaxy-scoped routing.

use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use nms_core::address::GalacticAddress;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_graph::{GalaxyModel, SystemId};
use nms_query::display::format_route;
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::theme::Theme;
use predicates::prelude::*;
use std::io::Write;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("test")
        .join(name)
}

fn single_target_save_json() -> &'static str {
    r#"{"Version":4720,"Platform":"Mac|Final","ActiveContext":"Main","CommonStateData":{"SaveName":"Single target route","TotalPlayTime":100},"BaseContext":{"GameMode":1,"PlayerStateData":{"UniverseAddress":{"RealityIndex":0,"GalacticAddress":{"VoxelX":-896,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":32,"PlanetIndex":0}},"Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]}},"ExpeditionContext":{"GameMode":6,"PlayerStateData":{"UniverseAddress":{"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":0,"PlanetIndex":0}},"Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]}},"DiscoveryManagerData":{"DiscoveryData-v1":{"ReserveStore":0,"ReserveManaged":0,"Store":{"Record":[{"DD":{"UA":"0x001000000064","DT":"SolarSystem","VP":[]},"DM":{},"OWS":{"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000},"FL":{"U":1}},{"DD":{"UA":"0x101000000064","DT":"Planet","VP":["0xAB",0]},"DM":{},"OWS":{"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000},"FL":{"U":1}},{"DD":{"UA":"0x002000000C80","DT":"SolarSystem","VP":[]},"DM":{},"OWS":{"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000},"FL":{"U":1}},{"DD":{"UA":"0x102000000C80","DT":"Planet","VP":["0xCD",1]},"DM":{},"OWS":{"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000},"FL":{"U":1}}]}}}}"#
}

#[test]
fn test_nms_route_lush_targets_succeed_for_nn_and_two_opt_round_trip() {
    let fixture = fixture_path("multi_system_save.json");
    for (algorithm, expected) in [("nn", "nearest-neighbor"), ("2opt", "2-opt")] {
        cargo_bin_cmd!("nms")
            .args([
                "route",
                "--save",
                fixture.to_str().unwrap(),
                "--biome",
                "Lush",
                "--algo",
                algorithm,
                "--round-trip",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("ROUTE ITINERARY"))
            .stdout(predicate::str::contains(expected))
            .stdout(predicate::str::contains("Route: 2 targets"));
    }
}

#[test]
fn test_single_target_route_cli_matches_parser_and_query_oracle() {
    let mut fixture = tempfile::NamedTempFile::new().unwrap();
    fixture
        .write_all(single_target_save_json().as_bytes())
        .unwrap();
    let save = nms_save::parse_save_file(fixture.path()).unwrap();
    let model = GalaxyModel::from_save(&save);
    let start = GalacticAddress::new(-896, 0, 0, 32, 0, 0);
    let target = GalacticAddress::new(100, 0, 0, 16, 0, 0);
    let start_id = SystemId::from_address(&start);
    let target_id = SystemId::from_address(&target);
    let query = RouteQuery {
        targets: TargetSelection::Biome(BiomeFilter {
            biome: Some(nms_core::biome::Biome::Lush),
            ..Default::default()
        }),
        from: RouteFrom::CurrentPosition,
        warp_range: None,
        within_ly: None,
        max_targets: None,
        algorithm: RoutingAlgorithm::NearestNeighbor,
        return_to_start: true,
    };
    assert_eq!(model.nearest_systems_from_address(&start, 1)[0].0, start_id);
    assert_eq!(
        model.nearest_systems_from_address(&target, 1)[0].0,
        target_id
    );
    assert_eq!(start.distance_ly(&target), 398_400.0);
    let result = execute_route(&model, &query).unwrap();
    assert_eq!(
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.system_id)
            .collect::<Vec<_>>(),
        vec![start_id, target_id, start_id]
    );
    assert_eq!(
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.leg_distance_ly)
            .collect::<Vec<_>>(),
        vec![0.0, 398_400.0, 398_400.0]
    );
    assert_eq!(result.targets_visited, 1);

    for algorithm in ["nn", "2opt"] {
        cargo_bin_cmd!("nms")
            .args([
                "route",
                "--save",
                fixture.path().to_str().unwrap(),
                "--biome",
                "Lush",
                "--algo",
                algorithm,
                "--round-trip",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Route: 1 targets"));
    }
}

#[test]
fn test_nms_route_missing_target_fails_without_itinerary() {
    let fixture = fixture_path("multi_system_save.json");
    cargo_bin_cmd!("nms")
        .args([
            "route",
            "--save",
            fixture.to_str().unwrap(),
            "--target",
            "No Such Target",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("ROUTE ITINERARY").not())
        .stderr(predicate::str::contains("target not found:"));
}

#[test]
fn test_route_formatter_preserves_qualified_identity_and_exact_400_ly_legs() {
    let mut model = GalaxyModel::new();
    let start_address = GalacticAddress::new(0, 0, 0, 100, 0, 9);
    let target_address = GalacticAddress::new(1, 0, 0, 101, 0, 9);
    let zero_start = GalacticAddress::new(0, 0, 0, 100, 0, 0);
    let zero_goal = GalacticAddress::new(1, 0, 0, 101, 0, 0);
    model.player_state = Some(PlayerState::new(zero_start, 0, None, None, 0, 0, 0));
    for (address, name) in [
        (start_address, "Start9"),
        (target_address, "Goal9"),
        (zero_start, "Start0"),
        (zero_goal, "Goal0"),
    ] {
        model.insert_system(System::new(
            address,
            Some(name.into()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(nms_core::biome::Biome::Lush),
                None,
                false,
                None,
                None,
            )],
        ));
    }
    model.set_active_galaxy(0);
    let target_id = SystemId::from_address(&target_address);
    let zero_id = SystemId::from_address(&zero_goal);

    let result = execute_route(
        &model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![target_id]),
            from: RouteFrom::Address(start_address),
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        },
    )
    .unwrap();

    assert_eq!(result.route.hops.len(), 2);
    assert_eq!(
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.system_id)
            .collect::<Vec<_>>(),
        vec![SystemId::from_address(&start_address), target_id]
    );
    assert!(
        result
            .route
            .hops
            .iter()
            .all(|hop| hop.system_id.reality_index() == 9)
    );
    assert!(!result.route.hops.iter().any(|hop| hop.system_id == zero_id));
    assert_eq!(result.route.hops[0].leg_distance_ly, 0.0);
    assert_eq!(result.route.hops[1].leg_distance_ly, 400.0);
    assert_eq!(result.targets_visited, 1);
    let output = format_route(&result, &model, &Theme::none());
    let start_row = output.lines().find(|line| line.contains("Start9")).unwrap();
    let goal_row = output.lines().find(|line| line.contains("Goal9")).unwrap();
    assert!(start_row.contains("< 1 ly"), "{start_row}");
    assert!(goal_row.contains("~400 ly"), "{goal_row}");
    assert!(!output.contains("Start0"));
    assert!(!output.contains("Goal0"));
}

#[test]
fn test_route_biome_filter_is_not_implicitly_limited_to_one_hundred() {
    let fixture = fixture_path("multi_system_save.json");
    let save = nms_save::parse_save_file(&fixture).unwrap();
    let mut model = GalaxyModel::from_save(&save);
    let origin = *model.player_position().unwrap();
    for index in 0..110_u16 {
        let address = GalacticAddress::new(index as i16 + 20, 0, 0, index, 0, 0);
        model.insert_system(System::new(
            address,
            None,
            None,
            None,
            vec![Planet::new(
                0,
                Some(nms_core::biome::Biome::Lush),
                None,
                false,
                None,
                None,
            )],
        ));
    }
    let result = execute_route(
        &model,
        &RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter {
                biome: Some(nms_core::biome::Biome::Lush),
                ..Default::default()
            }),
            from: RouteFrom::Address(origin),
            warp_range: None,
            within_ly: None,
            max_targets: Some(105),
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        },
    )
    .unwrap();
    assert_eq!(result.targets_visited, 105);
}
