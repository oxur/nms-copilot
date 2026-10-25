//! Additive delta computation between save snapshots.
//!
//! [`compute_delta`] reports newly keyed systems, planets, and bases,
//! changed bases under an existing lowercase name key, and player movement.
//! It does not report removals or changed metadata under existing system or
//! planet keys; result-vector order follows hash-map iteration.
//!
//! # Examples
//!
//! The literal snapshots below show the positive, reverse, and unchanged
//! cases. The changed Atlas/Garden metadata at an existing key is not a
//! reported change, while the new Beta/Cinder records and base changes are.
//!
//! ```
//! use std::collections::HashMap;
//! use nms_core::address::GalacticAddress;
//! use nms_core::biome::Biome;
//! use nms_core::player::{BaseType, PlayerBase};
//! use nms_core::system::{Planet, System, SystemId};
//! use nms_watch::{compute_delta, SaveSnapshot};
//!
//! fn address(planet: u8, galaxy: u8) -> GalacticAddress {
//!     GalacticAddress::new(3, 0, 0, 1, planet, galaxy)
//! }
//! fn original() -> SaveSnapshot {
//!     let address = address(2, 9);
//!     let id = SystemId::from_address(&address);
//!     let planet = Planet::new(2, Some(Biome::Lush), None, false, Some("Garden".into()), Some(7));
//!     let system = System::new(address, Some("Atlas".into()), None, None, vec![planet.clone()]);
//!     let base = PlayerBase::new("Home".into(), BaseType::HomePlanetBase, address, [0.0, 0.0, 0.0], None);
//!     SaveSnapshot {
//!         systems: HashMap::from([(id, system)]),
//!         planets: HashMap::from([((id, 2), planet)]),
//!         bases: HashMap::from([("home".into(), base)]),
//!         player_address: address,
//!     }
//! }
//! fn changed() -> SaveSnapshot {
//!     let old_address = address(2, 9);
//!     let old_id = SystemId::from_address(&old_address);
//!     let old_planet = Planet::new(2, Some(Biome::Lush), None, false, Some("Renamed Garden".into()), Some(7));
//!     let old_system = System::new(old_address, Some("Renamed".into()), None, None, vec![old_planet.clone()]);
//!     let new_address = address(1, 0);
//!     let new_id = SystemId::from_address(&new_address);
//!     let new_planet = Planet::new(1, Some(Biome::Toxic), None, false, Some("Cinder".into()), Some(8));
//!     let new_system = System::new(new_address, Some("Beta".into()), None, None, vec![new_planet.clone()]);
//!     let home = PlayerBase::new("HOME".into(), BaseType::HomePlanetBase, old_address, [1.0, 2.0, 3.0], None);
//!     let outpost = PlayerBase::new("Outpost".into(), BaseType::ExternalPlanetBase, new_address, [4.0, 5.0, 6.0], None);
//!     SaveSnapshot {
//!         systems: HashMap::from([(old_id, old_system), (new_id, new_system)]),
//!         planets: HashMap::from([((old_id, 2), old_planet), ((new_id, 1), new_planet)]),
//!         bases: HashMap::from([("home".into(), home), ("outpost".into(), outpost)]),
//!         player_address: new_address,
//!     }
//! }
//! let old = original();
//! let new = changed();
//! let forward = compute_delta(&old, &new);
//! let beta_address = GalacticAddress::new(3, 0, 0, 1, 1, 0);
//! let beta_planet = Planet::new(1, Some(Biome::Toxic), None, false, Some("Cinder".into()), Some(8));
//! let beta = System::new(beta_address, Some("Beta".into()), None, None, vec![beta_planet.clone()]);
//! assert_eq!(forward.new_systems, vec![beta]);
//! assert_eq!(forward.new_planets, vec![(SystemId::from_address(&beta_address), beta_planet)]);
//! assert_eq!(forward.player_moved.as_ref().map(|m| (m.from, m.to)), Some((GalacticAddress::new(3, 0, 0, 1, 2, 9), GalacticAddress::new(3, 0, 0, 1, 1, 0))));
//! assert_eq!(forward.new_bases, vec![PlayerBase::new("Outpost".into(), BaseType::ExternalPlanetBase, beta_address, [4.0, 5.0, 6.0], None)]);
//! assert_eq!(forward.modified_bases, vec![PlayerBase::new("HOME".into(), BaseType::HomePlanetBase, GalacticAddress::new(3, 0, 0, 1, 2, 9), [1.0, 2.0, 3.0], None)]);
//! assert_eq!(forward.change_count(), 5);
//! assert!(!forward.is_empty());
//! let reverse = compute_delta(&new, &old);
//! assert!(reverse.new_systems.is_empty());
//! assert!(reverse.new_planets.is_empty());
//! assert_eq!(reverse.player_moved.as_ref().map(|m| (m.from, m.to)), Some((GalacticAddress::new(3, 0, 0, 1, 1, 0), GalacticAddress::new(3, 0, 0, 1, 2, 9))));
//! assert!(reverse.new_bases.is_empty());
//! assert_eq!(reverse.modified_bases, vec![PlayerBase::new("Home".into(), BaseType::HomePlanetBase, GalacticAddress::new(3, 0, 0, 1, 2, 9), [0.0, 0.0, 0.0], None)]);
//! assert_eq!(reverse.change_count(), 2);
//! let unchanged = compute_delta(&new, &new);
//! assert!(unchanged.is_empty());
//! assert_eq!(unchanged.change_count(), 0);
//! ```

