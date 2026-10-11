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
        .stderr(predicate::str::contains("route target not found"));
}

#[test]
fn test_route_formatter_preserves_qualified_identity_and_exact_400_ly_legs() {
    let mut model = GalaxyModel::new();
    let start_address = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let target_address = GalacticAddress::new(1, 0, 0, 1, 0, 0);
    let foreign_address = GalacticAddress::new(1, 0, 0, 1, 0, 9);
    model.player_state = Some(PlayerState::new(start_address, 0, None, None, 0, 0, 0));
    model.insert_system(System::new(
        start_address,
        Some("Shared".into()),
        None,
        None,
        vec![Planet::new(0, None, None, false, None, None)],
    ));
    model.insert_system(System::new(
        target_address,
        Some("Twin".into()),
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
    model.insert_system(System::new(
        foreign_address,
        Some("Twin".into()),
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
    let local_target = SystemId::from_address(&target_address);
    let foreign_target = SystemId::from_address(&foreign_address);

    let result = execute_route(
        &model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![local_target]),
            from: RouteFrom::Address(start_address),
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: true,
        },
    )
    .unwrap();

    assert_eq!(result.route.hops.len(), 3);
    assert!(
        result
            .route
            .hops
            .iter()
            .all(|hop| hop.system_id.reality_index() == 0)
    );
    assert!(
        !result
            .route
            .hops
            .iter()
            .any(|hop| hop.system_id == foreign_target)
    );
    assert_eq!(result.route.hops[1].leg_distance_ly, 400.0);
    assert_eq!(result.route.hops[2].leg_distance_ly, 400.0);
    assert_eq!(result.targets_visited, 1);
    let output = format_route(&result, &model, &Theme::none());
    assert!(output.contains("Twin"));
    assert!(output.contains("~400 ly"));
    // Two legs are exactly 400 ly; the first cumulative column is also 400 ly.
    assert_eq!(output.matches("~400 ly").count(), 3);
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
