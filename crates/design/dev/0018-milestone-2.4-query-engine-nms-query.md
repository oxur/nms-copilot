# Milestone 2.4 -- Query Engine (nms-query)

Pure, stateless query types and execution functions over `&GalaxyModel`. Three query types: `FindQuery` (search planets), `ShowQuery` (detail views), `StatsQuery` (aggregate statistics).

## Crate: `nms-query`

Path: `crates/nms-query/`

### Dependencies to update in `crates/nms-query/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-graph = { workspace = true }
nms-save = { workspace = true }

[dev-dependencies]
serde_json = "1"
```

---

## Module Structure

### File: `crates/nms-query/src/lib.rs`

```rust
//! Shared query engine for NMS Copilot.
//!
//! Pure, stateless query layer consumed by all three interfaces (CLI, REPL, MCP).
//! Takes an immutable reference to the `GalaxyModel` and returns typed results.

pub mod find;
pub mod show;
pub mod stats;

pub use find::{FindQuery, FindResult, ReferencePoint};
pub use show::{ShowQuery, ShowResult};
pub use stats::{StatsQuery, StatsResult};
```

---

## File: `crates/nms-query/src/find.rs`

```rust
//! Planet/system search queries.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::system::{Planet, System};
use nms_graph::query::BiomeFilter;
use nms_graph::{GalaxyModel, GraphError};

/// How to determine the "from" point for distance calculations.
#[derive(Debug, Clone)]
pub enum ReferencePoint {
    /// Use the player's current position from the save file.
    CurrentPosition,
    /// Use a named base's position.
    Base(String),
    /// Use an explicit galactic address.
    Address(GalacticAddress),
}

impl Default for ReferencePoint {
    fn default() -> Self {
        Self::CurrentPosition
    }
}

/// Parameters for a planet/system search.
#[derive(Debug, Clone, Default)]
pub struct FindQuery {
    /// Filter by biome type.
    pub biome: Option<Biome>,
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
    /// Portal glyphs as hex (12 digits). Caller renders as emoji.
    pub portal_hex: String,
}

/// Execute a find query against the galaxy model.
///
/// Returns results sorted by distance ascending.
pub fn execute_find(
    model: &GalaxyModel,
    query: &FindQuery,
) -> Result<Vec<FindResult>, GraphError> {
    // Resolve the reference point
    let from = match &query.from {
        ReferencePoint::CurrentPosition => model
            .player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition)?,
        ReferencePoint::Base(name) => model
            .get_base(name)
            .map(|b| b.address)
            .ok_or_else(|| GraphError::BaseNotFound(name.clone()))?,
        ReferencePoint::Address(addr) => *addr,
    };

    let biome_filter = BiomeFilter {
        biome: query.biome,
        infested: query.infested,
        named_only: query.named_only,
    };

    // Choose between nearest-N or within-radius
    let planet_matches = if let Some(n) = query.nearest {
        model.nearest_planets(&from, n * 2, &biome_filter) // over-fetch for post-filtering
    } else if let Some(radius) = query.within_ly {
        model.planets_within_radius(&from, radius, &biome_filter)
    } else {
        // No spatial constraint: get all matching planets by biome
        let mut all = Vec::new();
        if let Some(biome) = query.biome {
            for planet in model.planets_by_biome(biome) {
                // Find the system for this planet
                for (&sys_id, system) in &model.systems {
                    if system.planets.iter().any(|p| std::ptr::eq(p, planet)) {
                        let dist = from.distance_ly(&system.address);
                        all.push(((sys_id, planet.index), planet, dist));
                        break;
                    }
                }
            }
        } else {
            // No biome filter, no spatial constraint: return all planets
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
            let system = model.get_system(&key.0)?;

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
            if let Some(radius) = query.within_ly {
                if dist > radius {
                    return None;
                }
            }

            let portal_hex = format!("{:012X}", system.address.packed());

            Some(FindResult {
                planet: planet.clone(),
                system: system.clone(),
                distance_ly: dist,
                portal_hex,
            })
        })
        .collect();

    // Sort by distance
    results.sort_by(|a, b| a.distance_ly.partial_cmp(&b.distance_ly).unwrap());

    // Apply nearest limit
    if let Some(n) = query.nearest {
        results.truncate(n);
    }

    Ok(results)
}
```

---

## File: `crates/nms-query/src/show.rs`

```rust
//! Detail view queries for systems, planets, and bases.