use nms_core::delta::{PlayerMoved, SaveDelta};

use crate::snapshot::SaveSnapshot;

/// Compute additive observations between two snapshots.
///
/// New system and planet records are selected by absent map keys, not by
/// comparing values at keys already present in `old`. Base changes use the
/// lowercase name key, and movement compares the full galactic address.
/// Removed records and changes to existing system or planet metadata are
/// omitted. Vectors are collected from hash maps and have no stable order.
///
/// # Examples
///
/// See the [module example](crate::delta#examples) for complete literal
/// forward, reverse, and unchanged expectations.
pub fn compute_delta(old: &SaveSnapshot, new: &SaveSnapshot) -> SaveDelta {
    // 1. Check player position change (cheapest check first)
    let player_moved = if old.player_address != new.player_address {
        Some(PlayerMoved {
            from: old.player_address,
            to: new.player_address,
        })
    } else {
        None
    };

    // 2. Find new systems (in new but not in old, keyed by SystemId)
    let new_systems = new
        .systems
        .iter()
        .filter(|(id, _)| !old.systems.contains_key(id))
        .map(|(_, sys)| sys.clone())
        .collect();

    // 3. Find new planets (in new but not in old, keyed by (SystemId, planet_index))
    let new_planets = new
        .planets
        .iter()
        .filter(|(key, _)| !old.planets.contains_key(key))
        .map(|((sys_id, _), planet)| (*sys_id, planet.clone()))
        .collect();

    // 4. Find new bases (by lowercase name key)
    let new_bases = new
        .bases
        .iter()
        .filter(|(name, _)| !old.bases.contains_key(name.as_str()))
        .map(|(_, base)| base.clone())
        .collect();

    // 5. Find modified bases (same name, different content)
    let modified_bases = new
        .bases
        .iter()
        .filter(|(name, new_base)| {
            old.bases
                .get(name.as_str())
                .is_some_and(|old_base| old_base != *new_base)
        })
        .map(|(_, base)| base.clone())
        .collect();

    SaveDelta {
        new_systems,
        new_planets,
        player_moved,
        new_bases,
        modified_bases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::SaveSnapshot;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::player::{BaseType, PlayerBase};
    use nms_core::system::{Planet, System, SystemId};
    use nms_save::model::SaveRoot;

    fn minimal_save() -> SaveRoot {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                    "Units": 1000000, "Nanites": 5000, "Specials": 200,
                    "PersistentPlayerBases": [
                        {
                            "BaseVersion": 8, "GalacticAddress": "0x050003AB8C07",
                            "Position": [0.0, 0.0, 0.0], "Forward": [1.0, 0.0, 0.0],
                            "LastUpdateTimestamp": 1700000000, "Objects": [], "RID": "",
                            "Owner": {"LID": "", "UID": "123", "USN": "Test", "PTK": "ST", "TS": 0},
                            "Name": "Home Base",
                            "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
                            "LastEditedById": "", "LastEditedByUsername": ""
                        }
                    ]
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100, "ReserveManaged": 100,
                    "Store": {
                        "Record": [
                            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": ["0xABCD"]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xDEAD", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                            {"DD": {"UA": "0x0A0002001234", "DT": "SolarSystem", "VP": ["0x1234"]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
                        ]
                    }
                }
            }
        }"#;
        nms_save::parse_save(json.as_bytes()).unwrap()
    }

    fn test_snapshot() -> SaveSnapshot {
        let save = minimal_save();
        SaveSnapshot::from_save(&save)
    }

    #[test]
    fn test_delta_empty_when_identical() {
        let snapshot = test_snapshot();
        let snapshot2 = test_snapshot();
        let delta = compute_delta(&snapshot, &snapshot2);
        assert!(delta.is_empty());
        assert_eq!(delta.change_count(), 0);
    }

    #[test]
    fn test_delta_detects_new_system() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let addr = GalacticAddress::new(500, 10, -300, 0x999, 0, 0);
        let sys_id = SystemId::from_address(&addr);
        new.systems.insert(
            sys_id,
            System::new(addr, Some("New".into()), None, None, vec![]),
        );

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_systems.len(), 1);
        assert!(!delta.is_empty());
    }

    #[test]
    fn test_delta_detects_player_moved() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        new.player_address = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = compute_delta(&old, &new);
        assert!(delta.player_moved.is_some());
        let moved = delta.player_moved.unwrap();
        assert_eq!(moved.to.voxel_x(), 999);
    }

    #[test]
    fn test_delta_detects_new_base() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let addr = GalacticAddress::new(100, 50, -200, 42, 0, 0);
        let base = PlayerBase::new(
            "New Base".into(),
            BaseType::HomePlanetBase,
            addr,
            [0.0, 0.0, 0.0],
            None,
        );
        new.bases.insert("new base".into(), base);

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_bases.len(), 1);
    }

    #[test]
    fn test_delta_detects_new_planet() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let sys_id = *new.systems.keys().next().unwrap();
        new.planets.insert(
            (sys_id, 5),
            Planet::new(5, Some(Biome::Lava), None, false, None, None),
        );

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_planets.len(), 1);
    }

    #[test]
    fn test_delta_detects_modified_base() {
        let old = test_snapshot();
        let mut new = test_snapshot();

        // Modify the existing "home base" by changing its position
        if let Some(base) = new.bases.get_mut("home base") {
            base.position = [99.0, 99.0, 99.0];
        }

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.modified_bases.len(), 1);
        // Not a new base, so new_bases should be empty
        assert!(delta.new_bases.is_empty());
    }

    #[test]
    fn test_delta_no_false_positives_on_existing() {
        let snapshot1 = test_snapshot();
        let snapshot2 = test_snapshot();
        let delta = compute_delta(&snapshot1, &snapshot2);
        assert!(delta.is_empty());
    }

    #[test]
    fn test_delta_change_count_sums_correctly() {
        let old = test_snapshot();
        let mut new = test_snapshot();

        // Add a new system
        let addr = GalacticAddress::new(500, 10, -300, 0x999, 0, 0);
        let sys_id = SystemId::from_address(&addr);
        new.systems
            .insert(sys_id, System::new(addr, None, None, None, vec![]));

        // Move player
        new.player_address = GalacticAddress::new(1, 0, 0, 1, 0, 0);

        // Add a new base
        let base = PlayerBase::new(
            "Another Base".into(),
            BaseType::ExternalPlanetBase,
            addr,
            [0.0, 0.0, 0.0],
            None,
        );
        new.bases.insert("another base".into(), base);

        let delta = compute_delta(&old, &new);
        // 1 system + 1 player_moved + 1 base = 3
        assert_eq!(delta.change_count(), 3);
    }

    fn identity_snapshot(galaxies: &[u8]) -> SaveSnapshot {
        let mut systems = std::collections::HashMap::new();
        let mut planets = std::collections::HashMap::new();
        for &galaxy in galaxies {
            let address = GalacticAddress::from_packed(0x150003AB8C07, galaxy);
            let name = if galaxy == 0 { "Zero" } else { "Nine" };
            let biome = if galaxy == 0 {
                Biome::Lush
            } else {
                Biome::Toxic
            };
            let planet = Planet::new(
                1,
                Some(biome),
                None,
                false,
                Some(format!("{name} marker")),
                None,
            );
            let system = System::new(address, Some(name.into()), None, None, vec![planet.clone()]);
            let system_id = SystemId::from_address(&address);
            systems.insert(system_id, system);
            planets.insert((system_id, 1), planet);
        }

        SaveSnapshot {
            systems,
            planets,
            bases: std::collections::HashMap::new(),
            player_address: GalacticAddress::new(0, 0, 0, 1, 0, 0),
        }
    }

    #[test]
    fn test_compute_delta_and_apply_add_same_coordinates_in_other_galaxy() {
        for (old_galaxy, added_galaxy) in [(0, 9), (9, 0)] {
            let old = identity_snapshot(&[old_galaxy]);
            let new = identity_snapshot(&[old_galaxy, added_galaxy]);
            let delta = compute_delta(&old, &new);
            assert_eq!(delta.new_systems.len(), 1);
            assert_eq!(delta.new_planets.len(), 1);
            assert!(delta.player_moved.is_none());
            assert!(delta.new_bases.is_empty());
            assert!(delta.modified_bases.is_empty());

            let old_system = old.systems.values().next().unwrap().clone();
            let mut model = nms_graph::GalaxyModel::new();
            model.insert_system(old_system);
            model.apply_delta(&delta);

            let old_address = GalacticAddress::from_packed(0x150003AB8C07, old_galaxy);
            let added_address = GalacticAddress::from_packed(0x150003AB8C07, added_galaxy);
            let old_id = SystemId::from_address(&old_address);
            let added_id = SystemId::from_address(&added_address);
            assert_eq!(model.system_count(), 2);
            assert_eq!(model.planet_count(), 2);
            assert_eq!(
                model.system(&old_id).unwrap().name.as_deref(),
                Some(if old_galaxy == 0 { "Zero" } else { "Nine" })
            );
            assert_eq!(
                model.system(&added_id).unwrap().name.as_deref(),
                Some(if added_galaxy == 0 { "Zero" } else { "Nine" })
            );
            assert_eq!(
                model.planets.get(&(old_id, 1)).unwrap().biome,
                Some(if old_galaxy == 0 {
                    Biome::Lush
                } else {
                    Biome::Toxic
                })
            );
            assert_eq!(
                model.planets.get(&(added_id, 1)).unwrap().biome,
                Some(if added_galaxy == 0 {
                    Biome::Lush
                } else {
                    Biome::Toxic
                })
            );
            assert_eq!(
                model.planets.get(&(old_id, 1)).unwrap().name.as_deref(),
                Some(if old_galaxy == 0 {
                    "Zero marker"
                } else {
                    "Nine marker"
                })
            );
            assert_eq!(
                model.planets.get(&(added_id, 1)).unwrap().name.as_deref(),
                Some(if added_galaxy == 0 {
                    "Zero marker"
                } else {
                    "Nine marker"
                })
            );
            assert_eq!(model.address_to_id.get(&old_id), Some(&old_id));
            assert_eq!(model.address_to_id.get(&added_id), Some(&added_id));

            model.apply_delta(&delta);
            assert_eq!(model.system_count(), 2);
            assert_eq!(model.planet_count(), 2);
            assert_eq!(model.address_to_id.len(), 2);
            assert_eq!(model.node_map.len(), 2);
            assert_eq!(model.spatial_size(), 2);
        }
    }

    fn movement_snapshot(
        addresses: &[GalacticAddress],
        player_address: GalacticAddress,
    ) -> SaveSnapshot {
        let mut systems = std::collections::HashMap::new();
        let mut planets = std::collections::HashMap::new();
        for &address in addresses {
            let (name, leaf, biome) = if address.reality_index == 0 {
                ("Origin-0", "Leaf-0", Biome::Lush)
            } else {
                ("Destination-9", "Leaf-9", Biome::Toxic)
            };
            let system_id = SystemId::from_address(&address);
            let planet = Planet::new(0, Some(biome), None, false, Some(leaf.into()), None);
            systems.insert(
                system_id,
                System::new(address, Some(name.into()), None, None, vec![planet.clone()]),
            );
            planets.insert((system_id, 0), planet);
        }

        SaveSnapshot {
            systems,
            planets,
            bases: std::collections::HashMap::new(),
            player_address,
        }
    }

    #[test]
    fn test_compute_delta_cross_reality_movement_preserves_qualified_discoveries() {
        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let id0 = SystemId::from_address(&a0);
        let id9 = SystemId::from_address(&a9);

        for (from, to, expected_zero, expected_nine) in [
            (a0, a9, "Origin-0", "Destination-9"),
            (a9, a0, "Origin-0", "Destination-9"),
        ] {
            let old = movement_snapshot(&[from], from);
            let new = movement_snapshot(&[a0, a9], to);
            let delta = compute_delta(&old, &new);
            let moved = delta.player_moved.as_ref().unwrap();
            assert_eq!(moved.from, from);
            assert_eq!(moved.to, to);
            assert_eq!(delta.new_systems.len(), 1);
            let added_id = SystemId::from_address(&delta.new_systems[0].address);
            assert_eq!(added_id, SystemId::from_address(&to));
            assert_eq!(delta.new_planets.len(), 1);
            assert_eq!(delta.new_planets[0].0, added_id);

            let existing = if from == a0 {
                System::new(
                    a0,
                    Some("Origin-0".into()),
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Leaf-0".into()),
                        None,
                    )],
                )
            } else {
                System::new(
                    a9,
                    Some("Destination-9".into()),
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(Biome::Toxic),
                        None,
                        false,
                        Some("Leaf-9".into()),
                        None,
                    )],
                )
            };
            let mut model = nms_graph::GalaxyModel::new();
            model.insert_system(existing);
            model.apply_delta(&delta);
            assert_eq!(
                model.system(&id0).unwrap().name.as_deref(),
                Some(expected_zero)
            );
            assert_eq!(
                model.system(&id9).unwrap().name.as_deref(),
                Some(expected_nine)
            );
            assert_eq!(
                model.planets.get(&(id0, 0)).unwrap().name.as_deref(),
                Some("Leaf-0")
            );
            assert_eq!(
                model.planets.get(&(id9, 0)).unwrap().name.as_deref(),
                Some("Leaf-9")
            );
            assert_eq!(
                model.planets.get(&(id0, 0)).unwrap().biome,
                Some(Biome::Lush)
            );
            assert_eq!(
                model.planets.get(&(id9, 0)).unwrap().biome,
                Some(Biome::Toxic)
            );
            assert_eq!(model.system_count(), 2);
            assert_eq!(model.planet_count(), 2);
        }

        let same_reality = movement_snapshot(&[a0], GalacticAddress::new(1, 0, 0, 1, 0, 0));
        let changed = movement_snapshot(&[a0], GalacticAddress::new(2, 0, 0, 1, 0, 0));
        let delta = compute_delta(&same_reality, &changed);
        assert_eq!(delta.player_moved.unwrap().to.voxel_x(), 2);
    }
}
