//! Mutable atlas snapshot for delta comparison.
//!
//! A [`SaveSnapshot`] contains system and flat-planet maps, bases keyed by
//! lowercased nonempty names, and the current physical player address. It is
//! not the complete player state, source version, or explicit discovery-name
//! observation map. Its public maps can be changed independently, so callers
//! are responsible for keeping related entries consistent.
//!
//! [`from_save`]: SaveSnapshot::from_save
//! [`from_validated_save`]: SaveSnapshot::from_validated_save
//! [`from_file`]: SaveSnapshot::from_file
//!
//! # Examples
//!
//! This in-memory example contrasts checked address extraction with the
//! legacy low-48-bit path. `SystemId` is galaxy-qualified and planet-zeroed;
//! the checked stored system address comes from this SolarSystem row at P0,
//! while the current player address retains P2/GG9. For these supplied packed
//! values only, legacy decoding yields P0/SSI265/GG0; that is not a universal
//! planet-zero rule, since an orphan row can supply a planet-bearing address.
//!
//! ```
//! use std::collections::HashMap;
//! use nms_core::address::GalacticAddress;
//! use nms_core::biome::Biome;
//! use nms_core::system::{Planet, System, SystemId};
//! use nms_watch::SaveSnapshot;
//!
//! let json = br#"{"Version":4720,"Platform":"Test","ActiveContext":"Main","CommonStateData":{},"BaseContext":{"PlayerStateData":{"UniverseAddress":{"RealityIndex":9,"GalacticAddress":{"VoxelX":3,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2}},"Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]}},"ExpeditionContext":{},"DiscoveryManagerData":{"DiscoveryData-v1":{"Store":{"Record":[{"DD":{"UA":"0x00010900000003","DT":"SolarSystem"},"DM":{"CN":"Atlas"},"OWS":{"USN":"Explorer"}},{"DD":{"UA":"0x20010900000003","DT":"Planet","VP":["0x2A","0x10000"]},"DM":{"CN":"Garden"},"OWS":{}}]}}}}"#;
//! let checked_save = nms_save::parse_validated_save(json)?;
//! let checked = SaveSnapshot::from_validated_save(&checked_save);
//! let checked_address = GalacticAddress::new(3, 0, 0, 1, 0, 9);
//! let checked_id = SystemId::from_address(&checked_address);
//! let checked_planet = Planet::new(2, Some(Biome::Lush), None, true, Some("Garden".into()), Some(42));
//! assert_eq!(checked.systems, HashMap::from([(checked_id, System::new(checked_address, Some("Atlas".into()), Some("Explorer".into()), None, vec![checked_planet.clone()]))]));
//! assert_eq!(checked.planets, HashMap::from([((checked_id, 2), checked_planet)]));
//! assert!(checked.bases.is_empty());
//! assert_eq!(checked.player_address, GalacticAddress::new(3, 0, 0, 1, 2, 9));
//!
//! let legacy = SaveSnapshot::from_save(checked_save.root());
//! let legacy_address = GalacticAddress::new(3, 0, 0, 265, 0, 0);
//! let legacy_id = SystemId::from_address(&legacy_address);
//! let legacy_planet = Planet::new(0, Some(Biome::Lush), None, true, None, Some(42));
//! assert_eq!(legacy.systems, HashMap::from([(legacy_id, System::new(legacy_address, None, Some("Explorer".into()), None, vec![legacy_planet.clone()]))]));
//! assert_eq!(legacy.planets, HashMap::from([((legacy_id, 0), legacy_planet)]));
//! assert!(legacy.bases.is_empty());
//! assert_eq!(legacy.player_address, GalacticAddress::new(3, 0, 0, 1, 2, 9));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::HashMap;
use std::path::Path;

use nms_core::address::GalacticAddress;
use nms_core::player::PlayerBase;
use nms_core::system::{Planet, System, SystemId};
use nms_graph::extract::extract_systems;
use nms_save::model::SaveRoot;