use nms_core::player::PlayerBase;
use nms_core::system::{Planet, System};
use nms_graph::{GalaxyModel, GraphError};
use nms_graph::spatial::SystemId;

/// What to show detail for.
#[derive(Debug, Clone)]
pub enum ShowQuery {
    /// Show a system by name or packed address.
    System(String),
    /// Show a base by name.
    Base(String),
}

/// Result of a show query.
#[derive(Debug, Clone)]
pub enum ShowResult {
    System(ShowSystemResult),
    Base(ShowBaseResult),
}

#[derive(Debug, Clone)]
pub struct ShowSystemResult {
    pub system: System,
    pub portal_hex: String,
    pub galaxy_name: String,
    pub distance_from_player: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ShowBaseResult {
    pub base: PlayerBase,
    pub portal_hex: String,
    pub galaxy_name: String,
    pub system: Option<System>,
    pub distance_from_player: Option<f64>,
}

/// Execute a show query.
pub fn execute_show(
    model: &GalaxyModel,
    query: &ShowQuery,
) -> Result<ShowResult, GraphError> {
    match query {
        ShowQuery::System(name_or_id) => show_system(model, name_or_id),
        ShowQuery::Base(name) => show_base(model, name),
    }
}

fn show_system(model: &GalaxyModel, name_or_id: &str) -> Result<ShowResult, GraphError> {
    // Try name lookup first
    let (sys_id, system) = if let Some(result) = model.get_system_by_name(name_or_id) {
        result
    } else {
        // Try as packed hex address
        let hex = name_or_id
            .strip_prefix("0x")
            .or_else(|| name_or_id.strip_prefix("0X"))
            .unwrap_or(name_or_id);
        let packed = u64::from_str_radix(hex, 16)
            .map_err(|_| GraphError::SystemNotFound(name_or_id.to_string()))?;
        let id = SystemId(packed & 0x0FFF_FFFF_FFFF);
        let system = model
            .get_system(&id)
            .ok_or_else(|| GraphError::SystemNotFound(name_or_id.to_string()))?;
        (&id, system)
    };

    let portal_hex = format!("{:012X}", system.address.packed());
    let galaxy = nms_core::galaxy::Galaxy::by_index(system.address.reality_index);

    let distance_from_player = model
        .player_position()
        .map(|pos| pos.distance_ly(&system.address));

    Ok(ShowResult::System(ShowSystemResult {
        system: system.clone(),
        portal_hex,
        galaxy_name: galaxy.name.to_string(),
        distance_from_player,
    }))
}

fn show_base(model: &GalaxyModel, name: &str) -> Result<ShowResult, GraphError> {
    let base = model
        .get_base(name)
        .ok_or_else(|| GraphError::BaseNotFound(name.to_string()))?;

    let portal_hex = format!("{:012X}", base.address.packed());
    let galaxy = nms_core::galaxy::Galaxy::by_index(base.address.reality_index);

    // Try to find the system this base is in
    let sys_id = SystemId::from_address(&base.address);
    let system = model.get_system(&sys_id).cloned();

    let distance_from_player = model
        .player_position()
        .map(|pos| pos.distance_ly(&base.address));

    Ok(ShowResult::Base(ShowBaseResult {
        base: base.clone(),
        portal_hex,
        galaxy_name: galaxy.name.to_string(),
        system,
        distance_from_player,
    }))
}
```

---

## File: `crates/nms-query/src/stats.rs`

```rust
//! Aggregate statistics queries.

use std::collections::HashMap;

use nms_core::biome::Biome;
use nms_core::discovery::Discovery;
use nms_graph::GalaxyModel;

/// What statistics to compute.
#[derive(Debug, Clone, Default)]
pub struct StatsQuery {
    /// Show biome distribution.
    pub biomes: bool,
    /// Show discovery counts by type.
    pub discoveries: bool,
}

/// Aggregate statistics result.
#[derive(Debug, Clone)]
pub struct StatsResult {
    /// Total systems in model.
    pub system_count: usize,
    /// Total planets in model.
    pub planet_count: usize,
    /// Total bases.
    pub base_count: usize,
    /// Biome distribution: biome -> count of planets.
    pub biome_counts: HashMap<Biome, usize>,
    /// Planets with no biome assigned.
    pub unknown_biome_count: usize,
    /// Named vs unnamed planets.
    pub named_planet_count: usize,
    /// Named vs unnamed systems.
    pub named_system_count: usize,
    /// Infested planet count.
    pub infested_count: usize,
}

/// Execute a stats query.
pub fn execute_stats(model: &GalaxyModel, _query: &StatsQuery) -> StatsResult {
    let mut biome_counts: HashMap<Biome, usize> = HashMap::new();
    let mut unknown_biome_count = 0;
    let mut named_planet_count = 0;
    let mut infested_count = 0;

    for planet in model.planets.values() {
        match planet.biome {
            Some(biome) => *biome_counts.entry(biome).or_default() += 1,
            None => unknown_biome_count += 1,
        }
        if planet.name.is_some() {
            named_planet_count += 1;
        }
        if planet.infested {
            infested_count += 1;
        }
    }

    let named_system_count = model
        .systems
        .values()
        .filter(|s| s.name.is_some())
        .count();

    StatsResult {
        system_count: model.system_count(),
        planet_count: model.planet_count(),
        base_count: model.base_count(),
        biome_counts,
        unknown_biome_count,
        named_planet_count,
        named_system_count,
        infested_count,
    }
}
```

---

## Tests

### File: `crates/nms-query/src/find.rs` (inline tests at bottom)

```rust
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
                    "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn find_all_planets() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn find_by_biome() {
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
    fn find_nearest_limit() {
        let model = test_model();
        let query = FindQuery {
            nearest: Some(1),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        assert!(results.len() <= 1);
    }

    #[test]
    fn find_from_base() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("Alpha Base".into()),
            ..Default::default()
        };
        let results = execute_find(&model, &query);
        assert!(results.is_ok());
    }

    #[test]
    fn find_from_nonexistent_base_errors() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("No Such Base".into()),
            ..Default::default()
        };
        assert!(execute_find(&model, &query).is_err());
    }

    #[test]
    fn find_results_sorted_by_distance() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for i in 1..results.len() {
            assert!(results[i].distance_ly >= results[i - 1].distance_ly);
        }
    }

    #[test]
    fn find_portal_hex_is_12_digits() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert_eq!(r.portal_hex.len(), 12);
        }
    }

    #[test]
    fn find_by_discoverer() {
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
}
```

### File: `crates/nms-query/src/show.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        // Same as find tests -- reuse the JSON
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn show_base_by_name() {
        let model = test_model();
        let result = execute_show(&model, &ShowQuery::Base("Alpha Base".into())).unwrap();
        match result {
            ShowResult::Base(b) => {
                assert_eq!(b.base.name, "Alpha Base");
                assert_eq!(b.galaxy_name, "Euclid");
                assert_eq!(b.portal_hex.len(), 12);
            }
            _ => panic!("Expected Base result"),
        }
    }

    #[test]
    fn show_base_case_insensitive() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::Base("alpha base".into())).is_ok());
    }

    #[test]
    fn show_base_not_found() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::Base("No Base".into())).is_err());
    }

    #[test]
    fn show_system_not_found() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::System("No System".into())).is_err());
    }
}
```

### File: `crates/nms-query/src/stats.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x201000000064", "DT": "Planet", "VP": ["0xCD", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn stats_basic_counts() {
        let model = test_model();
        let query = StatsQuery { biomes: true, discoveries: true };
        let stats = execute_stats(&model, &query);
        assert_eq!(stats.system_count, 1);
        assert_eq!(stats.planet_count, 2);
    }

    #[test]
    fn stats_biome_counts_sum() {
        let model = test_model();
        let stats = execute_stats(&model, &StatsQuery::default());
        let total: usize = stats.biome_counts.values().sum::<usize>() + stats.unknown_biome_count;
        assert_eq!(total, stats.planet_count);
    }
}
```

---

## Implementation Notes

1. **All queries are pure functions** -- they take `&GalaxyModel` and return owned result types. No mutation, no side effects. This makes them trivially shareable across CLI/REPL/MCP.

2. **`FindResult` includes cloned data** -- cloning System/Planet is intentional. Results must outlive the model borrow for display. These are small structs.

3. **`portal_hex` is raw hex, not emoji** -- the display layer (milestone 2.5) handles emoji rendering. The query layer just provides the 12-digit hex string.

4. **Over-fetch then filter** -- `nearest_planets` fetches `n * 2` candidates to account for post-filtering (name pattern, discoverer). This is a heuristic; if results are sparse, the spatial iterator naturally explores further.

5. **`ReferencePoint::Base` is case-insensitive** -- base lookup goes through `model.get_base()` which lowercases the name.

6. **`StatsResult` is always fully populated** -- even if the query only asks for biomes, we compute everything. Stats are cheap on hundreds of systems.
