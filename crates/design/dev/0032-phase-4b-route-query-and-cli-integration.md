# Phase 4B -- Route Query & CLI Integration

Milestones 4.5-4.7: `RouteQuery` type in nms-query, route display formatter, and `nms route` command in both CLI and REPL.

**Depends on:** Phase 4A (graph routing algorithms in nms-graph).

---

## Milestone 4.5: Route Query Type

### Crate: `nms-query`

Path: `crates/nms-query/`

### New File: `crates/nms-query/src/route.rs`

The query type that bridges user intent to the routing algorithms in nms-graph.

```rust
//! Route planning queries.
//!
//! Translates user-facing parameters (biome, base names, warp range)
//! into calls to `GalaxyModel`'s routing methods.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_graph::query::BiomeFilter;
use nms_graph::route::{Route, RouteError, RoutingAlgorithm};
use nms_graph::{GalaxyModel, GraphError, SystemId};

/// How to select route targets.
#[derive(Debug, Clone)]
pub enum TargetSelection {
    /// Visit all planets matching a biome filter.
    Biome(BiomeFilter),
    /// Visit specific systems/bases by name.
    Named(Vec<String>),
    /// Visit specific system IDs (from a previous find result).
    SystemIds(Vec<SystemId>),
}

/// Parameters for route planning.
#[derive(Debug, Clone)]
pub struct RouteQuery {
    /// How to select targets.
    pub targets: TargetSelection,

    /// Reference point for the route start.
    pub from: RouteFrom,

    /// Maximum warp range per hop (light-years). None = no constraint.
    pub warp_range: Option<f64>,

    /// Only include targets within this radius from start (light-years).
    pub within_ly: Option<f64>,

    /// Maximum number of targets to include.
    pub max_targets: Option<usize>,

    /// Routing algorithm.
    pub algorithm: RoutingAlgorithm,

    /// Whether to return to starting system after visiting all targets.
    pub return_to_start: bool,
}

/// Where the route starts from.
#[derive(Debug, Clone, Default)]
pub enum RouteFrom {
    /// Player's current position.
    #[default]
    CurrentPosition,
    /// A named base.
    Base(String),
    /// An explicit address.
    Address(GalacticAddress),
}

impl Default for RouteQuery {
    fn default() -> Self {
        Self {
            targets: TargetSelection::Biome(BiomeFilter::default()),
            from: RouteFrom::CurrentPosition,
            warp_range: None,
            within_ly: None,
            max_targets: None,
            algorithm: RoutingAlgorithm::default(),
            return_to_start: false,
        }
    }
}

/// A route result with metadata for display.
#[derive(Debug)]
pub struct RouteResult {
    /// The planned route.
    pub route: Route,
    /// Warp range used (if constrained).
    pub warp_range: Option<f64>,
    /// Total warp jumps needed (if warp range specified).
    pub warp_jumps: Option<usize>,
    /// Algorithm used.
    pub algorithm: RoutingAlgorithm,
    /// Number of targets visited (excluding waypoints).
    pub targets_visited: usize,
}

/// Execute a route query against the galaxy model.
pub fn execute_route(
    model: &GalaxyModel,
    query: &RouteQuery,
) -> Result<RouteResult, Box<dyn std::error::Error>> {
    // 1. Resolve start position
    let start_addr = resolve_start(model, &query.from)?;
    let start_id = find_nearest_system(model, &start_addr)?;

    // 2. Resolve targets to SystemIds
    let mut target_ids = resolve_targets(model, &query.targets, &start_addr, query.within_ly)?;

    // Remove start from targets if present
    target_ids.retain(|id| *id != start_id);

    // Apply max_targets limit
    if let Some(max) = query.max_targets {
        target_ids.truncate(max);
    }

    if target_ids.is_empty() {
        return Err("no targets found matching criteria".into());
    }

    let targets_visited = target_ids.len();

    // 3. Run routing algorithm
    let route = match query.algorithm {
        RoutingAlgorithm::NearestNeighbor => {
            model.tsp_nearest_neighbor(start_id, &target_ids, query.return_to_start)?
        }
        RoutingAlgorithm::TwoOpt => {
            model.tsp_two_opt(start_id, &target_ids, query.return_to_start)?
        }
    };

    // 4. Apply hop constraints if warp range specified
    let route = if let Some(warp_range) = query.warp_range {
        model.constrain_hops(&route, warp_range)
    } else {
        route
    };

    // 5. Compute warp jump count
    let warp_jumps = query.warp_range.map(|wr| GalaxyModel::warp_jump_count(&route, wr));

    Ok(RouteResult {
        route,
        warp_range: query.warp_range,
        warp_jumps,
        algorithm: query.algorithm,
        targets_visited,
    })
}

/// Resolve the starting position to a GalacticAddress.
fn resolve_start(
    model: &GalaxyModel,
    from: &RouteFrom,
) -> Result<GalacticAddress, GraphError> {
    match from {
        RouteFrom::CurrentPosition => model
            .player_position()
            .copied()
            .ok_or(GraphError::NoPlayerPosition),
        RouteFrom::Base(name) => model
            .base(name)
            .map(|b| b.address)
            .ok_or_else(|| GraphError::BaseNotFound(name.clone())),
        RouteFrom::Address(addr) => Ok(*addr),
    }
}

/// Find the nearest known system to an address.
fn find_nearest_system(
    model: &GalaxyModel,
    addr: &GalacticAddress,
) -> Result<SystemId, Box<dyn std::error::Error>> {
    let nearest = model.nearest_systems(addr, 1);
    nearest
        .first()
        .map(|(id, _)| *id)
        .ok_or_else(|| "no systems in model".into())
}

/// Resolve target selection to a list of SystemIds.
fn resolve_targets(
    model: &GalaxyModel,
    targets: &TargetSelection,
    from: &GalacticAddress,
    within_ly: Option<f64>,
) -> Result<Vec<SystemId>, Box<dyn std::error::Error>> {
    match targets {
        TargetSelection::Biome(filter) => {
            let planets = if let Some(radius) = within_ly {
                model.planets_within_radius(from, radius, filter)
            } else {
                // Get all matching planets, sorted by distance
                model.nearest_planets(from, usize::MAX, filter)
            };
            // Deduplicate by system (one entry per system)
            let mut seen = std::collections::HashSet::new();
            let ids: Vec<SystemId> = planets
                .into_iter()
                .filter_map(|(key, _, _)| {
                    if seen.insert(key.0) {
                        Some(key.0)
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ids)
        }
        TargetSelection::Named(names) => {
            let mut ids = Vec::with_capacity(names.len());
            for name in names {
                // Try as base name first, then system name
                if let Some(base) = model.base(name) {
                    let sys_id = SystemId::from_address(&base.address);
                    ids.push(sys_id);
                } else if let Some((sys_id, _)) = model.system_by_name(name) {
                    ids.push(*sys_id);
                } else {
                    return Err(format!("target not found: \"{name}\"").into());
                }
            }
            Ok(ids)
        }
        TargetSelection::SystemIds(ids) => Ok(ids.clone()),
    }
}
```

