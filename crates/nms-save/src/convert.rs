//! Legacy conversion from permissively deserialized raw save types to `nms-core` domain types.
//!
//! These compatibility helpers intentionally preserve the original parser's
//! permissive address behavior. Application code that needs checked source
//! addresses should use [`crate::ValidatedSave`] and its paired conversions.

use chrono::DateTime;

use crate::model::*;

impl RawDiscoveryRecord {
    /// Convert one recognized raw discovery row through the legacy address layout.
    ///
    /// The converter recognizes exactly `Planet`, `SolarSystem`, `Sector`,
    /// `Animal`, `Flora`, and `Mineral`; other `DT` strings return `None`. It
    /// uses galaxy zero for the packed address and does not extract a discovery
    /// display name from `DM.CN`. A nonempty `OWS.USN` becomes the discoverer,
    /// `FL.U` greater than zero becomes uploaded. A positive `OWS.TS` is cast
    /// from `u64` to `i64` and then converted when chrono can represent it, so
    /// not every positive source value yields a timestamp. For source-aware
    /// addresses, use the row/address pair from
    /// [`crate::ValidatedSave::discoveries`].
    ///
    /// # Examples
    ///
    /// ```
    /// let raw: nms_save::model::RawDiscoveryRecord = serde_json::from_str(
    ///     r#"{"DD":{"UA":"0x513300F79B1D82","DT":"Flora"},"OWS":{"USN":"Pilot","TS":1700000000},"FL":{"U":1}}"#,
    /// )?;
    /// let record = raw.to_core_record().ok_or_else(|| {
    ///     std::io::Error::new(std::io::ErrorKind::Other, "unrecognized discovery type")
    /// })?;
    /// assert_eq!(record.discoverer.as_deref(), Some("Pilot"));
    /// assert!(record.is_uploaded);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    /// Convert a typed base to a core base using the legacy packed-address layout.
    ///
    /// `HomePlanetBase` and `FreighterBase` map to their matching core variants;
    /// every other source string maps to `ExternalPlanetBase`. The address uses
    /// galaxy zero, position and name are copied, and an empty owner UID becomes
    /// `None`. The checked base projection is available from
    /// [`crate::ValidatedSave::bases`].
    ///
    /// # Examples
    ///
    /// ```
    /// let raw: nms_save::model::PersistentPlayerBase = serde_json::from_str(
    ///     r#"{"GalacticAddress":"0x40050003AB8C07","Name":"Home","Owner":{"UID":"123"},"BaseType":{"PersistentBaseTypes":"HomePlanetBase"}}"#,
    /// )?;
    /// let base = raw.to_core_base();
    /// assert_eq!(base.name, "Home");
    /// assert_eq!(base.owner_uid.as_deref(), Some("123"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    /// Return player-state data for the active context using legacy selection.
    ///
    /// Only the exact string `Expedition` selects the expedition context; every
    /// other value selects the base context. The selected fields may already
    /// contain serde defaults, so this accessor is not source validation.
    ///
    /// # Examples
    ///
    /// ```
    /// let root = nms_save::parse_save(br#"{"Version":1,"Platform":"Test","ActiveContext":"Expedition","CommonStateData":{},"BaseContext":{"PlayerStateData":{"Units":10}},"ExpeditionContext":{"PlayerStateData":{"Units":20}},"DiscoveryManagerData":{}}"#)?;
    /// assert_eq!(root.active_player_state().units, 20);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn active_player_state(&self) -> &PlayerStateData {
        match self.active_context.as_str() {
            "Expedition" => &self.expedition_context.player_state_data,
            _ => &self.base_context.player_state_data,
        }
    }

    /// Convert the selected typed state using the legacy address conversion.
    ///
    /// Current and previous addresses use the expanded typed components and
    /// the supplied reality index through the core 48-bit conversion; they do
    /// not decode a packed source address. Previous-address absence is detected
    /// only from zero X, Y, Z, and solar-system components; planet and galaxy
    /// do not participate in that sentinel. Currency values are cast from
    /// signed to unsigned, and the freighter address is currently `None`. Use
    /// [`crate::ValidatedSave::player_state`] when original source members must
    /// be checked before serde defaults or narrowing.
    ///
    /// # Examples
    ///
    /// ```
    /// let root = nms_save::parse_save(br#"{"Version":1,"Platform":"Test","ActiveContext":"Main","CommonStateData":{},"BaseContext":{"PlayerStateData":{"UniverseAddress":{"RealityIndex":9,"GalacticAddress":{"VoxelX":3,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2}},"Units":25}},"ExpeditionContext":{},"DiscoveryManagerData":{}}"#)?;
    /// let state = root.to_core_player_state();
    /// assert_eq!(state.current_reality, 9);
    /// assert_eq!(state.current_address.voxel_x(), 3);
    /// assert_eq!(state.freighter_address, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
