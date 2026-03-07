//! Extract System/Planet data from raw save discovery records.

use std::collections::HashMap;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::system::{Planet, System};
use nms_save::model::SaveRoot;

use crate::spatial::SystemId;

/// Temporary accumulator for building a System from multiple discovery records.
#[derive(Debug)]
struct SystemBuilder {
    address: GalacticAddress,
    name: Option<String>,
    discoverer: Option<String>,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    planets: Vec<Planet>,
}

/// Extract biome and infested flag from a discovery record's VP array.
///
/// VP array format (for Planet discovery type):
///   VP[0]: seed hash (hex string or integer)
///   VP[1]: biome/flags packed integer
///     - bits 0..15 (mask 0xFFFF): biome type index (GcBiomeType enum)
///     - bit 16 (mask 0x10000): infested flag
///
/// Returns `(biome, infested)`. Returns `(None, false)` if VP is empty or
/// the format is unrecognized.
pub fn extract_biome_from_vp(vp: &[serde_json::Value]) -> (Option<Biome>, bool) {
    if vp.len() < 2 {
        return (None, false);
    }

    // VP[1] can be a hex string "0x..." or integer
    let flags = match &vp[1] {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => {
            let hex = s
                .strip_prefix("0x")
                .or_else(|| s.strip_prefix("0X"))
                .unwrap_or(s);
            u64::from_str_radix(hex, 16).ok()
        }
        _ => None,
    };

    let Some(flags) = flags else {
        return (None, false);
    };

    let infested = (flags >> 16) & 1 == 1;

    // Biome type is in the lower 16 bits (mask 0xFFFF).
    // Mapping matches GcBiomeType::BiomeEnum ordering from game data.
    let biome_index = (flags & 0xFFFF) as u16;
    let biome = match biome_index {
        0 => Some(Biome::Lush),
        1 => Some(Biome::Toxic),
        2 => Some(Biome::Scorched),
        3 => Some(Biome::Radioactive),
        4 => Some(Biome::Frozen),
        5 => Some(Biome::Barren),
        6 => Some(Biome::Dead),
        7 => Some(Biome::Weird),
        8 => Some(Biome::Red),
        9 => Some(Biome::Green),
        10 => Some(Biome::Blue),
        11 => None, // "Test" biome in game data -- skip
        12 => Some(Biome::Swamp),
        13 => Some(Biome::Lava),
        14 => Some(Biome::Waterworld),
        15 => Some(Biome::GasGiant),
        _ => None,
    };

    (biome, infested)
}

/// Extract seed hash from VP[0].
pub fn extract_seed_from_vp(vp: &[serde_json::Value]) -> Option<u64> {
    if vp.is_empty() {
        return None;
    }
    match &vp[0] {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => {
            let hex = s
                .strip_prefix("0x")
                .or_else(|| s.strip_prefix("0X"))
                .unwrap_or(s);
            u64::from_str_radix(hex, 16).ok()
        }
        _ => None,
    }
}

