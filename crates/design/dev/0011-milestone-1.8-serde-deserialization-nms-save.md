# Milestone 1.8 -- Serde Deserialization (nms-save)

Typed Rust structs for the deobfuscated save file JSON, deserialized with serde. We only deserialize the fields NMS Copilot uses -- not the entire save.

## Crate: `nms-save`

Path: `crates/nms-save/`

### Dependencies to add to `crates/nms-save/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
serde_json = "1"
```

---

## Actual Save File JSON Structure (Deobfuscated, post-Worlds Part II / "Omega" format)

The save file is decompressed and deobfuscated JSON. Top-level keys (in order):

```json
{
  "Version": 4720,
  "Platform": "Mac|Final",
  "ActiveContext": "Main",
  "CommonStateData": { "SaveName": "...", "TotalPlayTime": 2464349, ... },
  "BaseContext": {
    "GameMode": 1,
    "PlayerStateData": { ... }
  },
  "ExpeditionContext": {
    "GameMode": 6,
    "PlayerStateData": { ... }
  },
  "DiscoveryManagerData": {
    "DiscoveryData-v1": {
      "ReserveStore": 3200,
      "ReserveManaged": 3250,
      "Store": {
        "Record": [ ... ]
      }
    }
  }
}
```

### Key observations from actual save data:

1. **GalacticAddress has TWO representations** depending on context:
   - **Object form** (in PlayerStateData.UniverseAddress): `{"VoxelX": 1699, "VoxelY": -2, "VoxelZ": 165, "SolarSystemIndex": 369, "PlanetIndex": 0}`
   - **Packed form** (in PersistentPlayerBases, DiscoveryManagerData, MarkerStack): either a hex string `"0x40050003AB8C07"` or a bare integer `4716909145249443`

2. **UniverseAddress** wraps GalacticAddress: `{"RealityIndex": 0, "GalacticAddress": { ... object form ... }}`

3. **Currency fields** in PlayerStateData: `"Units": -919837762` (i64, can go negative!), `"Nanites": 272127`, `"Specials": 2230` (Specials = Quicksilver)

4. **Discovery records** use short field names (NOT obfuscated -- these are the actual field names):
   - `DD` = Discovery Data: `{"UA": "0x513300F79B1D82", "DT": "Flora", "VP": [...]}`
   - `OWS` = Ownership: `{"LID": "", "UID": "76561197977678185", "USN": "Allasar", "PTK": "ST", "TS": 1757022865}`
   - `DM` = Discovery Metadata: usually `{}`
   - `FL` = Flags: `{"C": 1, "U": 1}` (C=created?, U=uploaded?)
   - `RID` = Record ID: base64 string

5. **Discovery types** (DD.DT values): `"Flora"`, `"Planet"`, `"Sector"`, `"SolarSystem"`, `"Mineral"`, `"Animal"`

6. **DD.UA** can be either a hex string `"0x513300F79B1D82"` or a bare integer `498082938293634` -- must handle both

7. **PersistentPlayerBases** structure:
   ```json
   {
     "BaseVersion": 8,
     "OriginalBaseVersion": 8,
     "GalacticAddress": "0x40050003AB8C07",
     "Position": [17267.421875, 3043.806640625, 63082.875],
     "Forward": [0.913, -0.333, -0.233],
     "UserData": 0,
     "LastUpdateTimestamp": 1738887563,
     "Objects": [ ... ],
     "RID": "",
     "Owner": {"LID": "...", "UID": "...", "USN": "", "PTK": "ST", "TS": 1700427307},
     "Name": "Gugestor Colony",
     "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
     "LastEditedById": "",
     "LastEditedByUsername": "",
     "ScreenshotAt": [...],
     "ScreenshotPos": [...],
     "GameMode": {"PresetGameMode": "Normal"},
     "Difficulty": { ... }
   }
   ```

8. **CommonStateData** contains `SaveName` and `TotalPlayTime` at the top level (shared across contexts).

9. **BaseContext** and **ExpeditionContext** each contain `GameMode` (integer) and a full `PlayerStateData`.

---

## Struct Hierarchy

### File: `crates/nms-save/src/model.rs`

All serde structs go here. Use `#[serde(default)]` liberally. Do NOT use `#[serde(deny_unknown_fields)]` -- PlayerStateData alone has 200+ fields.

