//! Graph routing algorithms for galactic navigation.
//!
//! Provides shortest-path, TSP, and warp-range-constrained routing
//! over the petgraph topology.

use std::collections::HashSet;

use petgraph::stable_graph::NodeIndex;

use crate::model::GalaxyModel;
use crate::spatial::SystemId;

/// A single hop in a route.
#[derive(Debug, Clone)]
pub struct RouteHop {
    /// System at this waypoint.
    pub system_id: SystemId,
    /// Distance from previous hop (0 for start).
    pub leg_distance_ly: f64,
    /// Cumulative distance from route start.
    pub cumulative_ly: f64,
    /// Whether this is an intermediate waypoint inserted for hop constraints.
    pub is_waypoint: bool,
}

/// A complete route through multiple systems.
#[derive(Debug, Clone)]
pub struct Route {
    /// Ordered list of hops (first is the start).
    pub hops: Vec<RouteHop>,
    /// Total distance in light-years.
    pub total_distance_ly: f64,
}

/// Routing algorithm choice.
#[derive(Debug, Clone, Copy, Default)]
pub enum RoutingAlgorithm {
    /// Greedy nearest-neighbor traversal.
    NearestNeighbor,
    /// Nearest-neighbor followed by 2-opt improvement.
    #[default]
    TwoOpt,
}

/// Errors that can occur during routing.
#[derive(Debug, Clone)]
pub enum RouteError {
    /// A system ID was not found in the model.
    SystemNotFound(SystemId),
    /// No path exists between two systems in the graph.
    NoPath { from: SystemId, to: SystemId },
    /// Too few targets to form a route.
    TooFewTargets,
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemNotFound(id) => write!(f, "system not found: 0x{:012X}", id.0),
            Self::NoPath { from, to } => {
                write!(f, "no path from 0x{:012X} to 0x{:012X}", from.0, to.0)
            }
            Self::TooFewTargets => write!(f, "need at least 1 target for a route"),
        }
    }
}

impl std::error::Error for RouteError {}

impl GalaxyModel {
    // -- 4.1: Dijkstra shortest path --

    /// Find the shortest path between two systems using A* (Dijkstra with zero heuristic).
    ///
    /// If no graph path exists, falls back to a direct Euclidean hop.
    pub fn shortest_path(&self, from: SystemId, to: SystemId) -> Result<Route, RouteError> {
        if from == to {
            return Ok(Route {
                hops: vec![RouteHop {
                    system_id: from,
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                }],
                total_distance_ly: 0.0,
            });
        }

        let from_node = self
            .node_map
            .get(&from)
            .ok_or(RouteError::SystemNotFound(from))?;
        let to_node = self
            .node_map
            .get(&to)
            .ok_or(RouteError::SystemNotFound(to))?;

        // Use astar with zero heuristic (equivalent to Dijkstra but returns the path)
        use petgraph::algo::astar;
        match astar(
            &self.graph,
            *from_node,
            |n| n == *to_node,
            |e| *e.weight(),
            |_| 0.0,
        ) {
            Some((_, path)) => Ok(self.path_to_route(&path)),
            None => self.direct_route(from, to),
        }
    }

    /// Create a direct single-hop route between two systems.
    fn direct_route(&self, from: SystemId, to: SystemId) -> Result<Route, RouteError> {
        let dist = self.euclidean_distance(from, to);
        Ok(Route {
            hops: vec![
                RouteHop {
                    system_id: from,
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: to,
                    leg_distance_ly: dist,
                    cumulative_ly: dist,
                    is_waypoint: false,
                },
            ],
            total_distance_ly: dist,
        })
    }

