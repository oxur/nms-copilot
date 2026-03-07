//! Typed Rust structs for deobfuscated NMS save file JSON.
//!
//! Only the fields NMS Copilot uses are deserialized — unknown fields
//! are silently ignored (no `deny_unknown_fields`).

use serde::{Deserialize, Serialize, de};
use std::fmt;

/// Top-level save file structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct SaveRoot {
    pub version: u32,
    pub platform: String,
    pub active_context: String,
    pub common_state_data: CommonStateData,
    pub base_context: GameContext,
    pub expedition_context: GameContext,
    pub discovery_manager_data: DiscoveryManagerData,
}

/// Shared state across game contexts (name, play time).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct CommonStateData {
    #[serde(default)]
    pub save_name: String,
    #[serde(default)]
    pub total_play_time: u64,
}

/// A game context (Base or Expedition), containing mode and player state.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct GameContext {
    #[serde(default)]
    pub game_mode: u32,
    #[serde(default)]
    pub player_state_data: PlayerStateData,
}

/// Subset of PlayerStateData fields needed by NMS Copilot.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct PlayerStateData {
    #[serde(default)]
    pub universe_address: UniverseAddress,

    #[serde(default)]
    pub previous_universe_address: UniverseAddress,

    #[serde(default)]
    pub save_summary: String,

    /// Units can be negative in actual saves.
    #[serde(default)]
    pub units: i64,

    #[serde(default)]
    pub nanites: i64,

    /// Quicksilver is stored as "Specials" in the JSON.
    #[serde(default)]
    pub specials: i64,

    #[serde(default)]
    pub persistent_player_bases: Vec<PersistentPlayerBase>,

    #[serde(default)]
    pub health: u32,

    #[serde(default)]
    pub time_alive: u64,
}

/// Universe address wrapping a galactic address with reality (galaxy) index.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct UniverseAddress {
    #[serde(default)]
    pub reality_index: u8,
    #[serde(default)]
    pub galactic_address: GalacticAddressObject,
}

/// Galactic address in expanded object form (used in PlayerStateData).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct GalacticAddressObject {
    #[serde(default)]
    pub voxel_x: i16,
    #[serde(default)]
    pub voxel_y: i8,
    #[serde(default)]
    pub voxel_z: i16,
    #[serde(default)]
    pub solar_system_index: u16,
    #[serde(default)]
    pub planet_index: u8,
}

impl GalacticAddressObject {
    /// Convert to the core `GalacticAddress` type.
    pub fn to_galactic_address(&self, reality_index: u8) -> nms_core::GalacticAddress {
        nms_core::GalacticAddress::new(
            self.voxel_x,
            self.voxel_y,
            self.voxel_z,
            self.solar_system_index,
            self.planet_index,
            reality_index,
        )
    }
}

/// Galactic address in packed form — hex string `"0x..."` or bare integer.
///
/// Used for bases and discoveries where the address is a single value.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct PackedGalacticAddress(pub u64);

impl<'de> Deserialize<'de> for PackedGalacticAddress {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = PackedGalacticAddress;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a hex string like \"0x...\" or an integer")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(PackedGalacticAddress(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(PackedGalacticAddress(v as u64))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let hex = v
                    .strip_prefix("0x")
                    .or_else(|| v.strip_prefix("0X"))
                    .unwrap_or(v);
                u64::from_str_radix(hex, 16)
                    .map(PackedGalacticAddress)
                    .map_err(|_| de::Error::custom(format!("invalid hex galactic address: {v}")))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl PackedGalacticAddress {
    /// Convert to the core `GalacticAddress` type.
    pub fn to_galactic_address(&self, reality_index: u8) -> nms_core::GalacticAddress {
        nms_core::GalacticAddress::from_packed(self.0, reality_index)
    }
}

/// Top-level discovery manager data.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryManagerData {
    #[serde(rename = "DiscoveryData-v1", default)]
    pub discovery_data_v1: DiscoveryDataV1,
}

/// Discovery data version 1 container.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct DiscoveryDataV1 {
    #[serde(default)]
    pub reserve_store: u32,
    #[serde(default)]
    pub reserve_managed: u32,
    #[serde(default)]
    pub store: DiscoveryStore,
}

