//! Planet/system search queries.

use nms_core::address::GalacticAddress;
use nms_core::biome::{Biome, BiomeSubType};
use nms_core::system::{Planet, System, SystemId};
use nms_graph::query::BiomeFilter;
use nms_graph::{GalaxyModel, GraphError};

/// How to determine the "from" point for distance calculations.
#[derive(Debug, Clone, Default)]
pub enum ReferencePoint {
    /// Use the player's current position from the save file.
    #[default]
    CurrentPosition,
    /// Use a named base's position.
    Base(String),
    /// Use an explicit galactic address.
    Address(GalacticAddress),
}

/// Parameters for a planet/system search.
#[derive(Debug, Clone, Default)]
pub struct FindQuery {
    /// Filter by biome type.
    pub biome: Option<Biome>,
    /// Filter by biome subtype/variant.
    pub biome_subtype: Option<BiomeSubType>,
    /// Filter by infested flag.
    pub infested: Option<bool>,
    /// Only include results within this radius (light-years).
    pub within_ly: Option<f64>,
    /// Return at most this many results (nearest first).
    pub nearest: Option<usize>,
    /// Filter by planet/system name pattern (case-insensitive substring).
    pub name_pattern: Option<String>,
    /// Filter by discoverer username (case-insensitive substring).
    pub discoverer: Option<String>,
    /// Only include named planets/systems.
    pub named_only: bool,
    /// Reference point for distance calculations.
    pub from: ReferencePoint,
}

/// A single result from a find query.
#[derive(Debug, Clone)]
pub struct FindResult {
    /// The matching planet.
    pub planet: Planet,
    /// The system containing the planet.
    pub system: System,
    /// Distance from the reference point in light-years.
    pub distance_ly: f64,
    /// Portal address of the planet as 12 hex digits (planet index in the first digit).
    /// Caller renders as emoji.
    pub portal_hex: String,
    /// Portal address of the system as 12 hex digits (planet digit zero). Identical for
    /// every planet in the same system, so it doubles as the group key and as a label
    /// for systems without a name.
    pub system_hex: String,
}

