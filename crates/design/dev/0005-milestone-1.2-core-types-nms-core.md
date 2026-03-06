# Milestone 1.2 — Core Types (nms-core)

Foundation types that every other crate depends on. Zero heavy deps beyond serde.

## Crate Setup

- Crate path: `crates/nms-core/`
- Add to workspace `Cargo.toml` members list
- Edition 2024, Rust 1.85+

### `crates/nms-core/Cargo.toml`

```toml
[package]
name = "nms-core"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
serde_json = "1"
```

## File Organization

```
crates/nms-core/src/
  lib.rs          — re-exports all public types
  address.rs      — GalacticAddress
  biome.rs        — Biome, BiomeSubType
  discovery.rs    — Discovery, DiscoveryRecord
  galaxy.rs       — Galaxy, GalaxyType
  system.rs       — System, Planet
  player.rs       — PlayerBase, BaseType, PlayerState
```

### `src/lib.rs`

Declare modules and re-export all public types:

```rust
pub mod address;
pub mod biome;
pub mod discovery;
pub mod galaxy;
pub mod player;
pub mod system;

pub use address::GalacticAddress;
pub use biome::{Biome, BiomeSubType};
pub use discovery::{Discovery, DiscoveryRecord};
pub use galaxy::{Galaxy, GalaxyType};
pub use player::{BaseType, PlayerBase, PlayerState};
pub use system::{Planet, System};
```

---

## Type Specifications

### GalacticAddress (`src/address.rs`)

A packed 48-bit galactic coordinate stored inside a `u64`, plus a separate `reality_index: u8` field representing the galaxy (0-255).

#### Struct Definition

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GalacticAddress {
    /// Packed 48-bit value encoding P-SSS-YY-ZZZ-XXX in portal glyph order.
    packed: u64,
    /// Galaxy index 0–255 (Euclid=0, Hilbert=1, ...). Not part of the 48-bit packed value.
    pub reality_index: u8,
}
```

#### Bit Layout (portal glyph order, 48 bits total)

Portal glyph hex string is 12 hex digits: `P SSS YY ZZZ XXX`

Reading from MSB to LSB of the 48-bit value:

| Bits (from MSB) | Width | Field              | Range                     |
|------------------|-------|--------------------|---------------------------|
| 47–44            | 4     | PlanetIndex        | 0x0–0xF (unsigned)        |
| 43–32            | 12    | SolarSystemIndex   | 0x000–0xFFE (unsigned)    |
| 31–24            | 8     | VoxelY             | -128..127 (signed)        |
| 23–12            | 12    | VoxelZ             | -2048..2047 (signed)      |
| 11–0             | 12    | VoxelX             | -2048..2047 (signed)      |

The packed value is exactly the numeric interpretation of the 12-digit portal hex string.

#### Field Extraction Methods

All return portal-coordinate-frame values (galaxy-center origin, signed for voxels):

```rust
impl GalacticAddress {
    /// Planet index (4-bit unsigned, 0–15). Bits 47–44.
    pub fn planet_index(&self) -> u8 {
        ((self.packed >> 44) & 0xF) as u8
    }

    /// Solar system index (12-bit unsigned, 0x000–0xFFE). Bits 43–32.
    pub fn solar_system_index(&self) -> u16 {
        ((self.packed >> 32) & 0xFFF) as u16
    }

    /// VoxelY (8-bit signed, -128..127). Bits 31–24.
    /// Stored as unsigned in the packed value; interpret as i8.
    pub fn voxel_y(&self) -> i8 {
        ((self.packed >> 24) & 0xFF) as u8 as i8
    }

    /// VoxelZ (12-bit signed, -2048..2047). Bits 23–12.
    /// Stored as unsigned 12-bit; interpret as signed via sign extension.
    pub fn voxel_z(&self) -> i16 {
        let raw = ((self.packed >> 12) & 0xFFF) as u16;
        // Sign-extend from 12 bits
        if raw & 0x800 != 0 {
            (raw | 0xF000) as i16
        } else {
            raw as i16
        }
    }

    /// VoxelX (12-bit signed, -2048..2047). Bits 11–0.
    /// Stored as unsigned 12-bit; interpret as signed via sign extension.
    pub fn voxel_x(&self) -> i16 {
        let raw = (self.packed & 0xFFF) as u16;
        if raw & 0x800 != 0 {
            (raw | 0xF000) as i16
        } else {
            raw as i16
        }
    }

