//! Pipeline integration tests for the NMS Copilot library stack.
//!
//! These tests exercise the full pipeline: parse fixture -> build model ->
//! run queries, validating that the library crates work together correctly
//! without going through the CLI binary.

use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::display::{format_find_results, format_stats};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::stats::{StatsQuery, execute_stats};
use nms_query::theme::Theme;

/// Helper: resolve a test fixture path from the workspace root.
fn fixture_path(name: &str) -> PathBuf {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    workspace.join("data").join("test").join(name)
}

/// Helper: parse a fixture and build a GalaxyModel.
fn model_from_fixture(name: &str) -> GalaxyModel {
    let path = fixture_path(name);
    let save = nms_save::parse_save_file(&path).unwrap();
    GalaxyModel::from_save(&save)
}

// ---- Minimal save fixture tests ----

#[test]
fn test_pipeline_parse_minimal_fixture_file_succeeds() {
    let path = fixture_path("minimal_save.json");
    let save = nms_save::parse_save_file(&path).unwrap();
    assert_eq!(save.version, 4720);
    assert_eq!(save.common_state_data.save_name, "Integration Test Save");
    assert_eq!(save.base_context.game_mode, 1);
}

#[test]
fn test_pipeline_minimal_model_has_expected_counts() {
    let model = model_from_fixture("minimal_save.json");
    assert_eq!(model.system_count(), 1);
    assert_eq!(model.planet_count(), 1);
    assert_eq!(model.base_count(), 1);
}

#[test]
fn test_pipeline_minimal_model_base_lookup_works() {
    let model = model_from_fixture("minimal_save.json");
    let base = model.base("Homestead Alpha").unwrap();
    assert_eq!(base.name, "Homestead Alpha");
}

#[test]
fn test_pipeline_minimal_model_player_position_set() {
    let model = model_from_fixture("minimal_save.json");
    let pos = model.player_position().unwrap();
    assert_eq!(pos.voxel_x(), 100);
    assert_eq!(pos.voxel_y(), 50);
    assert_eq!(pos.voxel_z(), -200);
}

// ---- Multi-system save fixture tests ----

#[test]
fn test_pipeline_parse_multi_system_fixture_file_succeeds() {
    let path = fixture_path("multi_system_save.json");
    let save = nms_save::parse_save_file(&path).unwrap();
    assert_eq!(save.version, 4720);
    assert_eq!(save.common_state_data.save_name, "Multi System Test");
}

#[test]
fn test_pipeline_multi_system_model_has_six_systems() {
    let model = model_from_fixture("multi_system_save.json");
    // 6 systems: player's system + 5 others
    assert_eq!(model.system_count(), 6);
}

#[test]
fn test_pipeline_multi_system_model_has_expected_planet_count() {
    let model = model_from_fixture("multi_system_save.json");
    // 6 systems with 2, 2, 2, 3, 2, 2 planets = 13
    assert_eq!(model.planet_count(), 13);
}

#[test]
fn test_pipeline_multi_system_model_has_two_bases() {
    let model = model_from_fixture("multi_system_save.json");
    assert_eq!(model.base_count(), 2);
}

#[test]
fn test_pipeline_multi_system_biome_index_has_lush() {
    let model = model_from_fixture("multi_system_save.json");
    let lush = model.planets_by_biome(Biome::Lush);
    // Player's system planet 1 (Lush), System 1 planet 0 (Lush), System 3 planet 2 (Lush)
    assert_eq!(lush.len(), 3);
}

#[test]
fn test_pipeline_multi_system_biome_index_has_toxic() {
    let model = model_from_fixture("multi_system_save.json");
    let toxic = model.planets_by_biome(Biome::Toxic);
    // System 2 planet index 1 (biome flag=1=Toxic),
    // System 5 planet index 1 (biome flag=0x10001=Toxic+infested)
    assert_eq!(toxic.len(), 2);
}

#[test]
fn test_pipeline_multi_system_biome_index_has_frozen() {
    let model = model_from_fixture("multi_system_save.json");
    let frozen = model.planets_by_biome(Biome::Frozen);
    // Player's system planet 2 (Frozen), System 3 planet 1 (Frozen)
    assert_eq!(frozen.len(), 2);
}

#[test]
fn test_pipeline_multi_system_biome_index_has_barren() {
    let model = model_from_fixture("multi_system_save.json");
    let barren = model.planets_by_biome(Biome::Barren);
    // System 1 planet index 2 (biome flag=5=Barren),
    // System 4 planet index 1 (biome flag=5=Barren)
    assert_eq!(barren.len(), 2);
}

#[test]
fn test_pipeline_multi_system_biome_index_has_scorched() {
    let model = model_from_fixture("multi_system_save.json");
    let scorched = model.planets_by_biome(Biome::Scorched);
    // System 2 planet index 2 (biome flag=2=Scorched)
    assert_eq!(scorched.len(), 1);
}