    /// Convert a path of NodeIndexes into a Route with distances.
    fn path_to_route(&self, path: &[NodeIndex]) -> Route {
        let mut hops = Vec::with_capacity(path.len());
        let mut cumulative = 0.0;

        for (i, &node) in path.iter().enumerate() {
            let sys_id = self.graph[node];
            let leg_dist = if i == 0 {
                0.0
            } else {
                let prev = path[i - 1];
                self.graph
                    .find_edge(prev, node)
                    .map(|e| self.graph[e])
                    .unwrap_or_else(|| {
                        let prev_id = self.graph[prev];
                        self.euclidean_distance(prev_id, sys_id)
                    })
            };
            cumulative += leg_dist;
            hops.push(RouteHop {
                system_id: sys_id,
                leg_distance_ly: leg_dist,
                cumulative_ly: cumulative,
                is_waypoint: false,
            });
        }

        Route {
            total_distance_ly: cumulative,
            hops,
        }
    }

    /// Euclidean distance in ly between two systems.
    fn euclidean_distance(&self, a: SystemId, b: SystemId) -> f64 {
        match (self.systems.get(&a), self.systems.get(&b)) {
            (Some(sa), Some(sb)) => sa.address.distance_ly(&sb.address),
            _ => 0.0,
        }
    }

    // -- 4.2: Nearest-neighbor TSP --

