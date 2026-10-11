//! Route query layer -- resolves targets and executes routing algorithms.

use std::collections::HashSet;

use nms_core::address::GalacticAddress;
use nms_graph::query::BiomeFilter;
use nms_graph::route::{Route, RoutingAlgorithm};
use nms_graph::{GalaxyModel, GraphError, SystemId};

/// How route targets are specified.
#[derive(Debug, Clone)]
pub enum TargetSelection {
    /// Find planets matching a biome filter, use their systems as targets.
    Biome(BiomeFilter),
    /// Named bases or systems (looked up case-insensitively).
    Named(Vec<String>),
    /// Explicit system IDs.
    SystemIds(Vec<SystemId>),
}

/// Where the route starts from.
#[derive(Debug, Clone, Default)]
pub enum RouteFrom {
    /// Use the player's current position from the save file.
    #[default]
    CurrentPosition,
    /// Use a named base's position.
    Base(String),
    /// Use an explicit galactic address.
    Address(GalacticAddress),
}

/// Parameters for a route query.
#[derive(Debug, Clone)]
pub struct RouteQuery {
    /// How targets are selected.
    pub targets: TargetSelection,
    /// Starting point for the route.
    pub from: RouteFrom,
    /// Ship warp range in light-years (for hop constraints and jump counts).
    pub warp_range: Option<f64>,
    /// Only consider targets within this radius in light-years.
    pub within_ly: Option<f64>,
    /// Maximum number of targets to visit.
    pub max_targets: Option<usize>,
    /// Which routing algorithm to use.
    pub algorithm: RoutingAlgorithm,
    /// Whether the route should return to the start.
    pub return_to_start: bool,
}

/// Result of a route query.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// The computed route.
    pub route: Route,
    /// Warp range used (if any).
    pub warp_range: Option<f64>,
    /// Total warp jumps needed at the given warp range.
    pub warp_jumps: Option<usize>,
    /// Which algorithm was used.
    pub algorithm: RoutingAlgorithm,
    /// Number of non-waypoint targets visited (excluding start).
    pub targets_visited: usize,
}

/// Resolve the starting address from a `RouteFrom` specification.
fn resolve_start(model: &GalaxyModel, from: &RouteFrom) -> Result<GalacticAddress, GraphError> {
    match from {
        RouteFrom::CurrentPosition => {
            let address = model
                .player_position()
                .copied()
                .ok_or(GraphError::NoPlayerPosition)?;
            if address.reality_index != model.active_galaxy {
                return Err(GraphError::ReferenceGalaxyMismatch {
                    reference: address.reality_index,
                    active: model.active_galaxy,
                });
            }
            Ok(address)
        }
        RouteFrom::Base(name) => model
            .base(name)
            .map(|b| b.address)
            .ok_or_else(|| GraphError::BaseNotFound(name.clone())),
        RouteFrom::Address(addr) => Ok(*addr),
    }
}

/// Resolve named targets (bases or systems) to `SystemId`s.
fn resolve_named_targets(
    model: &GalaxyModel,
    names: &[String],
    galaxy: u8,
    start: SystemId,
) -> Result<Vec<SystemId>, Box<dyn std::error::Error>> {
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        // Try as a base first
        if let Some(base) = model.base(name) {
            let id = SystemId::from_address(&base.address);
            if base.address.reality_index != galaxy {
                return Err(nms_graph::route::RouteError::NoPath {
                    from: start,
                    to: id,
                }
                .into());
            }
            ids.push(id);
            continue;
        }
        // Try as a system name
        if let Some((id, _)) = model.system_by_name_in_galaxy(name, galaxy) {
            ids.push(*id);
            continue;
        }
        return Err(GraphError::TargetNotFound(name.clone()).into());
    }
    Ok(ids)
}