#### Top-Level: `SaveFile`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SaveFile {
    pub version: u32,
    pub platform: String,
    pub active_context: String,
    pub common_state_data: CommonStateData,
    pub base_context: GameContext,
    pub expedition_context: GameContext,
    pub discovery_manager_data: DiscoveryManagerData,
}
```

#### `CommonStateData`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonStateData {
    #[serde(default)]
    pub save_name: String,
    #[serde(default)]
    pub total_play_time: u64,
    // All other fields are ignored via default serde behavior (no deny_unknown_fields)
}
```

#### `GameContext`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameContext {
    #[serde(default)]
    pub game_mode: u32,
    pub player_state_data: PlayerStateData,
}
```

#### `PlayerStateData` (subset -- only fields we need)

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerStateData {
    #[serde(default)]
    pub universe_address: UniverseAddress,

    #[serde(default)]
    pub previous_universe_address: UniverseAddress,

    #[serde(default)]
    pub save_summary: String,

    /// Units can be negative (observed: -919837762 in actual saves)
    #[serde(default)]
    pub units: i64,

    #[serde(default)]
    pub nanites: i64,

    /// Quicksilver is stored as "Specials" in the JSON
    #[serde(default)]
    pub specials: i64,

    #[serde(default)]
    pub persistent_player_bases: Vec<PersistentPlayerBase>,

    #[serde(default)]
    pub health: u32,

    #[serde(default)]
    pub time_alive: u64,
}
```

**Important:** `PlayerStateData` has hundreds of other fields (inventory, missions, knowledge, etc). We skip them all -- serde will silently ignore unknown fields by default (since we do NOT use `deny_unknown_fields`).

#### `UniverseAddress`

The PlayerStateData version uses the object form of GalacticAddress:

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UniverseAddress {
    #[serde(default)]
    pub reality_index: u8,
    #[serde(default)]
    pub galactic_address: GalacticAddressObject,
}
```

#### `GalacticAddressObject`

The expanded object form (used only in UniverseAddress within PlayerStateData):

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
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
```

Provide conversion to `nms_core::GalacticAddress`:

```rust
impl GalacticAddressObject {
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
```

#### `PackedGalacticAddress`

For bases and discoveries, GalacticAddress is stored as either a hex string `"0x..."` or a bare integer. Implement a custom deserializer:

```rust
use serde::de;

/// Galactic address in packed form -- can be a hex string "0x..." or a bare integer.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct PackedGalacticAddress(pub u64);

impl<'de> Deserialize<'de> for PackedGalacticAddress {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = PackedGalacticAddress;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a hex string like \"0x...\" or an integer")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(PackedGalacticAddress(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(PackedGalacticAddress(v as u64))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let hex = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")).unwrap_or(v);
                u64::from_str_radix(hex, 16)
                    .map(PackedGalacticAddress)
                    .map_err(|_| de::Error::custom(format!("invalid hex galactic address: {v}")))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl PackedGalacticAddress {
    pub fn to_galactic_address(&self, reality_index: u8) -> nms_core::GalacticAddress {
        nms_core::GalacticAddress::from_packed(self.0, reality_index)
    }
}
```

#### `DiscoveryManagerData`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DiscoveryManagerData {
    #[serde(rename = "DiscoveryData-v1", default)]
    pub discovery_data_v1: DiscoveryDataV1,
}
```

#### `DiscoveryDataV1`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DiscoveryDataV1 {
    #[serde(default)]
    pub reserve_store: u32,
    #[serde(default)]
    pub reserve_managed: u32,
    #[serde(default)]
    pub store: DiscoveryStore,
}
```

#### `DiscoveryStore`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DiscoveryStore {
    #[serde(default)]
    pub record: Vec<RawDiscoveryRecord>,
}
```

#### `RawDiscoveryRecord`

This is the raw JSON structure. Field names ARE the actual JSON keys (short names, not obfuscated):

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawDiscoveryRecord {
    /// Discovery Data
    #[serde(rename = "DD")]
    pub dd: DiscoveryData,

    /// Discovery Metadata (usually empty object)
    #[serde(rename = "DM", default)]
    pub dm: serde_json::Value,

    /// Ownership data
    #[serde(rename = "OWS")]
    pub ows: OwnershipData,

    /// Flags (C=created, U=uploaded)
    #[serde(rename = "FL", default)]
    pub fl: DiscoveryFlags,

    /// Record ID (base64 hash)
    #[serde(rename = "RID", default)]
    pub rid: Option<String>,
}
```