    /// Return the raw 48-bit packed value.
    pub fn packed(&self) -> u64 {
        self.packed
    }

    /// Voxel coordinates as (x, y, z) signed integers (center-origin).
    pub fn voxel_position(&self) -> (i16, i8, i16) {
        (self.voxel_x(), self.voxel_y(), self.voxel_z())
    }
}
```

#### Constructor

```rust
impl GalacticAddress {
    /// Create from individual field values (portal coordinate frame).
    /// `voxel_x`: signed -2048..2047
    /// `voxel_y`: signed -128..127
    /// `voxel_z`: signed -2048..2047
    /// `solar_system_index`: 0..0xFFE
    /// `planet_index`: 0..15
    /// `reality_index`: 0..255
    pub fn new(
        voxel_x: i16,
        voxel_y: i8,
        voxel_z: i16,
        solar_system_index: u16,
        planet_index: u8,
        reality_index: u8,
    ) -> Self {
        let x_bits = (voxel_x as u16 & 0xFFF) as u64;
        let y_bits = (voxel_y as u8 as u64) & 0xFF;
        let z_bits = (voxel_z as u16 & 0xFFF) as u64;
        let ssi_bits = (solar_system_index as u64) & 0xFFF;
        let p_bits = (planet_index as u64) & 0xF;

        let packed = (p_bits << 44)
            | (ssi_bits << 32)
            | (y_bits << 24)
            | (z_bits << 12)
            | x_bits;

        Self { packed, reality_index }
    }

