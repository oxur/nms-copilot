//! Extract System/Planet data from raw save discovery records.
//!
//! The checked entry points preserve parser-validated addresses; the legacy
//! entry points use the older typed-save address path.
//!
//! # Checked extraction example
//!
//! ```
//! use nms_core::{GalacticAddress, SystemId};
//! use nms_graph::GalaxyModel;
//!
//! let json = br#"{
//!   "Version":4720,"Platform":"Test","ActiveContext":"Main",
//!   "CommonStateData":{},
//!   "BaseContext":{"PlayerStateData":{"UniverseAddress":{
//!     "RealityIndex":9,"GalacticAddress":{"VoxelX":3,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2}},
//!     "Units":0,"Nanites":0,"Specials":0,"PersistentPlayerBases":[]}},
//!   "ExpeditionContext":{},
//!   "DiscoveryManagerData":{"DiscoveryData-v1":{"Store":{"Record":[
//!     {"DD":{"UA":"0x00010900000003","DT":"SolarSystem"},"DM":{"CN":"Atlas"},"OWS":{"USN":"Explorer"}},
//!     {"DD":{"UA":"0x20010900000003","DT":"Planet","VP":["0x2A","0x10000"]},"DM":{"CN":"Garden"},"OWS":{}}
//!   ]}}}
//! }"#;
//! let checked = nms_save::parse_validated_save(json)?;
//! let model = GalaxyModel::from_validated_save(&checked);
//! let expected = SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 0, 9));
//! let system = model.system(&expected).ok_or("missing checked system")?;
//! assert_eq!(expected.to_string(), "9:000100000003");
//! assert_eq!(system.address.planet_index(), 0);
//! assert_eq!(system.name.as_deref(), Some("Atlas"));
//! assert_eq!(system.discoverer.as_deref(), Some("Explorer"));
//! assert_eq!(system.source, nms_core::DiscoverySource::Personal);
//! assert_eq!(model.active_galaxy, 9);
//! assert_eq!(model.player_position().map(|address| (address.reality_index, address.planet_index(), address.voxel_x())), Some((9, 2, 3)));
//! let planet = model.planets.get(&(expected, 2)).ok_or("missing checked planet")?;
//! assert_eq!((planet.index, planet.name.as_deref(), planet.biome, planet.infested, planet.seed_hash), (2, Some("Garden"), Some(nms_core::Biome::Lush), true, Some(42)));
//! assert_eq!(model.system_count(), 1);
//! assert_eq!(model.planet_count(), 1);
//! assert_eq!(system.planets.iter().map(|planet| (planet.index, planet.name.as_deref(), planet.biome, planet.infested, planet.seed_hash)).collect::<Vec<_>>(), vec![(2, Some("Garden"), Some(nms_core::Biome::Lush), true, Some(42))]);
//! let names = nms_graph::extract::extract_discovery_names_from_validated(&checked);
//! assert_eq!(names.system_names, std::collections::HashMap::from([(expected, "Atlas".to_owned())]));
//! assert_eq!(names.planet_names, std::collections::HashMap::from([((expected, 2), "Garden".to_owned())]));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::{HashMap, HashSet};

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::delta::DiscoveryNames;
use nms_core::system::{Planet, System};
use nms_save::ValidatedSave;
use nms_save::model::{RawDiscoveryRecord, SaveRoot};

use crate::spatial::SystemId;

/// Extract literal first-row discovery names from a validated save.
///
/// System and planet rows are tracked independently. The first row for each
/// effective key reserves it even when `DM.CN` is absent, null, or non-string;
/// unknown discovery types reserve neither kind of key. Planet names remain
/// keyed to their exact system and planet index; an orphan planet row does not
/// create a system-name observation. Addresses come from parser-validated
/// record/address pairs, so raw packed addresses are
/// not decoded again. Name strings are retained literally, including whitespace
/// and empty strings.
///
/// During system extraction, solar-system rows are collected first and planet
/// rows second. The first solar-system row supplies system metadata; an orphan
/// planet row can create a system using that row's address. Nonempty `OWS.USN`
/// supplies the discoverer, positive `OWS.TS` is converted through `u64`-to-`i64`
/// and may not fit a timestamp, and biome subtype is unavailable from `VP`.
/// Checked extraction uses literal names and paired source addresses; legacy
/// extraction assumes galaxy 0 and does not apply `DM.CN` names. These examples
/// describe this parser path, not authentic game-format completeness.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let bytes = std::fs::read("save.json")?;
/// let save = nms_save::parse_validated_save(&bytes)?;
/// let names = nms_graph::extract::extract_discovery_names_from_validated(&save);
/// println!("{} system names", names.system_names.len());
/// # Ok(())
/// # }
/// ```
pub fn extract_discovery_names_from_validated(save: &ValidatedSave) -> DiscoveryNames {
    let mut names = DiscoveryNames::default();
    let mut seen_systems = HashSet::new();
    let mut seen_planets = HashSet::new();

    for (record, address) in save.discoveries() {
        match record.dd.dt.as_str() {
            "SolarSystem" => {
                let system_id = SystemId::from_address(&address);
                if seen_systems.insert(system_id)
                    && let Some(name) = record.dm.get("CN").and_then(serde_json::Value::as_str)
                {
                    names.system_names.insert(system_id, name.to_owned());
                }
            }
            "Planet" => {
                let key = (SystemId::from_address(&address), address.planet_index());
                if seen_planets.insert(key)
                    && let Some(name) = record.dm.get("CN").and_then(serde_json::Value::as_str)
                {
                    names.planet_names.insert(key, name.to_owned());
                }
            }
            _ => {}
        }
    }

    names
}

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
///   `VP[0]`: seed hash (hex string or integer)
///   `VP[1]`: biome/flags packed integer
///     - bits 0..15 (mask 0xFFFF): biome type index (GcBiomeType enum)
///     - bit 16 (mask 0x10000): infested flag
///
/// Returns `(None, false)` when the VP array has fewer than two entries.
/// Otherwise returns `(biome, infested)`. Flags accept an unsigned JSON integer or a
/// hexadecimal string, optionally prefixed with `0x` or `0X`; a bare string is
/// parsed as hexadecimal, not decimal. Unknown biome indices return `None`
/// while a recognized infested bit is retained. Invalid flags return
/// `(None, false)`.
///
/// # Examples
///
/// ```
/// use nms_core::Biome;
/// use nms_graph::extract::extract_biome_from_vp;
///
/// assert_eq!(extract_biome_from_vp(&[serde_json::json!(0), serde_json::json!("0x10000")]), (Some(Biome::Lush), true));
/// assert_eq!(extract_biome_from_vp(&[serde_json::json!(0), serde_json::json!("0x1000B")]), (None, true));
/// assert_eq!(extract_biome_from_vp(&[serde_json::json!(0), serde_json::json!("not-hex")]), (None, false));
/// ```
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