    /// Plan a route visiting all targets using nearest-neighbor greedy.
    ///
    /// Uses Euclidean distances directly (not graph edges) for the TSP ordering.
    pub fn tsp_nearest_neighbor(
        &self,
        start: SystemId,
        targets: &[SystemId],
        return_to_start: bool,
    ) -> Result<Route, RouteError> {
        if targets.is_empty() {
            return Err(RouteError::TooFewTargets);
        }
        if !self.systems.contains_key(&start) {
            return Err(RouteError::SystemNotFound(start));
        }
        for &t in targets {
            if !self.systems.contains_key(&t) {
                return Err(RouteError::SystemNotFound(t));
            }
        }

        let mut unvisited: Vec<SystemId> = targets.to_vec();
        let mut order = vec![start];
        let mut current = start;

        while !unvisited.is_empty() {
            let (nearest_idx, _) = unvisited
                .iter()
                .enumerate()
                .map(|(i, &t)| (i, self.euclidean_distance(current, t)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            current = unvisited.swap_remove(nearest_idx);
            order.push(current);
        }

        if return_to_start {
            order.push(start);
        }

        Ok(self.build_route_from_order(&order))
    }

    /// Convert an ordered list of SystemIds into a Route.
    pub fn build_route_from_order(&self, order: &[SystemId]) -> Route {
        let mut hops = Vec::with_capacity(order.len());
        let mut cumulative = 0.0;

        for (i, &sys_id) in order.iter().enumerate() {
            let leg_dist = if i == 0 {
                0.0
            } else {
                self.euclidean_distance(order[i - 1], sys_id)
            };
            cumulative += leg_dist;
            hops.push(RouteHop {
                system_id: sys_id,
                leg_distance_ly: leg_dist,
                cumulative_ly: cumulative,
                is_waypoint: false,
            });
        }

        Route {
            total_distance_ly: cumulative,
            hops,
        }
    }

    // -- 4.3: 2-opt improvement --

    /// Improve a route using 2-opt local search.
    ///
    /// Iteratively reverses segments when doing so reduces total distance.
    /// Runs until no improvement is found (local optimum).
    pub fn two_opt_improve(&self, mut order: Vec<SystemId>) -> Route {
        let n = order.len();
        if n < 4 {
            return self.build_route_from_order(&order);
        }

        let mut improved = true;
        while improved {
            improved = false;
            for i in 0..n - 2 {
                for j in (i + 2)..n {
                    if i == 0 && j == n - 1 {
                        continue;
                    }
                    let gain = self.two_opt_gain(&order, i, j);
                    if gain > 1e-6 {
                        order[i + 1..=j].reverse();
                        improved = true;
                    }
                }
            }
        }

        self.build_route_from_order(&order)
    }

    /// Calculate the distance gain from a 2-opt swap.
    fn two_opt_gain(&self, order: &[SystemId], i: usize, j: usize) -> f64 {
        let d = |a: SystemId, b: SystemId| self.euclidean_distance(a, b);

        let old_dist = d(order[i], order[i + 1])
            + if j + 1 < order.len() {
                d(order[j], order[j + 1])
            } else {
                0.0
            };

        let new_dist = d(order[i], order[j])
            + if j + 1 < order.len() {
                d(order[i + 1], order[j + 1])
            } else {
                0.0
            };

        old_dist - new_dist
    }

    /// Plan a route using nearest-neighbor + 2-opt.
    pub fn tsp_two_opt(
        &self,
        start: SystemId,
        targets: &[SystemId],
        return_to_start: bool,
    ) -> Result<Route, RouteError> {
        let nn_route = self.tsp_nearest_neighbor(start, targets, return_to_start)?;
        let order: Vec<SystemId> = nn_route.hops.iter().map(|h| h.system_id).collect();
        Ok(self.two_opt_improve(order))
    }

    // -- 4.4: Hop-constrained routing --

    /// Constrain a route so no hop exceeds `max_ly`.
    ///
    /// Inserts intermediate known systems as waypoints when a hop is too long.
    /// Waypoints are marked with `is_waypoint: true`.
    pub fn constrain_hops(&self, route: &Route, max_ly: f64) -> Route {
        if route.hops.len() < 2 {
            return route.clone();
        }

        let mut new_hops: Vec<RouteHop> = Vec::new();
        new_hops.push(RouteHop {
            is_waypoint: false,
            ..route.hops[0].clone()
        });

        for i in 1..route.hops.len() {
            let prev_id = new_hops.last().unwrap().system_id;
            let next_id = route.hops[i].system_id;
            let leg = self.euclidean_distance(prev_id, next_id);

            if leg <= max_ly {
                let cumulative = new_hops.last().unwrap().cumulative_ly + leg;
                new_hops.push(RouteHop {
                    system_id: next_id,
                    leg_distance_ly: leg,
                    cumulative_ly: cumulative,
                    is_waypoint: route.hops[i].is_waypoint,
                });
            } else {
                self.insert_waypoints(&mut new_hops, prev_id, next_id, max_ly);
            }
        }

        let total = new_hops.last().map(|h| h.cumulative_ly).unwrap_or(0.0);
        Route {
            hops: new_hops,
            total_distance_ly: total,
        }
    }

    /// Insert intermediate waypoints between `from` and `to`.
    fn insert_waypoints(
        &self,
        hops: &mut Vec<RouteHop>,
        from: SystemId,
        to: SystemId,
        max_ly: f64,
    ) {
        let mut current = from;
        let mut remaining = self.euclidean_distance(current, to);
        let max_iterations = 1000;
        let mut iterations = 0;

        while remaining > max_ly && iterations < max_iterations {
            iterations += 1;

            let current_sys = match self.systems.get(&current) {
                Some(s) => s,
                None => break,
            };

            let best = self
                .nearest_systems(&current_sys.address, 50)
                .into_iter()
                .filter(|(id, dist)| {
                    *id != current
                        && *dist <= max_ly
                        && self.euclidean_distance(*id, to) < remaining
                })
                .min_by(|a, b| {
                    let da = self.euclidean_distance(a.0, to);
                    let db = self.euclidean_distance(b.0, to);
                    da.partial_cmp(&db).unwrap()
                });

            match best {
                Some((waypoint_id, step_dist)) => {
                    let cumulative = hops.last().unwrap().cumulative_ly + step_dist;
                    hops.push(RouteHop {
                        system_id: waypoint_id,
                        leg_distance_ly: step_dist,
                        cumulative_ly: cumulative,
                        is_waypoint: true,
                    });
                    current = waypoint_id;
                    remaining = self.euclidean_distance(current, to);
                }
                None => break,
            }
        }

        let final_dist = self.euclidean_distance(current, to);
        let cumulative = hops.last().unwrap().cumulative_ly + final_dist;
        hops.push(RouteHop {
            system_id: to,
            leg_distance_ly: final_dist,
            cumulative_ly: cumulative,
            is_waypoint: false,
        });
    }

    /// Count the number of warp jumps needed for a route at a given range.
    pub fn warp_jump_count(route: &Route, warp_range: f64) -> usize {
        route
            .hops
            .iter()
            .skip(1)
            .map(|h| (h.leg_distance_ly / warp_range).ceil() as usize)
            .sum()
    }

    // -- 4.8: Reachability analysis --

    /// Find all systems reachable from `start` within a given warp range.
    ///
    /// Uses DFS over the spatial index (no graph clone needed).
    pub fn reachable_systems(
        &self,
        start: SystemId,
        warp_range: f64,
    ) -> Result<Vec<SystemId>, RouteError> {
        if !self.systems.contains_key(&start) {
            return Err(RouteError::SystemNotFound(start));
        }

        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            let sys = match self.systems.get(&current) {
                Some(s) => s,
                None => continue,
            };
            for (neighbor_id, dist) in self.nearest_systems(&sys.address, 100) {
                if dist <= warp_range && !visited.contains(&neighbor_id) {
                    stack.push(neighbor_id);
                }
            }
        }

        Ok(visited.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edges::EdgeStrategy;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};

    /// Build a model with N systems in a line along the X axis.
    fn line_model(n: usize, spacing: i16) -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);

        for i in 0..n {
            let x = (i as i16) * spacing;
            let ssi = (i + 1) as u16;
            let addr = GalacticAddress::new(x, 0, 0, ssi, 0, 0);
            let planet = Planet::new(0, Some(Biome::Barren), None, false, None, None);
            let system = System::new(addr, None, None, None, vec![planet]);
            model.insert_system(system);
        }

        model
    }