    /// Create from raw packed 48-bit value and reality index.
    pub fn from_packed(packed: u64, reality_index: u8) -> Self {
        Self {
            packed: packed & 0xFFFF_FFFF_FFFF, // mask to 48 bits
            reality_index,
        }
    }
}
```

#### Signal Booster Coordinate Conversion

Signal booster uses corner-origin unsigned coordinates in format `XXXX:YYYY:ZZZZ:SSSS`.

Conversion formulas (portal = center-origin, sb = signal booster = corner-origin):

- Portal_X_unsigned = (SB_X + 0x801) mod 0x1000
- Portal_Y_unsigned = (SB_Y + 0x81) mod 0x100
- Portal_Z_unsigned = (SB_Z + 0x801) mod 0x1000

Where Portal_X_unsigned is the raw 12-bit field in the packed value (before sign interpretation).

Reverse:

- SB_X = (Portal_X_unsigned + 0x7FF) mod 0x1000
- SB_Y = (Portal_Y_unsigned + 0x7F) mod 0x100
- SB_Z = (Portal_Z_unsigned + 0x7FF) mod 0x1000

```rust
impl GalacticAddress {
    /// Parse signal booster format "XXXX:YYYY:ZZZZ:SSSS".
    /// All four groups are 16-bit hex values (zero-padded, unsigned corner-origin).
    /// Does NOT include planet index or reality index; caller must supply those.
    pub fn from_signal_booster(s: &str, planet_index: u8, reality_index: u8) -> Result<Self, AddressParseError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err(AddressParseError::InvalidFormat);
        }
        let sb_x = u16::from_str_radix(parts[0], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let sb_y = u16::from_str_radix(parts[1], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let sb_z = u16::from_str_radix(parts[2], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let ssi  = u16::from_str_radix(parts[3], 16).map_err(|_| AddressParseError::InvalidHex)?;

        let portal_x = (sb_x.wrapping_add(0x801)) & 0xFFF;
        let portal_y = ((sb_y.wrapping_add(0x81)) & 0xFF) as u8;
        let portal_z = (sb_z.wrapping_add(0x801)) & 0xFFF;

        let packed = ((planet_index as u64 & 0xF) << 44)
            | ((ssi as u64 & 0xFFF) << 32)
            | ((portal_y as u64) << 24)
            | ((portal_z as u64) << 12)
            | (portal_x as u64);

        Ok(Self { packed, reality_index })
    }

    /// Format as signal booster string "XXXX:YYYY:ZZZZ:SSSS".
    pub fn to_signal_booster(&self) -> String {
        let portal_x = (self.packed & 0xFFF) as u16;
        let portal_y = ((self.packed >> 24) & 0xFF) as u16;
        let portal_z = ((self.packed >> 12) & 0xFFF) as u16;
        let ssi = ((self.packed >> 32) & 0xFFF) as u16;

        let sb_x = portal_x.wrapping_add(0x7FF) & 0xFFF;
        let sb_y = portal_y.wrapping_add(0x7F) & 0xFF;
        let sb_z = portal_z.wrapping_add(0x7FF) & 0xFFF;

        format!("{:04X}:{:04X}:{:04X}:{:04X}", sb_x, sb_y, sb_z, ssi)
    }
}
```

#### Error Type

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressParseError {
    InvalidFormat,
    InvalidHex,
    InvalidLength,
}

impl fmt::Display for AddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid address format"),
            Self::InvalidHex => write!(f, "invalid hex digit in address"),
            Self::InvalidLength => write!(f, "address has wrong number of digits"),
        }
    }
}

impl std::error::Error for AddressParseError {}
```

#### Display and FromStr

```rust
/// Display as "0x" followed by 12 uppercase hex digits (the 48-bit packed value).
impl fmt::Display for GalacticAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:012X}", self.packed)
    }
}

/// Parse from:
/// - "0x" or "0X" prefix followed by 12 hex digits (galactic address format)
/// - 12 hex digits without prefix (portal glyph hex)
/// Reality index defaults to 0 when parsing (caller can override).
impl FromStr for GalacticAddress {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_str = if s.starts_with("0x") || s.starts_with("0X") {
            &s[2..]
        } else {
            s
        };

        if hex_str.len() != 12 {
            return Err(AddressParseError::InvalidLength);
        }

        let packed = u64::from_str_radix(hex_str, 16)
            .map_err(|_| AddressParseError::InvalidHex)?;

        Ok(Self { packed, reality_index: 0 })
    }
}
```

#### From<u64> and Into<u64>

```rust
/// From raw packed u64 (reality_index defaults to 0).
impl From<u64> for GalacticAddress {
    fn from(packed: u64) -> Self {
        Self {
            packed: packed & 0xFFFF_FFFF_FFFF,
            reality_index: 0,
        }
    }
}

/// Into raw packed u64 (drops reality_index).
impl From<GalacticAddress> for u64 {
    fn from(addr: GalacticAddress) -> u64 {
        addr.packed
    }
}
```

#### Special System Indices (constants)

```rust
pub const SSI_BLACK_HOLE: u16 = 0x079;
pub const SSI_ATLAS_INTERFACE: u16 = 0x07A;
/// Purple system SSI range (inclusive).
pub const SSI_PURPLE_START: u16 = 0x3E8;
pub const SSI_PURPLE_END: u16 = 0x429;

impl GalacticAddress {
    pub fn is_black_hole(&self) -> bool {
        self.solar_system_index() == SSI_BLACK_HOLE
    }
    pub fn is_atlas_interface(&self) -> bool {
        self.solar_system_index() == SSI_ATLAS_INTERFACE
    }
    pub fn is_purple_system(&self) -> bool {
        let ssi = self.solar_system_index();
        ssi >= SSI_PURPLE_START && ssi <= SSI_PURPLE_END
    }
}
```

---

### Biome (`src/biome.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Matches GcBiomeType from game data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Lush,
    Toxic,
    Scorched,
    Radioactive,
    Frozen,
    Barren,
    Dead,
    Weird,
    Red,
    Green,
    Blue,
    Swamp,
    Lava,
    Waterworld,
    GasGiant,
}

impl fmt::Display for Biome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Lush => "Lush",
            Self::Toxic => "Toxic",
            Self::Scorched => "Scorched",
            Self::Radioactive => "Radioactive",
            Self::Frozen => "Frozen",
            Self::Barren => "Barren",
            Self::Dead => "Dead",
            Self::Weird => "Weird",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Swamp => "Swamp",
            Self::Lava => "Lava",
            Self::Waterworld => "Waterworld",
            Self::GasGiant => "GasGiant",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeParseError(pub String);

impl fmt::Display for BiomeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown biome: {}", self.0)
    }
}

impl std::error::Error for BiomeParseError {}