/// Build systems and planets from a parsed save file's discovery records.
///
/// Groups discovery records by system address, extracts planet biome data,
/// and returns a map of SystemId -> System.
pub fn extract_systems(save: &SaveRoot) -> HashMap<SystemId, System> {
    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    let mut builders: HashMap<SystemId, SystemBuilder> = HashMap::new();

    // First pass: collect SolarSystem discoveries (for system names/discoverers)
    for rec in records {
        if rec.dd.dt != "SolarSystem" {
            continue;
        }
        let addr = GalacticAddress::from_packed(rec.dd.ua.0, 0);
        let sys_id = SystemId::from_address(&addr);

        let timestamp = if rec.ows.ts > 0 {
            chrono::DateTime::from_timestamp(rec.ows.ts as i64, 0)
        } else {
            None
        };

        let discoverer = if rec.ows.usn.is_empty() {
            None
        } else {
            Some(rec.ows.usn.clone())
        };

        builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None, // System names aren't in discovery records
            discoverer,
            timestamp,
            planets: Vec::new(),
        });
    }

    // Second pass: collect Planet discoveries and attach to systems
    for rec in records {
        if rec.dd.dt != "Planet" {
            continue;
        }
        let addr = GalacticAddress::from_packed(rec.dd.ua.0, 0);
        let sys_id = SystemId::from_address(&addr);
        let planet_index = addr.planet_index();

        let (biome, infested) = extract_biome_from_vp(&rec.dd.vp);
        let seed_hash = extract_seed_from_vp(&rec.dd.vp);

        let planet = Planet::new(
            planet_index,
            biome,
            None, // BiomeSubType not extractable from VP
            infested,
            None, // Planet names aren't in discovery records
            seed_hash,
        );

        let builder = builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None,
            discoverer: None,
            timestamp: None,
            planets: Vec::new(),
        });

        // Avoid duplicate planet indices
        if !builder.planets.iter().any(|p| p.index == planet_index) {
            builder.planets.push(planet);
        }
    }

    // Convert builders to Systems
    builders
        .into_iter()
        .map(|(id, b)| {
            let system = System::new(b.address, b.name, b.discoverer, b.timestamp, b.planets);
            (id, system)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_biome_from_vp_empty() {
        assert_eq!(extract_biome_from_vp(&[]), (None, false));
    }

    #[test]
    fn test_extract_biome_from_vp_single_element() {
        let vp = vec![serde_json::json!("0xABCD")];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn test_extract_biome_lush_not_infested() {
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(0)];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Lush), false));
    }

    #[test]
    fn test_extract_biome_toxic_infested() {
        // bit 16 set = infested, low byte = 1 = Toxic
        let flags = (1u64 << 16) | 1;
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(flags)];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Toxic), true));
    }

    #[test]
    fn test_extract_biome_from_hex_string() {
        // 0x00010005 = bit 16 set (infested) + 5 (Barren)
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!("0x10005")];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Barren), true));
    }

    #[test]
    fn test_extract_biome_unknown_index() {
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(255)];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn test_extract_biome_test_index_skipped() {
        // Index 11 is "Test" biome -- should return None
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(11)];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn test_extract_biome_all_valid_indices() {
        let expected = [
            (0, Some(Biome::Lush)),
            (1, Some(Biome::Toxic)),
            (2, Some(Biome::Scorched)),
            (3, Some(Biome::Radioactive)),
            (4, Some(Biome::Frozen)),
            (5, Some(Biome::Barren)),
            (6, Some(Biome::Dead)),
            (7, Some(Biome::Weird)),
            (8, Some(Biome::Red)),
            (9, Some(Biome::Green)),
            (10, Some(Biome::Blue)),
            (12, Some(Biome::Swamp)),
            (13, Some(Biome::Lava)),
            (14, Some(Biome::Waterworld)),
            (15, Some(Biome::GasGiant)),
        ];
        for (idx, biome) in expected {
            let vp = vec![serde_json::json!("0x0"), serde_json::json!(idx)];
            assert_eq!(
                extract_biome_from_vp(&vp),
                (biome, false),
                "Failed for biome index {idx}"
            );
        }
    }

    #[test]
    fn test_extract_biome_vp1_not_number_or_string() {
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(true)];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn test_extract_seed_from_vp_hex() {
        let vp = vec![serde_json::json!("0xD6911E7B1D31085E")];
        assert_eq!(extract_seed_from_vp(&vp), Some(0xD6911E7B1D31085E));
    }

    #[test]
    fn test_extract_seed_from_vp_integer() {
        let vp = vec![serde_json::json!(12345)];
        assert_eq!(extract_seed_from_vp(&vp), Some(12345));
    }

    #[test]
    fn test_extract_seed_from_vp_empty() {
        assert_eq!(extract_seed_from_vp(&[]), None);
    }

    #[test]
    fn test_extract_seed_from_vp_not_number_or_string() {
        let vp = vec![serde_json::json!(null)];
        assert_eq!(extract_seed_from_vp(&vp), None);
    }
}
