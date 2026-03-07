//! Delta computation between save snapshots.
//!
//! Compares two `SaveSnapshot` values and produces a `SaveDelta` listing
//! all changes: new discoveries, player movement, new/modified bases.

use nms_core::delta::{PlayerMoved, SaveDelta};

use crate::snapshot::SaveSnapshot;

/// Compute the delta between two snapshots.
///
/// Compares discovery records by system address, player position, and base
/// lists. Checks player position first (O(1)) since it changes most often.
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
}
