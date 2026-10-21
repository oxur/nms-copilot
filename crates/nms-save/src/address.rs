//! Strict, field-specific decoding of save-file galactic addresses.
//!
//! These functions are an opt-in boundary for callers that still hold the
//! original JSON. They validate the save representation before constructing
//! the core 48-bit portal-frame address. Packed field family is selected by
//! the named function, never inferred from a value's width or an active-galaxy
//! fallback.
//!
//! # Packed save-field values
//!
//! Packed discovery and base fields use the 56-bit
//! `P-SSS-GG-YY-ZZZ-XXX` layout: planet index (4 bits), solar-system index
//! (12), recorded galaxy/reality index (8), signed voxel Y (8), signed voxel
//! Z (12), and signed voxel X (12). The recorded galaxy is kept in
//! [`GalacticAddress::reality_index`]; the core portal address is a separate
//! 48-bit representation and does not contain that galaxy byte.
//!
//! A JSON integer is accepted only from 0 through 72,057,594,037,927,935
//! (2^56 - 1). A JSON string is hexadecimal: one through fourteen ASCII
//! hexadecimal digits, optionally preceded by exactly one `0x` or `0X`.
//! Characters are checked before width, so malformed text returns
//! [`AddressDecodeError::InvalidHex`] even when it also has too many digits;
//! valid fifteen-digit text returns [`AddressDecodeError::PackedWidth`]. No
//! whitespace, sign, separator, fraction, exponent, or decimal-string coercion
//! is accepted. Thus bare string `"10"` means hexadecimal 16, while integer
//! `10` means decimal 10. Zero is a valid packed address.
//!
//! # Expanded address objects
//!
//! Current and previous values must be objects with an integer
//! `RealityIndex` in 0..=255 and a `GalacticAddress` object containing integer
//! `VoxelX` and `VoxelZ` in -2048..=2047, `VoxelY` in -128..=127,
//! `SolarSystemIndex` in 0..=4095, and `PlanetIndex` in 0..=15. Members are
//! required; strings, booleans, nulls, fractions, and exponent-form numbers
//! are not coerced to integers. Extra unrelated keys are ignored. Validation
//! follows the fixed order RealityIndex, GalacticAddress, X, Y, Z, SSI, P, so
//! the first reported error is deterministic.
//!
//! # Presence, paths, and scope
//!
//! The `Option` argument represents whether the caller found the JSON member:
//! `None` is absence and `Some(Null)` is a present value with the wrong type.
//! Previous absence returns `None`; a present previous object returns `None`
//! only when all six validated components, including planet and galaxy, are
//! zero. Any nonzero component is retained. A current all-zero object is a
//! valid address. The supplied path is a diagnostic label; fixed child names
//! are appended to it, and the path is not parsed as a selector or filesystem
//! location. Inputs are borrowed and are not modified; decoding performs no
//! I/O or global-state changes.
//!
//! This module decodes one address value only. Existing legacy parsers and
//! conversions are unchanged, and this module does not provide whole-save
//! atomic validation or application of the decoded values; that belongs to
//! the caller's higher-level update boundary.
//!
//! # Examples
//!
//! ```
//! use nms_save::address::decode_base_address;
//! let value = serde_json::json!("0x40050003AB8C07");
//! let address = decode_base_address(Some(&value), "/Base/GalacticAddress")?;
//! assert_eq!(address.solar_system_index(), 5);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use nms_core::GalacticAddress;
use serde_json::Value;

const MAX_PACKED: u64 = 0x00FF_FFFF_FFFF_FFFF;
const MAX_PACKED_I128: i128 = 72_057_594_037_927_935;
const PACKED_EXPECTED: &str = "an unsigned integer or hexadecimal string";

