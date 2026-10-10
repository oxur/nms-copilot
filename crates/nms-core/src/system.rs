use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::address::GalacticAddress;
use crate::biome::{Biome, BiomeSubType};

/// Identifies a star system by galaxy and its canonical planet-zeroed address.
///
/// `system_address` contains only the low 44 coordinate and solar-system bits
/// from a [`GalacticAddress`]. The reality index is stored separately; it is
/// never packed into portal-coordinate bits. Serde uses an object with numeric
/// `reality_index` and `system_address` fields. This replaces the former public
/// tuple/scalar representation and its scalar serialized form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct SystemId {
    reality_index: u8,
    system_address: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemIdFields {
    reality_index: u8,
    system_address: u64,
}

impl<'de> Deserialize<'de> for SystemId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = SystemIdFields::deserialize(deserializer)?;
        if fields.system_address & !0x0FFF_FFFF_FFFF != 0 {
            return Err(serde::de::Error::custom(
                "system_address must contain only the 44 system-coordinate bits",
            ));
        }

        Ok(Self {
            reality_index: fields.reality_index,
            system_address: fields.system_address,
        })
    }
}

impl SystemId {
    /// Create a galaxy-qualified key from a galactic address.
    ///
    /// The galaxy is retained separately and planet-index bits are cleared.
    pub fn from_address(addr: &GalacticAddress) -> Self {
        Self {
            reality_index: addr.reality_index,
            system_address: addr.packed() & 0x0FFF_FFFF_FFFF,
        }
    }

    /// Return the galaxy (reality) index for this system.
    pub fn reality_index(&self) -> u8 {
        self.reality_index
    }

    /// Return the planet-zeroed 44-bit portal coordinate for this system.
    pub fn system_address(&self) -> u64 {
        self.system_address
    }
}

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:012X}", self.reality_index, self.system_address)
    }
}

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
    use serde_json::json;

    #[test]
    fn system_reality_index_from_address() {
        let addr = GalacticAddress::new(0, 0, 0, 0x123, 0, 42);
        let sys = System::new(addr, None, None, None, vec![]);
        assert_eq!(sys.reality_index(), 42);
    }

    #[test]
    fn test_system_id_qualifies_planet_zeroed_address_by_reality() {
        let zero = GalacticAddress::from_packed(0x150003AB8C07, 0);
        let nine = GalacticAddress::from_packed(0x150003AB8C07, 9);
        let another_planet = GalacticAddress::from_packed(0xF50003AB8C07, 9);

        let id_zero = SystemId::from_address(&zero);
        let id_nine = SystemId::from_address(&nine);
        assert_ne!(id_zero, id_nine);
        assert_eq!(id_zero.system_address(), 0x050003AB8C07);
        assert_eq!(id_nine.system_address(), 0x050003AB8C07);
        assert_eq!(id_nine.reality_index(), 9);
        assert_eq!(id_nine, SystemId::from_address(&another_planet));
        assert_eq!(id_nine.to_string(), "9:050003AB8C07");

        let mut ids = std::collections::HashSet::new();
        ids.insert(id_zero);
        ids.insert(id_nine);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_system_id_serializes_as_exact_numeric_object_and_roundtrips() {
        let address = GalacticAddress::from_packed(0x150003AB8C07, 9);
        let id = SystemId::from_address(&address);
        let value = serde_json::to_value(id).unwrap();
        assert_eq!(
            value,
            json!({"reality_index": 9, "system_address": 0x050003AB8C07_u64})
        );
        assert_eq!(serde_json::from_value::<SystemId>(value).unwrap(), id);
    }

    #[test]
    fn test_system_id_deserialization_rejects_noncanonical_values() {
        for value in [
            json!(0x050003AB8C07_u64),
            json!({"system_address": 0x050003AB8C07_u64}),
            json!({"reality_index": 0}),
            json!({"reality_index": 0, "system_address": 0, "extra": 1}),
            json!({"reality_index": "9", "system_address": 0}),
            json!({"reality_index": -1, "system_address": 0}),
            json!({"reality_index": 256, "system_address": 0}),
            json!({"reality_index": 9, "system_address": -1}),
            json!({"reality_index": 9, "system_address": 0x1000000000000_u64}),
            json!({"reality_index": 9, "system_address": 0x150003AB8C07_u64}),
        ] {
            assert!(
                serde_json::from_value::<SystemId>(value).is_err(),
                "expected invalid SystemId JSON to be rejected"
            );
        }

        let error = serde_json::from_value::<SystemId>(json!({
            "reality_index": 9,
            "system_address": 0x150003AB8C07_u64
        }))
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "system_address must contain only the 44 system-coordinate bits"
        );
    }

    #[test]
    fn test_system_id_deserialization_accepts_reality_boundaries_and_zero_address() {
        for reality_index in [0, 255] {
            let value = json!({"reality_index": reality_index, "system_address": 0});
            let id = serde_json::from_value::<SystemId>(value).unwrap();
            assert_eq!(id.reality_index(), reality_index);
            assert_eq!(id.system_address(), 0);
        }
    }

    #[test]
    fn test_system_id_deserialization_rejects_coordinate_overflow() {
        let result = serde_json::from_str::<SystemId>(
            r#"{"reality_index":9,"system_address":18446744073709551616}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn planet_constructor() {
        let p = Planet::new(3, Some(Biome::Lush), None, false, Some("Eden".into()), None);
        assert_eq!(p.index, 3);
        assert_eq!(p.biome, Some(Biome::Lush));
        assert!(!p.infested);
    }
}
