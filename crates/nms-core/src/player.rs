//! Player bases and the player-state values represented by a save.
//!
//! Constructors copy their arguments. In particular, [PlayerState] stores
//! current_reality separately from the galaxy index in current_address and
//! does not require those values to agree.
//!
//! # Examples
//!
//! ```
//! use nms_core::{GalacticAddress, PlayerState};
//!
//! let address = GalacticAddress::new(0, 0, 0, 1, 0, 9);
//! let state = PlayerState::new(address, 3, None, None, 100, 20, 5);
//! assert_eq!(state.current_address.reality_index, 9);
//! assert_eq!(state.current_reality, 3);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::address::GalacticAddress;

/// Maps to PersistentBaseTypes in the save file.
///
/// Parsing is case-insensitive, accepts the full variant names and the short
/// aliases home, freighter, and external, and does not trim input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum BaseType {
    /// A home-planet base.
    HomePlanetBase,
    /// A freighter base.
    FreighterBase,
    /// An external-planet base.
    ExternalPlanetBase,
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HomePlanetBase => write!(f, "HomePlanetBase"),
            Self::FreighterBase => write!(f, "FreighterBase"),
            Self::ExternalPlanetBase => write!(f, "ExternalPlanetBase"),
        }
    }
}

/// Error returned when parsing a base type string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseTypeParseError(
    /// The original string that did not match a base type.
    pub String,
);

impl fmt::Display for BaseTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown base type: {}", self.0)
    }
}

impl std::error::Error for BaseTypeParseError {}

/// Parse a base type name without regard to case.
///
/// Accepts each full variant name and the aliases `home`, `freighter`, and
/// `external`. Input is not trimmed.
///
/// # Errors
///
/// Returns [BaseTypeParseError] containing the original input when no name
/// matches.
impl FromStr for BaseType {
    type Err = BaseTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "homeplanetbase" | "home" => Ok(Self::HomePlanetBase),
            "freighterbase" | "freighter" => Ok(Self::FreighterBase),
            "externalplanetbase" | "external" => Ok(Self::ExternalPlanetBase),
            _ => Err(BaseTypeParseError(s.to_string())),
        }
    }
}

/// A player-owned base at a specific galactic location.
///
/// The galaxy (reality index) is encoded in the `address` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub struct PlayerBase {
    /// Name assigned to the base.
    pub name: String,
    /// Base classification.
    pub base_type: BaseType,
    /// Galactic address, including the galaxy index.
    pub address: GalacticAddress,
    /// Position in local planet coordinates [x, y, z].
    pub position: [f32; 3],
    /// Platform-specific user ID.
    pub owner_uid: Option<String>,
}

impl PlayerBase {
    /// Copy the supplied base fields into a player-base record.
    pub fn new(
        name: String,
        base_type: BaseType,
        address: GalacticAddress,
        position: [f32; 3],
        owner_uid: Option<String>,
    ) -> Self {
        Self {
            name,
            base_type,
            address,
            position,
            owner_uid,
        }
    }

    /// Return the galaxy index stored in the base address.
    pub fn reality_index(&self) -> u8 {
        self.address.reality_index
    }
}

/// Snapshot of supported player-state values from the save file.
///
/// This is a value record rather than a validating constructor: current_reality
/// is copied independently from current_address.reality_index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub struct PlayerState {
    /// Current galactic address.
    pub current_address: GalacticAddress,
    /// Current reality value copied independently from current_address.
    pub current_reality: u8,
    /// Previous galactic address, when present.
    pub previous_address: Option<GalacticAddress>,
    /// Freighter galactic address, when present.
    pub freighter_address: Option<GalacticAddress>,
    /// Units currency balance.
    pub units: u64,
    /// Nanites currency balance.
    pub nanites: u64,
    /// Quicksilver currency balance.
    pub quicksilver: u64,
}

impl PlayerState {
    /// Copy the supplied values into a player-state snapshot.
    ///
    /// No validation or synchronization is performed between current_reality
    /// and current_address.reality_index.
    pub fn new(
        current_address: GalacticAddress,
        current_reality: u8,
        previous_address: Option<GalacticAddress>,
        freighter_address: Option<GalacticAddress>,
        units: u64,
        nanites: u64,
        quicksilver: u64,
    ) -> Self {
        Self {
            current_address,
            current_reality,
            previous_address,
            freighter_address,
            units,
            nanites,
            quicksilver,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_type_display_fromstr_roundtrip() {
        for bt in [
            BaseType::HomePlanetBase,
            BaseType::FreighterBase,
            BaseType::ExternalPlanetBase,
        ] {
            let s = bt.to_string();
            let parsed: BaseType = s.parse().unwrap();
            assert_eq!(bt, parsed);
        }
    }

    #[test]
    fn base_type_short_names() {
        assert_eq!(
            "home".parse::<BaseType>().unwrap(),
            BaseType::HomePlanetBase
        );
        assert_eq!(
            "freighter".parse::<BaseType>().unwrap(),
            BaseType::FreighterBase
        );
        assert_eq!(
            "external".parse::<BaseType>().unwrap(),
            BaseType::ExternalPlanetBase
        );
    }

    #[test]
    fn player_base_reality_index() {
        let addr = GalacticAddress::new(0, 0, 0, 0, 0, 7);
        let base = PlayerBase::new(
            "My Base".into(),
            BaseType::HomePlanetBase,
            addr,
            [1.0, 2.0, 3.0],
            None,
        );
        assert_eq!(base.reality_index(), 7);
    }

    #[test]
    fn player_state_constructor() {
        let addr = GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let state = PlayerState::new(addr, 0, None, None, 1_000_000, 5000, 200);
        assert_eq!(state.units, 1_000_000);
        assert_eq!(state.nanites, 5000);
        assert_eq!(state.quicksilver, 200);
    }
}
