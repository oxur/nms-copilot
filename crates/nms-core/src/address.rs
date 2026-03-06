use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Special system index: always contains a black hole.
pub const SSI_BLACK_HOLE: u16 = 0x079;
/// Special system index: always contains an Atlas Interface.
pub const SSI_ATLAS_INTERFACE: u16 = 0x07A;
/// Purple system SSI range start (inclusive).
pub const SSI_PURPLE_START: u16 = 0x3E8;
/// Purple system SSI range end (inclusive).
pub const SSI_PURPLE_END: u16 = 0x429;

/// Mask for the 48-bit packed galactic address.
const PACKED_MASK: u64 = 0xFFFF_FFFF_FFFF;

// Bit-field shifts within the 48-bit packed value.
const PLANET_SHIFT: u32 = 44;
const SSI_SHIFT: u32 = 32;
const VOXEL_Y_SHIFT: u32 = 24;
const VOXEL_Z_SHIFT: u32 = 12;

// Bit-field masks (applied after shifting).
const MASK_4BIT: u64 = 0xF;
const MASK_8BIT: u64 = 0xFF;
const MASK_12BIT: u64 = 0xFFF;

// 12-bit sign extension constants.
const SIGN_BIT_12: u16 = 0x800;
const SIGN_EXTEND_12: u16 = 0xF000;

/// Offset added to signal-booster X/Z to convert to portal-frame X/Z.
const SB_TO_PORTAL_XZ: u16 = 0x801;
/// Offset added to signal-booster Y to convert to portal-frame Y.
const SB_TO_PORTAL_Y: u16 = 0x81;
/// Offset added to portal-frame X/Z to convert to signal-booster X/Z.
const PORTAL_TO_SB_XZ: u16 = 0x7FF;
/// Offset added to portal-frame Y to convert to signal-booster Y.
const PORTAL_TO_SB_Y: u16 = 0x7F;

/// A packed 48-bit galactic coordinate plus a galaxy (reality) index.
///
/// The 48-bit value encodes fields in portal glyph order: `P-SSS-YY-ZZZ-XXX`
/// where P=PlanetIndex, SSS=SolarSystemIndex, YY=VoxelY, ZZZ=VoxelZ, XXX=VoxelX.
///
/// The `reality_index` identifies the galaxy (0=Euclid, 1=Hilbert, etc.) and is
/// stored separately from the packed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GalacticAddress {
    packed: u64,
    pub reality_index: u8,
}

impl GalacticAddress {
    /// Create from individual field values (portal coordinate frame).
    pub fn new(
        voxel_x: i16,
        voxel_y: i8,
        voxel_z: i16,
        solar_system_index: u16,
        planet_index: u8,
        reality_index: u8,
    ) -> Self {
        let x_bits = (voxel_x as u16 as u64) & MASK_12BIT;
        let y_bits = (voxel_y as u8 as u64) & MASK_8BIT;
        let z_bits = (voxel_z as u16 as u64) & MASK_12BIT;
        let ssi_bits = (solar_system_index as u64) & MASK_12BIT;
        let p_bits = (planet_index as u64) & MASK_4BIT;

        let packed = (p_bits << PLANET_SHIFT)
            | (ssi_bits << SSI_SHIFT)
            | (y_bits << VOXEL_Y_SHIFT)
            | (z_bits << VOXEL_Z_SHIFT)
            | x_bits;

        Self {
            packed,
            reality_index,
        }
    }

    /// Create from raw packed 48-bit value and reality index.
    pub fn from_packed(packed: u64, reality_index: u8) -> Self {
        Self {
            packed: packed & PACKED_MASK,
            reality_index,
        }
    }

    /// Return the raw 48-bit packed value.
    pub fn packed(&self) -> u64 {
        self.packed
    }

    /// Planet index (4-bit unsigned, 0-15). Bits 47-44.
    pub fn planet_index(&self) -> u8 {
        ((self.packed >> PLANET_SHIFT) & MASK_4BIT) as u8
    }

    /// Solar system index (12-bit unsigned, 0x000-0xFFE). Bits 43-32.
    pub fn solar_system_index(&self) -> u16 {
        ((self.packed >> SSI_SHIFT) & MASK_12BIT) as u16
    }

    /// VoxelY (8-bit signed, -128..127). Bits 31-24.
    pub fn voxel_y(&self) -> i8 {
        ((self.packed >> VOXEL_Y_SHIFT) & MASK_8BIT) as u8 as i8
    }

    /// VoxelZ (12-bit signed, -2048..2047). Bits 23-12.
    pub fn voxel_z(&self) -> i16 {
        let raw = ((self.packed >> VOXEL_Z_SHIFT) & MASK_12BIT) as u16;
        if raw & SIGN_BIT_12 != 0 {
            (raw | SIGN_EXTEND_12) as i16
        } else {
            raw as i16
        }
    }

