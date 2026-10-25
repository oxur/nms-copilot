//! Legacy typed Rust structs for deobfuscated NMS save file JSON.
//!
//! Only the fields NMS Copilot uses are deserialized — unknown fields
//! are silently ignored (no `deny_unknown_fields`). Application input should
//! cross the checked [`crate::ValidatedSave`] boundary before these compatibility
//! types are used to construct queryable state.

use serde::{Deserialize, Serialize, de};
use std::fmt;

/// Top-level save file structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct SaveRoot {
    /// Save version from the `Version` member.
    pub version: u32,
    /// Platform string from `Platform`, preserved verbatim.
    pub platform: String,
    /// Active context name, such as `Main` or `Expedition`.
    pub active_context: String,
    /// Shared save name and play-time data.
    pub common_state_data: CommonStateData,
    /// Typed subset of the `BaseContext` object.
    pub base_context: GameContext,
    /// Typed subset of the `ExpeditionContext` object.
    pub expedition_context: GameContext,
    /// Modeled discovery-manager subset; unknown source members are not retained.
    pub discovery_manager_data: DiscoveryManagerData,
}

/// Shared state across game contexts (name, play time).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct CommonStateData {
    /// Save display name from `SaveName`, defaulting to empty when absent.
    #[serde(default)]
    pub save_name: String,
    /// Accumulated play time from `TotalPlayTime`, defaulting to zero.
    #[serde(default)]
    pub total_play_time: u64,
}

/// A game context (Base or Expedition), containing mode and player state.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct GameContext {
    /// Numeric `GameMode` value, defaulting to zero when absent.
    #[serde(default)]
    pub game_mode: u32,
    /// Modeled player-state subset, defaulting to its empty value.
    #[serde(default)]
    pub player_state_data: PlayerStateData,
}

/// Subset of PlayerStateData fields needed by NMS Copilot.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct PlayerStateData {
    /// Current universe address; legacy serde defaults it when absent.
    #[serde(default)]
    pub universe_address: UniverseAddress,

    /// Previous universe address; legacy conversion applies its own zero sentinel.
    #[serde(default)]
    pub previous_universe_address: UniverseAddress,

    /// Opaque save-summary text, defaulting to empty.
    #[serde(default)]
    pub save_summary: String,

    /// Units can be negative in saves; legacy conversion casts this to `u64`.
    #[serde(default)]
    pub units: i64,

    /// Nanites balance, defaulting to zero.
    #[serde(default)]
    pub nanites: i64,

    /// Quicksilver is stored as `Specials` in JSON; legacy conversion casts to `u64`.
    #[serde(default)]
    pub specials: i64,

    /// Persistent bases modeled from this player-state object.
    #[serde(default)]
    pub persistent_player_bases: Vec<PersistentPlayerBase>,

    /// Health value, defaulting to zero.
    #[serde(default)]
    pub health: u32,

    /// Elapsed lifetime value, defaulting to zero.
    #[serde(default)]
    pub time_alive: u64,
}

/// Universe address wrapping a galactic address with reality (galaxy) index.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct UniverseAddress {
    /// Reality/galaxy index, defaulting to zero in legacy serde parsing.
    #[serde(default)]
    pub reality_index: u8,
    /// Expanded address components, defaulting component-by-component to zero.
    #[serde(default)]
    pub galactic_address: GalacticAddressObject,
}

/// Galactic address in expanded object form (used in PlayerStateData).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct GalacticAddressObject {
    /// Signed X voxel coordinate using the legacy Rust `i16` range.
    #[serde(default)]
    pub voxel_x: i16,
    /// Signed Y voxel coordinate using the legacy Rust `i8` range.
    #[serde(default)]
    pub voxel_y: i8,
    /// Signed Z voxel coordinate using the legacy Rust `i16` range.
    #[serde(default)]
    pub voxel_z: i16,
    /// Solar-system index, defaulting to zero.
    #[serde(default)]
    pub solar_system_index: u16,
    /// Planet index, defaulting to zero.
    #[serde(default)]
    pub planet_index: u8,
}

