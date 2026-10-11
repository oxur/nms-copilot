//! Spatial query methods on the GalaxyModel.

use nms_core::address::GalacticAddress;
use nms_core::biome::{Biome, BiomeSubType};
use nms_core::system::Planet;

use crate::error::GraphError;
use crate::model::{GalaxyModel, PlanetKey};
use crate::spatial::SystemId;

/// Filter criteria for planet queries.
#[derive(Debug, Clone, Default)]
pub struct BiomeFilter {
    /// Require this biome when set.
    pub biome: Option<Biome>,
    /// Require this biome subtype when set.
    pub biome_subtype: Option<BiomeSubType>,
    /// Require the planet's infested flag to equal this value when set.
    pub infested: Option<bool>,
    /// Require a planet name; a containing system name does not count.
    pub named_only: bool,
}

impl BiomeFilter {
    /// Return whether `planet` satisfies every configured planet predicate.
    ///
    /// `named_only` requires a planet name; a containing system name does not
    /// satisfy this filter. `Some(false)` matches non-infested planets.
    pub fn matches(&self, planet: &Planet) -> bool {
        if let Some(biome) = self.biome
            && planet.biome != Some(biome)
        {
            return false;
        }
        if let Some(subtype) = self.biome_subtype
            && planet.biome_subtype != Some(subtype)
        {
            return false;
        }
        if let Some(infested) = self.infested
            && planet.infested != infested
        {
            return false;
        }
        if self.named_only && planet.name.is_none() {
            return false;
        }
        true
    }
}

impl GalaxyModel {
    /// Resolve a reference point to a `GalacticAddress`.
    ///
    /// Accepts a direct address, a base name, or neither to use player position.
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

    /// Find the N nearest systems in the active galaxy to a reference point.
    ///
    /// Returns `(SystemId, distance_in_ly)` pairs in deterministic distance order.
    /// A reference in another galaxy is excluded with an empty result before
    /// distance calculation; use [`Self::nearest_systems_from_address`] to
    /// query an explicit galaxy.
    pub fn nearest_systems(&self, from: &GalacticAddress, n: usize) -> Vec<(SystemId, f64)> {
        if from.reality_index != self.active_galaxy {
            return Vec::new();
        }
        self.nearest_systems_from_address(from, n)
    }

    /// Find the N nearest systems in `from`'s galaxy without changing selection.
    ///
    /// Returns `(SystemId, distance_in_ly)` pairs ordered by distance and then
    /// by galaxy-qualified system identity.
    pub fn nearest_systems_from_address(
        &self,
        from: &GalacticAddress,
        n: usize,
    ) -> Vec<(SystemId, f64)> {
        if n == 0 {
            return Vec::new();
        }
        let mut results: Vec<_> = self
            .systems
            .iter()
            .filter(|(id, _)| id.reality_index() == from.reality_index)
            .map(|(&id, system)| (id, from.distance_ly(&system.address)))
            .collect();
        sort_system_distances(&mut results);
        results.truncate(n);
        results
    }

    /// Find all active-galaxy systems within `radius_ly` of `from`.
    ///
    /// The radius boundary is inclusive; negative or NaN radii return no rows.
    /// A reference in another galaxy is excluded before distance calculation;
    /// use [`Self::systems_within_radius_from_address`] for explicit scope.
    pub fn systems_within_radius(
        &self,
        from: &GalacticAddress,
        radius_ly: f64,
    ) -> Vec<(SystemId, f64)> {
        if from.reality_index != self.active_galaxy {
            return Vec::new();
        }
        self.systems_within_radius_from_address(from, radius_ly)
    }

    /// Find every system within `radius_ly` in `from`'s galaxy.
    ///
    /// The radius boundary is inclusive. Negative and NaN radii return no rows;
    /// positive infinity includes every system in the selected galaxy.
    pub fn systems_within_radius_from_address(
        &self,
        from: &GalacticAddress,
        radius_ly: f64,
    ) -> Vec<(SystemId, f64)> {
        if radius_ly.is_nan() || radius_ly < 0.0 {
            return Vec::new();
        }
        let mut results: Vec<_> = self
            .systems
            .iter()
            .filter(|(id, _)| id.reality_index() == from.reality_index)
            .filter_map(|(&id, system)| {
                let distance = from.distance_ly(&system.address);
                (distance <= radius_ly).then_some((id, distance))
            })
            .collect();
        sort_system_distances(&mut results);
        results
    }

