# Phase 4A -- Graph Routing Algorithms (nms-graph)

Milestones 4.1-4.4: Dijkstra shortest path, nearest-neighbor TSP, 2-opt improvement, and hop-constrained routing. All implemented in `nms-graph` as methods on `GalaxyModel`.

## Crate: `nms-graph`

Path: `crates/nms-graph/`

### Dependencies to add

```toml
# crates/nms-graph/Cargo.toml
# No new dependencies -- petgraph already provides Dijkstra
# rstar already provides spatial queries for waypoint insertion
```

### Overview

The graph already has:
- `StableGraph<SystemId, f64, Undirected>` with edge weights in light-years
- `EdgeStrategy::Knn` and `EdgeStrategy::WarpRange` for edge construction
- `nearest_systems()` and `systems_within_radius()` for spatial queries

This phase adds four routing capabilities that build on that foundation:
1. Dijkstra shortest path (point-to-point)
2. Nearest-neighbor TSP (visit a set of targets)
3. 2-opt local improvement (shorten TSP routes)
4. Hop-constrained routing (respect warp range limits per hop)

---

## New File: `crates/nms-graph/src/route.rs`

All routing algorithms and types live here.

### Types

```rust
//! Graph routing algorithms for galactic navigation.
//!
//! Provides shortest-path, TSP, and warp-range-constrained routing
//! over the petgraph topology.

use std::collections::HashMap;

use petgraph::algo::dijkstra;
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
            Self::NoPath { from, to } => write!(
                f,
                "no path from 0x{:012X} to 0x{:012X}",
                from.0, to.0
            ),
            Self::TooFewTargets => write!(f, "need at least 2 targets for a route"),
        }
    }
}

impl std::error::Error for RouteError {}
```

---

### Milestone 4.1: Dijkstra Shortest Path

Point-to-point shortest path using petgraph's built-in Dijkstra.

**Key design decision:** The graph may not have edges connecting distant systems (depending on the `EdgeStrategy` used at build time). If Dijkstra finds no path, we fall back to direct Euclidean distance as a single hop. This is realistic -- the player can always warp directly, just may need multiple jumps.

```rust
impl GalaxyModel {
    /// Find the shortest path between two systems using Dijkstra.
    ///
    /// Returns the path as a list of SystemIds with distances.
    /// If no graph path exists, returns a direct hop (Euclidean distance).
    pub fn shortest_path(
        &self,
        from: SystemId,
        to: SystemId,
    ) -> Result<Route, RouteError> {
        let from_node = self.node_map.get(&from)
            .ok_or(RouteError::SystemNotFound(from))?;
        let to_node = self.node_map.get(&to)
            .ok_or(RouteError::SystemNotFound(to))?;

        // Run Dijkstra from `from_node`
        let predecessors = dijkstra(
            &self.graph,
            *from_node,
            Some(*to_node),
            |e| *e.weight(),
        );

        if predecessors.contains_key(to_node) {
            // Reconstruct path by walking predecessors
            let path = self.reconstruct_dijkstra_path(*from_node, *to_node);
            Ok(self.path_to_route(&path))
        } else {
            // No graph path -- fall back to direct Euclidean hop
            self.direct_route(from, to)
        }
    }

    /// Reconstruct a path from Dijkstra's result.
    ///
    /// petgraph's dijkstra returns a cost map, not predecessors.
    /// We use astar instead for path reconstruction.
    fn reconstruct_dijkstra_path(
        &self,
        from: NodeIndex,
        to: NodeIndex,
    ) -> Vec<NodeIndex> {
        // Use petgraph::algo::astar which returns the actual path
        use petgraph::algo::astar;
        match astar(
            &self.graph,
            from,
            |n| n == to,
            |e| *e.weight(),
            |_| 0.0, // no heuristic (reduces to Dijkstra)
        ) {
            Some((_, path)) => path,
            None => vec![from, to], // shouldn't happen -- caller checked
        }
    }

    /// Create a direct single-hop route between two systems.
    fn direct_route(&self, from: SystemId, to: SystemId) -> Result<Route, RouteError> {
        let from_sys = self.systems.get(&from)
            .ok_or(RouteError::SystemNotFound(from))?;
        let to_sys = self.systems.get(&to)
            .ok_or(RouteError::SystemNotFound(to))?;

        let dist = from_sys.address.distance_ly(&to_sys.address);

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
                // Get edge weight between consecutive nodes
                let prev = path[i - 1];
                self.graph
                    .find_edge(prev, node)
                    .map(|e| self.graph[e])
                    .unwrap_or_else(|| {
                        // No direct edge -- compute Euclidean
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

    /// Euclidean distance in ly between two systems (R-tree independent).
    fn euclidean_distance(&self, a: SystemId, b: SystemId) -> f64 {
        match (self.systems.get(&a), self.systems.get(&b)) {
            (Some(sa), Some(sb)) => sa.address.distance_ly(&sb.address),
            _ => 0.0,
        }
    }
}
```

