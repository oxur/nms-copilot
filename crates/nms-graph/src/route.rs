//! Graph routing algorithms for galactic navigation.
//!
//! Provides shortest-path, TSP, and warp-range-constrained routing
//! over the petgraph topology.

use std::collections::HashSet;

use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;

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
    /// No same-galaxy path exists, or the requested systems are in different galaxies.
    NoPath { from: SystemId, to: SystemId },
    /// Too few targets to form a route.
    TooFewTargets,
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemNotFound(id) => write!(f, "system not found: {id}"),
            Self::NoPath { from, to } => {
                write!(f, "no path from {from} to {to}")
            }
            Self::TooFewTargets => write!(f, "need at least 1 target for a route"),
        }
    }
}

impl std::error::Error for RouteError {}

impl GalaxyModel {
    // -- 4.1: Dijkstra shortest path --

    /// Find a same-galaxy route between two systems using A* with a zero heuristic.
    ///
    /// Disconnected systems retain the direct Euclidean fallback, which does
    /// not assert that the leg is warp-feasible.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::SystemNotFound`] when either ID is absent and
    /// [`RouteError::NoPath`] when the IDs belong to different galaxies.
    pub fn shortest_path(&self, from: SystemId, to: SystemId) -> Result<Route, RouteError> {
        if !self.systems.contains_key(&from) {
            return Err(RouteError::SystemNotFound(from));
        }
        if !self.systems.contains_key(&to) {
            return Err(RouteError::SystemNotFound(to));
        }
        let from_node = *self
            .node_map
            .get(&from)
            .ok_or(RouteError::SystemNotFound(from))?;
        let to_node = *self
            .node_map
            .get(&to)
            .ok_or(RouteError::SystemNotFound(to))?;
        self.validate_galaxy_pair(from, to)?;

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

        // The borrowed view excludes accidental cross-galaxy shortcuts without
        // mutating or cloning the shared topology.
        let filtered = petgraph::visit::EdgeFiltered::from_fn(&self.graph, |edge| {
            let source = self.graph[edge.source()].reality_index();
            let target = self.graph[edge.target()].reality_index();
            source == from.reality_index() && target == from.reality_index()
        });
        use petgraph::algo::astar;
        match astar(
            &filtered,
            from_node,
            |n| n == to_node,
            |e| *e.weight(),
            |_| 0.0,
        ) {
            Some((_, path)) => {
                for &node in &path {
                    let id = self.graph[node];
                    if !self.systems.contains_key(&id) {
                        return Err(RouteError::SystemNotFound(id));
                    }
                    self.validate_galaxy_pair(from, id)?;
                }
                self.path_to_route(&path)
            }
            None => self.direct_route(from, to),
        }
    }

    /// Create a direct single-hop route between two systems.
    fn direct_route(&self, from: SystemId, to: SystemId) -> Result<Route, RouteError> {
        let dist = self.euclidean_distance(from, to)?;
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
    fn path_to_route(&self, path: &[NodeIndex]) -> Result<Route, RouteError> {
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
                    .map(|e| Ok(self.graph[e]))
                    .unwrap_or_else(|| {
                        let prev_id = self.graph[prev];
                        self.euclidean_distance(prev_id, sys_id)
                    })?
            };
            cumulative += leg_dist;
            hops.push(RouteHop {
                system_id: sys_id,
                leg_distance_ly: leg_dist,
                cumulative_ly: cumulative,
                is_waypoint: false,
            });
        }

