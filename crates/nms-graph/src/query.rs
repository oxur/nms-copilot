//! Spatial query methods on the GalaxyModel.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::system::Planet;
use rstar::PointDistance;

use crate::error::GraphError;
use crate::model::{GalaxyModel, PlanetKey};
use crate::spatial::SystemId;

/// Filter criteria for planet queries.
#[derive(Debug, Clone, Default)]
pub struct BiomeFilter {
    pub biome: Option<Biome>,
    pub infested: Option<bool>,
    pub named_only: bool,
}

impl GalaxyModel {
    /// Resolve a reference point to a `GalacticAddress`.
    ///
    /// Accepts:
    /// - A direct address (returned as-is)
    /// - A base name (looked up in the base index)
    /// - None for both (uses player position)
    pub fn resolve_position(
        &self,
        address: Option<&GalacticAddress>,
        base_name: Option<&str>,
    ) -> Result<GalacticAddress, GraphError> {
        if let Some(addr) = address {
            return Ok(*addr);
        }
        if let Some(name) = base_name {
            return self
                .base(name)
                .map(|b| b.address)
                .ok_or_else(|| GraphError::BaseNotFound(name.to_string()));
        }
        self.player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition)
    }

    /// Find the N nearest systems to a reference point.
    ///
    /// Uses the active galaxy's spatial index. Returns empty if the active
    /// galaxy has no systems.
    ///
    /// Returns `(SystemId, distance_in_ly)` pairs sorted by distance ascending.
    pub fn nearest_systems(&self, from: &GalacticAddress, n: usize) -> Vec<(SystemId, f64)> {
        let Some(spatial) = self.active_spatial() else {
            return Vec::new();
        };

        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];

        spatial
            .nearest_neighbor_iter(&query_point)
            .take(n)
            .map(|sp| {
                let voxel_dist_sq = sp.distance_2(&query_point);
                let ly = voxel_dist_sq.sqrt() * 400.0;
                (sp.id, ly)
            })
            .collect()
    }

    /// Find all systems within a radius (in light-years) of a reference point.
    ///
    /// Uses the active galaxy's spatial index. Returns empty if the active
    /// galaxy has no systems.
    ///
    /// Returns `(SystemId, distance_in_ly)` pairs sorted by distance ascending.
    pub fn systems_within_radius(
        &self,
        from: &GalacticAddress,
        radius_ly: f64,
    ) -> Vec<(SystemId, f64)> {
        let Some(spatial) = self.active_spatial() else {
            return Vec::new();
        };

        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];
        let voxel_radius = radius_ly / 400.0;
        let voxel_radius_sq = voxel_radius * voxel_radius;

        let mut results: Vec<(SystemId, f64)> = spatial
            .nearest_neighbor_iter(&query_point)
            .map(|sp| {
                let dist_sq = sp.distance_2(&query_point);
                (sp.id, dist_sq)
            })
            .take_while(|&(_, dist_sq)| dist_sq <= voxel_radius_sq)
            .map(|(id, dist_sq)| (id, dist_sq.sqrt() * 400.0))
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }

    /// Find the N nearest planets to a reference point, with optional filtering.
    ///
    /// Uses the active galaxy's spatial index. Returns empty if the active
    /// galaxy has no systems.
    ///
    /// Iterates systems by proximity, then checks their planets against the filter.
    /// Returns `(PlanetKey, &Planet, system_distance_ly)` tuples.
    pub fn nearest_planets<'a>(
        &'a self,
        from: &GalacticAddress,
        n: usize,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        let Some(spatial) = self.active_spatial() else {
            return Vec::new();
        };

        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];

        let mut results = Vec::with_capacity(n);

        for sp in spatial.nearest_neighbor_iter(&query_point) {
            if results.len() >= n {
                break;
            }

            let dist_ly = sp.distance_2(&query_point).sqrt() * 400.0;

            // Get all planets in this system
            if let Some(system) = self.systems.get(&sp.id) {
                for planet in &system.planets {
                    if results.len() >= n {
                        break;
                    }
                    if matches_filter(planet, filter) {
                        let key = (sp.id, planet.index);
                        results.push((key, planet, dist_ly));
                    }
                }
            }
        }

        results
    }

    /// Find all planets within a radius that match a filter.
    pub fn planets_within_radius<'a>(
        &'a self,
        from: &GalacticAddress,
        radius_ly: f64,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        let systems = self.systems_within_radius(from, radius_ly);
        let mut results = Vec::new();

        for (sys_id, dist_ly) in systems {
            if let Some(system) = self.systems.get(&sys_id) {
                for planet in &system.planets {
                    if matches_filter(planet, filter) {
                        let key = (sys_id, planet.index);
                        results.push((key, planet, dist_ly));
                    }
                }
            }
        }

        results
    }
}