/// Store containing all discovery records.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct DiscoveryStore {
    #[serde(default)]
    pub record: Vec<RawDiscoveryRecord>,
}

/// A raw discovery record from the save file.
///
/// Field names (DD, DM, OWS, FL, RID) are the actual JSON keys — not obfuscated.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RawDiscoveryRecord {
    /// Discovery data.
    #[serde(rename = "DD")]
    pub dd: DiscoveryData,

    /// Discovery metadata (usually empty object).
    #[serde(rename = "DM", default)]
    pub dm: serde_json::Value,

    /// Ownership data.
    #[serde(rename = "OWS")]
    pub ows: OwnershipData,

    /// Flags (C=created, U=uploaded).
    #[serde(rename = "FL", default)]
    pub fl: DiscoveryFlags,

    /// Record ID (base64 hash).
    #[serde(rename = "RID", default)]
    pub rid: Option<String>,
}

/// Discovery data sub-object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryData {
    /// Universe address (packed galactic address).
    #[serde(rename = "UA")]
    pub ua: PackedGalacticAddress,

    /// Discovery type: "Flora", "Planet", "Sector", "SolarSystem", "Mineral", "Animal".
    #[serde(rename = "DT")]
    pub dt: String,

    /// Variable-purpose data array — opaque for now.
    #[serde(rename = "VP", default)]
    pub vp: Vec<serde_json::Value>,
}

/// Ownership data for discoveries and bases.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OwnershipData {
    /// Local ID (may be empty).
    #[serde(rename = "LID", default)]
    pub lid: String,

    /// User ID (Steam ID, PSN ID, etc.).
    #[serde(rename = "UID", default)]
    pub uid: String,

    /// Username.
    #[serde(rename = "USN", default)]
    pub usn: String,

    /// Platform token: "ST" = Steam, "PS" = PlayStation, etc.
    #[serde(rename = "PTK", default)]
    pub ptk: String,

    /// Timestamp (Unix epoch seconds).
    #[serde(rename = "TS", default)]
    pub ts: u64,
}

/// Discovery flags sub-object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryFlags {
    /// Created flag.
    #[serde(rename = "C", default)]
    pub created: Option<u8>,

    /// Uploaded flag.
    #[serde(rename = "U", default)]
    pub uploaded: Option<u8>,
}

/// A player-owned base from PersistentPlayerBases.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct PersistentPlayerBase {
    #[serde(default)]
    pub base_version: u32,

    pub galactic_address: PackedGalacticAddress,

    #[serde(default)]
    pub position: [f32; 3],

    #[serde(default)]
    pub forward: [f32; 3],

    #[serde(default)]
    pub last_update_timestamp: u64,

    /// Base objects — stored as opaque JSON.
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,

    #[serde(default, rename = "RID")]
    pub rid: String,

    #[serde(default)]
    pub owner: OwnershipData,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub base_type: BaseTypeWrapper,

    #[serde(default)]
    pub last_edited_by_id: String,

    #[serde(default)]
    pub last_edited_by_username: String,

    #[serde(default)]
    pub game_mode: Option<GameModeWrapper>,
}

/// Wrapper for `{"PersistentBaseTypes": "HomePlanetBase"}`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseTypeWrapper {
    #[serde(rename = "PersistentBaseTypes", default)]
    pub persistent_base_types: String,
}