/// A failure while validating or decoding a save-file address.
///
/// Paths identify the exact JSON member that was absent or invalid. The
/// variants may grow as the decoder gains additional validation details.
///
/// # Examples
///
/// ```
/// use nms_save::address::{decode_discovery_address, AddressDecodeError};
/// let value = serde_json::json!(true);
/// let error = decode_discovery_address(Some(&value), "/Discovery/UA")
///     .err()
///     .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "expected a type error"))?;
/// assert!(matches!(error, AddressDecodeError::InvalidType { .. }));
/// assert_eq!(error.path(), "/Discovery/UA");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AddressDecodeError {
    /// A required JSON value or member was absent.
    #[error("missing required address value at {path}")]
    Missing {
        /// Caller path with any missing child member appended.
        path: String,
    },
    /// A value had the wrong JSON kind.
    #[error("expected {expected} at {path}")]
    InvalidType {
        /// Caller path with the invalid member appended.
        path: String,
        /// Human-readable JSON kind required by this decoder.
        expected: &'static str,
    },
    /// A packed-address string was empty or contained non-hexadecimal text.
    #[error("invalid hexadecimal address at {path}")]
    InvalidHex {
        /// Caller path identifying the malformed packed field.
        path: String,
    },
    /// A packed integer or hexadecimal string exceeded its 56-bit width.
    #[error("packed address exceeds 56 bits at {path}")]
    PackedWidth {
        /// Caller path identifying the oversized packed field.
        path: String,
    },
    /// A decoded numeric component was outside its encoded range.
    #[error("value {value} at {path} is outside {minimum}..={maximum}")]
    OutOfRange {
        /// Caller path with the invalid component appended.
        path: String,
        /// Exact integer observed in the JSON value before narrowing.
        value: i128,
        /// Inclusive minimum permitted by the encoded field.
        minimum: i128,
        /// Inclusive maximum permitted by the encoded field.
        maximum: i128,
    },
}

impl AddressDecodeError {
    /// Return the JSON path associated with this error.
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_save::address::decode_base_address;
    /// let error = decode_base_address(None, "/Bases/0/UA")
    ///     .err()
    ///     .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "expected a missing value"))?;
    /// assert_eq!(error.path(), "/Bases/0/UA");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn path(&self) -> &str {
        match self {
            Self::Missing { path }
            | Self::InvalidType { path, .. }
            | Self::InvalidHex { path }
            | Self::PackedWidth { path }
            | Self::OutOfRange { path, .. } => path,
        }
    }
}

/// Decode a discovery `DD.UA` packed address from its original JSON value.
///
/// The field family is selected by this function, not by the value's width.
/// A valid value preserves the galaxy byte encoded in the save address.
/// See the module-level [packed-field contract](index.html#packed-save-field-values) for
/// accepted JSON types, lexical rules, width, and bit layout.
///
/// # Errors
///
/// Returns [`AddressDecodeError`] for a missing value, unsupported JSON kind,
/// malformed hexadecimal string, packed value wider than 56 bits, or negative
/// integer.
///
/// # Examples
///
/// ```
/// use nms_save::address::decode_discovery_address;
/// let value = serde_json::json!("0x212B0004B2446A");
/// let address = decode_discovery_address(Some(&value), "/DD/UA")?;
/// assert_eq!(address.reality_index, 0);
/// assert_eq!(address.voxel_x(), 1130);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_discovery_address(
    value: Option<&Value>,
    path: &str,
) -> Result<GalacticAddress, AddressDecodeError> {
    decode_packed(value, path)
}

/// Decode a persistent-base `GalacticAddress` packed value from original JSON.
///
/// This entry point deliberately remains distinct from discovery decoding so
/// callers select semantics from the source field, never numeric width.
/// See the module-level [packed-field contract](index.html#packed-save-field-values) for
/// accepted JSON types, lexical rules, width, and bit layout.
///
/// # Errors
///
/// Returns [`AddressDecodeError`] for a missing value, unsupported JSON kind,
/// malformed hexadecimal string, packed value wider than 56 bits, or negative
/// integer.
///
/// # Examples
///
/// ```
/// use nms_save::address::decode_base_address;
/// let value = serde_json::json!(0x40050003AB8C07_u64);
/// let address = decode_base_address(Some(&value), "/Bases/0/GalacticAddress")?;
/// assert_eq!(address.planet_index(), 4);
/// assert_eq!(address.voxel_x(), -1017);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_base_address(
    value: Option<&Value>,
    path: &str,
) -> Result<GalacticAddress, AddressDecodeError> {
    decode_packed(value, path)
}