    /// VoxelX (12-bit signed, -2048..2047). Bits 11-0.
    pub fn voxel_x(&self) -> i16 {
        let raw = (self.packed & MASK_12BIT) as u16;
        if raw & SIGN_BIT_12 != 0 {
            (raw | SIGN_EXTEND_12) as i16
        } else {
            raw as i16
        }
    }

    /// Voxel coordinates as (x, y, z) signed integers (center-origin).
    pub fn voxel_position(&self) -> (i16, i8, i16) {
        (self.voxel_x(), self.voxel_y(), self.voxel_z())
    }

    /// Parse signal booster format `XXXX:YYYY:ZZZZ:SSSS`.
    ///
    /// Signal booster uses corner-origin unsigned coordinates. The conversion
    /// adds fixed offsets to translate into portal-frame (center-origin) values.
    /// Does NOT include planet index or reality index; caller must supply those.
    pub fn from_signal_booster(
        s: &str,
        planet_index: u8,
        reality_index: u8,
    ) -> Result<Self, AddressParseError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err(AddressParseError::InvalidFormat);
        }
        let sb_x = u16::from_str_radix(parts[0], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let sb_y = u16::from_str_radix(parts[1], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let sb_z = u16::from_str_radix(parts[2], 16).map_err(|_| AddressParseError::InvalidHex)?;
        let ssi = u16::from_str_radix(parts[3], 16).map_err(|_| AddressParseError::InvalidHex)?;

        let portal_x = sb_x.wrapping_add(SB_TO_PORTAL_XZ) & MASK_12BIT as u16;
        let portal_y = (sb_y.wrapping_add(SB_TO_PORTAL_Y) & MASK_8BIT as u16) as u8;
        let portal_z = sb_z.wrapping_add(SB_TO_PORTAL_XZ) & MASK_12BIT as u16;

        let packed = ((planet_index as u64 & MASK_4BIT) << PLANET_SHIFT)
            | ((ssi as u64 & MASK_12BIT) << SSI_SHIFT)
            | ((portal_y as u64) << VOXEL_Y_SHIFT)
            | ((portal_z as u64) << VOXEL_Z_SHIFT)
            | (portal_x as u64);

        Ok(Self {
            packed,
            reality_index,
        })
    }

    /// Format as signal booster string `XXXX:YYYY:ZZZZ:SSSS`.
    ///
    /// Converts portal-frame (center-origin) coordinates back to the
    /// corner-origin unsigned format used by the in-game signal booster.
    pub fn to_signal_booster(&self) -> String {
        let portal_x = (self.packed & MASK_12BIT) as u16;
        let portal_y = ((self.packed >> VOXEL_Y_SHIFT) & MASK_8BIT) as u16;
        let portal_z = ((self.packed >> VOXEL_Z_SHIFT) & MASK_12BIT) as u16;
        let ssi = ((self.packed >> SSI_SHIFT) & MASK_12BIT) as u16;

        let sb_x = portal_x.wrapping_add(PORTAL_TO_SB_XZ) & MASK_12BIT as u16;
        let sb_y = portal_y.wrapping_add(PORTAL_TO_SB_Y) & MASK_8BIT as u16;
        let sb_z = portal_z.wrapping_add(PORTAL_TO_SB_XZ) & MASK_12BIT as u16;

        format!("{sb_x:04X}:{sb_y:04X}:{sb_z:04X}:{ssi:04X}")
    }

    /// Distance in light-years to another address.
    ///
    /// Uses Euclidean distance in voxel space multiplied by 400.
    /// Only meaningful for addresses in the same galaxy.
    pub fn distance_ly(&self, other: &GalacticAddress) -> f64 {
        let (x1, y1, z1) = self.voxel_position();
        let (x2, y2, z2) = other.voxel_position();
        let dx = (x1 as f64) - (x2 as f64);
        let dy = (y1 as f64) - (y2 as f64);
        let dz = (z1 as f64) - (z2 as f64);
        (dx * dx + dy * dy + dz * dz).sqrt() * 400.0
    }

    /// Whether two addresses are in the same region (same VoxelX/Y/Z).
    pub fn same_region(&self, other: &GalacticAddress) -> bool {
        self.voxel_x() == other.voxel_x()
            && self.voxel_y() == other.voxel_y()
            && self.voxel_z() == other.voxel_z()
    }

    /// Whether two addresses are in the same system (same region + same SSI).
    pub fn same_system(&self, other: &GalacticAddress) -> bool {
        self.same_region(other) && self.solar_system_index() == other.solar_system_index()
    }

    /// Whether another address is within N light-years.
    pub fn within(&self, other: &GalacticAddress, ly: f64) -> bool {
        self.distance_ly(other) <= ly
    }

    /// Whether this address points to a black hole system (SSI 0x079).
    pub fn is_black_hole(&self) -> bool {
        self.solar_system_index() == SSI_BLACK_HOLE
    }

    /// Whether this address points to an Atlas Interface system (SSI 0x07A).
    pub fn is_atlas_interface(&self) -> bool {
        self.solar_system_index() == SSI_ATLAS_INTERFACE
    }

    /// Whether this address is in the purple system SSI range (0x3E8-0x429).
    pub fn is_purple_system(&self) -> bool {
        let ssi = self.solar_system_index();
        (SSI_PURPLE_START..=SSI_PURPLE_END).contains(&ssi)
    }
}