/// Resolve biome filter targets to `SystemId`s (deduplicated by system).
fn resolve_biome_targets(
    model: &GalaxyModel,
    filter: &BiomeFilter,
    from: &GalacticAddress,
) -> Vec<SystemId> {
    model
        .nearest_planets_from_address(from, usize::MAX, filter)
        .into_iter()
        .map(|(key, _, _)| key.0)
        .fold(Vec::new(), |mut ids, id| {
            if !ids.contains(&id) {
                ids.push(id);
            }
            ids
        })
}

/// Execute a route query against the galaxy model.
///
/// Resolves targets, runs the selected routing algorithm, and optionally
/// applies hop constraints based on warp range.
///
/// # Errors
///
/// Returns graph or route errors for an unavailable start, empty start galaxy,
/// unresolved target, missing system, cross-galaxy target, or empty target set.
pub fn execute_route(
    model: &GalaxyModel,
    query: &RouteQuery,
) -> Result<RouteResult, Box<dyn std::error::Error>> {
    // 1. Resolve start position
    let start_addr = resolve_start(model, &query.from)?;

    // 2. Find nearest system in the start address's own galaxy.
    let nearest = model.nearest_systems_from_address(&start_addr, 1);
    let start_id = nearest
        .first()
        .map(|(id, _)| *id)
        .ok_or(GraphError::NoSystemsInGalaxy(start_addr.reality_index))?;

    // 3. Resolve targets to SystemIds
    let mut target_ids = match &query.targets {
        TargetSelection::Biome(filter) => resolve_biome_targets(model, filter, &start_addr),
        TargetSelection::Named(names) => {
            resolve_named_targets(model, names, start_addr.reality_index, start_id)?
        }
        TargetSelection::SystemIds(ids) => ids.clone(),
    };

    // Validate every explicit/resolved ID before filtering or applying limits.
    for id in &target_ids {
        if model.system(id).is_none() {
            return Err(nms_graph::route::RouteError::SystemNotFound(*id).into());
        }
    }
    for id in &target_ids {
        if id.reality_index() != start_addr.reality_index {
            return Err(nms_graph::route::RouteError::NoPath {
                from: start_id,
                to: *id,
            }
            .into());
        }
    }

    let mut seen_targets = HashSet::new();
    target_ids.retain(|id| seen_targets.insert(*id));

    if let Some(radius) = query.within_ly {
        target_ids.retain(|id| {
            model
                .system(id)
                .is_some_and(|system| start_addr.distance_ly(&system.address) <= radius)
        });
    }

    // Biome targets are ranked from the origin after complete planet matching
    // and system deduplication. Other selections retain caller order.
    if let TargetSelection::Biome(_) = &query.targets {
        target_ids.sort_by(|a, b| {
            let da = start_addr.distance_ly(&model.system(a).expect("validated target").address);
            let db = start_addr.distance_ly(&model.system(b).expect("validated target").address);
            da.total_cmp(&db).then_with(|| {
                a.reality_index()
                    .cmp(&b.reality_index())
                    .then_with(|| a.system_address().cmp(&b.system_address()))
            })
        });
    }

    // 4. Remove start from targets
    target_ids.retain(|id| *id != start_id);

    // 5. Apply max_targets limit
    if let Some(max) = query.max_targets {
        target_ids.truncate(max);
    }

    // 6. Run routing algorithm
    let route = match query.algorithm {
        RoutingAlgorithm::NearestNeighbor => {
            model.tsp_nearest_neighbor(start_id, &target_ids, query.return_to_start)?
        }
        RoutingAlgorithm::TwoOpt => {
            model.tsp_two_opt(start_id, &target_ids, query.return_to_start)?
        }
    };

    // 7. Apply hop constraints if warp_range specified
    let route = if let Some(warp_range) = query.warp_range {
        model.constrain_hops(&route, warp_range)?
    } else {
        route
    };

    // 8. Compute warp jump count
    let warp_jumps = query
        .warp_range
        .map(|wr| GalaxyModel::warp_jump_count(&route, wr));

    // Count non-waypoint targets (excluding start)
    let targets_visited = route
        .hops
        .iter()
        .skip(1)
        .filter(|hop| !hop.is_waypoint && hop.system_id != start_id)
        .count();

    Ok(RouteResult {
        route,
        warp_range: query.warp_range,
        warp_jumps,
        algorithm: query.algorithm,
        targets_visited,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::biome::Biome;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x003000001900", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x103000001900", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn test_execute_route_with_system_ids() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();
        assert!(ids.len() >= 3, "Need at least 3 systems for routing");

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: false,
        };

        let result = execute_route(&model, &query).unwrap();
        assert!(!result.route.hops.is_empty());
        assert!(result.targets_visited > 0);
    }

    #[test]
    fn test_execute_route_from_base() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::Base("Alpha Base".into()),
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        };

        let result = execute_route(&model, &query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_route_from_nonexistent_base_errors() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::Base("No Such Base".into()),
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: false,
        };

        assert!(execute_route(&model, &query).is_err());
    }

    #[test]
    fn test_execute_route_with_warp_range() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::CurrentPosition,
            warp_range: Some(2500.0),
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: false,
        };

        let result = execute_route(&model, &query).unwrap();
        assert!(result.warp_range.is_some());
        assert!(result.warp_jumps.is_some());
    }

    #[test]
    fn test_execute_route_round_trip() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: true,
        };

        let result = execute_route(&model, &query).unwrap();
        let first = result.route.hops.first().unwrap().system_id;
        let last = result.route.hops.last().unwrap().system_id;
        assert_eq!(first, last);
        assert_eq!(result.targets_visited, 2);
    }

    #[test]
    fn test_execute_route_with_biome_filter() {
        let model = test_model();

        let filter = BiomeFilter {
            biome: Some(Biome::Lush),
            ..Default::default()
        };

        let query = RouteQuery {
            targets: TargetSelection::Biome(filter),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: Some(5),
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        };

        // This may or may not find Lush planets depending on test data;
        // we just check it doesn't panic
        let _ = execute_route(&model, &query);
    }

    #[test]
    fn test_execute_route_max_targets_limit() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids.clone()),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: Some(1),
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: false,
        };

        let result = execute_route(&model, &query).unwrap();
        // Start + at most 1 target
        assert!(result.targets_visited <= 1);
    }

    #[test]
    fn test_execute_route_from_explicit_address() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let addr = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::Address(addr),
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            return_to_start: false,
        };

        assert!(execute_route(&model, &query).is_ok());
    }

    #[test]
    fn test_execute_route_nearest_neighbor_algorithm() {
        let model = test_model();
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();

        let query = RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        };

        let result = execute_route(&model, &query).unwrap();
        assert!(matches!(
            result.algorithm,
            RoutingAlgorithm::NearestNeighbor
        ));
    }

    #[test]
    fn test_resolve_start_current_position() {
        let model = test_model();
        let addr = resolve_start(&model, &RouteFrom::CurrentPosition);
        assert!(addr.is_ok());
    }

    #[test]
    fn test_resolve_start_base() {
        let model = test_model();
        let addr = resolve_start(&model, &RouteFrom::Base("Alpha Base".into()));
        assert!(addr.is_ok());
    }

    #[test]
    fn test_resolve_start_base_not_found() {
        let model = test_model();
        let addr = resolve_start(&model, &RouteFrom::Base("No Such Base".into()));
        assert!(addr.is_err());
    }

    #[test]
    fn test_resolve_start_explicit_address() {
        let model = test_model();
        let ga = GalacticAddress::new(42, 10, -5, 0x100, 0, 0);
        let addr = resolve_start(&model, &RouteFrom::Address(ga));
        assert!(addr.is_ok());
        assert_eq!(addr.unwrap(), ga);
    }

    #[test]
    fn test_execute_route_reports_typed_empty_foreign_and_missing_targets() {
        let mut model = test_model();
        let foreign_address = GalacticAddress::new(0, 0, 0, 7, 0, 9);
        let foreign = SystemId::from_address(&foreign_address);
        model.insert_system(nms_core::system::System::new(
            foreign_address,
            Some("Foreign".into()),
            None,
            None,
            vec![],
        ));
        let query_for = |ids, from| RouteQuery {
            targets: TargetSelection::SystemIds(ids),
            from,
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        };
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let foreign_error = execute_route(
            &model,
            &query_for(vec![foreign], RouteFrom::Address(origin)),
        )
        .unwrap_err();
        assert!(matches!(
            foreign_error.downcast_ref::<nms_graph::route::RouteError>(),
            Some(nms_graph::route::RouteError::NoPath { to, .. }) if *to == foreign
        ));

        let missing = SystemId::from_address(&GalacticAddress::new(50, 0, 0, 1, 0, 0));
        let missing_error = execute_route(
            &model,
            &query_for(vec![missing, foreign], RouteFrom::Address(origin)),
        )
        .unwrap_err();
        assert!(matches!(
            missing_error.downcast_ref::<nms_graph::route::RouteError>(),
            Some(nms_graph::route::RouteError::SystemNotFound(id)) if *id == missing
        ));

        let empty_error = execute_route(
            &model,
            &query_for(
                vec![],
                RouteFrom::Address(GalacticAddress::new(0, 0, 0, 0, 0, 42)),
            ),
        )
        .unwrap_err();
        assert!(matches!(
            empty_error.downcast_ref::<GraphError>(),
            Some(GraphError::NoSystemsInGalaxy(42))
        ));
        assert_eq!(model.active_galaxy, 0);
    }

    #[test]
    fn test_execute_route_current_position_requires_active_galaxy_match() {
        let mut model = test_model();
        model.set_active_galaxy(9);
        let result = execute_route(
            &model,
            &RouteQuery {
                targets: TargetSelection::SystemIds(vec![]),
                from: RouteFrom::CurrentPosition,
                warp_range: None,
                within_ly: None,
                max_targets: None,
                algorithm: RoutingAlgorithm::NearestNeighbor,
                return_to_start: false,
            },
        );
        assert!(matches!(
            result.unwrap_err().downcast_ref::<GraphError>(),
            Some(GraphError::ReferenceGalaxyMismatch {
                reference: 0,
                active: 9
            })
        ));
    }

    #[test]
    fn test_route_from_foreign_base_uses_its_galaxy_without_selection_mutation() {
        let mut model = GalaxyModel::new();
        let start = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let target = GalacticAddress::new(1, 0, 0, 1, 0, 9);
        model.insert_system(nms_core::system::System::new(
            start,
            None,
            None,
            None,
            vec![],
        ));
        model.insert_system(nms_core::system::System::new(
            target,
            None,
            None,
            None,
            vec![],
        ));
        model.insert_base(nms_core::player::PlayerBase::new(
            "Far Home".into(),
            nms_core::player::BaseType::HomePlanetBase,
            start,
            [0.0; 3],
            None,
        ));
        let target_id = SystemId::from_address(&target);
        let result = execute_route(
            &model,
            &RouteQuery {
                targets: TargetSelection::SystemIds(vec![target_id]),
                from: RouteFrom::Base("Far Home".into()),
                warp_range: None,
                within_ly: None,
                max_targets: None,
                algorithm: RoutingAlgorithm::TwoOpt,
                return_to_start: false,
            },
        )
        .unwrap();
        assert_eq!(result.route.hops[0].system_id.reality_index(), 9);
        assert_eq!(result.route.hops[1].system_id, target_id);
        assert_eq!(model.active_galaxy, 0);
    }

    #[test]
    fn test_foreign_base_name_wins_over_same_named_local_system() {
        let mut model = GalaxyModel::new();
        let start = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let local_target = GalacticAddress::new(1, 0, 0, 1, 0, 0);
        let foreign_base = GalacticAddress::new(1, 0, 0, 1, 0, 9);
        model.insert_system(nms_core::system::System::new(
            start,
            None,
            None,
            None,
            vec![],
        ));
        model.insert_system(nms_core::system::System::new(
            local_target,
            Some("Shadow".into()),
            None,
            None,
            vec![],
        ));
        model.insert_base(nms_core::player::PlayerBase::new(
            "Shadow".into(),
            nms_core::player::BaseType::HomePlanetBase,
            foreign_base,
            [0.0; 3],
            None,
        ));
        let error = execute_route(
            &model,
            &RouteQuery {
                targets: TargetSelection::Named(vec!["Shadow".into()]),
                from: RouteFrom::Address(start),
                warp_range: None,
                within_ly: None,
                max_targets: None,
                algorithm: RoutingAlgorithm::NearestNeighbor,
                return_to_start: false,
            },
        )
        .unwrap_err();
        let foreign_id = SystemId::from_address(&foreign_base);
        assert!(matches!(
            error.downcast_ref::<nms_graph::route::RouteError>(),
            Some(nms_graph::route::RouteError::NoPath { to, .. }) if *to == foreign_id
        ));
    }

    #[test]
    fn test_biome_route_deduplicates_all_planets_before_target_limit() {
        let mut model = GalaxyModel::new();
        let start = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        model.player_state = Some(nms_core::player::PlayerState::new(
            start, 0, None, None, 0, 0, 0,
        ));
        let many_planets = (0..220)
            .map(|index| {
                nms_core::system::Planet::new(index, Some(Biome::Lush), None, false, None, None)
            })
            .collect();
        model.insert_system(nms_core::system::System::new(
            start,
            Some("Start".into()),
            None,
            None,
            many_planets,
        ));
        let target = GalacticAddress::new(1, 0, 0, 1, 0, 0);
        model.insert_system(nms_core::system::System::new(
            target,
            Some("Target".into()),
            None,
            None,
            vec![nms_core::system::Planet::new(
                0,
                Some(Biome::Lush),
                None,
                false,
                None,
                None,
            )],
        ));

        let result = execute_route(
            &model,
            &RouteQuery {
                targets: TargetSelection::Biome(BiomeFilter {
                    biome: Some(Biome::Lush),
                    ..Default::default()
                }),
                from: RouteFrom::Address(start),
                warp_range: None,
                within_ly: None,
                max_targets: Some(1),
                algorithm: RoutingAlgorithm::NearestNeighbor,
                return_to_start: false,
            },
        )
        .unwrap();
        assert_eq!(result.targets_visited, 1);
        assert_eq!(
            result.route.hops[1].system_id,
            SystemId::from_address(&target)
        );
    }

    #[test]
    fn test_route_radius_filters_all_targets_before_target_limit() {
        let mut model = GalaxyModel::new();
        let start = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let near = GalacticAddress::new(1, 0, 0, 1, 0, 0);
        let far = GalacticAddress::new(10, 0, 0, 1, 0, 0);
        model.player_state = Some(nms_core::player::PlayerState::new(
            start, 0, None, None, 0, 0, 0,
        ));
        for address in [start, near, far] {
            model.insert_system(nms_core::system::System::new(
                address,
                None,
                None,
                None,
                vec![],
            ));
        }
        let near_id = SystemId::from_address(&near);
        let far_id = SystemId::from_address(&far);
        let query = RouteQuery {
            targets: TargetSelection::SystemIds(vec![far_id, near_id]),
            from: RouteFrom::Address(start),
            warp_range: None,
            within_ly: Some(400.0),
            max_targets: Some(1),
            algorithm: RoutingAlgorithm::NearestNeighbor,
            return_to_start: false,
        };
        let result = execute_route(&model, &query).unwrap();
        assert_eq!(result.route.hops[1].system_id, near_id);

        let missing = SystemId::from_address(&GalacticAddress::new(50, 0, 0, 1, 0, 0));
        let invalid_even_outside_radius = RouteQuery {
            targets: TargetSelection::SystemIds(vec![missing]),
            within_ly: Some(0.0),
            ..query
        };
        let error = execute_route(&model, &invalid_even_outside_radius).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<nms_graph::route::RouteError>(),
            Some(nms_graph::route::RouteError::SystemNotFound(id)) if *id == missing
        ));
    }
}
