//! Conversion from raw save model types to `nms-core` domain types.

use chrono::DateTime;

use crate::model::*;

impl RawDiscoveryRecord {
    /// Convert to an `nms_core::DiscoveryRecord`, or `None` if the discovery
    /// type is unrecognized.
    pub fn to_core_record(&self) -> Option<nms_core::DiscoveryRecord> {
        let discovery_type = match self.dd.dt.as_str() {
            "Planet" => nms_core::Discovery::Planet,
            "SolarSystem" => nms_core::Discovery::SolarSystem,
            "Sector" => nms_core::Discovery::Sector,
            "Animal" => nms_core::Discovery::Animal,
            "Flora" => nms_core::Discovery::Flora,
            "Mineral" => nms_core::Discovery::Mineral,
            _ => return None,
        };

        let timestamp = if self.ows.ts > 0 {
            DateTime::from_timestamp(self.ows.ts as i64, 0)
        } else {
            None
        };

        let discoverer = if self.ows.usn.is_empty() {
            None
        } else {
            Some(self.ows.usn.clone())
        };

        let is_uploaded = self.fl.uploaded.unwrap_or(0) > 0;

        Some(nms_core::DiscoveryRecord::new(
            discovery_type,
            self.dd.ua.to_galactic_address(0),
            timestamp,
            None, // Discovery records in the save don't carry display names
            discoverer,
            is_uploaded,
        ))
    }
}

impl PersistentPlayerBase {
    /// Convert to an `nms_core::PlayerBase`.
    pub fn to_core_base(&self) -> nms_core::PlayerBase {
        let base_type = match self.base_type.persistent_base_types.as_str() {
            "HomePlanetBase" => nms_core::BaseType::HomePlanetBase,
            "FreighterBase" => nms_core::BaseType::FreighterBase,
            _ => nms_core::BaseType::ExternalPlanetBase,
        };

        nms_core::PlayerBase::new(
            self.name.clone(),
            base_type,
            self.galactic_address.to_galactic_address(0),
            self.position,
            if self.owner.uid.is_empty() {
                None
            } else {
                Some(self.owner.uid.clone())
            },
        )
    }
}

impl SaveRoot {
    /// Get `PlayerStateData` for the active context.
    pub fn active_player_state(&self) -> &PlayerStateData {
        match self.active_context.as_str() {
            "Expedition" => &self.expedition_context.player_state_data,
            _ => &self.base_context.player_state_data,
        }
    }