    /// Find the N nearest active-galaxy planets matching `filter`.
    ///
    /// Returns `(PlanetKey, &Planet, system_distance_ly)` tuples.
    /// A reference in another galaxy is excluded before distance calculation;
    /// use [`Self::nearest_planets_from_address`] for explicit scope.
    pub fn nearest_planets<'a>(
        &'a self,
        from: &GalacticAddress,
        n: usize,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        if from.reality_index != self.active_galaxy {
            return Vec::new();
        }
        self.nearest_planets_from_address(from, n, filter)
    }

    /// Find the N nearest matching planets in `from`'s galaxy without selection changes.
    pub fn nearest_planets_from_address<'a>(
        &'a self,
        from: &GalacticAddress,
        n: usize,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        if n == 0 {
            return Vec::new();
        }
        let mut results = Vec::new();
        for (&(system_id, planet_index), planet) in &self.planets {
            if system_id.reality_index() != from.reality_index || !filter.matches(planet) {
                continue;
            }
            let Some(system) = self.systems.get(&system_id) else {
                continue;
            };
            let distance = from.distance_ly(&system.address);
            results.push(((system_id, planet_index), planet, distance));
        }
        sort_planet_distances(&mut results);
        results.truncate(n);
        results
    }

    /// Find every active-galaxy planet within a radius matching `filter`.
    /// A reference in another galaxy is excluded before distance calculation;
    /// use [`Self::planets_within_radius_from_address`] for explicit scope.
    pub fn planets_within_radius<'a>(
        &'a self,
        from: &GalacticAddress,
        radius_ly: f64,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        if from.reality_index != self.active_galaxy {
            return Vec::new();
        }
        self.planets_within_radius_from_address(from, radius_ly, filter)
    }

    /// Find every matching planet within `radius_ly` in `from`'s galaxy.
    pub fn planets_within_radius_from_address<'a>(
        &'a self,
        from: &GalacticAddress,
        radius_ly: f64,
        filter: &BiomeFilter,
    ) -> Vec<(PlanetKey, &'a Planet, f64)> {
        let mut results = Vec::new();
        if radius_ly.is_nan() || radius_ly < 0.0 {
            return results;
        }
        for (&(system_id, planet_index), planet) in &self.planets {
            if system_id.reality_index() != from.reality_index || !filter.matches(planet) {
                continue;
            }
            let Some(system) = self.systems.get(&system_id) else {
                continue;
            };
            let distance = from.distance_ly(&system.address);
            if distance <= radius_ly {
                results.push(((system_id, planet_index), planet, distance));
            }
        }
        sort_planet_distances(&mut results);
        results
    }
}

fn sort_system_distances(results: &mut [(SystemId, f64)]) {
    results.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| system_id_order(a.0, b.0)));
}

fn sort_planet_distances(results: &mut [(PlanetKey, &Planet, f64)]) {
    results.sort_by(|a, b| {
        a.2.total_cmp(&b.2)
            .then_with(|| system_id_order(a.0.0, b.0.0))
            .then_with(|| a.0.1.cmp(&b.0.1))
    });
}

pub(crate) fn system_id_order(a: SystemId, b: SystemId) -> std::cmp::Ordering {
    a.reality_index()
        .cmp(&b.reality_index())
        .then_with(|| a.system_address().cmp(&b.system_address()))
}