/// Execute a find query against the galaxy model.
///
/// Returns results sorted by distance ascending.
pub fn execute_find(model: &GalaxyModel, query: &FindQuery) -> Result<Vec<FindResult>, GraphError> {
    // Resolve the reference point
    let from = match &query.from {
        ReferencePoint::CurrentPosition => model
            .player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition)?,
        ReferencePoint::Base(name) => model
            .base(name)
            .map(|b| b.address)
            .ok_or_else(|| GraphError::BaseNotFound(name.clone()))?,
        ReferencePoint::Address(addr) => *addr,
    };

    let biome_filter = BiomeFilter {
        biome: query.biome,
        biome_subtype: query.biome_subtype,
        infested: query.infested,
        named_only: query.named_only,
    };

    // Choose between nearest-N or within-radius
    let planet_matches = if let Some(n) = query.nearest {
        model.nearest_planets(&from, n * 2, &biome_filter) // over-fetch for post-filtering
    } else if let Some(radius) = query.within_ly {
        model.planets_within_radius(&from, radius, &biome_filter)
    } else {
        // No spatial constraint: get all matching planets
        let mut all = Vec::new();
        if let Some(biome) = query.biome {
            for planet in model.planets_by_biome(biome) {
                for (&sys_id, system) in &model.systems {
                    if system.planets.iter().any(|p| p.index == planet.index) {
                        let dist = from.distance_ly(&system.address);
                        all.push(((sys_id, planet.index), planet, dist));
                        break;
                    }
                }
            }
        } else {
            for (&sys_id, system) in &model.systems {
                for planet in &system.planets {
                    let dist = from.distance_ly(&system.address);
                    all.push(((sys_id, planet.index), planet, dist));
                }
            }
        }
        all
    };

    let mut results: Vec<FindResult> = planet_matches
        .into_iter()
        .filter_map(|(key, planet, dist)| {
            let system = model.system(&key.0)?;

            // Apply name pattern filter
            if let Some(ref pattern) = query.name_pattern {
                let pattern_lower = pattern.to_lowercase();
                let name_matches = planet
                    .name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&pattern_lower))
                    .unwrap_or(false)
                    || system
                        .name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&pattern_lower))
                        .unwrap_or(false);
                if !name_matches {
                    return None;
                }
            }

            // Apply discoverer filter
            if let Some(ref disc) = query.discoverer {
                let disc_lower = disc.to_lowercase();
                let disc_matches = system
                    .discoverer
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&disc_lower))
                    .unwrap_or(false);
                if !disc_matches {
                    return None;
                }
            }

            // Apply within_ly if both nearest and within are specified
            if let Some(radius) = query.within_ly
                && dist > radius
            {
                return None;
            }

            let sys_addr = &system.address;
            let planet_addr = GalacticAddress::new(
                sys_addr.voxel_x(),
                sys_addr.voxel_y(),
                sys_addr.voxel_z(),
                sys_addr.solar_system_index(),
                planet.index,
                sys_addr.reality_index,
            );
            let portal_hex = format!("{:012X}", planet_addr.packed());
            let system_hex = format!("{:012X}", SystemId::from_address(sys_addr).0);

            Some(FindResult {
                planet: planet.clone(),
                system: system.clone(),
                distance_ly: dist,
                portal_hex,
                system_hex,
            })
        })
        .collect();

    // Sort by distance, keeping each system's planets contiguous and in index order
    // so the display can group them.
    results.sort_by(|a, b| {
        a.distance_ly
            .partial_cmp(&b.distance_ly)
            .unwrap()
            .then_with(|| a.system_hex.cmp(&b.system_hex))
            .then_with(|| a.planet.index.cmp(&b.planet.index))
    });

    // Apply nearest limit
    if let Some(n) = query.nearest {
        results.truncate(n);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x00100000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x00100000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x10100000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x00200000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x10200000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn test_find_all_planets() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_find_by_biome() {
        let model = test_model();
        let query = FindQuery {
            biome: Some(Biome::Lush),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert_eq!(r.planet.biome, Some(Biome::Lush));
        }
    }

    #[test]
    fn test_find_nearest_limit() {
        let model = test_model();
        let query = FindQuery {
            nearest: Some(1),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        assert!(results.len() <= 1);
    }

    #[test]
    fn test_find_from_base() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("Alpha Base".into()),
            ..Default::default()
        };
        let results = execute_find(&model, &query);
        assert!(results.is_ok());
    }

    #[test]
    fn test_find_from_nonexistent_base_errors() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("No Such Base".into()),
            ..Default::default()
        };
        assert!(execute_find(&model, &query).is_err());
    }

    #[test]
    fn test_find_results_sorted_by_distance() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for i in 1..results.len() {
            assert!(results[i].distance_ly >= results[i - 1].distance_ly);
        }
    }

    #[test]
    fn test_find_portal_hex_is_12_digits() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert_eq!(r.portal_hex.len(), 12);
            assert_eq!(r.system_hex.len(), 12);
            // The planet address carries the planet index in its first digit; the
            // system address has a zero there. The rest is identical.
            assert_eq!(&r.portal_hex[1..], &r.system_hex[1..]);
            assert_eq!(&r.system_hex[..1], "0");
            assert_eq!(
                u8::from_str_radix(&r.portal_hex[..1], 16).unwrap(),
                r.planet.index
            );
        }
    }

    #[test]
    fn test_find_by_discoverer() {
        let model = test_model();
        let query = FindQuery {
            discoverer: Some("Explorer".into()),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert!(r.system.discoverer.as_ref().unwrap().contains("Explorer"));
        }
    }

    #[test]
    fn test_find_results_group_planets_by_system() {
        let model = test_model();
        let query = FindQuery {
            biome: None,
            biome_subtype: None,
            infested: None,
            within_ly: None,
            nearest: None,
            name_pattern: None,
            discoverer: None,
            named_only: false,
            from: ReferencePoint::CurrentPosition,
        };
        let results = execute_find(&model, &query).unwrap();
        assert!(results.len() >= 2);
        // Every system's planets appear contiguously: once a system_hex changes it
        // must never reappear later in the list.
        let mut seen: Vec<&str> = Vec::new();
        for r in &results {
            match seen.last() {
                Some(last) if *last == r.system_hex => {}
                _ => {
                    assert!(
                        !seen.contains(&r.system_hex.as_str()),
                        "system {} appears in two separate runs",
                        r.system_hex
                    );
                    seen.push(&r.system_hex);
                }
            }
        }
        // Distances never decrease.
        for w in results.windows(2) {
            assert!(w[0].distance_ly <= w[1].distance_ly);
        }
    }
}