impl FromStr for Biome {
    type Err = BiomeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lush" => Ok(Self::Lush),
            "toxic" => Ok(Self::Toxic),
            "scorched" => Ok(Self::Scorched),
            "radioactive" => Ok(Self::Radioactive),
            "frozen" => Ok(Self::Frozen),
            "barren" => Ok(Self::Barren),
            "dead" => Ok(Self::Dead),
            "weird" => Ok(Self::Weird),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "swamp" => Ok(Self::Swamp),
            "lava" => Ok(Self::Lava),
            "waterworld" => Ok(Self::Waterworld),
            "gasgiant" | "gas_giant" => Ok(Self::GasGiant),
            _ => Err(BiomeParseError(s.to_string())),
        }
    }
}
```

### BiomeSubType (`src/biome.rs`, same file)

```rust
/// Matches GcBiomeSubType from game data (31 variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeSubType {
    LushRoomTemp,
    LushHumid,
    LushInactive,
    ToxicTentacles,
    ToxicFungus,
    ToxicEggs,
    ScorchedSinged,
    ScorchedCharred,
    ScorchedBlasted,
    RadioactiveFungal,
    RadioactiveContaminated,
    RadioactiveIrradiated,
    FrozenIce,
    FrozenSnow,
    FrozenGlacial,
    BarrenDusty,
    BarrenRocky,
    BarrenMountainous,
    DeadEmpty,
    DeadCorroded,
    DeadVoid,
    WeirdHexagonal,
    WeirdCabled,
    WeirdBubbling,
    WeirdFractured,
    WeirdShattered,
    WeirdContorted,
    WeirdWireCell,
    SwampMurky,
    LavaVolcanic,
    WaterworldOcean,
}

