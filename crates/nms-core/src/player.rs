use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::address::GalacticAddress;

/// Maps to PersistentBaseTypes in the save file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum BaseType {
    HomePlanetBase,
    FreighterBase,
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
pub struct BaseTypeParseError(pub String);

impl fmt::Display for BaseTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown base type: {}", self.0)
    }
}

impl std::error::Error for BaseTypeParseError {}

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
    pub name: String,
    pub base_type: BaseType,
    pub address: GalacticAddress,
    /// Position in local planet coordinates [x, y, z].
    pub position: [f32; 3],
    /// Platform-specific user ID.
    pub owner_uid: Option<String>,
}

impl PlayerBase {
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

    /// Galaxy index (convenience accessor for `address.reality_index`).
    pub fn reality_index(&self) -> u8 {
        self.address.reality_index
    }
}

/// Snapshot of the player's current state from the save file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub struct PlayerState {
    pub current_address: GalacticAddress,
    pub current_reality: u8,
    pub previous_address: Option<GalacticAddress>,
    pub freighter_address: Option<GalacticAddress>,
    pub units: u64,
    pub nanites: u64,
    pub quicksilver: u64,
}

impl PlayerState {
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
