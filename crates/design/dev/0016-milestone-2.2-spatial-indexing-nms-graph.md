# Milestone 2.2 -- Spatial Indexing (nms-graph)

Nearest-neighbor and within-radius queries on the R-tree spatial index. Methods on `GalaxyModel` that find systems and planets by proximity.

## Crate: `nms-graph`

Path: `crates/nms-graph/`

### Dependencies

No new dependencies -- uses `rstar` and `nms-core` already added in 2.1.

---

## New File: `crates/nms-graph/src/query.rs`

Spatial query methods, kept in a separate module for clarity:

```rust
//! Spatial query methods on the GalaxyModel.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::system::Planet;

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
    /// Resolve a reference point to a GalacticAddress.
    ///
    /// Accepts:
    /// - A direct address (returned as-is)
    /// - A base name (looked up in the base index)
    /// - None (uses player position)
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
                .get_base(name)
                .map(|b| b.address)
                .ok_or_else(|| GraphError::BaseNotFound(name.to_string()));
        }
        self.player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition)
    }

    /// Find the N nearest systems to a reference point.
    ///
    /// Returns `(SystemId, distance_in_ly)` pairs sorted by distance ascending.
    pub fn nearest_systems(
        &self,
        from: &GalacticAddress,
        n: usize,
    ) -> Vec<(SystemId, f64)> {
        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];

        self.spatial
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
    /// Returns `(SystemId, distance_in_ly)` pairs sorted by distance ascending.
    pub fn systems_within_radius(
        &self,
        from: &GalacticAddress,
        radius_ly: f64,
    ) -> Vec<(SystemId, f64)> {
        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];
        let voxel_radius = radius_ly / 400.0;
        let voxel_radius_sq = voxel_radius * voxel_radius;

        let mut results: Vec<(SystemId, f64)> = self
            .spatial
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
    /// Iterates systems by proximity, then checks their planets against the filter.
    /// Returns `(PlanetKey, &Planet, system_distance_ly)` tuples.
    pub fn nearest_planets<'a>(
        &'a self,
        from: &GalacticAddress,
        n: usize,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        let query_point = [
            from.voxel_x() as f64,
            from.voxel_y() as f64,
            from.voxel_z() as f64,
        ];

        let mut results = Vec::with_capacity(n);

        for sp in self.spatial.nearest_neighbor_iter(&query_point) {
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
```

### Update `crates/nms-graph/src/lib.rs`

Add the new module:

```rust
pub mod query;
pub use query::BiomeFilter;
```

---

## Tests

### File: `crates/nms-graph/src/query.rs` (inline tests at bottom)

```rust
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
                    "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []
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

        // Insert systems at known positions
        let positions = [
            (10, 0, 0, 0x100, "Near"),      // 10 voxels = 4000 ly
            (50, 0, 0, 0x200, "Mid"),        // 50 voxels = 20000 ly
            (200, 0, 0, 0x300, "Far"),       // 200 voxels = 80000 ly
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
        model.planets.insert(key, scorched);
        model.biome_index.entry(Biome::Scorched).or_default().push(key);
        // Also add to the system's planet list
        if let Some(sys) = model.systems.get_mut(&near_id) {
            sys.planets.push(Planet::new(1, Some(Biome::Scorched), None, true, None, None));
        }

        model
    }

    #[test]
    fn nearest_systems_returns_sorted() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = model.nearest_systems(&origin, 10);
        // Should be sorted by distance (nearest first)
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i - 1].1);
        }
    }

    #[test]
    fn nearest_systems_limit() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = model.nearest_systems(&origin, 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn systems_within_radius_filters() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        // 10 voxels = 4000 ly, so radius of 5000 should include "Near" but not "Mid"
        let results = model.systems_within_radius(&origin, 5000.0);
        assert!(results.len() >= 1);
        for (_, dist) in &results {
            assert!(*dist <= 5000.0);
        }
    }

    #[test]
    fn systems_within_radius_zero() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(999, 999, 999, 1, 0, 0);
        let results = model.systems_within_radius(&origin, 0.0);
        assert!(results.is_empty());
    }

    #[test]
    fn nearest_planets_with_biome_filter() {
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
    fn nearest_planets_with_infested_filter() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter {
            infested: Some(true),
            ..Default::default()
        };
        let results = model.nearest_planets(&origin, 10, &filter);
        for (_, planet, _) in &results {
            assert!(planet.infested);
        }
    }

    #[test]
    fn nearest_planets_no_filter() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let filter = BiomeFilter::default();
        let results = model.nearest_planets(&origin, 5, &filter);
        assert!(!results.is_empty());
    }

    #[test]
    fn resolve_position_direct_address() {
        let model = spatial_test_model();
        let addr = GalacticAddress::new(42, 0, 0, 1, 0, 0);
        let resolved = model.resolve_position(Some(&addr), None).unwrap();
        assert_eq!(resolved, addr);
    }

    #[test]
    fn resolve_position_no_player_state_errors() {
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
    fn matches_filter_all_pass() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, Some("Eden".into()), None);
        let filter = BiomeFilter::default();
        assert!(matches_filter(&planet, &filter));
    }

    #[test]
    fn matches_filter_biome_mismatch() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let filter = BiomeFilter { biome: Some(Biome::Toxic), ..Default::default() };
        assert!(!matches_filter(&planet, &filter));
    }

    #[test]
    fn matches_filter_named_only() {
        let unnamed = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let named = Planet::new(0, Some(Biome::Lush), None, false, Some("X".into()), None);
        let filter = BiomeFilter { named_only: true, ..Default::default() };
        assert!(!matches_filter(&unnamed, &filter));
        assert!(matches_filter(&named, &filter));
    }
}
```

---

## Implementation Notes

1. **Distance conversion** -- the R-tree stores voxel coordinates (not light-years). Convert: `sqrt(voxel_dist_sq) * 400.0 = light-years`. This matches `GalacticAddress::distance_ly()` in nms-core.

2. **`nearest_neighbor_iter()`** -- rstar's iterator yields points in ascending distance order. This is lazy, so `take(n)` is efficient even for large trees.

3. **`systems_within_radius` uses `take_while`** -- since `nearest_neighbor_iter()` returns ascending distances, we can stop as soon as we exceed the radius. This is O(k log n) where k is the result count.

4. **Planet queries iterate systems** -- there's no separate R-tree for planets. Instead, we iterate systems by proximity and check their planets. This is fine for hundreds of systems; if performance matters, a planet-level spatial index can be added later.

5. **`BiomeFilter` uses `Option` fields** -- `None` means "don't filter on this criterion". This composes naturally: `BiomeFilter { biome: Some(Lush), infested: Some(true), ..Default::default() }` finds infested Lush planets.