### Tests (Milestone 4.5)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        // Same test model pattern used in find.rs
        // Systems with Lush and Scorched planets at known positions
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
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn test_route_by_named_targets() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Named(vec!["Alpha Base".into()]),
            ..Default::default()
        };
        let result = execute_route(&model, &query).unwrap();
        assert!(result.route.hops.len() >= 2);
    }

    #[test]
    fn test_route_by_biome() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter {
                biome: Some(Biome::Lush),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = execute_route(&model, &query);
        // May succeed or fail depending on whether Lush systems exist
        // In our test model they should exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_route_no_targets_errors() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter {
                biome: Some(Biome::Lava), // no Lava planets in test model
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(execute_route(&model, &query).is_err());
    }

    #[test]
    fn test_route_named_target_not_found_errors() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Named(vec!["No Such Base".into()]),
            ..Default::default()
        };
        assert!(execute_route(&model, &query).is_err());
    }

    #[test]
    fn test_route_with_warp_range() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter {
                biome: Some(Biome::Lush),
                ..Default::default()
            }),
            warp_range: Some(2500.0),
            ..Default::default()
        };
        let result = execute_route(&model, &query).unwrap();
        assert!(result.warp_jumps.is_some());
    }

    #[test]
    fn test_route_from_base() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter::default()),
            from: RouteFrom::Base("Alpha Base".into()),
            ..Default::default()
        };
        assert!(execute_route(&model, &query).is_ok());
    }

    #[test]
    fn test_route_from_nonexistent_base_errors() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter::default()),
            from: RouteFrom::Base("No Such Base".into()),
            ..Default::default()
        };
        assert!(execute_route(&model, &query).is_err());
    }

    #[test]
    fn test_route_max_targets() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter::default()),
            max_targets: Some(1),
            ..Default::default()
        };
        let result = execute_route(&model, &query).unwrap();
        assert_eq!(result.targets_visited, 1);
    }

    #[test]
    fn test_route_nearest_neighbor_algorithm() {
        let model = test_model();
        let query = RouteQuery {
            targets: TargetSelection::Biome(BiomeFilter::default()),
            algorithm: RoutingAlgorithm::NearestNeighbor,
            ..Default::default()
        };
        let result = execute_route(&model, &query).unwrap();
        assert!(matches!(result.algorithm, RoutingAlgorithm::NearestNeighbor));
    }
}
```

---

## Milestone 4.6: Route Display

### Modified File: `crates/nms-query/src/display.rs`

Add a route itinerary formatter.

```rust
use crate::route::RouteResult;
use nms_graph::GalaxyModel;
use nms_graph::route::RouteHop;

