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
    /// Whether `discoverer`/`timestamp` came from a record carrying a custom name.
    named_credit: bool,
    planets: Vec<Planet>,
}

/// Unix time of No Man's Sky's release (2016-08-09). Discovery timestamps before this
/// are placeholders and are ignored.
const NMS_RELEASE_TS: i64 = 1_470_700_800;

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
        let addr = rec.dd.ua.to_galactic_address(0);
        let sys_id = SystemId::from_address(&addr);

        let timestamp = if rec.ows.ts as i64 >= NMS_RELEASE_TS {
            chrono::DateTime::from_timestamp(rec.ows.ts as i64, 0)
        } else {
            None
        };

        let discoverer = if rec.ows.usn.is_empty() {
            None
        } else {
            Some(rec.ows.usn.clone())
        };

        let name = rec.dm.name();
        let named = name.is_some();
        let builder = builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None,
            discoverer: None,
            timestamp: None,
            named_credit: false,
            planets: Vec::new(),
        });
        if builder.name.is_none() {
            builder.name = name;
        }
        // The same system can have several SolarSystem records (the player's own plus
        // one synced from the discovery server). Credit the record that carries the
        // custom name; failing that, the earliest plausible timestamp.
        let take_credit = if named != builder.named_credit {
            named
        } else {
            match (timestamp, builder.timestamp) {
                (Some(t), Some(existing)) => t < existing,
                (Some(_), None) => true,
                (None, _) => builder.discoverer.is_none(),
            }
        };
        if take_credit {
            builder.discoverer = discoverer;
            builder.timestamp = timestamp;
            builder.named_credit = named;
        }
    }

    // Second pass: collect Planet discoveries and attach to systems
    for rec in records {
        if rec.dd.dt != "Planet" {
            continue;
        }
        let addr = rec.dd.ua.to_galactic_address(0);
        let sys_id = SystemId::from_address(&addr);
        let planet_index = addr.planet_index();

        let (biome, infested) = extract_biome_from_vp(&rec.dd.vp);
        let seed_hash = extract_seed_from_vp(&rec.dd.vp);

        let planet = Planet::new(
            planet_index,
            biome,
            None, // BiomeSubType not extractable from VP
            infested,
            rec.dm.name(),
            seed_hash,
        );

        let builder = builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None,
            discoverer: None,
            timestamp: None,
            named_credit: false,
            planets: Vec::new(),
        });

        // Avoid duplicate planet indices
        if !builder.planets.iter().any(|p| p.index == planet_index) {
            builder.planets.push(planet);
        }
    }

    // Third pass: fill in generated system names from space station teleporter
    // endpoints. A station is named after its system ("Atlasa Stellar Observer"),
    // and the endpoint list is the only place the save records a generated name.
    // A player-assigned custom name from the discovery record always wins.
    for context in [&save.base_context, &save.expedition_context] {
        for endpoint in &context.player_state_data.teleport_endpoints {
            let Some(name) = endpoint.system_name() else {
                continue;
            };
            let ua = &endpoint.universe_address;
            let addr = ua.galactic_address.to_galactic_address(ua.reality_index);
            let sys_id = SystemId::from_address(&addr);
            if let Some(builder) = builders.get_mut(&sys_id)
                && builder.name.is_none()
            {
                builder.name = Some(name);
            }
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

    #[test]
    fn test_extract_systems_uses_custom_names() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Win|Final",
            "ActiveContext": "Main",
            "CommonStateData": {},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {}},
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100,
                    "ReserveManaged": 100,
                    "Store": {"Record": [
                        {"DD": {"UA": 606934656187883, "DT": "SolarSystem", "VP": ["0x77C0A655CCBDA20F"]}, "DM": {"CN": "Best Rest"}, "OWS": {"UID": "1", "USN": "Santa", "PTK": "ST", "TS": 1471091917}, "FL": {"C": 1}},
                        {"DD": {"UA": 606934656187883, "DT": "Planet", "VP": ["0x1234", 6]}, "DM": {"CN": "Rest Stop"}, "OWS": {"UID": "1", "USN": "Santa", "PTK": "ST", "TS": 1471091917}, "FL": {"C": 1}},
                        {"DD": {"UA": 498082938293634, "DT": "SolarSystem", "VP": ["0xD9F543C64FB79748"]}, "DM": {}, "OWS": {"UID": "2", "USN": "Someone", "PTK": "ST", "TS": 1756915149}, "FL": {"C": 1}}
                    ]}
                }
            }
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        let systems = extract_systems(&save);
        assert_eq!(systems.len(), 2);

        let named = systems
            .values()
            .find(|s| s.name.is_some())
            .expect("named system");
        assert_eq!(named.name.as_deref(), Some("Best Rest"));
        assert_eq!(named.planets.len(), 1);
        assert_eq!(named.planets[0].name.as_deref(), Some("Rest Stop"));
        assert_eq!(named.planets[0].biome, Some(Biome::Dead));

        let unnamed = systems
            .values()
            .find(|s| s.name.is_none())
            .expect("unnamed system");
        assert_eq!(unnamed.discoverer.as_deref(), Some("Someone"));
    }

    #[test]
    fn test_extract_systems_names_from_station_endpoints() {
        // Two discovered systems in voxel (-532,-4,-1706): index 47 (no custom name) and
        // index 46 (custom-named "PIRATEBAY"), station endpoints for both, a base endpoint,
        // and a station in a system that was never discovered.
        let json = r#"{
            "Version": 4720,
            "Platform": "Win|Final",
            "ActiveContext": "Main",
            "CommonStateData": {},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {
                "TeleportEndpoints": [
                    {"Name": "Hiuship-Naw Exchange", "TeleporterType": "Spacestation", "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": -532, "VoxelY": -4, "VoxelZ": -1706, "SolarSystemIndex": 47, "PlanetIndex": 0}}},
                    {"Name": "PIRATEBAY Exchange", "TeleporterType": "Spacestation", "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": -532, "VoxelY": -4, "VoxelZ": -1706, "SolarSystemIndex": 46, "PlanetIndex": 0}}},
                    {"Name": "Radioactive Base", "TeleporterType": "Base", "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": -532, "VoxelY": -4, "VoxelZ": -1706, "SolarSystemIndex": 47, "PlanetIndex": 2}}},
                    {"Name": "Nowhere Orbital", "TeleporterType": "Spacestation", "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 1, "VoxelY": 1, "VoxelZ": 1, "SolarSystemIndex": 1, "PlanetIndex": 0}}}
                ]
            }},
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100,
                    "ReserveManaged": 100,
                    "Store": {"Record": [
                        {"DD": {"UA": 51681284156908, "DT": "SolarSystem", "VP": ["0x1"]}, "DM": {}, "OWS": {"UID": "1", "USN": "nevenall", "PTK": "ST", "TS": 1}, "FL": {"C": 1}},
                        {"DD": {"UA": 50581772529132, "DT": "SolarSystem", "VP": ["0x2"]}, "DM": {"CN": "PIRATEBAY"}, "OWS": {"UID": "2", "USN": "Santa", "PTK": "ST", "TS": 1}, "FL": {"C": 1}}
                    ]}
                }
            }
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        let systems = extract_systems(&save);
        assert_eq!(
            systems.len(),
            2,
            "station-only endpoints must not create systems"
        );

        let endpoints = &save.base_context.player_state_data.teleport_endpoints;
        let id_of = |i: usize| {
            let ua = &endpoints[i].universe_address;
            SystemId::from_address(&ua.galactic_address.to_galactic_address(ua.reality_index))
        };
        assert_eq!(systems[&id_of(0)].name.as_deref(), Some("Hiuship-Naw"));
        assert_eq!(systems[&id_of(0)].address.solar_system_index(), 47);
        assert_eq!(
            systems[&id_of(1)].name.as_deref(),
            Some("PIRATEBAY"),
            "custom name wins over station name"
        );
    }

    #[test]
    fn test_extract_systems_credits_earliest_solar_system_record() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Win|Final",
            "ActiveContext": "Main",
            "CommonStateData": {},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {}},
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100,
                    "ReserveManaged": 100,
                    "Store": {"Record": [
                        {"DD": {"UA": "0x00022801FC957DEB", "DT": "SolarSystem", "VP": ["0x1"]}, "DM": {}, "OWS": {"UID": "1", "USN": "nevenall", "PTK": "ST", "TS": 1417496658}, "FL": {"C": 1}},
                        {"DD": {"UA": "0x00022800FC957DEB", "DT": "SolarSystem", "VP": ["0x1"]}, "DM": {"CN": "Best Rest"}, "OWS": {"UID": "2", "USN": "Santa", "PTK": "ST", "TS": 1757022865}, "FL": {"C": 1}},
                        {"DD": {"UA": "0x00032801FC957DEB", "DT": "SolarSystem", "VP": ["0x1"]}, "DM": {}, "OWS": {"UID": "1", "USN": "nevenall", "PTK": "ST", "TS": 1417496658}, "FL": {"C": 1}},
                        {"DD": {"UA": "0x00032800FC957DEB", "DT": "SolarSystem", "VP": ["0x1"]}, "DM": {}, "OWS": {"UID": "3", "USN": "Later", "PTK": "ST", "TS": 1600000000}, "FL": {"C": 1}}
                    ]}
                }
            }
        }"#;
        let save: SaveRoot = serde_json::from_str(json).unwrap();
        let systems = extract_systems(&save);
        assert_eq!(systems.len(), 2, "flag byte must not split a system in two");

        // The record carrying the custom name is credited even though it is newer.
        let named = systems.values().find(|s| s.name.is_some()).unwrap();
        assert_eq!(named.name.as_deref(), Some("Best Rest"));
        assert_eq!(named.discoverer.as_deref(), Some("Santa"));
        assert_eq!(named.address.solar_system_index(), 552);

        // Without a name, a pre-release timestamp is a placeholder and loses to a real one.
        let unnamed = systems.values().find(|s| s.name.is_none()).unwrap();
        assert_eq!(unnamed.discoverer.as_deref(), Some("Later"));
        assert_eq!(unnamed.timestamp.map(|t| t.timestamp()), Some(1600000000));
        assert_eq!(unnamed.address.solar_system_index(), 808);
    }
}