#[test]
fn test_pipeline_multi_system_infested_planet_exists() {
    let model = model_from_fixture("multi_system_save.json");
    let toxic = model.planets_by_biome(Biome::Toxic);
    let infested_count = toxic.iter().filter(|p| p.infested).count();
    // System 5 planet index 1 (0x10001 = Toxic + infested)
    assert_eq!(infested_count, 1);
}

// ---- Find query pipeline tests ----

#[test]
fn test_pipeline_find_all_returns_all_planets() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery::default();
    let results = execute_find(&model, &query).unwrap();
    assert_eq!(results.len(), 13);
}

#[test]
fn test_pipeline_find_lush_biome_filter() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery {
        biome: Some(Biome::Lush),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert_eq!(results.len(), 3);
    for r in &results {
        assert_eq!(r.planet.biome, Some(Biome::Lush));
    }
}

#[test]
fn test_pipeline_find_nearest_respects_limit() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery {
        nearest: Some(3),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 3);
}

#[test]
fn test_pipeline_find_from_base_reference_works() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery {
        from: ReferencePoint::Base("Lush Haven".into()),
        nearest: Some(2),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 2);
}

#[test]
fn test_pipeline_find_infested_filter() {
    let model = model_from_fixture("multi_system_save.json");
    // Use nearest to ensure spatial-path filtering is applied
    let query = FindQuery {
        infested: Some(true),
        nearest: Some(20),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].planet.infested);
}

#[test]
fn test_pipeline_find_results_have_portal_hex() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery {
        nearest: Some(1),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(!results.is_empty());
    // Portal hex should be 12 characters
    assert_eq!(results[0].portal_hex.len(), 12);
}

#[test]
fn test_pipeline_find_formatted_output_contains_headers() {
    let model = model_from_fixture("multi_system_save.json");
    let query = FindQuery::default();
    let results = execute_find(&model, &query).unwrap();
    let output = format_find_results(&results, &Theme::none());
    assert!(output.contains("Portal Glyphs"));
}

// ---- Stats query pipeline tests ----

#[test]
fn test_pipeline_stats_counts_match_model() {
    let model = model_from_fixture("multi_system_save.json");
    let query = StatsQuery {
        biomes: true,
        discoveries: true,
    };
    let result = execute_stats(&model, &query);
    assert_eq!(result.system_count, 6);
    assert_eq!(result.planet_count, 13);
    assert_eq!(result.base_count, 2);
}

#[test]
fn test_pipeline_stats_biome_counts_sum_to_planet_count() {
    let model = model_from_fixture("multi_system_save.json");
    let query = StatsQuery::default();
    let result = execute_stats(&model, &query);
    let counted: usize = result.biome_counts.values().sum::<usize>() + result.unknown_biome_count;
    assert_eq!(counted, result.planet_count);
}

#[test]
fn test_pipeline_stats_formatted_output_contains_galaxy_stats() {
    let model = model_from_fixture("multi_system_save.json");
    let query = StatsQuery {
        biomes: true,
        discoveries: true,
    };
    let result = execute_stats(&model, &query);
    let output = format_stats(&result, &Theme::none());
    assert!(output.contains("GALAXY STATISTICS"));
}

// ---- Cross-crate integration: parse -> graph -> spatial ----

#[test]
fn test_pipeline_spatial_index_matches_system_count() {
    let model = model_from_fixture("multi_system_save.json");
    assert_eq!(model.spatial_size(), model.system_count());
}

#[test]
fn test_pipeline_graph_node_count_matches_system_count() {
    let model = model_from_fixture("multi_system_save.json");
    assert_eq!(model.graph.node_count(), model.system_count());
}

#[test]
fn test_pipeline_every_system_has_node_map_entry() {
    let model = model_from_fixture("multi_system_save.json");
    for sys_id in model.systems.keys() {
        assert!(
            model.node_map.contains_key(sys_id),
            "System {sys_id:?} missing from node_map"
        );
    }
}

#[test]
fn test_pipeline_every_system_has_address_to_id_entry() {
    let model = model_from_fixture("multi_system_save.json");
    for sys_id in model.systems.keys() {
        assert!(
            model.address_to_id.contains_key(&sys_id.0),
            "System {sys_id:?} missing from address_to_id"
        );
    }
}

#[test]
fn test_pipeline_discovered_galaxies_contains_euclid() {
    let model = model_from_fixture("multi_system_save.json");
    let galaxies = model.discovered_galaxies();
    assert!(
        galaxies.contains(&0),
        "Euclid (galaxy 0) should be in discovered galaxies"
    );
}