/// Format a route result as a step-by-step itinerary table.
///
/// ```text
///   Hop  System              Distance    Cumulative   Portal Glyphs
///    1   Gugestor Colony        0 ly         0 ly     🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
///    2   Esurad               18K ly       18K ly     🌅🕊️🐜🦕🌜🎈⛵🐜🦋🌀🕋🐋
///   *3   (waypoint)            4K ly       22K ly     🌅😑🐜🕊️🐜🌳🌜🕋🌅🔺🕋🦕
///    4   Ogsjov XV             2K ly       24K ly     🌅🦕🐜🕊️🐜🌅🦋🕋🌜🔺🕋🐜
///
///   Route: 4 stops, 24K ly total (10 warp jumps at 2500 ly range)
///   Algorithm: 2-opt
/// ```
pub fn format_route(result: &RouteResult, model: &GalaxyModel) -> String {
    if result.route.hops.is_empty() {
        return "  No route to display.\n".to_string();
    }

    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "  {:<5} {:<22} {:<12} {:<13} {}\n",
        "Hop", "System", "Distance", "Cumulative", "Portal Glyphs"
    ));

    for (i, hop) in result.route.hops.iter().enumerate() {
        let system_name = model
            .system(&hop.system_id)
            .and_then(|s| s.name.as_deref())
            .unwrap_or("(unnamed)");

        let display_name = if hop.is_waypoint {
            format!("↳ {system_name}")
        } else {
            system_name.to_string()
        };

        let hop_num = if hop.is_waypoint {
            format!("  *")
        } else {
            format!("{:>3}", i + 1)
        };

        let portal_hex = model
            .system(&hop.system_id)
            .map(|s| format!("{:012X}", s.address.packed()))
            .unwrap_or_default();
        let emoji = hex_to_emoji(&portal_hex);

        out.push_str(&format!(
            "  {hop_num}  {:<22} {:>10}  {:>11}   {}\n",
            truncate(&display_name, 22),
            format_distance(hop.leg_distance_ly),
            format_distance(hop.cumulative_ly),
            emoji,
        ));
    }

    // Summary line
    out.push('\n');
    let algo_name = match result.algorithm {
        RoutingAlgorithm::NearestNeighbor => "nearest-neighbor",
        RoutingAlgorithm::TwoOpt => "2-opt",
    };

    let total = format_distance(result.route.total_distance_ly);
    let stops = result.targets_visited;

    if let (Some(wr), Some(jumps)) = (result.warp_range, result.warp_jumps) {
        out.push_str(&format!(
            "  Route: {stops} targets, {total} total ({jumps} warp jumps at {} range)\n",
            format_distance(wr),
        ));
    } else {
        out.push_str(&format!("  Route: {stops} targets, {total} total\n"));
    }
    out.push_str(&format!("  Algorithm: {algo_name}\n"));

    out
}

