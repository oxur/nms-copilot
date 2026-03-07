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
        RouteFrom::CurrentPosition => model
            .player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition),
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
) -> Result<Vec<SystemId>, Box<dyn std::error::Error>> {
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        // Try as a base first
        if let Some(base) = model.base(name) {
            ids.push(SystemId::from_address(&base.address));
            continue;
        }
        // Try as a system name
        if let Some((id, _)) = model.system_by_name(name) {
            ids.push(*id);
            continue;
        }
        return Err(format!("Target not found: \"{name}\"").into());
    }
    Ok(ids)
}

/// Resolve biome filter targets to `SystemId`s (deduplicated by system).
fn resolve_biome_targets(
    model: &GalaxyModel,
    filter: &BiomeFilter,
    from: &GalacticAddress,
    within_ly: Option<f64>,
    max_targets: Option<usize>,
) -> Vec<SystemId> {
    let limit = max_targets.unwrap_or(100) * 2; // over-fetch for dedup

    let planet_matches = if let Some(radius) = within_ly {
        model.planets_within_radius(from, radius, filter)
    } else {
        model.nearest_planets(from, limit, filter)
    };

    // Deduplicate by system
    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for (key, _, _) in &planet_matches {
        if seen.insert(key.0) {
            ids.push(key.0);
        }
    }

    ids
}

/// Execute a route query against the galaxy model.
///
/// Resolves targets, runs the selected routing algorithm, and optionally
/// applies hop constraints based on warp range.
pub fn execute_route(
    model: &GalaxyModel,
    query: &RouteQuery,
) -> Result<RouteResult, Box<dyn std::error::Error>> {
    // 1. Resolve start position
    let start_addr = resolve_start(model, &query.from)?;

    // 2. Find nearest system to start address
    let nearest = model.nearest_systems(&start_addr, 1);
    let start_id = nearest
        .first()
        .map(|(id, _)| *id)
        .ok_or("No systems in model")?;

    // 3. Resolve targets to SystemIds
    let mut target_ids = match &query.targets {
        TargetSelection::Biome(filter) => resolve_biome_targets(
            model,
            filter,
            &start_addr,
            query.within_ly,
            query.max_targets,
        ),
        TargetSelection::Named(names) => resolve_named_targets(model, names)?,
        TargetSelection::SystemIds(ids) => ids.clone(),
    };

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
        model.constrain_hops(&route, warp_range)
    } else {
        route
    };

    // 8. Compute warp jump count
    let warp_jumps = query
        .warp_range
        .map(|wr| GalaxyModel::warp_jump_count(&route, wr));

    // Count non-waypoint targets (excluding start)
    let targets_visited = route.hops.iter().skip(1).filter(|h| !h.is_waypoint).count();

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
}
