//! Cache compatibility and route reconstruction regressions.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::query::BiomeFilter;
use nms_graph::route::{RouteError, RoutingAlgorithm};
use nms_graph::{GalaxyModel, SystemId};
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};

const PREDECESSOR_ARCHIVE: &[u8] = include_bytes!("fixtures/predecessor-schema2-semantic2.rkyv");

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn predecessor_fresh_model() -> GalaxyModel {
    let zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let mut model = GalaxyModel::new();
    model.player_state = Some(PlayerState::new(zero, 0, None, None, 7, 8, 9));
    for (address, infested, name) in [(zero, false, "Zero"), (nine, true, "Nine")] {
        model.insert_system(System::new(
            address,
            Some("Shared".into()),
            Some("Explorer".into()),
            None,
            vec![Planet::new(
                1,
                Some(Biome::Lush),
                None,
                infested,
                Some(name.into()),
                None,
            )],
        ));
    }
    model.set_active_galaxy(9);
    model
}

fn id(address: GalacticAddress) -> SystemId {
    SystemId::from_address(&address)
}

fn predecessor_projection(model: &GalaxyModel) -> String {
    let zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let zero_id = id(zero);
    let nine_id = id(nine);
    let explicit_zero = model.nearest_systems_from_address(&zero, usize::MAX);
    let explicit_nine = model.nearest_systems_from_address(&nine, usize::MAX);
    let active_nine = model.nearest_systems(&nine, usize::MAX);
    let active_zero_excluded = model.nearest_systems(&zero, usize::MAX);
    let planet_zero =
        model.nearest_planets_from_address(&zero, usize::MAX, &BiomeFilter::default());
    let planet_nine =
        model.nearest_planets_from_address(&nine, usize::MAX, &BiomeFilter::default());
    let current_query = RouteQuery {
        targets: TargetSelection::SystemIds(vec![]),
        from: RouteFrom::CurrentPosition,
        warp_range: None,
        within_ly: None,
        max_targets: Some(0),
        algorithm: RoutingAlgorithm::NearestNeighbor,
        return_to_start: false,
    };
    let mismatch = execute_route(model, &current_query)
        .unwrap_err()
        .downcast::<nms_graph::GraphError>()
        .unwrap()
        .to_string();
    let foreign = execute_route(
        model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![zero_id]),
            from: RouteFrom::Address(nine),
            ..current_query.clone()
        },
    )
    .unwrap_err()
    .downcast::<RouteError>()
    .unwrap()
    .to_string();
    let same_start = execute_route(
        model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![nine_id]),
            from: RouteFrom::Address(nine),
            ..current_query
        },
    )
    .unwrap_err()
    .downcast::<RouteError>()
    .unwrap()
    .to_string();
    let reachable_nine = model.reachable_systems(nine_id, 400.0).unwrap();
    format!(
        "systems={:?}/{:?}; planets={:?}/{:?}; active={:?}; excluded={:?}; state={:?}/{:?}; errors={mismatch:?}/{foreign:?}/{same_start:?}; reachable={reachable_nine:?}",
        explicit_zero,
        explicit_nine,
        planet_zero
            .iter()
            .map(|(key, planet, distance)| (*key, planet.name.as_deref(), *distance))
            .collect::<Vec<_>>(),
        planet_nine
            .iter()
            .map(|(key, planet, distance)| (*key, planet.name.as_deref(), *distance))
            .collect::<Vec<_>>(),
        active_nine,
        active_zero_excluded,
        model.player_position(),
        model.active_galaxy,
    )
}

fn rich_route_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    model.player_state = Some(PlayerState::new(
        GalacticAddress::new(0, 0, 0, 100, 0, 0),
        0,
        None,
        None,
        7,
        8,
        9,
    ));
    for (x, galaxy, name) in [
        (0, 9, "Start9"),
        (1, 9, "Bridge9"),
        (2, 9, "Goal9"),
        (0, 0, "Start0"),
        (1, 0, "Bridge0"),
        (2, 0, "Goal0"),
    ] {
        let address = GalacticAddress::new(x, 0, 0, 100 + x as u16, 0, galaxy);
        model.insert_system(System::new(
            address,
            Some(name.into()),
            Some("Explorer".into()),
            None,
            vec![Planet::new(
                0,
                Some(if galaxy == 9 {
                    Biome::Lush
                } else {
                    Biome::Barren
                }),
                None,
                false,
                Some(name.into()),
                None,
            )],
        ));
    }
    model.set_active_galaxy(0);
    model
}