/// Truncate a string to `max` characters, appending "…" if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{truncated}…")
    }
}
```

### Tests (Milestone 4.6)

```rust
#[cfg(test)]
mod route_display_tests {
    use super::*;
    use nms_graph::route::{Route, RouteHop, RoutingAlgorithm};
    use nms_graph::SystemId;

    #[test]
    fn test_format_route_empty() {
        let result = RouteResult {
            route: Route { hops: vec![], total_distance_ly: 0.0 },
            warp_range: None,
            warp_jumps: None,
            algorithm: RoutingAlgorithm::TwoOpt,
            targets_visited: 0,
        };
        let model = /* minimal model */;
        let output = format_route(&result, &model);
        assert!(output.contains("No route"));
    }

    #[test]
    fn test_format_route_contains_header() {
        // Build a minimal RouteResult with test model
        let output = /* ... */;
        assert!(output.contains("Hop"));
        assert!(output.contains("System"));
        assert!(output.contains("Portal Glyphs"));
    }

    #[test]
    fn test_format_route_shows_algorithm() {
        let output = /* ... */;
        assert!(output.contains("2-opt") || output.contains("nearest-neighbor"));
    }

    #[test]
    fn test_format_route_shows_warp_jumps() {
        // RouteResult with warp_range and warp_jumps set
        let output = /* ... */;
        assert!(output.contains("warp jumps"));
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("Hello", 10), "Hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("Very Long System Name Here", 10);
        assert!(result.len() <= 12); // 10 chars + potential multi-byte "…"
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_format_route_waypoint_marker() {
        // A route with is_waypoint=true should show "↳" prefix
        let output = /* route with waypoints */;
        assert!(output.contains("↳") || output.contains("*"));
    }
}
```

---

## Milestone 4.7: `nms route` Command

### Modified File: `crates/nms-query/src/lib.rs`

```rust
pub mod display;
pub mod find;
pub mod route;
pub mod show;
pub mod stats;

pub use display::{
    format_distance, format_find_results, format_route, format_show_result, format_stats,
    hex_to_emoji,
};
pub use find::{FindQuery, FindResult, ReferencePoint};
pub use route::{RouteFrom, RouteQuery, RouteResult, TargetSelection};
pub use show::{ShowQuery, ShowResult};
pub use stats::{StatsQuery, StatsResult};
```

### New File: `crates/nms-cli/src/route.rs`

CLI handler for the `route` subcommand.

```rust
//! `nms route` command handler.

use std::path::PathBuf;

use nms_graph::route::RoutingAlgorithm;
use nms_graph::query::BiomeFilter;
use nms_query::{RouteFrom, RouteQuery, TargetSelection, execute_route, format_route};

pub struct RouteArgs {
    pub save: Option<PathBuf>,
    pub biome: Option<String>,
    pub targets: Vec<String>,
    pub from: Option<String>,
    pub warp_range: Option<f64>,
    pub within: Option<f64>,
    pub max_targets: Option<usize>,
    pub algo: Option<String>,
    pub round_trip: bool,
}

pub fn run(args: RouteArgs) -> Result<(), Box<dyn std::error::Error>> {
    let model = crate::load_model(args.save)?;

    let targets = if !args.targets.is_empty() {
        TargetSelection::Named(args.targets)
    } else if let Some(ref biome_str) = args.biome {
        let biome: nms_core::biome::Biome = biome_str.parse()
            .map_err(|_| format!("unknown biome: {biome_str}"))?;
        TargetSelection::Biome(BiomeFilter {
            biome: Some(biome),
            ..Default::default()
        })
    } else {
        return Err("specify --biome or --targets".into());
    };

    let from = match args.from {
        Some(name) => RouteFrom::Base(name),
        None => RouteFrom::CurrentPosition,
    };

    let algorithm = match args.algo.as_deref() {
        Some("nn") | Some("nearest-neighbor") => RoutingAlgorithm::NearestNeighbor,
        Some("2opt") | Some("two-opt") | None => RoutingAlgorithm::TwoOpt,
        Some(other) => return Err(format!("unknown algorithm: {other}").into()),
    };

    let query = RouteQuery {
        targets,
        from,
        warp_range: args.warp_range,
        within_ly: args.within,
        max_targets: args.max_targets,
        algorithm,
        return_to_start: args.round_trip,
    };

    let result = execute_route(&model, &query)?;
    print!("{}", format_route(&result, &model));

    Ok(())
}
```

### Modified File: `crates/nms-cli/src/main.rs`

Add the `Route` subcommand to the CLI.

```rust
mod route;

#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Plan a route through target systems.
    Route {
        /// Path to save file (auto-detects if omitted).
        #[arg(long)]
        save: Option<PathBuf>,

        /// Filter targets by biome (e.g., Lush, Scorched).
        #[arg(long)]
        biome: Option<String>,

        /// Specific target systems/bases by name.
        #[arg(long = "target", num_args = 1)]
        targets: Vec<String>,

        /// Start from this base (default: current position).
        #[arg(long)]
        from: Option<String>,

        /// Maximum warp range per hop (light-years).
        #[arg(long)]
        warp_range: Option<f64>,

        /// Only include targets within this radius (light-years).
        #[arg(long)]
        within: Option<f64>,

        /// Maximum number of targets.
        #[arg(long)]
        max_targets: Option<usize>,

        /// Routing algorithm: "2opt" (default), "nn" / "nearest-neighbor".
        #[arg(long)]
        algo: Option<String>,

        /// Return to starting system after visiting all targets.
        #[arg(long)]
        round_trip: bool,
    },
}