/// Mutable subset of save state used for additive atlas comparisons.
///
/// This public value type does not enforce agreement between `systems`,
/// `planets`, and `bases`. Constructors populate the maps from a save, but
/// callers that mutate them must maintain any desired consistency.
#[derive(Debug)]
pub struct SaveSnapshot {
    /// Systems keyed by galaxy-qualified, planet-zeroed [`nms_core::system::SystemId`].
    pub systems: HashMap<SystemId, System>,
    /// Flat planet records keyed by `(SystemId, planet_index)`.
    pub planets: HashMap<(SystemId, u8), Planet>,
    /// Nonempty base names lowercased as keys, without trimming.
    ///
    /// Equal lowercase names replace an earlier entry, including across
    /// galaxies.
    pub bases: HashMap<String, PlayerBase>,
    /// Current physical address; unlike [`nms_core::system::SystemId`], this may retain P bits.
    pub player_address: GalacticAddress,
}

impl SaveSnapshot {
    /// Parse a save file with address validation and build its atlas snapshot.
    ///
    /// This uses [`nms_save::parse_validated_save_file`] and then the checked
    /// extraction path; it does not write the save or coordinate an app cache.
    /// See the [module example](crate::snapshot#examples) for the checked-versus-legacy map
    /// comparison.
    ///
    /// # Errors
    ///
    /// Returns the parser's boxed save error for file I/O, decompression, JSON,
    /// or address-validation failures.
    ///
    /// # Panics
    ///
    /// The underlying parser can panic for its documented malformed
    /// compressed-size offset on 32-bit targets or an invalid bundled mapping.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let save = nms_save::parse_validated_save_file(path)?;
        Ok(Self::from_validated_save(&save))
    }

    /// Build a snapshot through the legacy save-model extraction path.
    ///
    /// This does not revalidate raw source addresses. Packed discoveries and
    /// base addresses use the legacy low-48-bit conversion with galaxy zero,
    /// and discovery `DM.CN` names are ignored. The expanded current player
    /// address retains its supplied reality and planet components. Use
    /// [`from_validated_save`](Self::from_validated_save) when source addresses
    /// must be checked before deserialization. The resulting maps contain
    /// atlas discoveries and bases, not complete player/version/name state.
    /// See the [module example](crate::snapshot#examples) for this fixture's
    /// address distinction.
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

    /// Build a snapshot from checked, pre-deserialization save addresses.
    ///
    /// This delegates discovery extraction to
    /// [`nms_graph::extract::extract_systems_from_validated`]. That two-pass
    /// extraction processes SolarSystem rows before Planet rows: the first
    /// SolarSystem row for a system key supplies its stored address and
    /// metadata; the first planet row for each system/index supplies that
    /// planet, and an orphan planet row can create a system from its own
    /// checked address. `SystemId` keys remain planet-zeroed even when such a
    /// stored address retains P bits. Extracted systems use Personal
    /// provenance. The first row for an effective name key reserves it even
    /// when `DM.CN` is absent; a nonempty literal name is modeled, while an
    /// empty or missing name becomes `None`. Biome subtype is unavailable from
    /// `VP`. The current address is copied from the converted player state.
    /// This does not add a current-location placeholder or build a graph or
    /// cache. See the [module example](crate::snapshot#examples) for complete
    /// checked and legacy maps.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
    /// let snapshot = nms_watch::SaveSnapshot::from_validated_save(&save);
    /// println!("{} systems", snapshot.systems.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_validated_save(save: &nms_save::ValidatedSave) -> Self {
        let extracted = nms_graph::extract::extract_systems_from_validated(save);
        let mut planets = HashMap::new();
        for (sys_id, system) in &extracted {
            for planet in &system.planets {
                planets.insert((*sys_id, planet.index), planet.clone());
            }
        }
        let mut bases = HashMap::new();
        for base in save.bases() {
            if !base.name.is_empty() {
                bases.insert(base.name.to_lowercase(), base.clone());
            }
        }
        Self {
            systems: extracted,
            planets,
            bases,
            player_address: save.player_state().current_address,
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
