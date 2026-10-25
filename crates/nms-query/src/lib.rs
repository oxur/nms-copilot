//! Shared query engine for NMS Copilot.
//!
//! Evaluates typed queries against an in-memory [`nms_graph::GalaxyModel`] and
//! returns typed results for callers such as the CLI, REPL, and MCP server.
//! Query execution takes an immutable model reference; it performs no save-file
//! or cache I/O. Formatting helpers return owned strings for callers to print.
//!
//! # Modules
//!
//! - [`find`] filters planet and system records.
//! - [`show`] resolves a system or base detail view.
//! - [`stats`] aggregates counts across the model.
//! - [`route`] resolves targets and computes an itinerary.
//! - [`display`] formats query results.
//! - [`theme`] provides semantic styles and terminal detection.
//! - [`table`] builds the crate's themed tables.
//!
//! # Examples
//!
//! This explicit-address example needs no player save and observes the complete
//! find result fields, a show result, and all statistics across the model.
//!
//! ```
//! use nms_core::{Biome, GalacticAddress, Planet, System};
//! use nms_graph::{GalaxyModel, GraphError, SystemId};
//! use nms_query::find::execute_find;
//! use nms_query::show::execute_show;
//! use nms_query::stats::execute_stats;
//! use nms_query::{FindQuery, ReferencePoint, ShowQuery, ShowResult, StatsQuery, StatsResult};
//!
//! let atlas = GalacticAddress::new(0, 0, 0, 1, 2, 9);
//! let beta = GalacticAddress::new(3, 4, 0, 2, 0, 9);
//! let opposed = GalacticAddress::new(0, 0, 0, 1, 2, 0);
//! let mut model = GalaxyModel::new();
//! model.insert_system(System::new(atlas, Some("Atlas".into()), Some("Explorer".into()), None, vec![
//!     Planet::new(1, Some(Biome::Lush), None, false, Some("Garden".into()), Some(7)),
//! ]));
//! model.insert_system(System::new(beta, Some("Beta".into()), Some("Explorer".into()), None, vec![
//!     Planet::new(2, Some(Biome::Toxic), None, true, Some("Cinder".into()), Some(8)),
//! ]));
//! model.insert_system(System::new(opposed, Some("Opposed".into()), Some("Other".into()), None, vec![
//!     Planet::new(1, Some(Biome::Lush), None, false, Some("Other".into()), Some(9)),
//! ]));
//! let query = FindQuery { biome: Some(Biome::Lush), biome_subtype: None, infested: Some(false),
//!     within_ly: Some(0.0), nearest: Some(1), name_pattern: Some("gard".into()),
//!     discoverer: Some("expl".into()), named_only: true, from: ReferencePoint::Address(atlas) };
//! let found = execute_find(&model, &query)?;
//! assert_eq!(found.len(), 1);
//! let result = &found[0];
//! assert_eq!(format!("{}", SystemId::from_address(&result.system.address)), "9:000100000000");
//! assert_eq!(result.system.address, atlas);
//! assert_eq!(result.system.source, nms_core::DiscoverySource::Personal);
//! assert_eq!(result.system.name.as_deref(), Some("Atlas"));
//! assert_eq!(result.system.discoverer.as_deref(), Some("Explorer"));
//! assert_eq!(result.system.timestamp, None);
//! let containing_planets: Vec<_> = result.system.planets.iter().map(|p|
//!     (p.index, p.name.as_deref(), p.biome, p.biome_subtype, p.infested, p.seed_hash)).collect();
//! assert_eq!(containing_planets, vec![(1, Some("Garden"), Some(Biome::Lush), None, false, Some(7))]);
//! assert_eq!(result.planet.index, 1);
//! assert_eq!(result.planet.name.as_deref(), Some("Garden"));
//! assert_eq!(result.planet.biome, Some(Biome::Lush));
//! assert_eq!(result.planet.biome_subtype, None);
//! assert!(!result.planet.infested);
//! assert_eq!(result.planet.seed_hash, Some(7));
//! assert_eq!(result.distance_ly, 0.0);
//! assert_eq!(result.portal_hex, "200100000000");
//! assert_eq!(model.active_galaxy, 0);
//! assert!(matches!(execute_find(&model, &FindQuery::default()), Err(GraphError::NoPlayerPosition)));
//! model.set_active_galaxy(9);
//! let shown = match execute_show(&model, &ShowQuery::System("ATLAS".into()))? {
//!     ShowResult::System(system) => system,
//!     ShowResult::Base(_) => return Err("expected system result".into()),
//! };
//! assert_eq!(format!("{}", SystemId::from_address(&shown.system.address)), "9:000100000000");
//! assert_eq!(shown.system.address, atlas);
//! assert_eq!(shown.system.source, nms_core::DiscoverySource::Personal);
//! assert_eq!(shown.system.name.as_deref(), Some("Atlas"));
//! assert_eq!(shown.system.discoverer.as_deref(), Some("Explorer"));
//! assert_eq!(shown.system.timestamp, None);
//! let shown_planets: Vec<_> = shown.system.planets.iter().map(|p|
//!     (p.index, p.name.as_deref(), p.biome, p.biome_subtype, p.infested, p.seed_hash)).collect();
//! assert_eq!(shown_planets, vec![(1, Some("Garden"), Some(Biome::Lush), None, false, Some(7))]);
//! assert_eq!(shown.portal_hex, "200100000000");
//! assert_eq!(shown.galaxy_name, "Eissentam");
//! assert_eq!(shown.distance_from_player, None);
//! assert!(matches!(execute_show(&model, &ShowQuery::Base("Missing".into())), Err(GraphError::BaseNotFound(ref n)) if n == "Missing"));
//! let expected = StatsResult { system_count: 3, planet_count: 3, base_count: 0,
//!     biome_counts: [(Biome::Lush, 2), (Biome::Toxic, 1)].into(), unknown_biome_count: 0,
//!     named_planet_count: 3, named_system_count: 3, infested_count: 1 };
//! for query in [StatsQuery::default(), StatsQuery { biomes: true, discoveries: true }] {
//!     let got = execute_stats(&model, &query);
//!     assert_eq!(got.system_count, expected.system_count);
//!     assert_eq!(got.planet_count, expected.planet_count);
//!     assert_eq!(got.base_count, expected.base_count);
//!     assert_eq!(got.biome_counts, expected.biome_counts);
//!     assert_eq!(got.unknown_biome_count, expected.unknown_biome_count);
//!     assert_eq!(got.named_planet_count, expected.named_planet_count);
//!     assert_eq!(got.named_system_count, expected.named_system_count);
//!     assert_eq!(got.infested_count, expected.infested_count);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod display;
pub mod find;
pub mod route;
pub mod show;
pub mod stats;
pub mod table;
pub mod theme;

/// Text-formatting functions for query results.
pub use display::{
    format_distance, format_find_results, format_route, format_show_result, format_stats,
    hex_to_emoji,
};
/// Planet and system search query types.
pub use find::{FindQuery, FindResult, ReferencePoint};
/// Route selection, input, and result types.
pub use route::{RouteFrom, RouteQuery, RouteResult, TargetSelection};
/// Detail-query input and result types.
pub use show::{ShowQuery, ShowResult};
/// Statistics input and result types.
pub use stats::{StatsQuery, StatsResult};
/// Semantic output theme and stdout color-detection helper.
pub use theme::{Theme, should_use_colors};