// In main():
Commands::Route {
    save, biome, targets, from, warp_range,
    within, max_targets, algo, round_trip,
} => route::run(route::RouteArgs {
    save, biome, targets, from, warp_range,
    within, max_targets, algo, round_trip,
}),
```

### Modified File: `crates/nms-copilot/src/commands.rs`

Add the `Route` action to the REPL.

```rust
#[derive(Subcommand, Debug)]
pub enum Action {
    // ... existing actions ...

    /// Plan a route through target systems.
    Route {
        /// Filter targets by biome.
        #[arg(long)]
        biome: Option<String>,

        /// Specific target systems/bases by name.
        #[arg(long = "target", num_args = 1)]
        targets: Vec<String>,

        /// Start from this base (default: session position).
        #[arg(long)]
        from: Option<String>,

        /// Maximum warp range per hop (light-years).
        #[arg(long)]
        warp_range: Option<f64>,

        /// Only include targets within this radius (light-years).
        #[arg(long)]
        within: Option<f64>,

        /// Maximum number of targets.
        #[arg(long)]
        max_targets: Option<usize>,

        /// Routing algorithm: "2opt" (default), "nn" / "nearest-neighbor".
        #[arg(long)]
        algo: Option<String>,

        /// Return to starting system after visiting all targets.
        #[arg(long)]
        round_trip: bool,
    },
}
```

### Modified File: `crates/nms-copilot/src/dispatch.rs`

Handle the `Route` action. Uses session position as default start, session warp range as default warp range.

```rust
Action::Route {
    biome, targets, from, warp_range,
    within, max_targets, algo, round_trip,
} => {
    let target_selection = if !targets.is_empty() {
        TargetSelection::Named(targets.clone())
    } else if let Some(ref biome_str) = biome {
        let biome: nms_core::biome::Biome = biome_str.parse()
            .map_err(|e| format!("unknown biome: {biome_str}"))?;
        let mut filter = BiomeFilter {
            biome: Some(biome),
            ..Default::default()
        };
        // Use session biome filter if no explicit biome given
        TargetSelection::Biome(filter)
    } else if let Some(session_biome) = session.biome_filter {
        TargetSelection::Biome(BiomeFilter {
            biome: Some(session_biome),
            ..Default::default()
        })
    } else {
        return Err("specify --biome or --target".into());
    };

    let route_from = match from {
        Some(name) => RouteFrom::Base(name.clone()),
        None => match &session.position {
            Some(pos) => RouteFrom::Address(*pos.address()),
            None => RouteFrom::CurrentPosition,
        },
    };

    // Session warp range as fallback
    let effective_warp_range = warp_range.or(session.warp_range);

    let algorithm = match algo.as_deref() {
        Some("nn") | Some("nearest-neighbor") => RoutingAlgorithm::NearestNeighbor,
        Some("2opt") | Some("two-opt") | None => RoutingAlgorithm::TwoOpt,
        Some(other) => return Err(format!("unknown algorithm: {other}")),
    };

    let query = RouteQuery {
        targets: target_selection,
        from: route_from,
        warp_range: effective_warp_range,
        within_ly: *within,
        max_targets: *max_targets,
        algorithm,
        return_to_start: *round_trip,
    };

    let result = nms_query::route::execute_route(model, &query)
        .map_err(|e| e.to_string())?;
    Ok(nms_query::format_route(&result, model))
}
```

### Modified File: `crates/nms-copilot/src/completer.rs`

Add `route` to commands and its flags to completion.

```rust
const COMMANDS: &[&str] = &[
    "convert", "exit", "find", "help", "info", "quit",
    "reset", "route", "set", "show", "stats", "status",
];