impl fmt::Display for BiomeSubType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for BiomeSubType {
    type Err = BiomeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive match against the Debug name
        let lower = s.to_lowercase();
        // List all variants and compare lowercased Debug names
        let variants = [
            BiomeSubType::LushRoomTemp,
            BiomeSubType::LushHumid,
            BiomeSubType::LushInactive,
            BiomeSubType::ToxicTentacles,
            BiomeSubType::ToxicFungus,
            BiomeSubType::ToxicEggs,
            BiomeSubType::ScorchedSinged,
            BiomeSubType::ScorchedCharred,
            BiomeSubType::ScorchedBlasted,
            BiomeSubType::RadioactiveFungal,
            BiomeSubType::RadioactiveContaminated,
            BiomeSubType::RadioactiveIrradiated,
            BiomeSubType::FrozenIce,
            BiomeSubType::FrozenSnow,
            BiomeSubType::FrozenGlacial,
            BiomeSubType::BarrenDusty,
            BiomeSubType::BarrenRocky,
            BiomeSubType::BarrenMountainous,
            BiomeSubType::DeadEmpty,
            BiomeSubType::DeadCorroded,
            BiomeSubType::DeadVoid,
            BiomeSubType::WeirdHexagonal,
            BiomeSubType::WeirdCabled,
            BiomeSubType::WeirdBubbling,
            BiomeSubType::WeirdFractured,
            BiomeSubType::WeirdShattered,
            BiomeSubType::WeirdContorted,
            BiomeSubType::WeirdWireCell,
            BiomeSubType::SwampMurky,
            BiomeSubType::LavaVolcanic,
            BiomeSubType::WaterworldOcean,
        ];
        for v in variants {
            if format!("{:?}", v).to_lowercase() == lower {
                return Ok(v);
            }
        }
        Err(BiomeParseError(s.to_string()))
    }
}
```

---

### Discovery (`src/discovery.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::address::GalacticAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Discovery {
    Planet,
    SolarSystem,
    Sector,
    Animal,
    Flora,
    Mineral,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryRecord {
    pub discovery_type: Discovery,
    pub universe_address: GalacticAddress,
    pub reality_index: u8,
    pub timestamp: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub discoverer: Option<String>,
    pub is_uploaded: bool,
}
```

---

### Galaxy (`src/galaxy.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GalaxyType {
    Norm,
    Lush,
    Harsh,
    Empty,
}

impl fmt::Display for GalaxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Norm => write!(f, "Normal"),
            Self::Lush => write!(f, "Lush"),
            Self::Harsh => write!(f, "Harsh"),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Galaxy {
    pub index: u8,
    pub name: &'static str,
    pub galaxy_type: GalaxyType,
}
```

Provide a `const` array or a lookup function for all 256 galaxies. The first several and the pattern:

```rust
/// Lookup galaxy by index (0–255). Always returns a valid Galaxy.
pub fn galaxy_by_index(index: u8) -> Galaxy {
    // First 10 galaxies have unique names:
    // 0  Euclid        Norm
    // 1  Hilbert Dimension  Norm
    // 2  Calypso       Harsh
    // 3  Hesperius Dimension  Norm
    // 4  Hyades        Norm
    // 5  Ickjamatew    Norm
    // 6  Budullangr    Norm
    // 7  Aptarkaba     Norm (Lush? — verify; include all 256 names)
    // 8  Ontiniangp    Norm
    // 9  Hitonskyer    Norm
    // 10 Rerasmutul    Lush
    // ...pattern continues
    //
    // Galaxy type pattern (repeating every 10 after the first few):
    // Lush galaxies at indices: 10, 19, 30, 39, 50, 59, 70, 79, 90, 99, ...
    //   i.e., (index % 10 == 0 && index >= 10) || (index % 10 == 9 && index >= 19)
    //   Simplified: Lush if (index + 1) % 10 == 1 for index >= 10 ... but verify exact pattern
    //
    // Harsh galaxies at index 2 and others following pattern
    // Empty galaxies at certain indices
    //
    // Implementation: Store a static array of (name, GalaxyType) for all 256.
    // For the plan, include at minimum the first ~20 with correct types and
    // the full list of lush galaxy indices.

    GALAXIES[index as usize].clone()
}
```

Store the full list as a `static` array. The implementing agent should populate all 256 entries from the NMS wiki data. Provide the known lush galaxy indices for validation:

Lush galaxy indices (confirmed): 10, 19, 30, 39, 50, 59, 70, 79, 90, 99, 110, 119, 130, 139, 150, 159, 170, 179, 190, 199, 210, 219, 230, 239, 250, 255.

The implementing agent must source the full 256 galaxy name list. If not available, use the pattern: galaxies 0–9 have known names (listed above), and galaxies 10–255 are named "Galaxy {index}" with the type derived from the lush/harsh/empty pattern.

---

### System (`src/system.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::address::GalacticAddress;
use crate::biome::{Biome, BiomeSubType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct System {
    pub address: GalacticAddress,
    pub reality_index: u8,
    pub name: Option<String>,
    pub discoverer: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub planets: Vec<Planet>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planet {
    /// Planet index within the system (0–15).
    pub index: u8,
    pub biome: Option<Biome>,
    pub biome_subtype: Option<BiomeSubType>,
    /// Whether the planet has infested (biological horror) variant.
    pub infested: bool,
    pub name: Option<String>,
    /// Procedural generation seed.
    pub seed_hash: Option<u64>,
}
```

---

### Player (`src/player.rs`)

```rust
use serde::{Deserialize, Serialize};

use crate::address::GalacticAddress;

/// Maps to PersistentBaseTypes in the save file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    HomePlanetBase,
    FreighterBase,
    ExternalPlanetBase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerBase {
    pub name: String,
    pub base_type: BaseType,
    pub address: GalacticAddress,
    pub reality_index: u8,
    /// Position in local planet coordinates [x, y, z].
    pub position: [f32; 3],
    /// Platform-specific user ID.
    pub owner_uid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    pub current_address: GalacticAddress,
    pub current_reality: u8,
    pub previous_address: Option<GalacticAddress>,
    pub freighter_address: Option<GalacticAddress>,
    pub units: u64,
    pub nanites: u64,
    pub quicksilver: u64,
}
```

---

## Tests

Create `crates/nms-core/src/address.rs` with `#[cfg(test)] mod tests { ... }` at the bottom, or create a `tests/` directory. Inline tests preferred.

### GalacticAddress Tests (`src/address.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let addr = GalacticAddress::new(
            -350,  // voxel_x
            42,    // voxel_y
            1000,  // voxel_z
            0x123, // solar_system_index
            3,     // planet_index
            0,     // reality_index
        );
        assert_eq!(addr.voxel_x(), -350);
        assert_eq!(addr.voxel_y(), 42);
        assert_eq!(addr.voxel_z(), 1000);
        assert_eq!(addr.solar_system_index(), 0x123);
        assert_eq!(addr.planet_index(), 3);
    }

    #[test]
    fn from_portal_hex_string() {
        // Known portal address: "01717D8A4EA2"
        let addr: GalacticAddress = "01717D8A4EA2".parse().unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
        assert_eq!(addr.planet_index(), 0);
        assert_eq!(addr.solar_system_index(), 0x171);
        // Verify voxel extraction
        let y_raw = ((0x01717D8A4EA2u64 >> 24) & 0xFF) as u8;
        assert_eq!(addr.voxel_y(), y_raw as i8);
    }

    #[test]
    fn display_as_hex() {
        let addr = GalacticAddress::from_packed(0x01717D8A4EA2, 0);
        assert_eq!(format!("{}", addr), "0x01717D8A4EA2");
    }

    #[test]
    fn from_u64_and_into_u64() {
        let packed: u64 = 0x01717D8A4EA2;
        let addr = GalacticAddress::from(packed);
        let back: u64 = addr.into();
        assert_eq!(back, packed);
    }

    #[test]
    fn parse_with_0x_prefix() {
        let addr: GalacticAddress = "0x01717D8A4EA2".parse().unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    #[test]
    fn signal_booster_roundtrip() {
        let addr = GalacticAddress::new(-350, 42, 1000, 0x123, 3, 0);
        let sb = addr.to_signal_booster();
        let addr2 = GalacticAddress::from_signal_booster(&sb, 3, 0).unwrap();
        assert_eq!(addr.packed(), addr2.packed());
    }

    #[test]
    fn signal_booster_format() {
        // Verify signal booster string is "XXXX:YYYY:ZZZZ:SSSS" format
        let addr = GalacticAddress::from_packed(0x01717D8A4EA2, 0);
        let sb = addr.to_signal_booster();
        let parts: Vec<&str> = sb.split(':').collect();
        assert_eq!(parts.len(), 4);
        // Each part should be 4 hex digits
        for part in &parts {
            assert_eq!(part.len(), 4);
            assert!(u16::from_str_radix(part, 16).is_ok());
        }
    }

    #[test]
    fn special_system_indices() {
        let bh = GalacticAddress::new(0, 0, 0, 0x079, 0, 0);
        assert!(bh.is_black_hole());
        assert!(!bh.is_atlas_interface());

        let atlas = GalacticAddress::new(0, 0, 0, 0x07A, 0, 0);
        assert!(atlas.is_atlas_interface());

        let purple = GalacticAddress::new(0, 0, 0, 0x400, 0, 0);
        assert!(purple.is_purple_system());
    }
}
```

### Biome Tests (`src/biome.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_fromstr_roundtrip() {
        for biome in [
            Biome::Lush, Biome::Toxic, Biome::Scorched, Biome::Radioactive,
            Biome::Frozen, Biome::Barren, Biome::Dead, Biome::Weird,
            Biome::Red, Biome::Green, Biome::Blue, Biome::Swamp,
            Biome::Lava, Biome::Waterworld, Biome::GasGiant,
        ] {
            let s = biome.to_string();
            let parsed: Biome = s.parse().unwrap();
            assert_eq!(biome, parsed);
        }
    }

    #[test]
    fn case_insensitive_parse() {
        assert_eq!("lush".parse::<Biome>().unwrap(), Biome::Lush);
        assert_eq!("LUSH".parse::<Biome>().unwrap(), Biome::Lush);
        assert_eq!("Lush".parse::<Biome>().unwrap(), Biome::Lush);
    }
}
```

### Galaxy Tests (`src/galaxy.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclid_is_index_zero() {
        let g = galaxy_by_index(0);
        assert_eq!(g.index, 0);
        assert_eq!(g.name, "Euclid");
        assert_eq!(g.galaxy_type, GalaxyType::Norm);
    }

    #[test]
    fn calypso_is_harsh() {
        let g = galaxy_by_index(2);
        assert_eq!(g.name, "Calypso");
        assert_eq!(g.galaxy_type, GalaxyType::Harsh);
    }

    #[test]
    fn index_10_is_lush() {
        let g = galaxy_by_index(10);
        assert_eq!(g.galaxy_type, GalaxyType::Lush);
    }

    #[test]
    fn all_256_valid() {
        for i in 0..=255u8 {
            let g = galaxy_by_index(i);
            assert_eq!(g.index, i);
            assert!(!g.name.is_empty());
        }
    }
}
```

### Serialization Tests (any module or separate test file)

```rust
#[test]
fn galactic_address_serde_roundtrip() {
    let addr = GalacticAddress::new(-350, 42, 1000, 0x123, 3, 5);
    let json = serde_json::to_string(&addr).unwrap();
    let addr2: GalacticAddress = serde_json::from_str(&json).unwrap();
    assert_eq!(addr, addr2);
}
```
