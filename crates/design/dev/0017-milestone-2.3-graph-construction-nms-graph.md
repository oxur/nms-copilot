# Milestone 2.3 -- Graph Construction (nms-graph)

Build petgraph edges between systems using spatial proximity. Two strategies: k-nearest-neighbor and warp-range-constrained. Edges are weighted by distance in light-years.

## Crate: `nms-graph`

Path: `crates/nms-graph/`

### Dependencies

No new dependencies -- uses `petgraph` and `rstar` from 2.1.

---

## New File: `crates/nms-graph/src/edges.rs`

```rust
//! Graph edge construction strategies.
//!
//! Edges connect systems in the petgraph. Edge weights are distances in
//! light-years. Two strategies are provided:
//!
//! - **KNN**: connect each system to its k nearest neighbors
//! - **Warp range**: connect systems within a maximum warp distance

use crate::model::GalaxyModel;
use crate::spatial::SystemId;

/// Strategy for generating graph edges.
#[derive(Debug, Clone, Copy)]
pub enum EdgeStrategy {
    /// Connect each system to its k nearest neighbors.
    Knn { k: usize },
    /// Connect all systems within warp range (light-years).
    WarpRange { max_ly: f64 },
}

impl Default for EdgeStrategy {
    fn default() -> Self {
        EdgeStrategy::Knn { k: 10 }
    }
}

impl GalaxyModel {
    /// Build edges using the given strategy. Clears existing edges first.
    pub fn build_edges(&mut self, strategy: EdgeStrategy) {
        // Remove all existing edges
        let edge_ids: Vec<_> = self.graph.edge_indices().collect();
        for edge_id in edge_ids {
            self.graph.remove_edge(edge_id);
        }

        match strategy {
            EdgeStrategy::Knn { k } => self.build_knn_edges(k),
            EdgeStrategy::WarpRange { max_ly } => self.build_warp_range_edges(max_ly),
        }
    }

    /// Connect each system to its k nearest neighbors.
    ///
    /// For each system, queries the R-tree for the k+1 nearest points
    /// (including itself), skips self, and adds undirected edges.
    /// Duplicate edges (A-B when B-A already exists) are avoided.
    fn build_knn_edges(&mut self, k: usize) {
        // Collect all system points so we can iterate without borrowing self
        let system_ids: Vec<SystemId> = self.systems.keys().copied().collect();

        for &sys_id in &system_ids {
            let system = match self.systems.get(&sys_id) {
                Some(s) => s,
                None => continue,
            };

            let query_point = [
                system.address.voxel_x() as f64,
                system.address.voxel_y() as f64,
                system.address.voxel_z() as f64,
            ];

            // Get k+1 nearest (first is self)
            let neighbors: Vec<_> = self
                .spatial
                .nearest_neighbor_iter(&query_point)
                .take(k + 1)
                .filter(|sp| sp.id != sys_id)
                .take(k)
                .map(|sp| {
                    let dist_ly = sp.distance_2(&query_point).sqrt() * 400.0;
                    (sp.id, dist_ly)
                })
                .collect();

            let from_node = match self.node_map.get(&sys_id) {
                Some(&n) => n,
                None => continue,
            };

            for (neighbor_id, dist_ly) in neighbors {
                let to_node = match self.node_map.get(&neighbor_id) {
                    Some(&n) => n,
                    None => continue,
                };

                // Avoid duplicate edges (check both directions)
                if self.graph.find_edge(from_node, to_node).is_none() {
                    self.graph.add_edge(from_node, to_node, dist_ly);
                }
            }
        }
    }

    /// Connect all pairs of systems within a warp range (in light-years).
    ///
    /// For each system, queries the R-tree for all neighbors within
    /// `max_ly / 400.0` voxel units and adds edges.
    fn build_warp_range_edges(&mut self, max_ly: f64) {
        let system_ids: Vec<SystemId> = self.systems.keys().copied().collect();

        for &sys_id in &system_ids {
            let system = match self.systems.get(&sys_id) {
                Some(s) => s,
                None => continue,
            };

            let query_point = [
                system.address.voxel_x() as f64,
                system.address.voxel_y() as f64,
                system.address.voxel_z() as f64,
            ];

            let voxel_radius = max_ly / 400.0;
            let voxel_radius_sq = voxel_radius * voxel_radius;

            let neighbors: Vec<_> = self
                .spatial
                .nearest_neighbor_iter(&query_point)
                .take_while(|sp| sp.distance_2(&query_point) <= voxel_radius_sq)
                .filter(|sp| sp.id != sys_id)
                .map(|sp| {
                    let dist_ly = sp.distance_2(&query_point).sqrt() * 400.0;
                    (sp.id, dist_ly)
                })
                .collect();

            let from_node = match self.node_map.get(&sys_id) {
                Some(&n) => n,
                None => continue,
            };

            for (neighbor_id, dist_ly) in neighbors {
                let to_node = match self.node_map.get(&neighbor_id) {
                    Some(&n) => n,
                    None => continue,
                };

                if self.graph.find_edge(from_node, to_node).is_none() {
                    self.graph.add_edge(from_node, to_node, dist_ly);
                }
            }
        }
    }

    /// Add KNN edges for a single newly inserted system.
    ///
    /// Call this after `insert_system()` to connect the new node to
    /// its neighbors without rebuilding the entire graph.
    pub fn connect_new_system(&mut self, sys_id: SystemId, k: usize) {
        let system = match self.systems.get(&sys_id) {
            Some(s) => s,
            None => return,
        };

        let query_point = [
            system.address.voxel_x() as f64,
            system.address.voxel_y() as f64,
            system.address.voxel_z() as f64,
        ];

        let neighbors: Vec<_> = self
            .spatial
            .nearest_neighbor_iter(&query_point)
            .take(k + 1)
            .filter(|sp| sp.id != sys_id)
            .take(k)
            .map(|sp| {
                let dist_ly = sp.distance_2(&query_point).sqrt() * 400.0;
                (sp.id, dist_ly)
            })
            .collect();

        let from_node = match self.node_map.get(&sys_id) {
            Some(&n) => n,
            None => return,
        };

        for (neighbor_id, dist_ly) in neighbors {
            let to_node = match self.node_map.get(&neighbor_id) {
                Some(&n) => n,
                None => continue,
            };

            if self.graph.find_edge(from_node, to_node).is_none() {
                self.graph.add_edge(from_node, to_node, dist_ly);
            }
        }
    }
}
```