        Ok(Route {
            total_distance_ly: cumulative,
            hops,
        })
    }

    /// Euclidean distance in ly between two systems.
    fn euclidean_distance(&self, a: SystemId, b: SystemId) -> Result<f64, RouteError> {
        match (self.systems.get(&a), self.systems.get(&b)) {
            (Some(sa), Some(sb)) => Ok(sa.address.distance_ly(&sb.address)),
            (None, _) => Err(RouteError::SystemNotFound(a)),
            (_, None) => Err(RouteError::SystemNotFound(b)),
        }
    }

    fn validate_galaxy_pair(&self, from: SystemId, to: SystemId) -> Result<(), RouteError> {
        if from.reality_index() != to.reality_index() {
            return Err(RouteError::NoPath { from, to });
        }
        Ok(())
    }

    fn validate_order(&self, order: &[SystemId]) -> Result<(), RouteError> {
        for &id in order {
            if !self.systems.contains_key(&id) || !self.node_map.contains_key(&id) {
                return Err(RouteError::SystemNotFound(id));
            }
        }
        if let Some(&first) = order.first() {
            for &id in &order[1..] {
                self.validate_galaxy_pair(first, id)?;
            }
        }
        Ok(())
    }

    // -- 4.2: Nearest-neighbor TSP --

    /// Plan a route visiting all targets using nearest-neighbor greedy.
    ///
    /// Uses Euclidean distances directly (not graph edges) for the TSP ordering.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::TooFewTargets`] for an empty target list,
    /// [`RouteError::SystemNotFound`] for absent IDs, and [`RouteError::NoPath`]
    /// when a target is in another galaxy.
    pub fn tsp_nearest_neighbor(
        &self,
        start: SystemId,
        targets: &[SystemId],
        return_to_start: bool,
    ) -> Result<Route, RouteError> {
        if targets.is_empty() {
            return Err(RouteError::TooFewTargets);
        }
        let order: Vec<_> = std::iter::once(start)
            .chain(targets.iter().copied())
            .collect();
        self.validate_order(&order)?;

        let mut unvisited: Vec<SystemId> = targets.to_vec();
        let mut order = vec![start];
        let mut current = start;

        while !unvisited.is_empty() {
            let mut nearest: Option<(usize, f64)> = None;
            for (index, &target) in unvisited.iter().enumerate() {
                let distance = self.euclidean_distance(current, target)?;
                if nearest.is_none_or(|(best_index, best_distance)| {
                    distance.total_cmp(&best_distance).is_lt()
                        || (distance.total_cmp(&best_distance).is_eq()
                            && crate::query::system_id_order(target, unvisited[best_index]).is_lt())
                }) {
                    nearest = Some((index, distance));
                }
            }
            let (nearest_idx, _) = nearest.expect("unvisited is nonempty");

            current = unvisited.swap_remove(nearest_idx);
            order.push(current);
        }

        if return_to_start {
            order.push(start);
        }

        self.build_route_from_order(&order)
    }

    /// Convert a validated ordered list of system IDs into a route.
    ///
    /// Empty input produces an empty zero-distance route.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::SystemNotFound`] for an absent system and
    /// [`RouteError::NoPath`] if IDs span galaxies.
    pub fn build_route_from_order(&self, order: &[SystemId]) -> Result<Route, RouteError> {
        self.validate_order(order)?;
        let mut hops = Vec::with_capacity(order.len());
        let mut cumulative = 0.0;

        for (i, &sys_id) in order.iter().enumerate() {
            let leg_dist = if i == 0 {
                0.0
            } else {
                self.euclidean_distance(order[i - 1], sys_id)?
            };
            cumulative += leg_dist;
            hops.push(RouteHop {
                system_id: sys_id,
                leg_distance_ly: leg_dist,
                cumulative_ly: cumulative,
                is_waypoint: false,
            });
        }

        Ok(Route {
            total_distance_ly: cumulative,
            hops,
        })
    }

    // -- 4.3: 2-opt improvement --

    /// Improve a route using 2-opt local search.
    ///
    /// Iteratively reverses segments when doing so reduces total distance.
    /// Runs until no improvement is found (local optimum).
    ///
    /// # Errors
    ///
    /// Returns a typed error if any system is missing or the order spans galaxies.
    pub fn two_opt_improve(&self, mut order: Vec<SystemId>) -> Result<Route, RouteError> {
        self.validate_order(&order)?;
        let n = order.len();
        if n < 4 {
            return self.build_route_from_order(&order);
        }
        let closed = order.first() == order.last();
        let movable_end = if closed { n - 1 } else { n };

        let mut improved = true;
        while improved {
            improved = false;
            for i in 0..movable_end.saturating_sub(2) {
                for j in (i + 2)..movable_end {
                    if closed && i == 0 && j == movable_end - 1 {
                        continue;
                    }
                    let gain = self.two_opt_gain(&order, i, j)?;
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
    fn two_opt_gain(&self, order: &[SystemId], i: usize, j: usize) -> Result<f64, RouteError> {
        let d = |a: SystemId, b: SystemId| self.euclidean_distance(a, b);

        let old_dist = d(order[i], order[i + 1])?
            + if j + 1 < order.len() {
                d(order[j], order[j + 1])?
            } else {
                0.0
            };

        let new_dist = d(order[i], order[j])?
            + if j + 1 < order.len() {
                d(order[i + 1], order[j + 1])?
            } else {
                0.0
            };

        Ok(old_dist - new_dist)
    }

    /// Plan a route using nearest-neighbor + 2-opt.
    ///
    /// # Errors
    ///
    /// Returns the validation errors produced by
    /// [`tsp_nearest_neighbor`](Self::tsp_nearest_neighbor) or
    /// [`two_opt_improve`](Self::two_opt_improve).
    pub fn tsp_two_opt(
        &self,
        start: SystemId,
        targets: &[SystemId],
        return_to_start: bool,
    ) -> Result<Route, RouteError> {
        let nn_route = self.tsp_nearest_neighbor(start, targets, return_to_start)?;
        let order: Vec<SystemId> = nn_route.hops.iter().map(|h| h.system_id).collect();
        self.two_opt_improve(order)
    }

    // -- 4.4: Hop-constrained routing --

    /// Add best-effort waypoints for legs longer than `max_ly`.
    ///
    /// Inserts intermediate known systems as waypoints when a hop is too long.
    /// Waypoints are marked with `is_waypoint: true`.
    /// Waypoints are selected from the complete radius neighborhood and must
    /// move closer to the destination. If progress is unavailable or the
    /// finite search guard is reached, the final direct leg is retained and may
    /// exceed `max_ly`; this method does not guarantee warp feasibility.
    ///
    /// # Errors
    ///
    /// Returns a typed error if a route system is missing or the route spans galaxies.
    pub fn constrain_hops(&self, route: &Route, max_ly: f64) -> Result<Route, RouteError> {
        let order: Vec<_> = route.hops.iter().map(|hop| hop.system_id).collect();
        self.validate_order(&order)?;
        if route.hops.len() < 2 {
            return Ok(Route {
                hops: route
                    .hops
                    .first()
                    .map(|hop| RouteHop {
                        system_id: hop.system_id,
                        leg_distance_ly: 0.0,
                        cumulative_ly: 0.0,
                        is_waypoint: hop.is_waypoint,
                    })
                    .into_iter()
                    .collect(),
                total_distance_ly: 0.0,
            });
        }

        let mut new_hops = Vec::with_capacity(route.hops.len());
        new_hops.push(RouteHop {
            system_id: route.hops[0].system_id,
            leg_distance_ly: 0.0,
            cumulative_ly: 0.0,
            is_waypoint: route.hops[0].is_waypoint,
        });

        for i in 1..route.hops.len() {
            let prev_id = new_hops.last().unwrap().system_id;
            let next_id = route.hops[i].system_id;
            let leg = self.euclidean_distance(prev_id, next_id)?;

            if leg <= max_ly {
                let cumulative = new_hops.last().unwrap().cumulative_ly + leg;
                new_hops.push(RouteHop {
                    system_id: next_id,
                    leg_distance_ly: leg,
                    cumulative_ly: cumulative,
                    is_waypoint: route.hops[i].is_waypoint,
                });
            } else {
                self.insert_waypoints(
                    &mut new_hops,
                    prev_id,
                    next_id,
                    max_ly,
                    route.hops[i].is_waypoint,
                )?;
            }
        }

        let total = new_hops.last().map(|h| h.cumulative_ly).unwrap_or(0.0);
        Ok(Route {
            hops: new_hops,
            total_distance_ly: total,
        })
    }

    /// Insert intermediate waypoints between `from` and `to`.
    fn insert_waypoints(
        &self,
        hops: &mut Vec<RouteHop>,
        from: SystemId,
        to: SystemId,
        max_ly: f64,
        destination_is_waypoint: bool,
    ) -> Result<(), RouteError> {
        let mut current = from;
        let mut remaining = self.euclidean_distance(current, to)?;
        let max_iterations = 1000;
        let mut iterations = 0;

        while remaining > max_ly && iterations < max_iterations {
            iterations += 1;

            let current_sys = match self.systems.get(&current) {
                Some(s) => s,
                None => break,
            };

            let mut candidates = Vec::new();
            for (id, distance) in
                self.systems_within_radius_from_address(&current_sys.address, max_ly)
            {
                if id == current || distance > max_ly {
                    continue;
                }
                let remaining_distance = self.euclidean_distance(id, to)?;
                if remaining_distance < remaining {
                    candidates.push((id, distance, remaining_distance));
                }
            }
            let best = candidates.into_iter().min_by(|a, b| {
                a.2.total_cmp(&b.2)
                    .then_with(|| crate::query::system_id_order(a.0, b.0))
            });

            match best {
                Some((waypoint_id, step_dist, _)) => {
                    let cumulative = hops.last().unwrap().cumulative_ly + step_dist;
                    hops.push(RouteHop {
                        system_id: waypoint_id,
                        leg_distance_ly: step_dist,
                        cumulative_ly: cumulative,
                        is_waypoint: true,
                    });
                    current = waypoint_id;
                    remaining = self.euclidean_distance(current, to)?;
                }
                None => break,
            }
        }

        let final_dist = self.euclidean_distance(current, to)?;
        let cumulative = hops.last().unwrap().cumulative_ly + final_dist;
        hops.push(RouteHop {
            system_id: to,
            leg_distance_ly: final_dist,
            cumulative_ly: cumulative,
            is_waypoint: destination_is_waypoint,
        });
        Ok(())
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
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::SystemNotFound`] if `start` is absent.
    pub fn reachable_systems(
        &self,
        start: SystemId,
        warp_range: f64,
    ) -> Result<Vec<SystemId>, RouteError> {
        if !self.systems.contains_key(&start) {
            return Err(RouteError::SystemNotFound(start));
        }

        let start_address = self
            .systems
            .get(&start)
            .ok_or(RouteError::SystemNotFound(start))?
            .address;
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
            for (neighbor_id, dist) in
                self.systems_within_radius_from_address(&sys.address, warp_range)
            {
                if dist <= warp_range && !visited.contains(&neighbor_id) {
                    stack.push(neighbor_id);
                }
            }
        }

        let mut result: Vec<_> = visited
            .into_iter()
            .filter(|id| id.reality_index() == start_address.reality_index)
            .collect();
        result.sort_by(|a, b| crate::query::system_id_order(*a, *b));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edges::EdgeStrategy;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};

    fn id(packed: u64) -> SystemId {
        SystemId::from_address(&GalacticAddress::from_packed(packed, 0))
    }

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
        ids.sort_by_key(|id| (id.reality_index(), id.system_address()));
        ids
    }

    #[test]
    fn test_route_error_display_includes_galaxy_qualified_identity() {
        let from = SystemId::from_address(&GalacticAddress::from_packed(0x050003AB8C07, 9));
        let to = SystemId::from_address(&GalacticAddress::from_packed(0x060003AB8C07, 0));
        assert_eq!(
            RouteError::SystemNotFound(from).to_string(),
            "system not found: 9:050003AB8C07"
        );
        assert_eq!(
            RouteError::NoPath { from, to }.to_string(),
            "no path from 9:050003AB8C07 to 0:060003AB8C07"
        );
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
        assert!(model.shortest_path(id(0xDEAD), id(0xBEEF)).is_err());
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
    fn test_shortest_path_same_galaxy_disconnected_fallback_is_exact_800_ly() {
        let model = line_model(2, 2);
        let ids = sorted_ids(&model);
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        assert_eq!(route.total_distance_ly, 800.0);
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

    #[test]
    fn test_shortest_path_ignores_injected_cross_galaxy_bridge() {
        let mut model = line_model(1, 10);
        let zero_id = sorted_ids(&model)[0];
        let a = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let b = GalacticAddress::new(2, 0, 0, 2, 0, 9);
        let a_id = SystemId::from_address(&a);
        let b_id = SystemId::from_address(&b);
        model.insert_system(System::new(a, None, None, None, vec![]));
        model.insert_system(System::new(b, None, None, None, vec![]));
        let a_node = model.node_map[&a_id];
        let b_node = model.node_map[&b_id];
        let zero_node = model.node_map[&zero_id];
        model.graph.add_edge(a_node, zero_node, 1.0);
        model.graph.add_edge(zero_node, b_node, 1.0);

        let route = model.shortest_path(a_id, b_id).unwrap();
        assert_eq!(route.hops.len(), 2);
        assert_eq!(route.hops[0].system_id, a_id);
        assert_eq!(route.hops[1].system_id, b_id);
        assert_eq!(route.total_distance_ly, 800.0);
        let reachable = model.reachable_systems(a_id, 800.0).unwrap();
        assert!(reachable.contains(&a_id));
        assert!(reachable.contains(&b_id));
        assert!(!reachable.contains(&zero_id));
    }

    #[test]
    fn test_shortest_path_validates_membership_before_same_id_and_galaxy() {
        let mut model = line_model(1, 1);
        let local = sorted_ids(&model)[0];
        assert!(matches!(
            model.shortest_path(id(0xDEAD), id(0xDEAD)),
            Err(RouteError::SystemNotFound(missing)) if missing == id(0xDEAD)
        ));
        let foreign_address = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let foreign = SystemId::from_address(&foreign_address);
        model.insert_system(System::new(foreign_address, None, None, None, vec![]));
        assert!(matches!(
            model.shortest_path(local, foreign),
            Err(RouteError::NoPath { from, to }) if from == local && to == foreign
        ));
    }

    #[test]
    fn test_route_constructors_reject_foreign_and_missing_ids() {
        let mut model = line_model(2, 1);
        let local = sorted_ids(&model)[0];
        let foreign_address = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let foreign = SystemId::from_address(&foreign_address);
        model.insert_system(System::new(foreign_address, None, None, None, vec![]));
        assert!(matches!(
            model.build_route_from_order(&[local, foreign]),
            Err(RouteError::NoPath { .. })
        ));
        assert!(matches!(
            model.build_route_from_order(&[local, id(0xDEAD)]),
            Err(RouteError::SystemNotFound(_))
        ));
        assert!(matches!(
            model.two_opt_improve(vec![local, foreign]),
            Err(RouteError::NoPath { .. })
        ));
        assert!(matches!(
            model.two_opt_improve(vec![id(0xDEAD)]),
            Err(RouteError::SystemNotFound(_))
        ));
        let invalid_route = Route {
            hops: vec![
                RouteHop {
                    system_id: local,
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: foreign,
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                },
            ],
            total_distance_ly: 0.0,
        };
        assert!(matches!(
            model.constrain_hops(&invalid_route, 1_000.0),
            Err(RouteError::NoPath { .. })
        ));
        let missing_route = Route {
            hops: vec![RouteHop {
                system_id: id(0xDEAD),
                leg_distance_ly: 0.0,
                cumulative_ly: 0.0,
                is_waypoint: false,
            }],
            total_distance_ly: 0.0,
        };
        assert!(matches!(
            model.constrain_hops(&missing_route, 1_000.0),
            Err(RouteError::SystemNotFound(_))
        ));
        let empty = model.build_route_from_order(&[]).unwrap();
        assert!(empty.hops.is_empty());
        assert_eq!(empty.total_distance_ly, 0.0);
        assert!(matches!(
            model.build_route_from_order(&[local, foreign, id(0xDEAD)]),
            Err(RouteError::SystemNotFound(missing)) if missing == id(0xDEAD)
        ));
    }

    #[test]
    fn test_constrain_hops_recomputes_totals_and_preserves_waypoint_markers() {
        let mut model = GalaxyModel::new();
        let addresses = [
            GalacticAddress::new(0, 0, 0, 100, 0, 9),
            GalacticAddress::new(1, 0, 0, 101, 0, 9),
            GalacticAddress::new(2, 0, 0, 102, 0, 9),
        ];
        for address in addresses {
            model.insert_system(System::new(address, None, None, None, vec![]));
        }
        let [a, b, c] = addresses.map(|address| SystemId::from_address(&address));
        let route = Route {
            hops: vec![
                RouteHop {
                    system_id: a,
                    leg_distance_ly: 55.0,
                    cumulative_ly: 99.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: c,
                    leg_distance_ly: 3.0,
                    cumulative_ly: 4.0,
                    is_waypoint: true,
                },
            ],
            total_distance_ly: 4.0,
        };
        let constrained = model.constrain_hops(&route, 400.0).unwrap();
        assert_eq!(
            constrained
                .hops
                .iter()
                .map(|h| h.system_id)
                .collect::<Vec<_>>(),
            vec![a, b, c]
        );
        assert_eq!(
            constrained
                .hops
                .iter()
                .map(|h| h.leg_distance_ly)
                .collect::<Vec<_>>(),
            vec![0.0, 400.0, 400.0]
        );
        assert_eq!(
            constrained
                .hops
                .iter()
                .map(|h| h.cumulative_ly)
                .collect::<Vec<_>>(),
            vec![0.0, 400.0, 800.0]
        );
        assert_eq!(
            constrained
                .hops
                .iter()
                .map(|h| h.is_waypoint)
                .collect::<Vec<_>>(),
            vec![false, true, true]
        );
        assert_eq!(constrained.total_distance_ly, 800.0);

        let single = Route {
            hops: vec![RouteHop {
                system_id: a,
                leg_distance_ly: 9.0,
                cumulative_ly: 12.0,
                is_waypoint: true,
            }],
            total_distance_ly: 12.0,
        };
        let normalized = model.constrain_hops(&single, 400.0).unwrap();
        assert_eq!(normalized.hops.len(), 1);
        assert_eq!(normalized.hops[0].leg_distance_ly, 0.0);
        assert_eq!(normalized.hops[0].cumulative_ly, 0.0);
        assert!(normalized.hops[0].is_waypoint);
        assert_eq!(normalized.total_distance_ly, 0.0);
    }

    #[test]
    fn test_nine_galaxy_closed_two_opt_keeps_each_target_once() {
        let mut model = GalaxyModel::new();
        let addresses = [0, 3, 1, 2].map(|x| GalacticAddress::new(x, 0, 0, x as u16 + 100, 0, 9));
        for address in addresses {
            model.insert_system(System::new(address, None, None, None, vec![]));
        }
        let ids = addresses.map(|address| SystemId::from_address(&address));
        let route = model
            .two_opt_improve(vec![ids[0], ids[1], ids[2], ids[3], ids[0]])
            .unwrap();
        assert_eq!(route.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(route.hops.last().unwrap().system_id, ids[0]);
        assert_eq!(route.total_distance_ly, 2_400.0);
        let interiors: HashSet<_> = route.hops[1..route.hops.len() - 1]
            .iter()
            .map(|hop| hop.system_id)
            .collect();
        assert_eq!(interiors.len(), 3);
        assert_eq!(interiors, HashSet::from([ids[1], ids[2], ids[3]]));
    }

    #[test]
    fn test_tsp_foreign_target_returns_no_path_after_missing_validation() {
        let mut model = line_model(1, 1);
        let start = sorted_ids(&model)[0];
        let foreign_address = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let foreign = SystemId::from_address(&foreign_address);
        model.insert_system(System::new(foreign_address, None, None, None, vec![]));
        assert!(matches!(
            model.tsp_nearest_neighbor(start, &[foreign], false),
            Err(RouteError::NoPath { from, to }) if from == start && to == foreign
        ));
        assert!(matches!(
            model.tsp_nearest_neighbor(start, &[id(0xDEAD), foreign], false),
            Err(RouteError::SystemNotFound(missing)) if missing == id(0xDEAD)
        ));
        assert!(matches!(
            model.tsp_nearest_neighbor(id(0xBEEF), &[], false),
            Err(RouteError::TooFewTargets)
        ));
    }

    #[test]
    fn test_reachable_systems_scans_more_than_one_hundred_neighbors() {
        let model = line_model(120, 1);
        let start = sorted_ids(&model)[0];
        let reachable = model.reachable_systems(start, 100_000.0).unwrap();
        assert_eq!(reachable.len(), 120);
        assert!(
            reachable
                .windows(2)
                .all(|pair| { crate::query::system_id_order(pair[0], pair[1]).is_lt() })
        );
    }

    #[test]
    fn test_reachable_nine_galaxy_returns_exact_complete_set_with_collisions() {
        let mut model = GalaxyModel::new();
        let mut expected = Vec::new();
        for x in 0..120_i16 {
            let address = GalacticAddress::new(x, 0, 0, 1000 + x as u16, 0, 9);
            expected.push(SystemId::from_address(&address));
            model.insert_system(System::new(address, None, None, None, vec![]));
        }
        for x in [0_i16, 10, 50] {
            let address = GalacticAddress::new(x, 0, 0, 1000 + x as u16, 0, 0);
            model.insert_system(System::new(address, None, None, None, vec![]));
        }
        let isolated = GalacticAddress::new(1000, 0, 0, 500, 0, 9);
        model.insert_system(System::new(isolated, None, None, None, vec![]));
        model.set_active_galaxy(0);
        let start = expected[0];
        let reachable = model.reachable_systems(start, 47_600.0).unwrap();
        expected.sort_by(|a, b| crate::query::system_id_order(*a, *b));
        assert_eq!(reachable, expected);
        assert!(reachable.iter().all(|id| id.reality_index() == 9));
        assert!(!reachable.contains(&SystemId::from_address(&isolated)));
        assert!(model.reachable_systems(id(0xDEAD), 47_600.0).is_err());
    }

    #[test]
    fn test_waypoint_search_finds_progress_after_sixty_nonprogress_decoys() {
        let mut model = line_model(0, 1);
        let start_address = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let target_address = GalacticAddress::new(100, 0, 0, 1, 0, 0);
        let start = SystemId::from_address(&start_address);
        let target = SystemId::from_address(&target_address);
        model.insert_system(System::new(start_address, None, None, None, vec![]));
        model.insert_system(System::new(target_address, None, None, None, vec![]));
        for ssi in 10..70 {
            let decoy = GalacticAddress::new(0, 0, 0, ssi, 0, 0);
            model.insert_system(System::new(decoy, None, None, None, vec![]));
        }
        let advancing = GalacticAddress::new(10, 0, 0, 1, 0, 0);
        model.insert_system(System::new(advancing, None, None, None, vec![]));

        let direct = model.build_route_from_order(&[start, target]).unwrap();
        let constrained = model.constrain_hops(&direct, 4_000.0).unwrap();
        assert!(constrained.hops.len() > 2);
        assert!(constrained.hops[1].is_waypoint);
        assert_eq!(
            constrained.hops[1].system_id,
            SystemId::from_address(&advancing)
        );
    }

    #[test]
    fn test_two_opt_keeps_closed_start_and_end() {
        let model = line_model(6, 10);
        let ids = sorted_ids(&model);
        let mut closed = vec![ids[0], ids[3], ids[1], ids[4], ids[2], ids[5], ids[0]];
        let improved = model.two_opt_improve(std::mem::take(&mut closed)).unwrap();
        assert_eq!(improved.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(improved.hops.last().unwrap().system_id, ids[0]);
    }

    #[test]
    fn test_two_opt_closed_route_preserves_exact_2400_ly_line() {
        let model = line_model(2, 3);
        let ids = sorted_ids(&model);
        let route = model.tsp_two_opt(ids[0], &[ids[1]], true).unwrap();
        assert_eq!(route.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(route.hops.last().unwrap().system_id, ids[0]);
        assert_eq!(route.total_distance_ly, 2_400.0);
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
                .tsp_nearest_neighbor(id, &[self::id(0xDEAD)], false)
                .is_err()
        );
    }

    #[test]
    fn test_tsp_nn_nonexistent_start_errors() {
        let model = line_model(3, 10);
        let ids = sorted_ids(&model);
        assert!(model.tsp_nearest_neighbor(id(0xDEAD), &ids, false).is_err());
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
        let opt_route = model.two_opt_improve(nn_order).unwrap();
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

        let scrambled_route = model.build_route_from_order(&scrambled).unwrap();
        let improved = model.two_opt_improve(scrambled).unwrap();
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
        let route = model.build_route_from_order(&ids).unwrap();
        let constrained = model.constrain_hops(&route, 100_000.0).unwrap();
        assert_eq!(constrained.hops.len(), route.hops.len());
    }

    #[test]
    fn test_constrain_hops_inserts_waypoints() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        // Route from first to last (9 * 4000 = 36000 ly)
        let route = model.build_route_from_order(&[ids[0], ids[9]]).unwrap();
        let constrained = model.constrain_hops(&route, 5000.0).unwrap();
        assert!(constrained.hops.len() > 2, "Should insert waypoints");
        for hop in &constrained.hops[1..constrained.hops.len() - 1] {
            assert!(hop.is_waypoint);
        }
    }

    #[test]
    fn test_constrain_hops_preserves_endpoints() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        let route = model.build_route_from_order(&[ids[0], ids[9]]).unwrap();
        let constrained = model.constrain_hops(&route, 5000.0).unwrap();
        assert_eq!(constrained.hops.first().unwrap().system_id, ids[0]);
        assert_eq!(constrained.hops.last().unwrap().system_id, ids[9]);
    }

    #[test]
    fn test_constrain_hops_respects_warp_range() {
        let model = line_model(10, 10);
        let ids = sorted_ids(&model);
        let route = model.build_route_from_order(&[ids[0], ids[9]]).unwrap();
        let constrained = model.constrain_hops(&route, 5000.0).unwrap();
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
                    system_id: id(0),
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: id(1),
                    leg_distance_ly: 5000.0,
                    cumulative_ly: 5000.0,
                    is_waypoint: false,
                },
                RouteHop {
                    system_id: id(2),
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
        let constrained = model.constrain_hops(&route, 5000.0).unwrap();
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
        assert!(model.reachable_systems(id(0xDEAD), 5000.0).is_err());
    }
}

#[cfg(test)]
mod iteration02_route_oracles {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::system::System;
    use std::collections::HashSet;

    fn addr(x: i16, ssi: u16, galaxy: u8) -> GalacticAddress {
        GalacticAddress::new(x, 0, 0, ssi, 0, galaxy)
    }
    fn id(a: GalacticAddress) -> SystemId {
        SystemId::from_address(&a)
    }

    #[test]
    fn positive_nine_galaxy_bridge_and_foreign_shortcut_have_exact_projection() {
        let a = addr(0, 100, 9);
        let b = addr(1, 101, 9);
        let c = addr(2, 102, 9);
        let foreign = addr(0, 100, 0);
        let (a_id, b_id, c_id, foreign_id) = (id(a), id(b), id(c), id(foreign));
        let mut model = GalaxyModel::new();
        for x in [a, c, foreign] {
            model.insert_system(System::new(x, None, None, None, vec![]));
        }
        let an = model.node_map[&a_id];
        let cn = model.node_map[&c_id];
        let fn_ = model.node_map[&foreign_id];
        model.graph.add_edge(an, fn_, 1.0);
        model.graph.add_edge(fn_, cn, 1.0);
        let fallback = model.shortest_path(a_id, c_id).unwrap();
        assert_eq!(
            fallback
                .hops
                .iter()
                .map(|h| (
                    h.system_id,
                    h.is_waypoint,
                    h.leg_distance_ly,
                    h.cumulative_ly
                ))
                .collect::<Vec<_>>(),
            vec![(a_id, false, 0.0, 0.0), (c_id, false, 800.0, 800.0)]
        );
        model.insert_system(System::new(b, None, None, None, vec![]));
        let bn = model.node_map[&b_id];
        model.graph.add_edge(an, bn, 400.0);
        model.graph.add_edge(bn, cn, 400.0);
        let route = model.shortest_path(a_id, c_id).unwrap();
        assert_eq!(
            route
                .hops
                .iter()
                .map(|h| (
                    h.system_id,
                    h.is_waypoint,
                    h.leg_distance_ly,
                    h.cumulative_ly
                ))
                .collect::<Vec<_>>(),
            vec![
                (a_id, false, 0.0, 0.0),
                (b_id, false, 400.0, 400.0),
                (c_id, false, 400.0, 800.0)
            ]
        );
        assert_eq!(route.total_distance_ly, 800.0);
        assert!(
            matches!(model.shortest_path(a_id, foreign_id), Err(RouteError::NoPath { from, to }) if from == a_id && to == foreign_id)
        );
    }

    #[test]
    fn route_entry_points_return_exact_errors_for_empty_one_hop_missing_and_foreign_ids() {
        let local_addr = addr(0, 1, 9);
        let foreign_addr = addr(0, 1, 0);
        let local = id(local_addr);
        let foreign = id(foreign_addr);
        let missing = id(addr(4, 99, 9));
        let mut model = GalaxyModel::new();
        model.insert_system(System::new(local_addr, None, None, None, vec![]));
        model.insert_system(System::new(foreign_addr, None, None, None, vec![]));
        assert!(
            matches!(model.shortest_path(missing, missing), Err(RouteError::SystemNotFound(id)) if id == missing)
        );
        assert!(
            matches!(model.shortest_path(local, foreign), Err(RouteError::NoPath { from, to }) if from == local && to == foreign)
        );
        assert!(model.build_route_from_order(&[]).unwrap().hops.is_empty());
        let one = model.build_route_from_order(&[local]).unwrap();
        assert_eq!(
            one.hops
                .iter()
                .map(|h| (
                    h.system_id,
                    h.is_waypoint,
                    h.leg_distance_ly,
                    h.cumulative_ly
                ))
                .collect::<Vec<_>>(),
            vec![(local, false, 0.0, 0.0)]
        );
        assert!(
            matches!(model.build_route_from_order(&[local, foreign]), Err(RouteError::NoPath { from, to }) if from == local && to == foreign)
        );
        assert!(
            matches!(model.build_route_from_order(&[local, foreign, missing]), Err(RouteError::SystemNotFound(id)) if id == missing)
        );
        assert!(
            matches!(model.two_opt_improve(vec![local, foreign]), Err(RouteError::NoPath { from, to }) if from == local && to == foreign)
        );
        assert!(matches!(
            model.tsp_nearest_neighbor(missing, &[], false),
            Err(RouteError::TooFewTargets)
        ));
        assert!(matches!(
            model.tsp_two_opt(missing, &[], true),
            Err(RouteError::TooFewTargets)
        ));
        for return_to_start in [false, true] {
            assert!(
                matches!(model.tsp_nearest_neighbor(local, &[foreign], return_to_start), Err(RouteError::NoPath { from, to }) if from == local && to == foreign)
            );
            assert!(
                matches!(model.tsp_two_opt(local, &[foreign], return_to_start), Err(RouteError::NoPath { from, to }) if from == local && to == foreign)
            );
            assert!(
                matches!(model.tsp_nearest_neighbor(missing, &[local], return_to_start), Err(RouteError::SystemNotFound(id)) if id == missing)
            );
            assert!(
                matches!(model.tsp_two_opt(missing, &[local], return_to_start), Err(RouteError::SystemNotFound(id)) if id == missing)
            );
        }
    }

    #[test]
    fn dense_galaxy_nine_reachability_returns_exact_120_member_set_and_preserves_selection() {
        let mut model = GalaxyModel::new();
        let mut expected = HashSet::new();
        for ssi in 1..=120 {
            let a = addr(0, ssi, 9);
            expected.insert(id(a));
            model.insert_system(System::new(a, None, None, None, vec![]));
        }
        let collision = addr(0, 1, 0);
        let isolated = addr(10, 121, 9);
        model.insert_system(System::new(collision, None, None, None, vec![]));
        model.insert_system(System::new(isolated, None, None, None, vec![]));
        let before = (model.active_galaxy, model.player_position().copied());
        let got = model.reachable_systems(id(addr(0, 1, 9)), 1000.0).unwrap();
        assert_eq!(got.len(), 120);
        assert_eq!(got.iter().copied().collect::<HashSet<_>>(), expected);
        assert!(!got.contains(&id(collision)));
        assert!(!got.contains(&id(isolated)));
        let missing = id(addr(77, 5, 9));
        assert!(
            matches!(model.reachable_systems(missing,1000.0),Err(RouteError::SystemNotFound(id)) if id==missing)
        );
        assert_eq!(
            (model.active_galaxy, model.player_position().copied()),
            before
        );
    }
}