impl GalacticAddressObject {
    /// Convert stored components to a core address with the supplied galaxy index.
    ///
    /// Core construction masks components to their encoded widths. This
    /// conversion does not validate original JSON member presence or narrower
    /// packed-save coordinate ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_save::model::GalacticAddressObject;
    /// let source: GalacticAddressObject = serde_json::from_str(
    ///     r#"{"VoxelX":3,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2}"#,
    /// )?;
    /// let address = source.to_galactic_address(9);
    /// assert_eq!(address.reality_index, 9);
    /// assert_eq!(address.voxel_x(), 3);
    /// assert_eq!(address.solar_system_index(), 1);
    /// assert_eq!(address.planet_index(), 2);
    /// assert_eq!(address.packed(), 0x200100000003);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

/// Legacy packed galactic-address value used by typed bases and discoveries.
///
/// Serde accepts an unsigned JSON integer, casts a signed integer to `u64`, or
/// parses a hexadecimal string with an optional `0x`/`0X` prefix. This
/// compatibility representation does not enforce the checked decoder's 56-bit
/// bound or validate original field presence before serde defaults.
/// Serialization emits the numeric `u64`, not a hexadecimal string.
///
/// Conversion masks into the core 48-bit portal layout and uses the galaxy
/// index supplied by the caller; it does not recover the galaxy byte from the
/// save's 56-bit discovery/base layout.
///
/// # Examples
///
/// ```
/// use nms_save::address::decode_discovery_address;
/// use nms_save::model::PackedGalacticAddress;
/// let source: serde_json::Value = serde_json::from_str(r#""0x20010900000003""#)?;
/// let legacy: PackedGalacticAddress = serde_json::from_value(source.clone())?;
/// assert_eq!(legacy.0, 0x20010900000003);
/// assert_eq!(legacy.to_galactic_address(0).reality_index, 0);
/// let checked = decode_discovery_address(Some(&source), "/DD/UA")?;
/// assert_eq!(checked.reality_index, 9);
/// assert_eq!(checked.planet_index(), 2);
/// assert_eq!(checked.voxel_x(), 3);
/// assert_eq!(checked.solar_system_index(), 1);
/// assert_eq!(checked.packed(), 0x200100000003);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// Convert the legacy packed value using the supplied galaxy index.
    ///
    /// This masks into the core 48-bit portal layout; it does not recover the
    /// source 56-bit galaxy byte or validate the original packed value. Use
    /// [`crate::address::decode_discovery_address`] or the corresponding
    /// checked decoder when the source JSON is available.
    pub fn to_galactic_address(&self, reality_index: u8) -> nms_core::GalacticAddress {
        nms_core::GalacticAddress::from_packed(self.0, reality_index)
    }
}

/// Top-level discovery manager data.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryManagerData {
    /// Version-1 discovery payload, defaulting when absent.
    #[serde(rename = "DiscoveryData-v1", default)]
    pub discovery_data_v1: DiscoveryDataV1,
}

/// Discovery data version 1 container.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct DiscoveryDataV1 {
    /// Reserved store count from `ReserveStore`, defaulting to zero.
    #[serde(default)]
    pub reserve_store: u32,
    /// Reserved managed count from `ReserveManaged`, defaulting to zero.
    #[serde(default)]
    pub reserve_managed: u32,
    /// Discovery rows held by this versioned container.
    #[serde(default)]
    pub store: DiscoveryStore,
}

/// Store containing all discovery records.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct DiscoveryStore {
    /// Modeled discovery rows from `Record`, defaulting to an empty list.
    #[serde(default)]
    pub record: Vec<RawDiscoveryRecord>,
}

/// A raw discovery record from the save file.
///
/// Field names (DD, DM, OWS, FL, RID) are the actual JSON keys — not obfuscated.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RawDiscoveryRecord {
    /// Discovery payload from the actual, non-obfuscated `DD` key.
    #[serde(rename = "DD")]
    pub dd: DiscoveryData,

    /// Opaque value from `DM`; serde defaults it to JSON null when absent.
    #[serde(rename = "DM", default)]
    pub dm: serde_json::Value,

    /// Observed ownership fields from `OWS`; these do not establish identity.
    #[serde(rename = "OWS")]
    pub ows: OwnershipData,

    /// Optional creation/upload flags from `FL`, defaulting when absent.
    #[serde(rename = "FL", default)]
    pub fl: DiscoveryFlags,

    /// Optional record identifier from `RID`; its contents remain opaque.
    #[serde(rename = "RID", default)]
    pub rid: Option<String>,
}