#### Tests (Milestone 4.1)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::system::{Planet, System};
    use nms_core::biome::Biome;
    use crate::edges::EdgeStrategy;

    /// Build a model with N systems in a line along the X axis.
    fn line_model(n: usize, spacing: i16) -> GalaxyModel {
        // ... same helper as edges.rs tests ...
    }

    #[test]
    fn test_shortest_path_adjacent_systems() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();
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
    }

    #[test]
    fn test_shortest_path_nonexistent_system_errors() {
        let model = line_model(3, 10);
        let result = model.shortest_path(SystemId(0xDEAD), SystemId(0xBEEF));
        assert!(result.is_err());
    }

    #[test]
    fn test_shortest_path_disconnected_falls_back_to_direct() {
        let mut model = line_model(3, 100);
        // Warp range too small to connect any systems
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 1.0 });
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        // Should still return a route (direct hop)
        assert_eq!(route.hops.len(), 2);
    }

    #[test]
    fn test_shortest_path_multi_hop() {
        // 5 systems, spacing 10 voxels, KNN k=1 (only nearest neighbor)
        // Path from first to last must go through intermediates
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 1 });
        let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
        // Sort by address so we know the order
        ids.sort_by_key(|id| id.0);
        let route = model.shortest_path(ids[0], ids[4]).unwrap();
        assert!(route.hops.len() >= 3, "Should route through intermediates");
    }

    #[test]
    fn test_shortest_path_cumulative_distances() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        for i in 1..route.hops.len() {
            let expected = route.hops[i - 1].cumulative_ly + route.hops[i].leg_distance_ly;
            assert!((route.hops[i].cumulative_ly - expected).abs() < 0.01);
        }
    }

    #[test]
    fn test_shortest_path_first_hop_zero_distance() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let ids: Vec<SystemId> = model.systems.keys().copied().collect();
        let route = model.shortest_path(ids[0], ids[1]).unwrap();
        assert_eq!(route.hops[0].leg_distance_ly, 0.0);
    }
}
```

---

### Milestone 4.2: Nearest-Neighbor TSP

Greedy traversal: always visit the closest unvisited target next.

```rust
impl GalaxyModel {
    /// Plan a route visiting all targets using nearest-neighbor greedy.
    ///
    /// `start` is the origin system. The route visits every target and
    /// returns to start if `return_to_start` is true.
    ///
    /// Does NOT use graph edges for ordering -- uses Euclidean distances
    /// directly for the TSP. This is the right call because TSP operates
    /// on the full distance matrix, not the topology.
    pub fn tsp_nearest_neighbor(
        &self,
        start: SystemId,
        targets: &[SystemId],
        return_to_start: bool,
    ) -> Result<Route, RouteError> {
        if targets.is_empty() {
            return Err(RouteError::TooFewTargets);
        }
        // Validate all systems exist
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
            // Find nearest unvisited
            let (nearest_idx, _nearest_dist) = unvisited
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
    fn build_route_from_order(&self, order: &[SystemId]) -> Route {
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
}
```

#### Tests (Milestone 4.2)

```rust
#[test]
fn test_tsp_nn_visits_all_targets() {
    let model = line_model(5, 10);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let start = ids[0];
    let targets = &ids[1..];
    let route = model.tsp_nearest_neighbor(start, targets, false).unwrap();
    // Route should have start + all targets
    assert_eq!(route.hops.len(), ids.len());
}

#[test]
fn test_tsp_nn_return_to_start() {
    let model = line_model(3, 10);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
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
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let route = model.tsp_nearest_neighbor(ids[0], &[ids[1]], false).unwrap();
    assert_eq!(route.hops.len(), 2);
}

#[test]
fn test_tsp_nn_nonexistent_target_errors() {
    let model = line_model(3, 10);
    let id = *model.systems.keys().next().unwrap();
    assert!(model.tsp_nearest_neighbor(id, &[SystemId(0xDEAD)], false).is_err());
}

#[test]
fn test_tsp_nn_nonexistent_start_errors() {
    let model = line_model(3, 10);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    assert!(model.tsp_nearest_neighbor(SystemId(0xDEAD), &ids, false).is_err());
}

#[test]
fn test_tsp_nn_greedy_picks_nearest() {
    // 3 systems at x=0, x=10, x=100. Starting at 0, greedy should visit 10 first.
    let model = line_model(3, 50);
    let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    let route = model.tsp_nearest_neighbor(ids[0], &[ids[2], ids[1]], false).unwrap();
    // Second hop should be the closer system (ids[1])
    assert_eq!(route.hops[1].system_id, ids[1]);
}
```

---

### Milestone 4.3: 2-Opt Improvement

Iteratively reverse segments of the route when doing so reduces total distance. Converges to a local optimum.

**Algorithm:**
1. Start with a route (e.g., from nearest-neighbor)
2. For each pair of edges (i, i+1) and (j, j+1):
   - Reverse the segment between i+1 and j
   - If the new route is shorter, keep it
3. Repeat until no improving swap is found

```rust
impl GalaxyModel {
    /// Improve a route using 2-opt local search.
    ///
    /// Iteratively reverses segments when doing so reduces total distance.
    /// Runs until no improvement is found (local optimum).
    ///
    /// Operates on the order of SystemIds, not graph edges.
    /// Takes ownership of the order and returns the improved Route.
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
                    // Skip if j is the wrap-around edge back to start
                    if i == 0 && j == n - 1 {
                        continue;
                    }
                    let gain = self.two_opt_gain(&order, i, j);
                    if gain > 1e-6 {
                        // Reverse the segment between i+1 and j
                        order[i + 1..=j].reverse();
                        improved = true;
                    }
                }
            }
        }

        self.build_route_from_order(&order)
    }

    /// Calculate the distance gain from a 2-opt swap.
    ///
    /// Compares the cost of edges (i, i+1) + (j, j+1) against
    /// the cost of (i, j) + (i+1, j+1) after reversal.
    /// Positive return means the swap improves the route.
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
        // First, get greedy order
        let nn_route = self.tsp_nearest_neighbor(start, targets, return_to_start)?;
        let order: Vec<SystemId> = nn_route.hops.iter().map(|h| h.system_id).collect();
        Ok(self.two_opt_improve(order))
    }
}
```

#### Tests (Milestone 4.3)

```rust
#[test]
fn test_two_opt_does_not_increase_distance() {
    let model = line_model(6, 20);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let nn_route = model.tsp_nearest_neighbor(ids[0], &ids[1..], false).unwrap();
    let nn_order: Vec<SystemId> = nn_route.hops.iter().map(|h| h.system_id).collect();
    let opt_route = model.two_opt_improve(nn_order);
    assert!(opt_route.total_distance_ly <= nn_route.total_distance_ly + 1e-6);
}