/// Display as `0x` followed by 12 uppercase hex digits.
impl fmt::Display for GalacticAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:012X}", self.packed)
    }
}

/// Parse from `0x`/`0X` prefix + 12 hex digits, or bare 12 hex digits.
/// Reality index defaults to 0.
impl FromStr for GalacticAddress {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_str = if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))
        {
            stripped
        } else {
            s
        };

        if hex_str.len() != 12 {
            return Err(AddressParseError::InvalidLength);
        }

        let packed = u64::from_str_radix(hex_str, 16).map_err(|_| AddressParseError::InvalidHex)?;

        Ok(Self {
            packed,
            reality_index: 0,
        })
    }
}

/// From raw packed u64 (reality_index defaults to 0).
impl From<u64> for GalacticAddress {
    fn from(packed: u64) -> Self {
        Self {
            packed: packed & PACKED_MASK,
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

/// Error returned when parsing a galactic address string fails.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let addr = GalacticAddress::new(-350, 42, 1000, 0x123, 3, 0);
        assert_eq!(addr.voxel_x(), -350);
        assert_eq!(addr.voxel_y(), 42);
        assert_eq!(addr.voxel_z(), 1000);
        assert_eq!(addr.solar_system_index(), 0x123);
        assert_eq!(addr.planet_index(), 3);
    }

    #[test]
    fn from_portal_hex_string() {
        let addr: GalacticAddress = "01717D8A4EA2".parse().unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
        assert_eq!(addr.planet_index(), 0);
        assert_eq!(addr.solar_system_index(), 0x171);
        let y_raw = ((0x01717D8A4EA2u64 >> 24) & 0xFF) as u8;
        assert_eq!(addr.voxel_y(), y_raw as i8);
    }

    #[test]
    fn display_as_hex() {
        let addr = GalacticAddress::from_packed(0x01717D8A4EA2, 0);
        assert_eq!(format!("{addr}"), "0x01717D8A4EA2");
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
        let addr = GalacticAddress::from_packed(0x01717D8A4EA2, 0);
        let sb = addr.to_signal_booster();
        let parts: Vec<&str> = sb.split(':').collect();
        assert_eq!(parts.len(), 4);
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

    #[test]
    fn distance_same_address_is_zero() {
        let addr = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        assert_eq!(addr.distance_ly(&addr), 0.0);
    }

    #[test]
    fn distance_one_voxel_x() {
        let a = GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let b = GalacticAddress::new(1, 0, 0, 0, 0, 0);
        assert!((a.distance_ly(&b) - 400.0).abs() < 0.01);
    }

    #[test]
    fn same_region_different_ssi() {
        let a = GalacticAddress::new(100, 50, 200, 0x001, 0, 0);
        let b = GalacticAddress::new(100, 50, 200, 0x002, 0, 0);
        assert!(a.same_region(&b));
        assert!(!a.same_system(&b));
    }

    #[test]
    fn same_system_same_everything() {
        let a = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        let b = GalacticAddress::new(100, 50, 200, 0x123, 5, 0);
        assert!(a.same_system(&b));
    }

    #[test]
    fn within_boundary() {
        let a = GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let b = GalacticAddress::new(1, 0, 0, 0, 0, 0);
        assert!(a.within(&b, 400.0));
        assert!(!a.within(&b, 399.0));
    }

    #[test]
    fn negative_voxel_roundtrip() {
        let addr = GalacticAddress::new(-2048, -128, -2048, 0, 0, 0);
        assert_eq!(addr.voxel_x(), -2048);
        assert_eq!(addr.voxel_y(), -128);
        assert_eq!(addr.voxel_z(), -2048);
    }

    #[test]
    fn serde_roundtrip() {
        let addr = GalacticAddress::new(-350, 42, 1000, 0x123, 3, 5);
        let json = serde_json::to_string(&addr).unwrap();
        let addr2: GalacticAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, addr2);
    }
}