/// Discovery data sub-object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryData {
    /// Packed address from `UA`; typed conversion uses the legacy 48-bit layout.
    #[serde(rename = "UA")]
    pub ua: PackedGalacticAddress,

    /// Source discovery-type string; the compatibility converter recognizes six exact values.
    #[serde(rename = "DT")]
    pub dt: String,

    /// Opaque variable-purpose values from `VP`; no typed interpretation is applied.
    #[serde(rename = "VP", default)]
    pub vp: Vec<serde_json::Value>,
}

/// Ownership data for discoveries and bases.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OwnershipData {
    /// Source-local identifier, defaulting to empty.
    #[serde(rename = "LID", default)]
    pub lid: String,

    /// Platform account identifier, defaulting to empty; it is observed metadata.
    #[serde(rename = "UID", default)]
    pub uid: String,

    /// Source username, defaulting to empty.
    #[serde(rename = "USN", default)]
    pub usn: String,

    /// Source platform token such as `ST` or `PS`, defaulting to empty.
    #[serde(rename = "PTK", default)]
    pub ptk: String,

    /// Source timestamp in Unix epoch seconds, defaulting to zero.
    #[serde(rename = "TS", default)]
    pub ts: u64,
}

/// Discovery flags sub-object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DiscoveryFlags {
    /// Optional numeric created flag as represented in the source JSON.
    #[serde(rename = "C", default)]
    pub created: Option<u8>,

    /// Optional numeric uploaded flag.
    ///
    /// The legacy conversion treats values greater than zero as uploaded.
    #[serde(rename = "U", default)]
    pub uploaded: Option<u8>,
}

/// A player-owned base from PersistentPlayerBases.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct PersistentPlayerBase {
    /// Base schema version, defaulting to zero when absent.
    #[serde(default)]
    pub base_version: u32,

    /// Packed source address; legacy conversion supplies galaxy zero.
    pub galactic_address: PackedGalacticAddress,

    /// Three stored position coordinates, defaulting to zeroes.
    #[serde(default)]
    pub position: [f32; 3],

    /// Three stored forward-vector coordinates, defaulting to zeroes.
    #[serde(default)]
    pub forward: [f32; 3],

    /// Last-update Unix timestamp, defaulting to zero.
    #[serde(default)]
    pub last_update_timestamp: u64,

    /// Base-object values retained as opaque JSON, not interpreted by this model.
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,

    /// Record identifier from `RID`, defaulting to an empty string.
    #[serde(default, rename = "RID")]
    pub rid: String,

    /// Observed owner metadata; no stronger identity inference is made.
    #[serde(default)]
    pub owner: OwnershipData,

    /// Base display name, defaulting to empty.
    #[serde(default)]
    pub name: String,

    /// Persistent base-type string, defaulting to its empty wrapper.
    #[serde(default)]
    pub base_type: BaseTypeWrapper,

    /// Last-editor identifier, defaulting to empty.
    #[serde(default)]
    pub last_edited_by_id: String,

    /// Last-editor name, defaulting to empty.
    #[serde(default)]
    pub last_edited_by_username: String,

    /// Optional preset game-mode wrapper; missing or JSON null becomes `None`.
    #[serde(default)]
    pub game_mode: Option<GameModeWrapper>,
}

/// Wrapper for `{"PersistentBaseTypes": "HomePlanetBase"}`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseTypeWrapper {
    /// Raw `PersistentBaseTypes` value, defaulting to empty.
    #[serde(rename = "PersistentBaseTypes", default)]
    pub persistent_base_types: String,
}

/// Wrapper for `{"PresetGameMode": "Normal"}`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GameModeWrapper {
    /// Raw `PresetGameMode` value, defaulting to empty.
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