fn rich_projection(model: &GalaxyModel, round_trip: bool, algorithm: RoutingAlgorithm) -> String {
    let start = GalacticAddress::new(0, 0, 0, 100, 0, 9);
    let target = id(GalacticAddress::new(2, 0, 0, 102, 0, 9));
    let result = execute_route(
        model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![target]),
            from: RouteFrom::Address(start),
            warp_range: Some(400.0),
            within_ly: None,
            max_targets: None,
            algorithm,
            return_to_start: round_trip,
        },
    )
    .unwrap();
    format!(
        "hops={:?}; flags={:?}; legs={:?}; cumulative={:?}; total={}; targets={}",
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.system_id)
            .collect::<Vec<_>>(),
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.is_waypoint)
            .collect::<Vec<_>>(),
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.leg_distance_ly)
            .collect::<Vec<_>>(),
        result
            .route
            .hops
            .iter()
            .map(|hop| hop.cumulative_ly)
            .collect::<Vec<_>>(),
        result.route.total_distance_ly,
        result.targets_visited,
    )
}

#[test]
fn predecessor_fixture_rebuild_matches_independent_fresh_and_current_round_trip() {
    // This computed checksum is over the actual fixture bytes; length/header
    // checks below fence the sealed predecessor artifact itself.
    assert_eq!(PREDECESSOR_ARCHIVE.len(), 360);
    assert_eq!(fnv1a64(PREDECESSOR_ARCHIVE), 0x80aafdbcef32c297);
    assert_eq!(&PREDECESSOR_ARCHIVE[..16], b"NMSCACHE\x02\0\0\0\x02\0\0\0");

    let expected = predecessor_fresh_model();
    let fixture =
        nms_cache::rebuild_model(&nms_cache::serialize::deserialize(PREDECESSOR_ARCHIVE).unwrap());
    let current_bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&expected, 4720)).unwrap();
    let current =
        nms_cache::rebuild_model(&nms_cache::serialize::deserialize(&current_bytes).unwrap());

    for model in [&expected, &fixture, &current] {
        assert_eq!(model.system_count(), 2);
        assert_eq!(model.planet_count(), 2);
        assert_eq!(model.active_galaxy, 9);
        assert_eq!(
            model.player_position(),
            Some(&GalacticAddress::new(0, 0, 0, 1, 0, 0))
        );
        assert_eq!(
            model.system_by_name_in_galaxy("shared", 0).unwrap().0,
            &id(GalacticAddress::new(0, 0, 0, 1, 0, 0))
        );
        assert_eq!(
            model.system_by_name_in_galaxy("shared", 9).unwrap().0,
            &id(GalacticAddress::new(0, 0, 0, 1, 0, 9))
        );
        assert_eq!(
            predecessor_projection(model),
            predecessor_projection(&expected)
        );
    }
}

#[test]
fn current_three_system_nine_galaxy_route_matches_fresh_and_checked_cache() {
    let fresh = rich_route_model();
    let bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    assert_eq!(&bytes[..16], b"NMSCACHE\x02\0\0\0\x02\0\0\0");
    let rebuilt = nms_cache::rebuild_model(&nms_cache::serialize::deserialize(&bytes).unwrap());
    for model in [&fresh, &rebuilt] {
        let direct = rich_projection(model, false, RoutingAlgorithm::NearestNeighbor);
        assert!(direct.contains("legs=[0.0, 400.0, 400.0]"), "{direct}");
        assert!(
            direct.contains("cumulative=[0.0, 400.0, 800.0]"),
            "{direct}"
        );
        assert!(direct.contains("flags=[false, true, false]"), "{direct}");
        assert!(direct.ends_with("total=800; targets=1"), "{direct}");
        assert_eq!(
            rich_projection(model, false, RoutingAlgorithm::TwoOpt),
            direct
        );
        let round_trip = rich_projection(model, true, RoutingAlgorithm::NearestNeighbor);
        assert!(round_trip.ends_with("targets=1"), "{round_trip}");
        assert_eq!(
            rich_projection(model, true, RoutingAlgorithm::TwoOpt),
            round_trip
        );
    }
    assert_eq!(
        rich_projection(&fresh, false, RoutingAlgorithm::NearestNeighbor),
        rich_projection(&rebuilt, false, RoutingAlgorithm::NearestNeighbor)
    );
}