#### `DiscoveryData` (the DD sub-object)

```rust
/// Discovery data sub-object.
/// UA can be hex string "0x..." or bare integer.
/// DT is the discovery type string.
/// VP is an array of mixed types (hex strings and integers) -- store as Value.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryData {
    /// Universe address (packed galactic address, no reality index at this level)
    #[serde(rename = "UA")]
    pub ua: PackedGalacticAddress,

    /// Discovery type: "Flora", "Planet", "Sector", "SolarSystem", "Mineral", "Animal"
    #[serde(rename = "DT")]
    pub dt: String,

    /// Variable-purpose data array (seeds, hashes, counts) -- opaque for now
    #[serde(rename = "VP", default)]
    pub vp: Vec<serde_json::Value>,
}
```

#### `OwnershipData` (the OWS sub-object)

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OwnershipData {
    /// Local ID (may be empty string)
    #[serde(rename = "LID", default)]
    pub lid: String,

    /// User ID (Steam ID, PSN ID, etc.)
    #[serde(rename = "UID", default)]
    pub uid: String,

    /// Username
    #[serde(rename = "USN", default)]
    pub usn: String,

    /// Platform token: "ST" = Steam, "PS" = PlayStation, etc.
    #[serde(rename = "PTK", default)]
    pub ptk: String,

    /// Timestamp (Unix epoch seconds)
    #[serde(rename = "TS", default)]
    pub ts: u64,
}
```

#### `DiscoveryFlags` (the FL sub-object)

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DiscoveryFlags {
    /// Created flag
    #[serde(rename = "C", default)]
    pub created: Option<u8>,

    /// Uploaded flag
    #[serde(rename = "U", default)]
    pub uploaded: Option<u8>,
}
```

#### `PersistentPlayerBase`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
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

    /// Base objects -- we store as Value since we don't need to inspect them
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
```

#### Wrapper types for nested enum-in-object patterns

The save file wraps enum values in objects like `{"PersistentBaseTypes": "HomePlanetBase"}`:

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BaseTypeWrapper {
    #[serde(rename = "PersistentBaseTypes", default)]
    pub persistent_base_types: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameModeWrapper {
    #[serde(rename = "PresetGameMode", default)]
    pub preset_game_mode: String,
}
```

---

## Conversion Functions

### File: `crates/nms-save/src/convert.rs`

Convert raw save types to `nms-core` domain types.

```rust
use crate::model::*;
use nms_core::{DiscoveryRecord, Discovery, PlayerBase, BaseType, PlayerState, GalacticAddress};
use chrono::{DateTime, Utc};

impl RawDiscoveryRecord {
    pub fn to_core_record(&self) -> Option<DiscoveryRecord> {
        let discovery_type = match self.dd.dt.as_str() {
            "Planet" => Discovery::Planet,
            "SolarSystem" => Discovery::SolarSystem,
            "Sector" => Discovery::Sector,
            "Animal" => Discovery::Animal,
            "Flora" => Discovery::Flora,
            "Mineral" => Discovery::Mineral,
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

        Some(DiscoveryRecord {
            discovery_type,
            universe_address: self.dd.ua.to_galactic_address(0),
            reality_index: 0, // Discovery records don't carry reality_index
            timestamp,
            name: None,       // Discoveries in save don't carry display names
            discoverer,
            is_uploaded,
        })
    }
}

impl PersistentPlayerBase {
    pub fn to_core_base(&self) -> PlayerBase {
        let base_type = match self.base_type.persistent_base_types.as_str() {
            "HomePlanetBase" => BaseType::HomePlanetBase,
            "FreighterBase" => BaseType::FreighterBase,
            _ => BaseType::ExternalPlanetBase,
        };

        PlayerBase {
            name: self.name.clone(),
            base_type,
            address: self.galactic_address.to_galactic_address(0),
            reality_index: 0,
            position: self.position,
            owner_uid: if self.owner.uid.is_empty() { None } else { Some(self.owner.uid.clone()) },
        }
    }
}

impl SaveFile {
    /// Get PlayerStateData for the active context.
    pub fn active_player_state(&self) -> &PlayerStateData {
        match self.active_context.as_str() {
            "Expedition" => &self.expedition_context.player_state_data,
            _ => &self.base_context.player_state_data, // "Main" and any other value
        }
    }

    /// Convert active player state to nms-core PlayerState.
    pub fn to_core_player_state(&self) -> PlayerState {
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
            Some(prev_ua.galactic_address.to_galactic_address(prev_ua.reality_index))
        };

        PlayerState {
            current_address,
            current_reality: ua.reality_index,
            previous_address,
            freighter_address: None, // not yet extracted
            units: ps.units as u64,  // cast back to u64 for core type
            nanites: ps.nanites as u64,
            quicksilver: ps.specials as u64,
        }
    }
}
```