const ROUTE_FLAGS: &[&str] = &[
    "--algo", "--biome", "--from", "--max-targets",
    "--round-trip", "--target", "--warp-range", "--within",
];

// In complete() method, add:
"route" => suggest_from(ROUTE_FLAGS, partial, span),
```

---

## Modified File: `crates/nms-query/Cargo.toml`

No new dependencies needed -- `nms-graph` is already a dependency and now exports the routing types.

---

## Usage Examples

### CLI

```bash
# Route through all Scorched planets
nms route --biome Scorched

# Route with warp range constraint
nms route --biome Lush --warp-range 2500

# Route between specific bases
nms route --target "Acadia" --target "Sealab" --target "Outpost"

# Route within radius, nearest-neighbor only
nms route --biome Swamp --within 100000 --algo nn

# Round trip from a specific base
nms route --biome Lava --from "Home Base" --round-trip --warp-range 2000
```

### REPL

```
[Euclid | Lush | 644 planets] 🚀 route --biome Scorched --warp-range 2500
  Hop  System              Distance    Cumulative   Portal Glyphs
    1  Gugestor Colony        0 ly         0 ly     🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑
    2  Esurad               18K ly       18K ly     🌅🕊️🐜🦕🌜🎈⛵🐜🦋🌀🕋🐋
   *   ↳ (waypoint)          2K ly       20K ly     🌅😑🐜🕊️🐜🌳🌜🕋🌅🔺🕋🦕
    3  Ogsjov XV              4K ly       24K ly     🌅🦕🐜🕊️🐜🌅🦋🕋🌜🔺🕋🐜

  Route: 3 targets, 24K ly total (10 warp jumps at 2K ly range)
  Algorithm: 2-opt
```

---

## Implementation Notes

1. **Session integration is key.** In the REPL, the route command should use `session.position` as the default start and `session.warp_range` as the default warp range. This means a user who has done `set position "Home Base"` and `set warp-range 2500` can just type `route --biome Lush` and get a personalized route.

2. **Target deduplication by system.** When routing by biome, multiple planets in the same system should not create duplicate stops. The `resolve_targets` function deduplicates by SystemId.

3. **`nearest_planets` with `usize::MAX` limit.** For the "all matching planets" case, we pass a very large limit. This is fine because the R-tree iterator is lazy -- it will stop when no more results exist.

4. **Route display formatting.** Waypoints (inserted by `constrain_hops`) get a `*` marker and `↳` prefix to distinguish them from actual targets. This helps the player understand which stops are destinations vs. intermediate fueling points.

5. **Algorithm names in CLI.** Accept both short (`nn`, `2opt`) and long (`nearest-neighbor`, `two-opt`) forms. Default is `2opt`.

6. **Error handling.** `execute_route` returns `Box<dyn Error>` because errors can come from multiple sources (GraphError, RouteError, string parsing). The CLI/REPL layers convert to strings for display.

7. **`nms-query` dependency on `nms-graph::route`.** The route module in nms-query depends on types from `nms-graph::route`. This follows the same pattern as `find.rs` depending on `nms_graph::query::BiomeFilter`.