#[derive(Debug, PartialEq)]
struct TypedPredecessorProjection {
    systems: usize,
    planets: usize,
    active_galaxy: u8,
    player_state: Option<PlayerState>,
    explicit_zero: Vec<(SystemId, f64)>,
    explicit_nine: Vec<(SystemId, f64)>,
    active_nine: Vec<(SystemId, f64)>,
    active_zero: Vec<(SystemId, f64)>,
    explicit_zero_planets: Vec<((SystemId, u8), f64)>,
    explicit_nine_planets: Vec<((SystemId, u8), f64)>,
    active_nine_planets: Vec<((SystemId, u8), f64)>,
    active_zero_planets: Vec<((SystemId, u8), f64)>,
    errors: Vec<TypedRouteError>,
    reachable_nine: std::collections::HashSet<SystemId>,
}

#[derive(Debug, PartialEq, Eq)]
enum TypedRouteError {
    ReferenceMismatch { reference: u8, active: u8 },
    NoPath { from: SystemId, to: SystemId },
    TooFewTargets,
}

fn typed_route_error(error: Box<dyn std::error::Error>) -> TypedRouteError {
    if let Some(error) = error.downcast_ref::<nms_graph::GraphError>() {
        return match error {
            nms_graph::GraphError::ReferenceGalaxyMismatch { reference, active } => {
                TypedRouteError::ReferenceMismatch {
                    reference: *reference,
                    active: *active,
                }
            }
            other => panic!("unexpected graph error: {other:?}"),
        };
    }
    match error
        .downcast_ref::<RouteError>()
        .expect("typed RouteError")
    {
        RouteError::NoPath { from, to } => TypedRouteError::NoPath {
            from: *from,
            to: *to,
        },
        RouteError::TooFewTargets => TypedRouteError::TooFewTargets,
        other => panic!("unexpected route error: {other:?}"),
    }
}

fn typed_predecessor_projection(model: &GalaxyModel) -> TypedPredecessorProjection {
    let zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let zero_id = id(zero);
    let nine_id = id(nine);
    let filter = BiomeFilter::default();
    let current = RouteQuery {
        targets: TargetSelection::SystemIds(vec![]),
        from: RouteFrom::CurrentPosition,
        warp_range: None,
        within_ly: None,
        max_targets: Some(0),
        algorithm: RoutingAlgorithm::NearestNeighbor,
        return_to_start: false,
    };
    let mismatch = typed_route_error(execute_route(model, &current).unwrap_err());
    let foreign = typed_route_error(
        execute_route(
            model,
            &RouteQuery {
                targets: TargetSelection::SystemIds(vec![zero_id]),
                from: RouteFrom::Address(nine),
                ..current.clone()
            },
        )
        .unwrap_err(),
    );
    let same_start = typed_route_error(
        execute_route(
            model,
            &RouteQuery {
                targets: TargetSelection::SystemIds(vec![nine_id]),
                from: RouteFrom::Address(nine),
                ..current
            },
        )
        .unwrap_err(),
    );
    let keys = |items: Vec<((SystemId, u8), &Planet, f64)>| {
        items
            .into_iter()
            .map(|(key, _, distance)| (key, distance))
            .collect()
    };
    TypedPredecessorProjection {
        systems: model.system_count(),
        planets: model.planet_count(),
        active_galaxy: model.active_galaxy,
        player_state: model.player_state.clone(),
        explicit_zero: model.nearest_systems_from_address(&zero, usize::MAX),
        explicit_nine: model.nearest_systems_from_address(&nine, usize::MAX),
        active_nine: model.nearest_systems(&nine, usize::MAX),
        active_zero: model.nearest_systems(&zero, usize::MAX),
        explicit_zero_planets: keys(model.nearest_planets_from_address(&zero, usize::MAX, &filter)),
        explicit_nine_planets: keys(model.nearest_planets_from_address(&nine, usize::MAX, &filter)),
        active_nine_planets: keys(model.nearest_planets(&nine, usize::MAX, &filter)),
        active_zero_planets: keys(model.nearest_planets(&zero, usize::MAX, &filter)),
        errors: vec![mismatch, foreign, same_start],
        reachable_nine: model
            .reachable_systems(nine_id, 400.0)
            .unwrap()
            .into_iter()
            .collect(),
    }
}

#[derive(Debug, PartialEq)]
struct TypedRouteProjection {
    ids: Vec<SystemId>,
    flags: Vec<bool>,
    legs: Vec<f64>,
    cumulative: Vec<f64>,
    total: f64,
    targets: usize,
}