### Update `crates/nms-graph/src/lib.rs`

Add the new module:

```rust
pub mod edges;
pub use edges::EdgeStrategy;
```

### Update `GalaxyModel::from_save()` in `model.rs`

After building the model, add default edges:

```rust
// At end of from_save(), before returning:
let mut model = Self { graph, spatial, systems, ... };
model.build_edges(EdgeStrategy::default());
model
```

---

## Tests

### File: `crates/nms-graph/src/edges.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::system::{System, Planet};
    use nms_core::biome::Biome;
    use crate::model::GalaxyModel;

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

    #[test]
    fn knn_edges_created() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        // Each of 5 systems connected to 2 nearest -> at most 10 directed edges
        // Undirected + dedup means fewer edges
        assert!(model.graph.edge_count() > 0);
        assert!(model.graph.edge_count() <= 10);
    }

    #[test]
    fn knn_edges_all_nodes_connected() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        // Every node should have at least one edge
        for (_, &node_idx) in &model.node_map {
            let degree = model.graph.edges(node_idx).count();
            assert!(degree >= 1, "Node with 0 edges found");
        }
    }

    #[test]
    fn warp_range_edges_respects_distance() {
        let mut model = line_model(5, 10);
        // 10 voxels * 400 = 4000 ly. Range 5000 should connect adjacent only.
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 5000.0 });
        for edge in model.graph.edge_indices() {
            let weight = model.graph[edge];
            assert!(weight <= 5000.0, "Edge weight {weight} exceeds warp range");
        }
    }

    #[test]
    fn warp_range_zero_no_edges() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 0.0 });
        assert_eq!(model.graph.edge_count(), 0);
    }

    #[test]
    fn build_edges_clears_previous() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        let count1 = model.graph.edge_count();
        model.build_edges(EdgeStrategy::Knn { k: 1 });
        let count2 = model.graph.edge_count();
        assert!(count2 < count1);
    }

    #[test]
    fn connect_new_system_adds_edges() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let edges_before = model.graph.edge_count();

        // Insert a new system
        let addr = GalacticAddress::new(5, 0, 0, 0xFFF, 0, 0);
        let system = System::new(addr, None, None, None, vec![]);
        let sys_id = crate::spatial::SystemId::from_address(&addr);
        model.insert_system(system);
        model.connect_new_system(sys_id, 2);

        assert!(model.graph.edge_count() > edges_before);
    }

    #[test]
    fn no_self_loops() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        for edge in model.graph.edge_indices() {
            let (a, b) = model.graph.edge_endpoints(edge).unwrap();
            assert_ne!(a, b, "Self-loop detected");
        }
    }

    #[test]
    fn no_duplicate_edges() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        let mut seen = std::collections::HashSet::new();
        for edge in model.graph.edge_indices() {
            let (a, b) = model.graph.edge_endpoints(edge).unwrap();
            let key = if a < b { (a, b) } else { (b, a) };
            assert!(seen.insert(key), "Duplicate edge found: {key:?}");
        }
    }
}
```

---

## Implementation Notes

1. **Edge deduplication** -- `find_edge(a, b)` checks for an existing edge between nodes. Since the graph is undirected, this also catches the reverse direction.

2. **KNN k=10 default** -- for a typical save with ~300 systems, k=10 creates a well-connected graph without being dense. Users can rebuild with different parameters.

3. **`build_edges` clears first** -- calling `build_edges` multiple times (e.g., switching strategy) doesn't accumulate edges. The old edges are removed before rebuilding.

4. **`connect_new_system`** -- for incremental updates from the file watcher, we don't rebuild the entire graph. Instead, we connect just the new node to its k nearest neighbors.

5. **Voxel distance → light-years** -- edges store distance in light-years (voxel distance * 400.0), matching the convention used everywhere else in the project.

6. **O(n * k * log n)** for KNN construction -- each of n systems does a KNN query costing O(k * log n) on the R-tree. For 300 systems with k=10, this is fast (<1ms).
