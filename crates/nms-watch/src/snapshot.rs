//! Save file snapshot for delta comparison.
//!
//! A `SaveSnapshot` captures the minimum state needed to detect changes
//! between two versions of a save file. Built by re-parsing the save
//! and extracting discovery records, player position, and bases.

use std::collections::HashMap;
use std::path::Path;

use nms_core::address::GalacticAddress;
use nms_core::player::PlayerBase;
use nms_core::system::{Planet, System, SystemId};
use nms_graph::extract::extract_systems;
use nms_save::model::SaveRoot;

/// A snapshot of save file state for diff comparison.
#[derive(Debug)]
pub struct SaveSnapshot {
    /// Systems keyed by SystemId.
    pub systems: HashMap<SystemId, System>,
    /// Planets keyed by (SystemId, planet_index).
    pub planets: HashMap<(SystemId, u8), Planet>,
    /// Bases keyed by lowercase name.
    pub bases: HashMap<String, PlayerBase>,
    /// Player's current galactic address.
    pub player_address: GalacticAddress,
}

impl SaveSnapshot {
    /// Build a snapshot by parsing a save file from disk.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let save = nms_save::parse_save_file(path)?;
        Ok(Self::from_save(&save))
    }

    /// Build a snapshot from an already-parsed save.
    pub fn from_save(save: &SaveRoot) -> Self {
        let extracted = extract_systems(save);

        let mut planets = HashMap::new();
        for (sys_id, system) in &extracted {
            for planet in &system.planets {
                planets.insert((*sys_id, planet.index), planet.clone());
            }
        }

        let ps = save.active_player_state();
        let mut bases = HashMap::new();
        for base in &ps.persistent_player_bases {
            let core_base = base.to_core_base();
            if !core_base.name.is_empty() {
                bases.insert(core_base.name.to_lowercase(), core_base);
            }
        }

        let player_address = save.to_core_player_state().current_address;

        Self {
            systems: extracted,
            planets,
            bases,
            player_address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_snapshot_from_save_systems() {
        let save = minimal_save();
        let snapshot = SaveSnapshot::from_save(&save);
        assert_eq!(snapshot.systems.len(), 2);
    }

    #[test]
    fn test_snapshot_from_save_planets() {
        let save = minimal_save();
        let snapshot = SaveSnapshot::from_save(&save);
        assert_eq!(snapshot.planets.len(), 1);
    }

    #[test]
    fn test_snapshot_from_save_bases() {
        let save = minimal_save();
        let snapshot = SaveSnapshot::from_save(&save);
        assert_eq!(snapshot.bases.len(), 1);
        assert!(snapshot.bases.contains_key("home base"));
    }

    #[test]
    fn test_snapshot_from_save_player_address() {
        let save = minimal_save();
        let snapshot = SaveSnapshot::from_save(&save);
        assert_eq!(snapshot.player_address.voxel_x(), 100);
        assert_eq!(snapshot.player_address.voxel_y(), 50);
        assert_eq!(snapshot.player_address.voxel_z(), -200);
    }
}