/// Check if a planet matches the given filter criteria.
fn matches_filter(planet: &Planet, filter: &BiomeFilter) -> bool {
    if let Some(biome) = filter.biome {
        if planet.biome != Some(biome) {
            return false;
        }
    }
    if let Some(infested) = filter.infested {
        if planet.infested != infested {
            return false;
        }
    }
    if filter.named_only && planet.name.is_none() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};

    /// Build a model with systems at known positions for spatial testing.
    fn spatial_test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [
                        {"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                    ]
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);

        // Insert systems at known positions along the X axis
        let positions = [
            (10, 0, 0, 0x100, "Near"), // 10 voxels = 4000 ly
            (50, 0, 0, 0x200, "Mid"),  // 50 voxels = 20000 ly
            (200, 0, 0, 0x300, "Far"), // 200 voxels = 80000 ly
        ];

        for (x, y, z, ssi, name) in positions {
            let addr = GalacticAddress::new(x, y, z, ssi, 0, 0);
            let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
            let system = System::new(addr, Some(name.into()), None, None, vec![planet]);
            model.insert_system(system);
        }

        // Add a Scorched infested planet to "Near"
        let near_addr = GalacticAddress::new(10, 0, 0, 0x100, 1, 0);
        let near_id = crate::spatial::SystemId::from_address(&near_addr);
        let scorched = Planet::new(1, Some(Biome::Scorched), None, true, None, None);
        let key = (near_id, 1);
        model.planets.insert(key, scorched.clone());
        model
            .biome_index
            .entry(Biome::Scorched)
            .or_default()
            .push(key);
        // Also add to the system's planet list
        if let Some(sys) = model.systems.get_mut(&near_id) {
            sys.planets.push(scorched);
        }

        model
    }

    #[test]
    fn test_nearest_systems_returns_sorted() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = model.nearest_systems(&origin, 10);
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i - 1].1);
        }
    }

    #[test]
    fn test_nearest_systems_limit() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = model.nearest_systems(&origin, 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_nearest_systems_distances_are_in_ly() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = model.nearest_systems(&origin, 10);
        // "Near" is at voxel 10 = 4000 ly; find the result with ~4000 ly
        let near_result = results.iter().find(|(_, d)| (*d - 4000.0).abs() < 1.0);
        assert!(near_result.is_some(), "Expected a system at ~4000 ly");
    }

    #[test]
    fn test_systems_within_radius_filters() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        // 10 voxels = 4000 ly, so radius of 5000 should include "Near" but not "Mid"
        let results = model.systems_within_radius(&origin, 5000.0);
        assert!(!results.is_empty());
        for (_, dist) in &results {
            assert!(*dist <= 5000.0);
        }
    }

    #[test]
    fn test_systems_within_radius_excludes_far() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        // Radius 5000 ly should not include "Mid" (20000 ly) or "Far" (80000 ly)
        let results = model.systems_within_radius(&origin, 5000.0);
        for (_, dist) in &results {
            assert!(*dist < 20000.0, "Far system should be excluded");
        }
    }

    #[test]
    fn test_systems_within_radius_zero() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(999, 127, 999, 1, 0, 0);
        let results = model.systems_within_radius(&origin, 0.0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_nearest_planets_with_biome_filter() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter {
            biome: Some(Biome::Scorched),
            ..Default::default()
        };
        let results = model.nearest_planets(&origin, 10, &filter);
        assert!(!results.is_empty());
        for (_, planet, _) in &results {
            assert_eq!(planet.biome, Some(Biome::Scorched));
        }
    }

    #[test]
    fn test_nearest_planets_with_infested_filter() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter {
            infested: Some(true),
            ..Default::default()
        };
        let results = model.nearest_planets(&origin, 10, &filter);
        assert!(!results.is_empty());
        for (_, planet, _) in &results {
            assert!(planet.infested);
        }
    }

    #[test]
    fn test_nearest_planets_no_filter() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter::default();
        let results = model.nearest_planets(&origin, 5, &filter);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_nearest_planets_limit_respected() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter::default();
        let results = model.nearest_planets(&origin, 1, &filter);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_planets_within_radius() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter {
            biome: Some(Biome::Lush),
            ..Default::default()
        };
        // 5000 ly should reach "Near" (4000 ly) but not "Mid" (20000 ly)
        let results = model.planets_within_radius(&origin, 5000.0, &filter);
        for (_, planet, dist) in &results {
            assert_eq!(planet.biome, Some(Biome::Lush));
            assert!(*dist <= 5000.0);
        }
    }

    #[test]
    fn test_planets_within_radius_no_match() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter {
            biome: Some(Biome::Lava),
            ..Default::default()
        };
        let results = model.planets_within_radius(&origin, 100000.0, &filter);
        assert!(results.is_empty());
    }

    #[test]
    fn test_resolve_position_direct_address() {
        let model = spatial_test_model();
        let addr = GalacticAddress::new(42, 0, 0, 1, 0, 0);
        let resolved = model.resolve_position(Some(&addr), None).unwrap();
        assert_eq!(resolved, addr);
    }

    #[test]
    fn test_resolve_position_from_base() {
        let model = spatial_test_model();
        let resolved = model.resolve_position(None, Some("Test Base")).unwrap();
        // Base was inserted with address 0x001000000064
        assert_eq!(resolved.packed(), 0x001000000064);
    }

    #[test]
    fn test_resolve_position_base_not_found() {
        let model = spatial_test_model();
        let result = model.resolve_position(None, Some("No Such Base"));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_position_player_position() {
        let model = spatial_test_model();
        let resolved = model.resolve_position(None, None).unwrap();
        // Player is at origin (0,0,0)
        assert_eq!(resolved.voxel_x(), 0);
        assert_eq!(resolved.voxel_y(), 0);
        assert_eq!(resolved.voxel_z(), 0);
    }

    #[test]
    fn test_resolve_position_no_player_state_errors() {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 0},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);
        model.player_state = None;
        assert!(model.resolve_position(None, None).is_err());
    }

    #[test]
    fn test_resolve_position_address_takes_priority() {
        let model = spatial_test_model();
        let addr = GalacticAddress::new(42, 10, -5, 0x999, 0, 0);
        // Even with a base name, direct address wins
        let resolved = model
            .resolve_position(Some(&addr), Some("Test Base"))
            .unwrap();
        assert_eq!(resolved, addr);
    }

    #[test]
    fn test_matches_filter_all_pass() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, Some("Eden".into()), None);
        let filter = BiomeFilter::default();
        assert!(matches_filter(&planet, &filter));
    }

    #[test]
    fn test_matches_filter_biome_mismatch() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let filter = BiomeFilter {
            biome: Some(Biome::Toxic),
            ..Default::default()
        };
        assert!(!matches_filter(&planet, &filter));
    }

    #[test]
    fn test_matches_filter_biome_match() {
        let planet = Planet::new(0, Some(Biome::Toxic), None, false, None, None);
        let filter = BiomeFilter {
            biome: Some(Biome::Toxic),
            ..Default::default()
        };
        assert!(matches_filter(&planet, &filter));
    }

    #[test]
    fn test_matches_filter_infested_mismatch() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let filter = BiomeFilter {
            infested: Some(true),
            ..Default::default()
        };
        assert!(!matches_filter(&planet, &filter));
    }

    #[test]
    fn test_matches_filter_named_only() {
        let unnamed = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let named = Planet::new(0, Some(Biome::Lush), None, false, Some("X".into()), None);
        let filter = BiomeFilter {
            named_only: true,
            ..Default::default()
        };
        assert!(!matches_filter(&unnamed, &filter));
        assert!(matches_filter(&named, &filter));
    }

    #[test]
    fn test_matches_filter_combined() {
        let planet = Planet::new(
            0,
            Some(Biome::Scorched),
            None,
            true,
            Some("Inferno".into()),
            None,
        );
        let filter = BiomeFilter {
            biome: Some(Biome::Scorched),
            infested: Some(true),
            named_only: true,
        };
        assert!(matches_filter(&planet, &filter));
    }
}