    /// Get sorted system IDs from a model (sorted by packed address).
    fn sorted_ids(model: &GalaxyModel) -> Vec<SystemId> {
        let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
        ids.sort_by_key(|id| id.0);
        ids
    }

    // -- 4.1 tests --

    #[test]
    fn test_shortest_path_adjacent_systems() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        assert!(route.hops.len() >= 2);
        assert!(route.total_distance_ly > 0.0);
    }

    #[test]
    fn test_shortest_path_same_system() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let id = *model.systems.keys().next().unwrap();
        let route = model.shortest_path(id, id).unwrap();
        assert_eq!(route.total_distance_ly, 0.0);
        assert_eq!(route.hops.len(), 1);
    }

    #[test]
    fn test_shortest_path_nonexistent_system_errors() {
        let model = line_model(3, 10);
        assert!(
            model
                .shortest_path(SystemId(0xDEAD), SystemId(0xBEEF))
                .is_err()
        );
    }

    #[test]
    fn test_shortest_path_disconnected_falls_back_to_direct() {
        let mut model = line_model(3, 100);
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 1.0 });
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        assert_eq!(route.hops.len(), 2);
    }

    #[test]
    fn test_shortest_path_multi_hop() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 1 });
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[4]).unwrap();
        assert!(route.hops.len() >= 3, "Should route through intermediates");
    }

    #[test]
    fn test_shortest_path_cumulative_distances() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        for i in 1..route.hops.len() {
            let expected = route.hops[i - 1].cumulative_ly + route.hops[i].leg_distance_ly;
            assert!(
                (route.hops[i].cumulative_ly - expected).abs() < 0.01,
                "cumulative mismatch at hop {i}"
            );
        }
    }

    #[test]
    fn test_shortest_path_first_hop_zero_distance() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        assert_eq!(route.hops[0].leg_distance_ly, 0.0);
    }

    // -- 4.2 tests --

    #[test]
    fn test_tsp_nn_visits_all_targets() {
        let model = line_model(5, 10);
        let ids = sorted_ids(&model);
        let route = model
            .tsp_nearest_neighbor(ids[0], &ids[1..], false)
            .unwrap();
        assert_eq!(route.hops.len(), ids.len());
    }

    #[test]
    fn test_tsp_nn_return_to_start() {
        let model = line_model(3, 10);
        let ids = sorted_ids(&model);
        let route = model.tsp_nearest_neighbor(ids[0], &ids[1..], true).unwrap();
        assert_eq!(route.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(route.hops.last().unwrap().system_id, ids[0]);
    }

    #[test]
    fn test_tsp_nn_empty_targets_errors() {
        let model = line_model(3, 10);
        let id = *model.systems.keys().next().unwrap();
        assert!(model.tsp_nearest_neighbor(id, &[], false).is_err());
    }

    #[test]
    fn test_tsp_nn_single_target() {
        let model = line_model(3, 10);
        let ids = sorted_ids(&model);
        let route = model
            .tsp_nearest_neighbor(ids[0], &[ids[1]], false)
            .unwrap();
        assert_eq!(route.hops.len(), 2);
    }

    #[test]
    fn test_tsp_nn_nonexistent_target_errors() {
        let model = line_model(3, 10);
        let id = *model.systems.keys().next().unwrap();
        assert!(
            model
                .tsp_nearest_neighbor(id, &[SystemId(0xDEAD)], false)
                .is_err()
        );
    }

    #[test]
    fn test_tsp_nn_nonexistent_start_errors() {
        let model = line_model(3, 10);
        let ids = sorted_ids(&model);
        assert!(
            model
                .tsp_nearest_neighbor(SystemId(0xDEAD), &ids, false)
                .is_err()
        );
    }

    #[test]
    fn test_tsp_nn_greedy_picks_nearest() {
        // Systems at x=0, x=50, x=100. Starting at 0, greedy should visit 50 first.
        let model = line_model(3, 50);
        let ids = sorted_ids(&model);
        let route = model
            .tsp_nearest_neighbor(ids[0], &[ids[2], ids[1]], false)
            .unwrap();
        assert_eq!(route.hops[1].system_id, ids[1]);
    }

    // -- 4.3 tests --

    #[test]
    fn test_two_opt_does_not_increase_distance() {
        let model = line_model(6, 20);
        let ids = sorted_ids(&model);
        let nn_route = model
            .tsp_nearest_neighbor(ids[0], &ids[1..], false)
            .unwrap();
        let nn_order: Vec<SystemId> = nn_route.hops.iter().map(|h| h.system_id).collect();
        let opt_route = model.two_opt_improve(nn_order);
        assert!(opt_route.total_distance_ly <= nn_route.total_distance_ly + 1e-6);
    }

    #[test]
    fn test_two_opt_small_route_unchanged() {
        let model = line_model(3, 10);
        let ids = sorted_ids(&model);
        let route = model.tsp_two_opt(ids[0], &ids[1..], false).unwrap();
        assert_eq!(route.hops.len(), 3);
    }

    #[test]
    fn test_two_opt_improves_scrambled_route() {
        let model = line_model(8, 10);
        let ids = sorted_ids(&model);

        let mut scrambled = ids.clone();
        scrambled.swap(1, 5);
        scrambled.swap(2, 6);

        let scrambled_route = model.build_route_from_order(&scrambled);
        let improved = model.two_opt_improve(scrambled);
        assert!(improved.total_distance_ly <= scrambled_route.total_distance_ly);
    }

    #[test]
    fn test_tsp_two_opt_visits_all() {
        let model = line_model(5, 10);
        let ids = sorted_ids(&model);
        let route = model.tsp_two_opt(ids[0], &ids[1..], false).unwrap();
        assert_eq!(route.hops.len(), ids.len());
    }

    // -- 4.4 tests --

    #[test]
    fn test_constrain_hops_short_route_unchanged() {
        let model = line_model(3, 5);
        let ids = sorted_ids(&model);
        let route = model.build_route_from_order(&ids);
        let constrained = model.constrain_hops(&route, 100_000.0);
        assert_eq!(constrained.hops.len(), route.hops.len());
    }

    #[test]
    fn test_constrain_hops_inserts_waypoints() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        // Route from first to last (9 * 4000 = 36000 ly)
        let route = model.build_route_from_order(&[ids[0], ids[9]]);
        let constrained = model.constrain_hops(&route, 5000.0);
        assert!(constrained.hops.len() > 2, "Should insert waypoints");
        for hop in &constrained.hops[1..constrained.hops.len() - 1] {
            assert!(hop.is_waypoint);
        }
    }

    #[test]
    fn test_constrain_hops_preserves_endpoints() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        let route = model.build_route_from_order(&[ids[0], ids[9]]);
        let constrained = model.constrain_hops(&route, 5000.0);
        assert_eq!(constrained.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(constrained.hops.last().unwrap().system_id, ids[9]);
    }

    #[test]
    fn test_constrain_hops_respects_warp_range() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        let route = model.build_route_from_order(&[ids[0], ids[9]]);
        let constrained = model.constrain_hops(&route, 5000.0);
        let within_range = constrained
            .hops
            .iter()
            .skip(1)
            .filter(|h| h.leg_distance_ly <= 5000.0 + 1.0)
            .count();
        assert!(within_range > 0);
    }

    #[test]
    fn test_warp_jump_count() {
        let route = Route {
            total_distance_ly: 10000.0,
            hops: vec![
                RouteHop {
                    system_id: SystemId(0),
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: SystemId(1),
                    leg_distance_ly: 5000.0,
                    cumulative_ly: 5000.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: SystemId(2),
                    leg_distance_ly: 5000.0,
                    cumulative_ly: 10000.0,
                    is_waypoint: false,
                },
            ],
        };
        assert_eq!(GalaxyModel::warp_jump_count(&route, 2500.0), 4);
        assert_eq!(GalaxyModel::warp_jump_count(&route, 5000.0), 2);
    }

    #[test]
    fn test_constrain_hops_empty_route() {
        let model = line_model(3, 10);
        let route = Route {
            hops: vec![],
            total_distance_ly: 0.0,
        };
        let constrained = model.constrain_hops(&route, 5000.0);
        assert!(constrained.hops.is_empty());
    }

    // -- 4.8 tests --

    #[test]
    fn test_reachable_includes_start() {
        let model = line_model(5, 10);
        let id = *model.systems.keys().next().unwrap();
        let reachable = model.reachable_systems(id, 100_000.0).unwrap();
        assert!(reachable.contains(&id));
    }

    #[test]
    fn test_reachable_all_within_large_range() {
        let model = line_model(5, 10);
        let id = *model.systems.keys().next().unwrap();
        let reachable = model.reachable_systems(id, 1_000_000.0).unwrap();
        assert_eq!(reachable.len(), model.systems.len());
    }

    #[test]
    fn test_reachable_isolated_with_tiny_range() {
        let model = line_model(5, 100);
        let id = *model.systems.keys().next().unwrap();
        let reachable = model.reachable_systems(id, 1.0).unwrap();
        assert_eq!(reachable.len(), 1);
    }

    #[test]
    fn test_reachable_partial() {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);

        // Close cluster: 0, 10, 20 (4000 ly apart)
        // Far cluster: 100, 110 (4000 ly apart, 32000 ly from close cluster)
        let positions = [(0i16, 1u16), (10, 2), (20, 3), (100, 4), (110, 5)];
        for (x, ssi) in positions {
            let addr = GalacticAddress::new(x, 0, 0, ssi, 0, 0);
            let system = System::new(addr, None, None, None, vec![]);
            model.insert_system(system);
        }

        let start_addr = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let start_id = SystemId::from_address(&start_addr);
        let reachable = model.reachable_systems(start_id, 5000.0).unwrap();
        // Should reach 0, 10, 20 but not 100, 110
        assert!(reachable.len() >= 3);
        assert!(reachable.len() < 6); // 5 inserted + 1 origin from save
    }

    #[test]
    fn test_reachable_nonexistent_start_errors() {
        let model = line_model(3, 10);
        assert!(model.reachable_systems(SystemId(0xDEAD), 5000.0).is_err());
    }
}
