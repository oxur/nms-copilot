use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::address::GalacticAddress;

/// The kind of object that was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Discovery {
    Planet,
    SolarSystem,
    Sector,
    Animal,
    Flora,
    Mineral,
}

impl fmt::Display for Discovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Planet => write!(f, "Planet"),
            Self::SolarSystem => write!(f, "SolarSystem"),
            Self::Sector => write!(f, "Sector"),
            Self::Animal => write!(f, "Animal"),
            Self::Flora => write!(f, "Flora"),
            Self::Mineral => write!(f, "Mineral"),
        }
    }
}

/// Error returned when parsing a discovery type string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryParseError(pub String);

impl fmt::Display for DiscoveryParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown discovery type: {}", self.0)
    }
}

impl std::error::Error for DiscoveryParseError {}

impl FromStr for Discovery {
    type Err = DiscoveryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planet" => Ok(Self::Planet),
            "solarsystem" | "solar_system" => Ok(Self::SolarSystem),
            "sector" => Ok(Self::Sector),
            "animal" => Ok(Self::Animal),
            "flora" => Ok(Self::Flora),
            "mineral" => Ok(Self::Mineral),
            _ => Err(DiscoveryParseError(s.to_string())),
        }
    }
}

/// A single discovery event from the player's save file.
///
/// The `reality_index` is derived from the `universe_address` and kept in sync
/// by the constructor. Both refer to the galaxy where the discovery was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DiscoveryRecord {
    pub discovery_type: Discovery,
    pub universe_address: GalacticAddress,
    pub timestamp: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub discoverer: Option<String>,
    pub is_uploaded: bool,
}

impl DiscoveryRecord {
    pub fn new(
        discovery_type: Discovery,
        universe_address: GalacticAddress,
        timestamp: Option<DateTime<Utc>>,
        name: Option<String>,
        discoverer: Option<String>,
        is_uploaded: bool,
    ) -> Self {
        Self {
            discovery_type,
            universe_address,
            timestamp,
            name,
            discoverer,
            is_uploaded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_display_fromstr_roundtrip() {
        for d in [
            Discovery::Planet,
            Discovery::SolarSystem,
            Discovery::Sector,
            Discovery::Animal,
            Discovery::Flora,
            Discovery::Mineral,
        ] {
            let s = d.to_string();
            let parsed: Discovery = s.parse().unwrap();
            assert_eq!(d, parsed);
        }
    }

    #[test]
    fn discovery_solar_system_alternate() {
        assert_eq!(
            "solar_system".parse::<Discovery>().unwrap(),
            Discovery::SolarSystem
        );
    }

    #[test]
    fn discovery_record_constructor() {
        let addr = GalacticAddress::new(0, 0, 0, 0x123, 0, 5);
        let rec = DiscoveryRecord::new(
            Discovery::Planet,
            addr,
            None,
            Some("TestPlanet".to_string()),
            None,
            false,
        );
        assert_eq!(rec.discovery_type, Discovery::Planet);
        assert_eq!(rec.universe_address.reality_index, 5);
        assert!(!rec.is_uploaded);
    }
}