/// Decode a complete expanded current-address wrapper from original JSON.
///
/// The wrapper must contain `RealityIndex` and a complete `GalacticAddress`
/// object. Validation occurs before the core constructor narrows components.
/// See the module-level [expanded-object contract](index.html#expanded-address-objects)
/// for required members, ranges, validation order, and diagnostic paths.
///
/// # Errors
///
/// Returns [`AddressDecodeError`] when the wrapper or any required member is
/// absent, has the wrong JSON kind, or falls outside its encoded component
/// range.
///
/// # Examples
///
/// ```
/// use nms_save::address::decode_current_address;
/// let value = serde_json::json!({
///     "RealityIndex": 9,
///     "GalacticAddress": {
///         "VoxelX": 1130, "VoxelY": 4, "VoxelZ": -1244,
///         "SolarSystemIndex": 299, "PlanetIndex": 2
///     }
/// });
/// let address = decode_current_address(Some(&value), "/Player/UniverseAddress")?;
/// assert_eq!(address.reality_index, 9);
/// assert_eq!(address.voxel_z(), -1244);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_current_address(
    value: Option<&Value>,
    path: &str,
) -> Result<GalacticAddress, AddressDecodeError> {
    decode_expanded(value, path).map(|decoded| decoded.address)
}

/// Decode a previous-address wrapper, preserving the explicit zero sentinel.
///
/// An absent wrapper or a complete all-six-components-zero wrapper is `None`.
/// Every present wrapper is fully validated before that sentinel is applied.
/// See the module-level [presence contract](index.html#presence-paths-and-scope) for the
/// distinction between absence and JSON null.
///
/// # Errors
///
/// Returns [`AddressDecodeError`] when a present wrapper or required member is
/// invalid or outside its encoded component range.
///
/// # Examples
///
/// ```
/// use nms_save::address::decode_previous_address;
/// let value = serde_json::json!({
///     "RealityIndex": 0,
///     "GalacticAddress": {
///         "VoxelX": 0, "VoxelY": 0, "VoxelZ": 0,
///         "SolarSystemIndex": 0, "PlanetIndex": 0
///     }
/// });
/// assert!(decode_previous_address(Some(&value), "/Player/PreviousUniverseAddress")?.is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_previous_address(
    value: Option<&Value>,
    path: &str,
) -> Result<Option<GalacticAddress>, AddressDecodeError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let decoded = decode_expanded(Some(value), path)?;
    Ok((decoded.reality_index != 0
        || decoded.planet_index != 0
        || decoded.solar_system_index != 0
        || decoded.voxel_x != 0
        || decoded.voxel_y != 0
        || decoded.voxel_z != 0)
        .then_some(decoded.address))
}

fn decode_packed(value: Option<&Value>, path: &str) -> Result<GalacticAddress, AddressDecodeError> {
    let value = value.ok_or_else(|| AddressDecodeError::Missing {
        path: path.to_owned(),
    })?;
    let packed = match value {
        Value::Number(number) => {
            if let Some(unsigned) = number.as_u64() {
                if unsigned > MAX_PACKED {
                    return Err(AddressDecodeError::PackedWidth {
                        path: path.to_owned(),
                    });
                }
                unsigned
            } else if let Some(signed) = number.as_i64() {
                if signed < 0 {
                    return Err(AddressDecodeError::OutOfRange {
                        path: path.to_owned(),
                        value: i128::from(signed),
                        minimum: 0,
                        maximum: MAX_PACKED_I128,
                    });
                }
                signed as u64
            } else {
                return Err(invalid_type(path, PACKED_EXPECTED));
            }
        }
        Value::String(text) => parse_packed_hex(text, path)?,
        _ => return Err(invalid_type(path, PACKED_EXPECTED)),
    };
    Ok(address_from_packed(packed))
}

fn parse_packed_hex(text: &str, path: &str) -> Result<u64, AddressDecodeError> {
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AddressDecodeError::InvalidHex {
            path: path.to_owned(),
        });
    }
    if digits.len() > 14 {
        return Err(AddressDecodeError::PackedWidth {
            path: path.to_owned(),
        });
    }
    u64::from_str_radix(digits, 16).map_err(|_| AddressDecodeError::PackedWidth {
        path: path.to_owned(),
    })
}