#[cfg(test)]
mod galaxy_scope_tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};

    /// Build a model with systems at known positions for spatial testing.
    fn spatial_test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);
        for x in [1, 5, 10] {
            let addr = GalacticAddress::new(x, 0, 0, x as u16, 0, 0);
            let planet = Planet::new(0, Some(Biome::Barren), None, false, None, None);
            model.insert_system(System::new(addr, None, None, None, vec![planet]));
        }
        model
    }

    #[test]
    fn test_address_query_is_galaxy_qualified_and_active_query_rejects_foreign() {
        let mut model = spatial_test_model();
        let foreign = GalacticAddress::new(1, 0, 0, 1, 0, 9);
        model.insert_system(System::new(
            foreign,
            None,
            None,
            None,
            vec![Planet::new(0, Some(Biome::Barren), None, false, None, None)],
        ));
        model.set_active_galaxy(0);
        assert!(model.nearest_systems(&foreign, 3).is_empty());
        assert!(
            model
                .nearest_planets(&foreign, 3, &BiomeFilter::default())
                .is_empty()
        );
        assert!(
            model
                .systems_within_radius(&foreign, f64::INFINITY)
                .is_empty()
        );
        assert!(
            model
                .planets_within_radius(&foreign, f64::INFINITY, &BiomeFilter::default())
                .is_empty()
        );
        let results = model.nearest_systems_from_address(&foreign, 3);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.reality_index(), 9);
        assert_eq!(
            model.nearest_planets_from_address(&foreign, 1, &BiomeFilter::default())[0]
                .0
                .0
                .reality_index(),
            9
        );
        assert_eq!(
            model
                .systems_within_radius_from_address(&foreign, f64::INFINITY)
                .len(),
            1
        );
        assert_eq!(
            model
                .planets_within_radius_from_address(
                    &foreign,
                    f64::INFINITY,
                    &BiomeFilter::default()
                )
                .len(),
            1
        );
        assert_eq!(model.active_galaxy, 0);
    }

    #[test]
    fn test_all_spatial_apis_return_empty_for_unpopulated_galaxy() {
        let model = GalaxyModel::new();
        let address = GalacticAddress::new(0, 0, 0, 1, 0, 7);
        assert!(
            model
                .nearest_systems_from_address(&address, usize::MAX)
                .is_empty()
        );
        assert!(
            model
                .nearest_planets_from_address(&address, usize::MAX, &BiomeFilter::default())
                .is_empty()
        );
        assert!(
            model
                .systems_within_radius_from_address(&address, f64::INFINITY)
                .is_empty()
        );
        assert!(
            model
                .planets_within_radius_from_address(
                    &address,
                    f64::INFINITY,
                    &BiomeFilter::default()
                )
                .is_empty()
        );
    }

    #[test]
    fn test_spatial_limits_and_radius_edges() {
        let model = spatial_test_model();
        let origin = GalacticAddress::new(0, 0, 0, 0, 0, 0);
        assert!(model.nearest_systems(&origin, usize::MAX).len() >= 3);
        assert!(model.nearest_systems(&origin, 0).is_empty());
        assert!(model.systems_within_radius(&origin, f64::NAN).is_empty());
        assert!(model.systems_within_radius(&origin, -1.0).is_empty());
        assert_eq!(model.systems_within_radius(&origin, 400.0).len(), 1);
        assert_eq!(model.systems_within_radius(&origin, f64::INFINITY).len(), 3);
    }

    #[test]
    fn test_galaxy_collision_radius_boundary_and_numeric_tie_order() {
        let addresses = [
            GalacticAddress::new(0, 0, 0, 1, 0, 0),
            GalacticAddress::new(0, 0, 0, 2, 0, 0),
            GalacticAddress::new(1, 0, 0, 1, 0, 0),
            GalacticAddress::new(0, 0, 0, 1, 0, 9),
        ];
        let mut reversed = addresses;
        reversed.reverse();
        for insertion_order in [addresses, reversed] {
            let mut model = GalaxyModel::new();
            for address in insertion_order {
                model.insert_system(System::new(
                    address,
                    None,
                    None,
                    None,
                    vec![Planet::new(0, Some(Biome::Lush), None, false, None, None)],
                ));
            }
            let origin = addresses[0];
            let at_399 = model.systems_within_radius_from_address(&origin, 399.0);
            assert_eq!(at_399.len(), 2);
            assert!(at_399.iter().all(|(id, _)| id.reality_index() == 0));
            let at_400 = model.systems_within_radius_from_address(&origin, 400.0);
            assert_eq!(at_400.len(), 3);
            assert!(at_400.windows(2).all(|pair| {
                pair[0].1.total_cmp(&pair[1].1).is_le()
                    && (pair[0].1 != pair[1].1 || system_id_order(pair[0].0, pair[1].0).is_lt())
            }));
            let planets =
                model.nearest_planets_from_address(&origin, usize::MAX, &BiomeFilter::default());
            assert!(planets.iter().all(|(key, _, _)| key.0.reality_index() == 0));
            assert!(
                planets
                    .iter()
                    .all(|(key, _, _)| model.system(&key.0).is_some())
            );
        }
    }
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
        assert!(filter.matches(&planet));
    }

    #[test]
    fn test_matches_filter_biome_mismatch() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let filter = BiomeFilter {
            biome: Some(Biome::Toxic),
            ..Default::default()
        };
        assert!(!filter.matches(&planet));
    }

    #[test]
    fn test_matches_filter_biome_match() {
        let planet = Planet::new(0, Some(Biome::Toxic), None, false, None, None);
        let filter = BiomeFilter {
            biome: Some(Biome::Toxic),
            ..Default::default()
        };
        assert!(filter.matches(&planet));
    }

    #[test]
    fn test_matches_filter_infested_mismatch() {
        let planet = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let filter = BiomeFilter {
            infested: Some(true),
            ..Default::default()
        };
        assert!(!filter.matches(&planet));
    }

    #[test]
    fn test_matches_filter_named_only() {
        let unnamed = Planet::new(0, Some(Biome::Lush), None, false, None, None);
        let named = Planet::new(0, Some(Biome::Lush), None, false, Some("X".into()), None);
        let filter = BiomeFilter {
            named_only: true,
            ..Default::default()
        };
        assert!(!filter.matches(&unnamed));
        assert!(filter.matches(&named));
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
            biome_subtype: None,
            infested: Some(true),
            named_only: true,
        };
        assert!(filter.matches(&planet));
    }

    #[test]
    fn test_explicit_spatial_apis_return_exact_canonical_galaxy_views() {
        use nms_core::biome::BiomeSubType;
        use nms_core::system::System;

        for reverse in [false, true] {
            let mut model = GalaxyModel::new();
            let mut addresses = vec![
                GalacticAddress::new(0, 0, 0, 1, 0, 0),
                GalacticAddress::new(1, 0, 0, 2, 0, 0),
                GalacticAddress::new(0, 0, 0, 1, 0, 9),
                GalacticAddress::new(1, 0, 0, 2, 0, 9),
            ];
            if reverse {
                addresses.reverse();
            }
            for address in addresses {
                model.insert_system(System::new(
                    address,
                    None,
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        false,
                        Some(format!("P{}", address.reality_index)),
                        None,
                    )],
                ));
            }

            let zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
            let nine = GalacticAddress::new(0, 0, 0, 1, 0, 9);
            let expected_zero = [
                (SystemId::from_address(&zero), 0.0),
                (
                    SystemId::from_address(&GalacticAddress::new(1, 0, 0, 2, 0, 0)),
                    400.0,
                ),
            ];
            let expected_nine = [
                (SystemId::from_address(&nine), 0.0),
                (
                    SystemId::from_address(&GalacticAddress::new(1, 0, 0, 2, 0, 9)),
                    400.0,
                ),
            ];
            let before = (model.active_galaxy, model.player_state.clone());
            assert_eq!(
                model.nearest_systems_from_address(&zero, usize::MAX),
                expected_zero
            );
            assert_eq!(
                model.nearest_systems_from_address(&nine, usize::MAX),
                expected_nine
            );
            assert_eq!(
                model.systems_within_radius_from_address(&zero, 400.0),
                expected_zero
            );
            assert_eq!(
                model.systems_within_radius_from_address(&nine, 400.0),
                expected_nine
            );
            assert_eq!(
                model.systems_within_radius_from_address(&zero, 0.0),
                vec![expected_zero[0]]
            );
            assert_eq!(
                model.systems_within_radius_from_address(&zero, 399.0),
                vec![expected_zero[0]]
            );
            assert!(
                model
                    .systems_within_radius_from_address(&zero, -1.0)
                    .is_empty()
            );
            assert!(
                model
                    .systems_within_radius_from_address(&zero, f64::NAN)
                    .is_empty()
            );
            assert_eq!(
                model.systems_within_radius_from_address(&zero, f64::INFINITY),
                expected_zero
            );
            assert!(model.nearest_systems_from_address(&zero, 0).is_empty());
            let zero_planets =
                model.nearest_planets_from_address(&zero, usize::MAX, &BiomeFilter::default());
            let nine_planets =
                model.nearest_planets_from_address(&nine, usize::MAX, &BiomeFilter::default());
            assert_eq!(
                zero_planets
                    .iter()
                    .map(|(key, _, d)| (*key, *d))
                    .collect::<Vec<_>>(),
                vec![
                    ((expected_zero[0].0, 0), 0.0),
                    ((expected_zero[1].0, 0), 400.0)
                ]
            );
            assert_eq!(
                nine_planets
                    .iter()
                    .map(|(key, _, d)| (*key, *d))
                    .collect::<Vec<_>>(),
                vec![
                    ((expected_nine[0].0, 0), 0.0),
                    ((expected_nine[1].0, 0), 400.0)
                ]
            );
            assert!(std::ptr::eq(
                zero_planets[0].1,
                &model.planets[&zero_planets[0].0]
            ));
            assert!(std::ptr::eq(
                nine_planets[0].1,
                &model.planets[&nine_planets[0].0]
            ));
            assert_eq!(
                model
                    .planets_within_radius_from_address(&zero, 400.0, &BiomeFilter::default())
                    .len(),
                2
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(&nine, 400.0, &BiomeFilter::default())
                    .len(),
                2
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(&zero, 0.0, &BiomeFilter::default())
                    .len(),
                1
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(&zero, 399.0, &BiomeFilter::default())
                    .len(),
                1
            );
            assert!(
                model
                    .planets_within_radius_from_address(&zero, -1.0, &BiomeFilter::default())
                    .is_empty()
            );
            assert!(
                model
                    .planets_within_radius_from_address(&zero, f64::NAN, &BiomeFilter::default())
                    .is_empty()
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(
                        &zero,
                        f64::INFINITY,
                        &BiomeFilter::default()
                    )
                    .len(),
                2
            );
            assert!(
                model
                    .nearest_planets_from_address(&zero, 0, &BiomeFilter::default())
                    .is_empty()
            );

            model.set_active_galaxy(0);
            assert!(model.nearest_systems(&nine, usize::MAX).is_empty());
            assert!(model.systems_within_radius(&nine, f64::INFINITY).is_empty());
            assert!(
                model
                    .nearest_planets(&nine, usize::MAX, &BiomeFilter::default())
                    .is_empty()
            );
            assert!(
                model
                    .planets_within_radius(&nine, f64::INFINITY, &BiomeFilter::default())
                    .is_empty()
            );
            model.set_active_galaxy(9);
            assert_eq!(model.nearest_systems(&nine, usize::MAX).len(), 2);
            assert_eq!(model.systems_within_radius(&nine, f64::INFINITY).len(), 2);
            assert_eq!(
                model
                    .nearest_planets(&nine, usize::MAX, &BiomeFilter::default())
                    .len(),
                2
            );
            assert_eq!(
                model
                    .planets_within_radius(&nine, f64::INFINITY, &BiomeFilter::default())
                    .len(),
                2
            );
            assert!(model.nearest_systems(&zero, usize::MAX).is_empty());
            assert!(model.systems_within_radius(&zero, f64::INFINITY).is_empty());
            assert!(
                model
                    .nearest_planets(&zero, usize::MAX, &BiomeFilter::default())
                    .is_empty()
            );
            assert!(
                model
                    .planets_within_radius(&zero, f64::INFINITY, &BiomeFilter::default())
                    .is_empty()
            );
            model.set_active_galaxy(7);
            assert!(model.nearest_systems(&zero, usize::MAX).is_empty());
            assert!(model.systems_within_radius(&zero, f64::INFINITY).is_empty());
            assert!(
                model
                    .nearest_planets(&zero, usize::MAX, &BiomeFilter::default())
                    .is_empty()
            );
            assert!(
                model
                    .planets_within_radius(&zero, f64::INFINITY, &BiomeFilter::default())
                    .is_empty()
            );
            assert!(
                model
                    .nearest_systems_from_address(
                        &GalacticAddress::new(0, 0, 0, 1, 0, 7),
                        usize::MAX
                    )
                    .is_empty()
            );
            model.set_active_galaxy(0);
            assert_eq!((model.active_galaxy, model.player_state.clone()), before);
        }
    }

    #[test]
    fn test_spatial_ssi_ties_and_all_planet_filter_fields_are_exact() {
        use nms_core::biome::BiomeSubType;
        use nms_core::system::System;

        for reverse in [false, true] {
            let mut model = GalaxyModel::new();
            let mut systems = vec![
                System::new(
                    GalacticAddress::new(1, 0, 0, 9, 0, 0),
                    Some("Alpha".into()),
                    None,
                    None,
                    vec![],
                ),
                System::new(
                    GalacticAddress::new(-1, 0, 0, 2, 0, 0),
                    Some("Zulu".into()),
                    None,
                    None,
                    vec![],
                ),
            ];
            if reverse {
                systems.reverse();
            }
            for system in systems {
                model.insert_system(system);
            }
            let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
            let tied = model.nearest_systems_from_address(&origin, usize::MAX);
            assert_eq!(tied.len(), 2);
            let left = SystemId::from_address(&GalacticAddress::new(-1, 0, 0, 2, 0, 0));
            let right = SystemId::from_address(&GalacticAddress::new(1, 0, 0, 9, 0, 0));
            let (first, second) = if system_id_order(left, right).is_lt() {
                (left, right)
            } else {
                (right, left)
            };
            assert_eq!(tied[0].0, first);
            assert_eq!(tied[0].1, 400.0);
            assert_eq!(tied[1].0, second);
            assert_eq!(tied[1].1, 400.0);
        }

        let base = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let ssi_neighbor = GalacticAddress::new(0, 0, 0, 2, 0, 0);
        assert!((base.distance_ly(&ssi_neighbor) - 264.6828728).abs() < 1e-6);
        let mut ssi_model = GalaxyModel::new();
        ssi_model.insert_system(System::new(ssi_neighbor, None, None, None, vec![]));
        assert_eq!(
            ssi_model.nearest_systems_from_address(&base, 1)[0].1,
            base.distance_ly(&ssi_neighbor)
        );

        let mut model = GalaxyModel::new();
        let near = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let far = GalacticAddress::new(2, 0, 0, 2, 0, 9);
        model.insert_system(System::new(
            near,
            None,
            None,
            None,
            vec![
                Planet::new(
                    0,
                    Some(Biome::Toxic),
                    Some(BiomeSubType::LushRoomTemp),
                    true,
                    Some("Wrong biome".into()),
                    None,
                ),
                Planet::new(
                    1,
                    Some(Biome::Lush),
                    Some(BiomeSubType::LushHumid),
                    true,
                    Some("Wrong subtype".into()),
                    None,
                ),
                Planet::new(
                    2,
                    Some(Biome::Lush),
                    Some(BiomeSubType::LushRoomTemp),
                    false,
                    Some("Wrong infested".into()),
                    None,
                ),
                Planet::new(
                    3,
                    Some(Biome::Lush),
                    Some(BiomeSubType::LushRoomTemp),
                    true,
                    None,
                    None,
                ),
            ],
        ));
        model.insert_system(System::new(
            far,
            None,
            None,
            None,
            vec![Planet::new(
                0,
                Some(Biome::Lush),
                Some(BiomeSubType::LushRoomTemp),
                true,
                Some("Match".into()),
                None,
            )],
        ));
        let filter = BiomeFilter {
            biome: Some(Biome::Lush),
            biome_subtype: Some(BiomeSubType::LushRoomTemp),
            infested: Some(true),
            named_only: true,
        };
        let nearest = model.nearest_planets_from_address(&near, 1, &filter);
        let far_id = SystemId::from_address(&far);
        assert_eq!(
            nearest
                .iter()
                .map(|(key, _, distance)| (*key, *distance))
                .collect::<Vec<_>>(),
            vec![((far_id, 0), 800.0)]
        );
        assert_eq!(
            model
                .planets_within_radius_from_address(&near, 799.0, &filter)
                .len(),
            0
        );
        assert_eq!(
            model
                .planets_within_radius_from_address(&near, 800.0, &filter)
                .len(),
            1
        );
    }
}