/// Extract a seed hash from `VP[0]` as an unsigned integer or hexadecimal string.
///
/// Hexadecimal strings follow `u64::from_str_radix` after an optional `0x` or
/// `0X` prefix; unprefixed strings are hexadecimal, and a leading plus is
/// accepted by that parser. Negative or nonintegral JSON numbers, malformed
/// strings, and other JSON types return `None`.
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
/// Groups discovery records by system key and returns a map of
/// [`SystemId`] to [`System`]. The first solar-system row for a key supplies its
/// stored address, discoverer, and timestamp; checked extraction also uses its
/// literal `DM.CN` name, while legacy extraction does not. Extracted systems
/// have [`nms_core::DiscoverySource::Personal`] provenance; community identity is not
/// inferred from discovery records. Planet rows are handled in a second pass
/// and may create an orphan system from their own address, without SolarSystem-row
/// name, discoverer, or timestamp. Biome subtype is unavailable from `VP`. The
/// legacy path decodes packed addresses with galaxy 0 and does not extract
/// literal `DM.CN` discovery names.
pub fn extract_systems(save: &SaveRoot) -> HashMap<SystemId, System> {
    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    extract_systems_with_records(
        || {
            records
                .iter()
                .map(|record| (record, GalacticAddress::from_packed(record.dd.ua.0, 0)))
        },
        None,
    )
}

/// Extract systems using the checked addresses paired by [`ValidatedSave`].
///
/// Solar-system and planet rows are processed in separate passes. The first
/// row for each effective system or planet key supplies its data; an orphan
/// planet row can create a system, and its stored address can retain that
/// planet index while the [`SystemId`] key remains planet-zeroed. It follows the
/// same SolarSystem metadata, orphan-row, and Personal provenance rules as
/// [`extract_systems`]. Biome subtype is unavailable from `VP`.
///
/// See the [module's checked extraction example](crate::extract#checked-extraction-example)
/// for a complete model and literal identity walkthrough.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
/// let systems = nms_graph::extract::extract_systems_from_validated(&save);
/// println!("{} systems", systems.len());
/// # Ok(())
/// # }
/// ```
pub fn extract_systems_from_validated(save: &ValidatedSave) -> HashMap<SystemId, System> {
    let names = extract_discovery_names_from_validated(save);
    extract_systems_with_records(|| save.discoveries(), Some(&names))
}

fn extract_systems_with_records<'a, F, I>(
    records: F,
    names: Option<&DiscoveryNames>,
) -> HashMap<SystemId, System>
where
    F: Fn() -> I,
    I: Iterator<Item = (&'a RawDiscoveryRecord, GalacticAddress)>,
{
    let mut builders: HashMap<SystemId, SystemBuilder> = HashMap::new();

    // First pass: collect SolarSystem discoveries (for system names/discoverers)
    for (rec, addr) in records() {
        if rec.dd.dt != "SolarSystem" {
            continue;
        }
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
            name: names
                .and_then(|names| names.system_names.get(&sys_id))
                .filter(|name| !name.is_empty())
                .cloned(),
            discoverer,
            timestamp,
            planets: Vec::new(),
        });
    }

    // Second pass: collect Planet discoveries and attach to systems
    for (rec, addr) in records() {
        if rec.dd.dt != "Planet" {
            continue;
        }
        let sys_id = SystemId::from_address(&addr);
        let planet_index = addr.planet_index();

        let (biome, infested) = extract_biome_from_vp(&rec.dd.vp);
        let seed_hash = extract_seed_from_vp(&rec.dd.vp);

        let planet = Planet::new(
            planet_index,
            biome,
            None, // BiomeSubType not extractable from VP
            infested,
            names
                .and_then(|names| names.planet_names.get(&(sys_id, planet_index)))
                .filter(|name| !name.is_empty())
                .cloned(),
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
