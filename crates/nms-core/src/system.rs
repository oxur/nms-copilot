use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::address::GalacticAddress;
use crate::biome::{Biome, BiomeSubType};

/// A star system containing one or more planets.
///
/// The galaxy (reality index) is encoded in the `address` field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct System {
    pub address: GalacticAddress,
    pub name: Option<String>,
    pub discoverer: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub planets: Vec<Planet>,
}

impl System {
    pub fn new(
        address: GalacticAddress,
        name: Option<String>,
        discoverer: Option<String>,
        timestamp: Option<DateTime<Utc>>,
        planets: Vec<Planet>,
    ) -> Self {
        Self {
            address,
            name,
            discoverer,
            timestamp,
            planets,
        }
    }

    /// Galaxy index (convenience accessor for `address.reality_index`).
    pub fn reality_index(&self) -> u8 {
        self.address.reality_index
    }
}

/// A planet within a star system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub struct Planet {
    /// Planet index within the system (0-15).
    pub index: u8,
    pub biome: Option<Biome>,
    pub biome_subtype: Option<BiomeSubType>,
    /// Whether the planet has infested (biological horror) variant.
    pub infested: bool,
    pub name: Option<String>,
    /// Procedural generation seed.
    pub seed_hash: Option<u64>,
}

impl Planet {
    pub fn new(
        index: u8,
        biome: Option<Biome>,
        biome_subtype: Option<BiomeSubType>,
        infested: bool,
        name: Option<String>,
        seed_hash: Option<u64>,
    ) -> Self {
        Self {
            index,
            biome,
            biome_subtype,
            infested,
            name,
            seed_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_reality_index_from_address() {
        let addr = GalacticAddress::new(0, 0, 0, 0x123, 0, 42);
        let sys = System::new(addr, None, None, None, vec![]);
        assert_eq!(sys.reality_index(), 42);
    }

    #[test]
    fn planet_constructor() {
        let p = Planet::new(3, Some(Biome::Lush), None, false, Some("Eden".into()), None);
        assert_eq!(p.index, 3);
        assert_eq!(p.biome, Some(Biome::Lush));
        assert!(!p.infested);
    }
}