#[test]
fn test_two_opt_small_route_unchanged() {
    // With only 2-3 systems, 2-opt has nothing to improve
    let model = line_model(3, 10);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let route = model.tsp_two_opt(ids[0], &ids[1..], false).unwrap();
    assert_eq!(route.hops.len(), 3);
}

#[test]
fn test_two_opt_improves_scrambled_route() {
    // Build a line of systems, scramble the order, then 2-opt should fix it
    let model = line_model(8, 10);
    let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
    ids.sort_by_key(|id| id.0);

    // Scramble: swap elements to create a clearly suboptimal order
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
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let route = model.tsp_two_opt(ids[0], &ids[1..], false).unwrap();
    assert_eq!(route.hops.len(), ids.len());
}
```

---

### Milestone 4.4: Hop-Constrained Routing

Subdivides a route so that no single hop exceeds the warp range. When a hop is too long, insert the nearest known system in the right direction as an intermediate waypoint. If no system exists within range along the path, the hop remains as-is (the player will need to make manual jumps).

**Design:** This is a post-processing step on any Route. It does not change the visit order — it only inserts waypoints between existing stops.

```rust
impl GalaxyModel {
    /// Constrain a route so no hop exceeds `max_ly`.
    ///
    /// When a hop exceeds the warp range, inserts intermediate known systems
    /// as waypoints. Uses the spatial index to find the best stepping stone.
    ///
    /// Waypoints are marked with `is_waypoint: true`.
    pub fn constrain_hops(&self, route: &Route, max_ly: f64) -> Route {
        if route.hops.len() < 2 {
            return route.clone();
        }

        let mut new_hops: Vec<RouteHop> = Vec::new();
        new_hops.push(RouteHop { is_waypoint: false, ..route.hops[0].clone() });

        for i in 1..route.hops.len() {
            let prev_id = new_hops.last().unwrap().system_id;
            let next_id = route.hops[i].system_id;
            let leg = self.euclidean_distance(prev_id, next_id);

            if leg <= max_ly {
                // Hop is within range, add directly
                let cumulative = new_hops.last().unwrap().cumulative_ly + leg;
                new_hops.push(RouteHop {
                    system_id: next_id,
                    leg_distance_ly: leg,
                    cumulative_ly: cumulative,
                    is_waypoint: route.hops[i].is_waypoint,
                });
            } else {
                // Need to subdivide -- find stepping stones
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
    ///
    /// Strategy: from the current position, find the nearest known system
    /// that is (a) closer to `to` than we are, and (b) within `max_ly`.
    /// Repeat until we can reach `to` directly, or no progress can be made.
    fn insert_waypoints(
        &self,
        hops: &mut Vec<RouteHop>,
        from: SystemId,
        to: SystemId,
        max_ly: f64,
    ) {
        let mut current = from;
        let mut remaining = self.euclidean_distance(current, to);

        // Safety limit to prevent infinite loops
        let max_iterations = 1000;
        let mut iterations = 0;

        while remaining > max_ly && iterations < max_iterations {
            iterations += 1;

            // Find the nearest system to `current` that is closer to `to`
            let current_sys = match self.systems.get(&current) {
                Some(s) => s,
                None => break,
            };
            let to_sys = match self.systems.get(&to) {
                Some(s) => s,
                None => break,
            };

            let best = self.nearest_systems(&current_sys.address, 50)
                .into_iter()
                .filter(|(id, dist)| {
                    *id != current
                        && *dist <= max_ly
                        && self.euclidean_distance(*id, to) < remaining
                })
                .min_by(|a, b| {
                    // Prefer the one that gets us closest to `to`
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
                None => {
                    // No stepping stone found -- emit the long hop as-is
                    break;
                }
            }
        }

        // Final hop to destination
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
    ///
    /// Each hop that exceeds `warp_range` requires ceil(distance / warp_range) jumps.
    pub fn warp_jump_count(route: &Route, warp_range: f64) -> usize {
        route.hops.iter()
            .skip(1) // skip start (0 distance)
            .map(|h| (h.leg_distance_ly / warp_range).ceil() as usize)
            .sum()
    }
}
```

#### Tests (Milestone 4.4)

```rust
#[test]
fn test_constrain_hops_short_route_unchanged() {
    let model = line_model(3, 5);
    let ids: Vec<SystemId> = model.systems.keys().copied().collect();
    let route = model.build_route_from_order(&ids);
    let constrained = model.constrain_hops(&route, 100_000.0);
    assert_eq!(constrained.hops.len(), route.hops.len());
}

#[test]
fn test_constrain_hops_inserts_waypoints() {
    // 2 systems far apart, with intermediate systems available
    let model = line_model(10, 10); // 10 systems, spacing 4000 ly
    let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    // Route from first to last (9 * 4000 = 36000 ly)
    let route = model.build_route_from_order(&[ids[0], ids[9]]);
    let constrained = model.constrain_hops(&route, 5000.0);
    assert!(constrained.hops.len() > 2, "Should insert waypoints");
    // All inserted hops should be marked as waypoints (except first and last)
    for hop in &constrained.hops[1..constrained.hops.len() - 1] {
        assert!(hop.is_waypoint);
    }
}

#[test]
fn test_constrain_hops_preserves_total_endpoint() {
    let model = line_model(10, 10);
    let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    let route = model.build_route_from_order(&[ids[0], ids[9]]);
    let constrained = model.constrain_hops(&route, 5000.0);
    // Should still start and end at the same systems
    assert_eq!(constrained.hops.first().unwrap().system_id, ids[0]);
    assert_eq!(constrained.hops.last().unwrap().system_id, ids[9]);
}

#[test]
fn test_constrain_hops_respects_warp_range() {
    let model = line_model(10, 10);
    let mut ids: Vec<SystemId> = model.systems.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    let route = model.build_route_from_order(&[ids[0], ids[9]]);
    let constrained = model.constrain_hops(&route, 5000.0);
    // Most hops should be within warp range (some may exceed if no waypoint available)
    let within_range = constrained.hops.iter()
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
            RouteHop { system_id: SystemId(0), leg_distance_ly: 0.0, cumulative_ly: 0.0, is_waypoint: false },
            RouteHop { system_id: SystemId(1), leg_distance_ly: 5000.0, cumulative_ly: 5000.0, is_waypoint: false },
            RouteHop { system_id: SystemId(2), leg_distance_ly: 5000.0, cumulative_ly: 10000.0, is_waypoint: false },
        ],
    };
    assert_eq!(GalaxyModel::warp_jump_count(&route, 2500.0), 4); // 2+2
    assert_eq!(GalaxyModel::warp_jump_count(&route, 5000.0), 2); // 1+1
}

#[test]
fn test_constrain_hops_empty_route() {
    let model = line_model(3, 10);
    let route = Route { hops: vec![], total_distance_ly: 0.0 };
    let constrained = model.constrain_hops(&route, 5000.0);
    assert!(constrained.hops.is_empty());
}
```

---

### Milestone 4.8: Reachability Analysis

Connected components within a given warp range. "Which systems can I reach from here?"

**Design:** Rebuild edges with `WarpRange` strategy for the given range, then find the connected component containing the start system using petgraph's DFS.

```rust
use petgraph::visit::Dfs;

impl GalaxyModel {
    /// Find all systems reachable from `start` within a given warp range.
    ///
    /// Uses a DFS on the graph after building warp-range edges.
    /// Returns SystemIds of all reachable systems (including `start`).
    ///
    /// Note: This temporarily rebuilds edges, so it clones the graph.
    /// For a non-destructive query, this operates on a local copy.
    pub fn reachable_systems(
        &self,
        start: SystemId,
        warp_range: f64,
    ) -> Result<Vec<SystemId>, RouteError> {
        let start_node = self.node_map.get(&start)
            .ok_or(RouteError::SystemNotFound(start))?;

        // Build a temporary graph with warp-range edges
        let mut temp = self.clone_graph_structure();
        temp.build_edges(crate::edges::EdgeStrategy::WarpRange { max_ly: warp_range });

        // DFS from start
        let mut reachable = Vec::new();
        let mut dfs = Dfs::new(&temp.graph, *start_node);
        while let Some(node) = dfs.next(&temp.graph) {
            reachable.push(temp.graph[node]);
        }

        Ok(reachable)
    }

    /// Clone just the graph/node structure (no systems/spatial data).
    /// Used for temporary edge experiments.
    fn clone_graph_structure(&self) -> GalaxyModel {
        // We need a full clone because build_edges operates on &mut self.
        // This is acceptable for a query operation.
        self.clone()
    }
}
```

**Note:** This requires `GalaxyModel` to derive or implement `Clone`. If that's too expensive, an alternative is to build a temporary `StableGraph` with just the warp-range edges and reuse the existing `node_map` for lookups. This is a design choice to be made during implementation -- the simpler `clone` approach first, optimize if needed.

An alternative lightweight approach (no clone):

```rust
/// Lightweight reachability without cloning the model.
///
/// Builds a separate adjacency set from the R-tree directly.
pub fn reachable_systems_fast(
    &self,
    start: SystemId,
    warp_range: f64,
) -> Result<Vec<SystemId>, RouteError> {
    let start_sys = self.systems.get(&start)
        .ok_or(RouteError::SystemNotFound(start))?;

    let mut visited = std::collections::HashSet::new();
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
```

This second approach is recommended for implementation -- it avoids cloning and uses the existing R-tree directly.

#### Tests (Milestone 4.8)

```rust
#[test]
fn test_reachable_includes_start() {
    let model = line_model(5, 10);
    let id = *model.systems.keys().next().unwrap();
    let reachable = model.reachable_systems_fast(id, 100_000.0).unwrap();
    assert!(reachable.contains(&id));
}

#[test]
fn test_reachable_all_within_large_range() {
    let model = line_model(5, 10);
    let id = *model.systems.keys().next().unwrap();
    let reachable = model.reachable_systems_fast(id, 1_000_000.0).unwrap();
    assert_eq!(reachable.len(), model.systems.len());
}

#[test]
fn test_reachable_isolated_with_tiny_range() {
    let model = line_model(5, 100); // 100 voxels = 40000 ly apart
    let id = *model.systems.keys().next().unwrap();
    // Range of 1 ly can't reach any neighbor
    let reachable = model.reachable_systems_fast(id, 1.0).unwrap();
    assert_eq!(reachable.len(), 1); // only self
}

#[test]
fn test_reachable_partial() {
    // Systems at 0, 10, 20, 100, 110. Range 5000 ly (12.5 voxels).
    // From system 0: can reach 10, 20 (chain), but not 100.
    let json = r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
    }"#;
    let save = nms_save::parse_save(json.as_bytes()).unwrap();
    let mut model = GalaxyModel::from_save(&save);

    let positions = [(0, 1), (10, 2), (20, 3), (100, 4), (110, 5)];
    for (x, ssi) in positions {
        let addr = GalacticAddress::new(x, 0, 0, ssi, 0, 0);
        let system = System::new(addr, None, None, None, vec![]);
        model.insert_system(system);
    }

    let start_addr = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let start_id = SystemId::from_address(&start_addr);
    let reachable = model.reachable_systems_fast(start_id, 5000.0).unwrap();
    // Should reach 0, 10, 20 (3 systems within chain distance)
    // but not 100, 110 (gap of 80 voxels = 32000 ly)
    assert!(reachable.len() >= 3);
    assert!(reachable.len() < 5);
}

#[test]
fn test_reachable_nonexistent_start_errors() {
    let model = line_model(3, 10);
    assert!(model.reachable_systems_fast(SystemId(0xDEAD), 5000.0).is_err());
}
```

---

## Modified File: `crates/nms-graph/src/lib.rs`

```rust
pub mod edges;
pub mod error;
pub mod extract;
pub mod model;
pub mod query;
pub mod route;
pub mod spatial;

pub use edges::EdgeStrategy;
pub use error::GraphError;
pub use model::GalaxyModel;
pub use query::BiomeFilter;
pub use route::{Route, RouteError, RouteHop, RoutingAlgorithm};
pub use spatial::{SystemId, SystemPoint};
```

---

## Implementation Notes

1. **TSP uses Euclidean distance, not graph edges.** The graph topology (KNN/WarpRange edges) is for pathfinding between specific pairs. TSP needs the full pairwise distance matrix, which is just Euclidean. Mixing the two would be confusing.

2. **2-opt is O(n^2) per iteration.** For typical player datasets (~100-1000 systems), this converges in milliseconds. No need for Or-opt or 3-opt unless performance becomes an issue.

3. **Hop constraint is a post-processing step.** It doesn't change visit order -- it only adds waypoints. This keeps the TSP and routing logic clean and composable: `TSP → 2-opt → constrain_hops`.

4. **Reachability uses DFS + spatial index.** The lightweight approach avoids cloning the model by using the R-tree directly. The `nearest_systems` query with limit 100 is generous enough to find all neighbors within warp range.

5. **`astar` with zero heuristic = Dijkstra.** petgraph's `dijkstra` returns only a cost map, not the path. Using `astar` with `|_| 0.0` heuristic gives identical results plus the actual path.

6. **Composable routing pipeline:**
   ```
   targets → tsp_nearest_neighbor → two_opt_improve → constrain_hops → Route
   ```
   Each step takes and returns the same types, so callers can mix and match.