fn typed_rich_projection(
    model: &GalaxyModel,
    return_to_start: bool,
    algorithm: RoutingAlgorithm,
) -> TypedRouteProjection {
    let start = GalacticAddress::new(0, 0, 0, 100, 0, 9);
    let target = id(GalacticAddress::new(2, 0, 0, 102, 0, 9));
    let result = execute_route(
        model,
        &RouteQuery {
            targets: TargetSelection::SystemIds(vec![target]),
            from: RouteFrom::Address(start),
            warp_range: Some(400.0),
            within_ly: None,
            max_targets: None,
            algorithm,
            return_to_start,
        },
    )
    .unwrap();
    TypedRouteProjection {
        ids: result.route.hops.iter().map(|h| h.system_id).collect(),
        flags: result.route.hops.iter().map(|h| h.is_waypoint).collect(),
        legs: result
            .route
            .hops
            .iter()
            .map(|h| h.leg_distance_ly)
            .collect(),
        cumulative: result.route.hops.iter().map(|h| h.cumulative_ly).collect(),
        total: result.route.total_distance_ly,
        targets: result.targets_visited,
    }
}

#[test]
fn predecessor_fixture_spatial_and_error_semantics_match_typed_literal_expectations() {
    let fresh = predecessor_fresh_model();
    let fixture =
        nms_cache::rebuild_model(&nms_cache::serialize::deserialize(PREDECESSOR_ARCHIVE).unwrap());
    let bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    let current = nms_cache::rebuild_model(&nms_cache::serialize::deserialize(&bytes).unwrap());
    let zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let zero_id = id(zero);
    let nine_id = id(nine);
    let expected = TypedPredecessorProjection {
        systems: 2,
        planets: 2,
        active_galaxy: 9,
        player_state: Some(PlayerState::new(zero, 0, None, None, 7, 8, 9)),
        explicit_zero: vec![(zero_id, 0.0)],
        explicit_nine: vec![(nine_id, 0.0)],
        active_nine: vec![(nine_id, 0.0)],
        active_zero: vec![],
        explicit_zero_planets: vec![((zero_id, 1), 0.0)],
        explicit_nine_planets: vec![((nine_id, 1), 0.0)],
        active_nine_planets: vec![((nine_id, 1), 0.0)],
        active_zero_planets: vec![],
        errors: vec![
            TypedRouteError::ReferenceMismatch {
                reference: 0,
                active: 9,
            },
            TypedRouteError::NoPath {
                from: nine_id,
                to: zero_id,
            },
            TypedRouteError::TooFewTargets,
        ],
        reachable_nine: [nine_id].into_iter().collect(),
    };
    for model in [&fresh, &fixture, &current] {
        assert_eq!(
            nms_cache::extract_cache_data(model, 4720).save_version,
            4720
        );
        assert_eq!(typed_predecessor_projection(model), expected);
        for (address, expected_key) in [(zero, (zero_id, 1)), (nine, (nine_id, 1))] {
            let planets =
                model.nearest_planets_from_address(&address, usize::MAX, &BiomeFilter::default());
            assert_eq!(planets.len(), 1);
            assert!(std::ptr::eq(planets[0].1, &model.planets[&expected_key]));
        }
    }
}

#[test]
fn current_route_cache_matches_exact_open_and_closed_typed_projection() {
    let fresh = rich_route_model();
    let bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    let rebuilt = nms_cache::rebuild_model(&nms_cache::serialize::deserialize(&bytes).unwrap());
    let a = id(GalacticAddress::new(0, 0, 0, 100, 0, 9));
    let b = id(GalacticAddress::new(1, 0, 0, 101, 0, 9));
    let c = id(GalacticAddress::new(2, 0, 0, 102, 0, 9));
    let open = TypedRouteProjection {
        ids: vec![a, b, c],
        flags: vec![false, true, false],
        legs: vec![0.0, 400.0, 400.0],
        cumulative: vec![0.0, 400.0, 800.0],
        total: 800.0,
        targets: 1,
    };
    let closed = TypedRouteProjection {
        ids: vec![a, b, c, b, a],
        flags: vec![false, true, false, true, false],
        legs: vec![0.0, 400.0, 400.0, 400.0, 400.0],
        cumulative: vec![0.0, 400.0, 800.0, 1200.0, 1600.0],
        total: 1600.0,
        targets: 1,
    };
    for model in [&fresh, &rebuilt] {
        for algorithm in [RoutingAlgorithm::NearestNeighbor, RoutingAlgorithm::TwoOpt] {
            assert_eq!(typed_rich_projection(model, false, algorithm), open);
            assert_eq!(typed_rich_projection(model, true, algorithm), closed);
        }
    }
}