    /// Convert active player state to `nms_core::PlayerState`.
    pub fn to_core_player_state(&self) -> nms_core::PlayerState {
        let ps = self.active_player_state();
        let ua = &ps.universe_address;
        let current_address = ua.galactic_address.to_galactic_address(ua.reality_index);

        let prev_ua = &ps.previous_universe_address;
        let previous_address = if prev_ua.galactic_address.voxel_x == 0
            && prev_ua.galactic_address.voxel_y == 0
            && prev_ua.galactic_address.voxel_z == 0
            && prev_ua.galactic_address.solar_system_index == 0
        {
            None
        } else {
            Some(
                prev_ua
                    .galactic_address
                    .to_galactic_address(prev_ua.reality_index),
            )
        };

        nms_core::PlayerState::new(
            current_address,
            ua.reality_index,
            previous_address,
            None, // freighter_address not yet extracted
            ps.units as u64,
            ps.nanites as u64,
            ps.specials as u64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_record_to_core() {
        let raw = RawDiscoveryRecord {
            dd: DiscoveryData {
                ua: PackedGalacticAddress(0x513300F79B1D82),
                dt: "Flora".into(),
                vp: vec![],
            },
            dm: serde_json::Value::Object(serde_json::Map::new()),
            ows: OwnershipData {
                lid: String::new(),
                uid: "12345".into(),
                usn: "TestUser".into(),
                ptk: "ST".into(),
                ts: 1700000000,
            },
            fl: DiscoveryFlags {
                created: Some(1),
                uploaded: Some(1),
            },
            rid: Some("abc".into()),
        };

        let core = raw.to_core_record().unwrap();
        assert_eq!(core.discovery_type, nms_core::Discovery::Flora);
        assert_eq!(core.discoverer.as_deref(), Some("TestUser"));
        assert!(core.is_uploaded);
        assert!(core.timestamp.is_some());
    }

    #[test]
    fn unknown_discovery_type_returns_none() {
        let raw = RawDiscoveryRecord {
            dd: DiscoveryData {
                ua: PackedGalacticAddress(0),
                dt: "UnknownType".into(),
                vp: vec![],
            },
            dm: serde_json::Value::Null,
            ows: OwnershipData::default(),
            fl: DiscoveryFlags::default(),
            rid: None,
        };
        assert!(raw.to_core_record().is_none());
    }

    #[test]
    fn base_to_core() {
        let base = PersistentPlayerBase {
            base_version: 8,
            galactic_address: PackedGalacticAddress(0x40050003AB8C07),
            position: [100.0, 200.0, 300.0],
            forward: [1.0, 0.0, 0.0],
            last_update_timestamp: 1700000000,
            objects: vec![],
            rid: String::new(),
            owner: OwnershipData {
                lid: String::new(),
                uid: "76561198025707979".into(),
                usn: String::new(),
                ptk: "ST".into(),
                ts: 0,
            },
            name: "My Base".into(),
            base_type: BaseTypeWrapper {
                persistent_base_types: "HomePlanetBase".into(),
            },
            last_edited_by_id: String::new(),
            last_edited_by_username: String::new(),
            game_mode: None,
        };

        let core = base.to_core_base();
        assert_eq!(core.name, "My Base");
        assert_eq!(core.base_type, nms_core::BaseType::HomePlanetBase);
        assert_eq!(core.owner_uid.as_deref(), Some("76561198025707979"));
    }

    #[test]
    fn galactic_address_object_to_core() {
        let obj = GalacticAddressObject {
            voxel_x: 1699,
            voxel_y: -2,
            voxel_z: 165,
            solar_system_index: 369,
            planet_index: 0,
        };
        let addr = obj.to_galactic_address(0);
        assert_eq!(addr.voxel_x(), 1699);
        assert_eq!(addr.voxel_y(), -2);
        assert_eq!(addr.voxel_z(), 165);
        assert_eq!(addr.solar_system_index(), 369);
        assert_eq!(addr.planet_index(), 0);
        assert_eq!(addr.reality_index, 0);
    }

    #[test]
    fn active_context_main() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "test"},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {"Units": 999}
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {"Units": 111}
            },
            "DiscoveryManagerData": {"DiscoveryData-v1": {"Store": {"Record": []}}}
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        assert_eq!(save.active_player_state().units, 999);
    }

    #[test]
    fn active_context_expedition() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Expedition",
            "CommonStateData": {"SaveName": "test"},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {"Units": 999}
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {"Units": 111}
            },
            "DiscoveryManagerData": {"DiscoveryData-v1": {"Store": {"Record": []}}}
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        assert_eq!(save.active_player_state().units, 111);
    }

    #[test]
    fn to_core_player_state_with_previous() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "test"},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {
                        "RealityIndex": 0,
                        "GalacticAddress": {"VoxelX": 100, "VoxelY": 10, "VoxelZ": 200, "SolarSystemIndex": 369, "PlanetIndex": 2}
                    },
                    "PreviousUniverseAddress": {
                        "RealityIndex": 0,
                        "GalacticAddress": {"VoxelX": 50, "VoxelY": 5, "VoxelZ": 100, "SolarSystemIndex": 505, "PlanetIndex": 0}
                    },
                    "Units": 1000000,
                    "Nanites": 5000,
                    "Specials": 200
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"Store": {"Record": []}}}
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        let state = save.to_core_player_state();
        assert_eq!(state.units, 1000000);
        assert_eq!(state.nanites, 5000);
        assert_eq!(state.quicksilver, 200);
        assert_eq!(state.current_address.voxel_x(), 100);
        assert!(state.previous_address.is_some());
        assert_eq!(state.previous_address.unwrap().voxel_x(), 50);
    }

    #[test]
    fn to_core_player_state_zero_previous_is_none() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {
                        "RealityIndex": 0,
                        "GalacticAddress": {"VoxelX": 100, "VoxelY": 10, "VoxelZ": 200, "SolarSystemIndex": 369, "PlanetIndex": 0}
                    },
                    "PreviousUniverseAddress": {
                        "RealityIndex": 0,
                        "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}
                    },
                    "Units": 500
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"Store": {"Record": []}}}
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        let state = save.to_core_player_state();
        assert!(state.previous_address.is_none());
    }
}