#[cfg(test)]
mod iteration02_oracle_tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::{Biome, BiomeSubType};
    use nms_core::player::PlayerState;
    use nms_core::system::{Planet, System};

    fn addr(x: i16, ssi: u16, galaxy: u8) -> GalacticAddress {
        GalacticAddress::new(x, 0, 0, ssi, 0, galaxy)
    }

    fn id(address: GalacticAddress) -> SystemId {
        SystemId::from_address(&address)
    }

    #[test]
    fn explicit_and_active_spatial_apis_return_literal_galaxy_memberships_and_preserve_state() {
        let zero = addr(0, 1, 0);
        let zero_far = addr(1, 2, 0);
        let nine = addr(0, 1, 9);
        let nine_far = addr(1, 2, 9);
        let mut model = GalaxyModel::new();
        model.player_state = Some(PlayerState::new(zero, 0, None, None, 7, 8, 9));
        for address in [zero, zero_far, nine, nine_far] {
            model.insert_system(System::new(
                address,
                Some(format!(
                    "{}:{}",
                    address.reality_index,
                    address.solar_system_index()
                )),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    Some(BiomeSubType::LushRoomTemp),
                    false,
                    Some("named".into()),
                    None,
                )],
            ));
        }
        let filter = BiomeFilter::default();
        let state = (model.player_state.clone(), model.active_galaxy);
        let expected_zero = vec![(id(zero), 0.0), (id(zero_far), 400.0)];
        let expected_nine = vec![(id(nine), 0.0), (id(nine_far), 400.0)];
        for active in [0, 9] {
            model.set_active_galaxy(active);
            let expected = if active == 0 {
                &expected_zero
            } else {
                &expected_nine
            };
            let active_reference = if active == 0 { zero } else { nine };
            assert_eq!(
                model.nearest_systems(&active_reference, usize::MAX),
                expected.clone()
            );
            assert_eq!(
                model.systems_within_radius(&active_reference, 400.0),
                expected.clone()
            );
            assert_eq!(
                model
                    .nearest_planets(&active_reference, usize::MAX, &filter)
                    .iter()
                    .map(|(k, _, d)| (*k, *d))
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|(sid, d)| ((*sid, 1), *d))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                model
                    .planets_within_radius(&active_reference, 400.0, &filter)
                    .iter()
                    .map(|(k, _, d)| (*k, *d))
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|(sid, d)| ((*sid, 1), *d))
                    .collect::<Vec<_>>()
            );
            for reference in [zero, nine] {
                let explicit = if reference.reality_index == 0 {
                    &expected_zero
                } else {
                    &expected_nine
                };
                assert_eq!(
                    model.nearest_systems_from_address(&reference, usize::MAX),
                    *explicit
                );
                assert_eq!(
                    model.systems_within_radius_from_address(&reference, 400.0),
                    *explicit
                );
                let planets = model.nearest_planets_from_address(&reference, usize::MAX, &filter);
                assert_eq!(
                    planets.iter().map(|(k, _, d)| (*k, *d)).collect::<Vec<_>>(),
                    explicit
                        .iter()
                        .map(|(sid, d)| ((*sid, 1), *d))
                        .collect::<Vec<_>>()
                );
                assert!(
                    planets
                        .iter()
                        .all(|(key, planet, _)| std::ptr::eq(*planet, &model.planets[key]))
                );
                assert_eq!(
                    model
                        .planets_within_radius_from_address(&reference, 400.0, &filter)
                        .iter()
                        .map(|(k, _, d)| (*k, *d))
                        .collect::<Vec<_>>(),
                    explicit
                        .iter()
                        .map(|(sid, d)| ((*sid, 1), *d))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    (model.player_state.clone(), model.active_galaxy),
                    (Some(PlayerState::new(zero, 0, None, None, 7, 8, 9)), active)
                );
            }
            assert_eq!(
                model.nearest_systems(&active_reference, 0),
                Vec::<(SystemId, f64)>::new()
            );
            assert_eq!(
                model.systems_within_radius(&active_reference, 0.0),
                vec![(if active == 0 { id(zero) } else { id(nine) }, 0.0)]
            );
            assert_eq!(state.0, model.player_state);
        }
        model.set_active_galaxy(7);
        assert_eq!(
            model.nearest_systems_from_address(&nine, usize::MAX),
            expected_nine
        );
        assert_eq!(
            model.nearest_systems(&nine, usize::MAX),
            Vec::<(SystemId, f64)>::new()
        );
        assert_eq!(
            model.systems_within_radius(&nine, 400.0),
            Vec::<(SystemId, f64)>::new()
        );
        assert!(model.nearest_planets(&nine, usize::MAX, &filter).is_empty());
        assert!(
            model
                .planets_within_radius(&nine, 400.0, &filter)
                .is_empty()
        );
        assert_eq!(state.0, model.player_state);
        assert_eq!(state.1, 0);
    }

    #[test]
    fn literal_tie_order_and_false_infested_filter_survive_both_insert_orders() {
        let left = addr(-1, 2, 9);
        let right = addr(1, 9, 9);
        let filter = BiomeFilter {
            biome: Some(Biome::Lush),
            biome_subtype: Some(BiomeSubType::LushRoomTemp),
            infested: Some(false),
            named_only: true,
        };
        for reverse in [false, true] {
            let mut model = GalaxyModel::new();
            let mut systems = vec![
                System::new(
                    left,
                    Some("Zulu".into()),
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        true,
                        Some("left".into()),
                        None,
                    )],
                ),
                System::new(
                    right,
                    Some("Alpha".into()),
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        true,
                        Some("right".into()),
                        None,
                    )],
                ),
            ];
            if reverse {
                systems.reverse();
            }
            for system in systems {
                model.insert_system(system);
            }
            let tied = model.nearest_systems_from_address(&addr(0, 5, 9), usize::MAX);
            assert_eq!(tied, vec![(id(left), 400.0), (id(right), 400.0)]);
            assert_eq!(
                model.nearest_systems_from_address(&addr(0, 5, 9), 1),
                vec![(id(left), 400.0)]
            );
            let mut uncertainty_model = GalaxyModel::new();
            uncertainty_model.insert_system(System::new(addr(0, 5, 9), None, None, None, vec![]));
            let uncertain =
                uncertainty_model.nearest_systems_from_address(&addr(0, 6, 9), usize::MAX);
            assert!((uncertain[0].1 - 264.6828728).abs() < 1e-6);

            let near = addr(0, 20, 9);
            let far = addr(2, 21, 9);
            model.insert_system(System::new(
                near,
                None,
                None,
                None,
                vec![
                    Planet::new(
                        0,
                        Some(Biome::Toxic),
                        Some(BiomeSubType::LushRoomTemp),
                        false,
                        Some("wrong biome".into()),
                        None,
                    ),
                    Planet::new(
                        1,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushHumid),
                        false,
                        Some("wrong subtype".into()),
                        None,
                    ),
                    Planet::new(
                        2,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        true,
                        Some("wrong infestation".into()),
                        None,
                    ),
                    Planet::new(
                        3,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        false,
                        None,
                        None,
                    ),
                ],
            ));
            model.insert_system(System::new(
                far,
                None,
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(Biome::Lush),
                    Some(BiomeSubType::LushRoomTemp),
                    false,
                    Some("match".into()),
                    None,
                )],
            ));
            let matches = model.nearest_planets_from_address(&near, 1, &filter);
            assert_eq!(
                matches
                    .iter()
                    .map(|(key, _, d)| (*key, *d))
                    .collect::<Vec<_>>(),
                vec![((id(far), 0), 800.0)]
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(&near, 799.0, &filter)
                    .len(),
                0
            );
            assert_eq!(
                model
                    .planets_within_radius_from_address(&near, 800.0, &filter)
                    .len(),
                1
            );
        }
    }
}
