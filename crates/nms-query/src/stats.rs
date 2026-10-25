//! Aggregate statistics across the complete in-memory model.

use std::collections::HashMap;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;

/// Compatibility flags for choosing statistics. The current [`execute_stats`]
/// implementation computes all fields and ignores both flags; see the [`crate#examples`] for both flag settings.
#[derive(Debug, Clone, Default)]
pub struct StatsQuery {
    /// Whether to include biome distribution (currently ignored).
    pub biomes: bool,
    /// Whether to include discovery counts by type (currently ignored).
    pub discoveries: bool,
}

/// Counts computed across all model records and galaxies. Biome-map entry order
/// is unspecified. See the [`crate#examples`] for complete
/// expected counts and biome entries.
#[derive(Debug, Clone)]
pub struct StatsResult {
    /// Number of indexed systems across all galaxies.
    pub system_count: usize,
    /// Number of entries in the model's flattened planet map.
    pub planet_count: usize,
    /// Number of indexed bases across all galaxies.
    pub base_count: usize,
    /// Counts for planets with a known biome.
    pub biome_counts: HashMap<Biome, usize>,
    /// Planets whose biome is `None`.
    pub unknown_biome_count: usize,
    /// Planets whose name is `Some`, including an empty string.
    pub named_planet_count: usize,
    /// Systems whose name is `Some`, including an empty string.
    pub named_system_count: usize,
    /// Planets with the infested flag set, independent of biome.
    pub infested_count: usize,
}

/// Compute every statistic over the entire model, regardless of active galaxy.
/// `query.biomes` and `query.discoveries` are currently ignored. The [`crate#examples`] checks both flag settings.
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

    let named_system_count = model.systems.values().filter(|s| s.name.is_some()).count();

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
    fn test_stats_basic_counts() {
        let model = test_model();
        let query = StatsQuery {
            biomes: true,
            discoveries: true,
        };
        let stats = execute_stats(&model, &query);
        assert_eq!(stats.system_count, 1);
        assert_eq!(stats.planet_count, 2);
    }

    #[test]
    fn test_stats_biome_counts_sum() {
        let model = test_model();
        let stats = execute_stats(&model, &StatsQuery::default());
        let total: usize = stats.biome_counts.values().sum::<usize>() + stats.unknown_biome_count;
        assert_eq!(total, stats.planet_count);
    }
}