/// Wrapper for `{"PresetGameMode": "Normal"}`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GameModeWrapper {
    #[serde(rename = "PresetGameMode", default)]
    pub preset_game_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_packed_galactic_address_hex_string() {
        let json = r#""0x40050003AB8C07""#;
        let addr: PackedGalacticAddress = serde_json::from_str(json).unwrap();
        assert_eq!(addr.0, 0x40050003AB8C07);
    }

    #[test]
    fn parse_packed_galactic_address_integer() {
        let json = "4716909145249443";
        let addr: PackedGalacticAddress = serde_json::from_str(json).unwrap();
        assert_eq!(addr.0, 4716909145249443);
    }

    #[test]
    fn parse_packed_galactic_address_zero() {
        let json = "0";
        let addr: PackedGalacticAddress = serde_json::from_str(json).unwrap();
        assert_eq!(addr.0, 0);
    }

    #[test]
    fn parse_galactic_address_object() {
        let json = r#"{"VoxelX": 1699, "VoxelY": -2, "VoxelZ": 165, "SolarSystemIndex": 369, "PlanetIndex": 0}"#;
        let addr: GalacticAddressObject = serde_json::from_str(json).unwrap();
        assert_eq!(addr.voxel_x, 1699);
        assert_eq!(addr.voxel_y, -2);
        assert_eq!(addr.voxel_z, 165);
        assert_eq!(addr.solar_system_index, 369);
        assert_eq!(addr.planet_index, 0);
    }

    #[test]
    fn parse_universe_address() {
        let json = r#"{
            "RealityIndex": 0,
            "GalacticAddress": {"VoxelX": 1699, "VoxelY": -2, "VoxelZ": 165, "SolarSystemIndex": 369, "PlanetIndex": 0}
        }"#;
        let ua: UniverseAddress = serde_json::from_str(json).unwrap();
        assert_eq!(ua.reality_index, 0);
        assert_eq!(ua.galactic_address.voxel_x, 1699);
    }

    #[test]
    fn parse_discovery_record() {
        let json = r#"{
            "DD": {"UA": "0x513300F79B1D82", "DT": "Flora", "VP": ["0xD6911E7B1D31085E", "0x6454A508A8EBE022"]},
            "DM": {},
            "OWS": {"LID": "", "UID": "76561197977678185", "USN": "Allasar", "PTK": "ST", "TS": 1757022865},
            "FL": {"C": 1, "U": 1},
            "RID": "RAyjId1/Ea20q4fOptVHGQ3K99CKxs8609foiDDzCDc="
        }"#;
        let rec: RawDiscoveryRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.dd.dt, "Flora");
        assert_eq!(rec.dd.ua.0, 0x513300F79B1D82);
        assert_eq!(rec.ows.usn, "Allasar");
        assert_eq!(rec.ows.ptk, "ST");
        assert_eq!(rec.ows.ts, 1757022865);
        assert_eq!(rec.fl.created, Some(1));
        assert_eq!(rec.fl.uploaded, Some(1));
    }

    #[test]
    fn parse_discovery_record_integer_ua() {
        let json = r#"{
            "DD": {"UA": 498082938293634, "DT": "SolarSystem", "VP": ["0xD9F543C64FB79748"]},
            "DM": {},
            "OWS": {"LID": "", "UID": "76561197962153408", "USN": "Cereal 4th", "PTK": "ST", "TS": 1756915149},
            "FL": {"C": 1, "U": 1}
        }"#;
        let rec: RawDiscoveryRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.dd.dt, "SolarSystem");
        assert_eq!(rec.dd.ua.0, 498082938293634);
        assert!(rec.rid.is_none());
    }

    #[test]
    fn parse_discovery_record_sector() {
        let json = r#"{
            "DD": {"UA": "0x61C100039060B9", "DT": "Sector", "VP": ["0x8665527833B28EE7", 512]},
            "DM": {},
            "OWS": {"LID": "76561198024880757", "UID": "76561198024880757", "USN": "Ascalon", "PTK": "ST", "TS": 1771036917},
            "FL": {"U": 1}
        }"#;
        let rec: RawDiscoveryRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.dd.dt, "Sector");
        assert_eq!(rec.fl.created, None);
        assert_eq!(rec.fl.uploaded, Some(1));
    }

    #[test]
    fn parse_persistent_player_base() {
        let json = r#"{
            "BaseVersion": 8,
            "OriginalBaseVersion": 8,
            "GalacticAddress": "0x40050003AB8C07",
            "Position": [17267.421875, 3043.806640625, 63082.875],
            "Forward": [0.913, -0.333, -0.233],
            "UserData": 0,
            "LastUpdateTimestamp": 1738887563,
            "Objects": [],
            "RID": "",
            "Owner": {"LID": "76561198025707979", "UID": "76561198025707979", "USN": "", "PTK": "ST", "TS": 1700427307},
            "Name": "Gugestor Colony",
            "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
            "LastEditedById": "",
            "LastEditedByUsername": "",
            "ScreenshotAt": [-0.601, 0.052, 0.797],
            "ScreenshotPos": [-16.56, 14.89, 95.18],
            "GameMode": {"PresetGameMode": "Normal"},
            "Difficulty": {}
        }"#;
        let base: PersistentPlayerBase = serde_json::from_str(json).unwrap();
        assert_eq!(base.name, "Gugestor Colony");
        assert_eq!(base.base_type.persistent_base_types, "HomePlanetBase");
        assert_eq!(base.galactic_address.0, 0x40050003AB8C07);
        assert_eq!(base.owner.uid, "76561198025707979");
    }

    #[test]
    fn parse_common_state_data() {
        let json = r#"{"SaveName": "main - Steam", "TotalPlayTime": 2464349, "UsesThirdPersonCharacterCam": true}"#;
        let csd: CommonStateData = serde_json::from_str(json).unwrap();
        assert_eq!(csd.save_name, "main - Steam");
        assert_eq!(csd.total_play_time, 2464349);
    }

    #[test]
    fn parse_player_state_data_minimal() {
        let json = r#"{
            "UniverseAddress": {
                "RealityIndex": 0,
                "GalacticAddress": {"VoxelX": 1699, "VoxelY": -2, "VoxelZ": 165, "SolarSystemIndex": 369, "PlanetIndex": 0}
            },
            "PreviousUniverseAddress": {
                "RealityIndex": 0,
                "GalacticAddress": {"VoxelX": 1699, "VoxelY": -2, "VoxelZ": 165, "SolarSystemIndex": 505, "PlanetIndex": 0}
            },
            "SaveSummary": "In the Rabirad-Motom system",
            "Units": -919837762,
            "Nanites": 272127,
            "Specials": 2230,
            "Health": 180,
            "TimeAlive": 1435361,
            "PersistentPlayerBases": []
        }"#;
        let ps: PlayerStateData = serde_json::from_str(json).unwrap();
        assert_eq!(ps.units, -919837762);
        assert_eq!(ps.nanites, 272127);
        assert_eq!(ps.specials, 2230);
        assert_eq!(ps.universe_address.galactic_address.voxel_x, 1699);
        assert_eq!(ps.universe_address.galactic_address.solar_system_index, 369);
    }

    #[test]
    fn parse_minimal_save_root() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 1000000,
                    "Nanites": 5000,
                    "Specials": 200,
                    "PersistentPlayerBases": []
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0,
                    "Nanites": 0,
                    "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100,
                    "ReserveManaged": 100,
                    "Store": {"Record": []}
                }
            }
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        assert_eq!(save.version, 4720);
        assert_eq!(save.platform, "Mac|Final");
        assert_eq!(save.active_context, "Main");
        assert_eq!(save.common_state_data.save_name, "test");
        assert_eq!(save.common_state_data.total_play_time, 100);
        assert_eq!(save.base_context.player_state_data.units, 1000000);
        assert_eq!(
            save.discovery_manager_data
                .discovery_data_v1
                .store
                .record
                .len(),
            0
        );
    }
}
