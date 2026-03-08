//! Graph edge construction strategies.
//!
//! Edges connect systems in the petgraph. Edge weights are distances in
//! light-years. Two strategies are provided:
//!
//! - **KNN**: connect each system to its k nearest neighbors
//! - **Warp range**: connect systems within a maximum warp distance

use rstar::PointDistance;

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
    /// Each system is matched against its own galaxy's spatial index.
    fn build_knn_edges(&mut self, k: usize) {
        let system_ids: Vec<SystemId> = self.systems.keys().copied().collect();

        for &sys_id in &system_ids {
            let system = match self.systems.get(&sys_id) {
                Some(s) => s,
                None => continue,
            };

            let galaxy = system.address.reality_index;
            let spatial = match self.spatial.get(&galaxy) {
                Some(s) => s,
                None => continue,
            };

            let query_point = [
                system.address.voxel_x() as f64,
                system.address.voxel_y() as f64,
                system.address.voxel_z() as f64,
            ];

            // Get k+1 nearest (first may be self)
            let neighbors: Vec<_> = spatial
                .nearest_neighbor_iter(&query_point)
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
    /// For each system, queries its galaxy's R-tree for all neighbors within
    /// `max_ly / 400.0` voxel units and adds edges.
    fn build_warp_range_edges(&mut self, max_ly: f64) {
        let system_ids: Vec<SystemId> = self.systems.keys().copied().collect();

        for &sys_id in &system_ids {
            let system = match self.systems.get(&sys_id) {
                Some(s) => s,
                None => continue,
            };

            let galaxy = system.address.reality_index;
            let spatial = match self.spatial.get(&galaxy) {
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

            let neighbors: Vec<_> = spatial
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
    /// Looks up the system's galaxy to use the correct spatial index.
    pub fn connect_new_system(&mut self, sys_id: SystemId, k: usize) {
        let system = match self.systems.get(&sys_id) {
            Some(s) => s,
            None => return,
        };

        let galaxy = system.address.reality_index;
        let spatial = match self.spatial.get(&galaxy) {
            Some(s) => s,
            None => return,
        };

        let query_point = [
            system.address.voxel_x() as f64,
            system.address.voxel_y() as f64,
            system.address.voxel_z() as f64,
        ];

        let neighbors: Vec<_> = spatial
            .nearest_neighbor_iter(&query_point)
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

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_knn_edges_created() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        assert!(model.graph.edge_count() > 0);
        assert!(model.graph.edge_count() <= 10);
    }

    #[test]
    fn test_knn_edges_all_nodes_connected() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        for (_, &node_idx) in &model.node_map {
            let degree = model.graph.edges(node_idx).count();
            assert!(degree >= 1, "Node with 0 edges found");
        }
    }

    #[test]
    fn test_warp_range_edges_respects_distance() {
        let mut model = line_model(5, 10);
        // 10 voxels * 400 = 4000 ly. Range 5000 should connect adjacent only.
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 5000.0 });
        for edge in model.graph.edge_indices() {
            let weight = model.graph[edge];
            assert!(weight <= 5000.0, "Edge weight {weight} exceeds warp range");
        }
    }

    #[test]
    fn test_warp_range_zero_no_edges() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 0.0 });
        assert_eq!(model.graph.edge_count(), 0);
    }

    #[test]
    fn test_build_edges_clears_previous() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        let count1 = model.graph.edge_count();
        model.build_edges(EdgeStrategy::Knn { k: 1 });
        let count2 = model.graph.edge_count();
        assert!(count2 < count1);
    }

    #[test]
    fn test_connect_new_system_adds_edges() {
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
    fn test_no_self_loops() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        for edge in model.graph.edge_indices() {
            let (a, b) = model.graph.edge_endpoints(edge).unwrap();
            assert_ne!(a, b, "Self-loop detected");
        }
    }

    #[test]
    fn test_no_duplicate_edges() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 4 });
        let mut seen = std::collections::HashSet::new();
        for edge in model.graph.edge_indices() {
            let (a, b) = model.graph.edge_endpoints(edge).unwrap();
            let key = if a < b { (a, b) } else { (b, a) };
            assert!(seen.insert(key), "Duplicate edge found: {key:?}");
        }
    }

    #[test]
    fn test_edge_strategy_default_is_knn_10() {
        let strategy = EdgeStrategy::default();
        match strategy {
            EdgeStrategy::Knn { k } => assert_eq!(k, 10),
            _ => panic!("Default should be Knn"),
        }
    }

    #[test]
    fn test_warp_range_large_connects_all() {
        let mut model = line_model(3, 10);
        // 3 systems, spacing 10 voxels = 4000 ly each. Range 100000 ly covers all.
        model.build_edges(EdgeStrategy::WarpRange { max_ly: 100_000.0 });
        // 3 systems fully connected = 3 edges
        assert_eq!(model.graph.edge_count(), 3);
    }

    #[test]
    fn test_connect_new_system_nonexistent_is_noop() {
        let mut model = line_model(3, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        let edges_before = model.graph.edge_count();

        // Try connecting a system that doesn't exist
        model.connect_new_system(SystemId(0xDEADBEEF), 2);

        assert_eq!(model.graph.edge_count(), edges_before);
    }

    #[test]
    fn test_edge_weights_are_positive() {
        let mut model = line_model(5, 10);
        model.build_edges(EdgeStrategy::Knn { k: 2 });
        for edge in model.graph.edge_indices() {
            let weight = model.graph[edge];
            assert!(weight > 0.0, "Edge weight should be positive, got {weight}");
        }
    }
}