fn address_from_packed(packed: u64) -> GalacticAddress {
    let planet = ((packed >> 52) & 0x0F) as u8;
    let solar_system = ((packed >> 40) & 0x0FFF) as u16;
    let reality = ((packed >> 32) & 0xFF) as u8;
    let voxel_y = sign_extend(((packed >> 24) & 0xFF) as u16, 8) as i8;
    let voxel_z = sign_extend(((packed >> 12) & 0x0FFF) as u16, 12);
    let voxel_x = sign_extend((packed & 0x0FFF) as u16, 12);
    GalacticAddress::new(voxel_x, voxel_y, voxel_z, solar_system, planet, reality)
}

fn sign_extend(raw: u16, bits: u32) -> i16 {
    let sign_bit = 1_u16 << (bits - 1);
    if raw & sign_bit == 0 {
        raw as i16
    } else {
        (raw as i32 - (1_i32 << bits)) as i16
    }
}

struct ExpandedAddress {
    address: GalacticAddress,
    reality_index: u8,
    planet_index: u8,
    solar_system_index: u16,
    voxel_x: i16,
    voxel_y: i8,
    voxel_z: i16,
}

fn decode_expanded(
    value: Option<&Value>,
    path: &str,
) -> Result<ExpandedAddress, AddressDecodeError> {
    let value = value.ok_or_else(|| AddressDecodeError::Missing {
        path: path.to_owned(),
    })?;
    let wrapper = value
        .as_object()
        .ok_or_else(|| invalid_type(path, "an object"))?;
    let reality_path = child_path(path, "RealityIndex");
    let reality = component(wrapper.get("RealityIndex"), &reality_path, 0, 255)? as u8;

    let galactic_path = child_path(path, "GalacticAddress");
    let galactic = wrapper
        .get("GalacticAddress")
        .ok_or_else(|| AddressDecodeError::Missing {
            path: galactic_path.clone(),
        })?
        .as_object()
        .ok_or_else(|| invalid_type(&galactic_path, "an object"))?;

    let x_path = child_path(&galactic_path, "VoxelX");
    let voxel_x = component(galactic.get("VoxelX"), &x_path, -2048, 2047)? as i16;
    let y_path = child_path(&galactic_path, "VoxelY");
    let voxel_y = component(galactic.get("VoxelY"), &y_path, -128, 127)? as i8;
    let z_path = child_path(&galactic_path, "VoxelZ");
    let voxel_z = component(galactic.get("VoxelZ"), &z_path, -2048, 2047)? as i16;
    let ssi_path = child_path(&galactic_path, "SolarSystemIndex");
    let solar_system_index =
        component(galactic.get("SolarSystemIndex"), &ssi_path, 0, 4095)? as u16;
    let planet_path = child_path(&galactic_path, "PlanetIndex");
    let planet_index = component(galactic.get("PlanetIndex"), &planet_path, 0, 15)? as u8;

    Ok(ExpandedAddress {
        address: GalacticAddress::new(
            voxel_x,
            voxel_y,
            voxel_z,
            solar_system_index,
            planet_index,
            reality,
        ),
        reality_index: reality,
        planet_index,
        solar_system_index,
        voxel_x,
        voxel_y,
        voxel_z,
    })
}

fn component(
    value: Option<&Value>,
    path: &str,
    minimum: i128,
    maximum: i128,
) -> Result<i128, AddressDecodeError> {
    let value = value.ok_or_else(|| AddressDecodeError::Missing {
        path: path.to_owned(),
    })?;
    let number = value
        .as_number()
        .ok_or_else(|| invalid_type(path, "an integer"))?;
    let integer = number
        .as_i64()
        .map(i128::from)
        .or_else(|| number.as_u64().map(i128::from))
        .ok_or_else(|| invalid_type(path, "an integer"))?;
    if integer < minimum || integer > maximum {
        return Err(AddressDecodeError::OutOfRange {
            path: path.to_owned(),
            value: integer,
            minimum,
            maximum,
        });
    }
    Ok(integer)
}

fn invalid_type(path: &str, expected: &'static str) -> AddressDecodeError {
    AddressDecodeError::InvalidType {
        path: path.to_owned(),
        expected,
    }
}

fn child_path(parent: &str, child: &str) -> String {
    format!("{parent}/{child}")
}