---

## Top-Level Parse Function

### File: `crates/nms-save/src/lib.rs`

```rust
pub mod model;
pub mod convert;

use model::SaveFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Parse a deobfuscated save file JSON string into a SaveFile struct.
/// The input must be valid UTF-8 JSON with deobfuscated (plaintext) keys.
pub fn parse_save(json: &[u8]) -> Result<SaveFile, SaveError> {
    let save: SaveFile = serde_json::from_slice(json)?;
    Ok(save)
}

/// Parse from a file path. Reads the file, assumes it is already decompressed
/// and deobfuscated JSON.
pub fn parse_save_file(path: &std::path::Path) -> Result<SaveFile, SaveError> {
    let bytes = std::fs::read(path)?;
    parse_save(&bytes)
}
```

---

## File Organization Summary

```
crates/nms-save/src/
    lib.rs       -- parse_save(), parse_save_file(), SaveError
    model.rs     -- all serde structs (SaveFile, CommonStateData, GameContext,
                    PlayerStateData, UniverseAddress, GalacticAddressObject,
                    PackedGalacticAddress, DiscoveryManagerData, DiscoveryDataV1,
                    DiscoveryStore, RawDiscoveryRecord, DiscoveryData,
                    OwnershipData, DiscoveryFlags, PersistentPlayerBase,
                    BaseTypeWrapper, GameModeWrapper)
    convert.rs   -- From impls: RawDiscoveryRecord -> nms_core::DiscoveryRecord,
                    PersistentPlayerBase -> nms_core::PlayerBase,
                    SaveFile -> nms_core::PlayerState
```

---

## Tests

### File: `crates/nms-save/src/model.rs` (inline tests at bottom)

```rust
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
    fn parse_minimal_save_file() {
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
        let save: SaveFile = serde_json::from_str(json).unwrap();
        assert_eq!(save.version, 4720);
        assert_eq!(save.platform, "Mac|Final");
        assert_eq!(save.active_context, "Main");
        assert_eq!(save.common_state_data.save_name, "test");
        assert_eq!(save.common_state_data.total_play_time, 100);
        assert_eq!(save.base_context.player_state_data.units, 1000000);
        assert_eq!(save.discovery_manager_data.discovery_data_v1.store.record.len(), 0);
    }
}
```

### File: `crates/nms-save/src/convert.rs` (inline tests at bottom)

```rust
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
            fl: DiscoveryFlags { created: Some(1), uploaded: Some(1) },
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
            base_type: BaseTypeWrapper { persistent_base_types: "HomePlanetBase".into() },
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
}
```

---

## Implementation Notes

1. **Do NOT use `#[serde(deny_unknown_fields)]`** on any struct. The save file has hundreds of fields we skip.

2. **Use `#[serde(default)]`** on every field except those guaranteed present (Version, Platform, ActiveContext).

3. **`serde_json::Value`** is used for fields we might inspect later but don't need to type now (Objects array, DM metadata, VP arrays).

4. **The `PackedGalacticAddress` custom deserializer** is critical -- the save file mixes hex strings and bare integers for the same field. Both must work.

5. **`i64` for currency fields** -- Units can go negative in actual saves (observed -919837762). The core type uses `u64`; the cast happens in conversion. Document this discrepancy.

6. **The DD/OWS/FL/DM/RID field names** in discovery records are NOT obfuscated -- they are the actual JSON keys in both obfuscated and deobfuscated saves. The deobfuscation mapping only applies to top-level structural keys like `PlayerStateData`, `DiscoveryManagerData`, etc.

7. **Discovery records carry no display name.** The `DD` object has no name field. Discovery names (if renamed by players) are handled differently in the save format and are not in the store records we parse.

8. **`voxel_y` is `i8`** in the object form. serde can deserialize JSON integers to i8 directly.
