# Phase 6A -- MCP Server & Tool Implementations

Milestones 6.1-6.7: Server scaffold using fabryk-mcp, all MCP tool implementations, and discoverability.

**Depends on:** Phases 1-4 (save parsing, galaxy model, query engine, routing).

---

## Architecture Overview

The `nms-mcp` binary is a standalone MCP server that loads the galaxy model once at startup and serves tool calls over stdio (primary) or streaming HTTP (optional). It uses the fabryk-mcp framework for server infrastructure, tool registration, health checks, and discoverability.

```
                    fabryk-mcp
                   +-----------+
  Claude/AI  <-->  |  Server   |  <-- ToolRegistry (NmsTools)
  (stdio/HTTP)     +-----------+       |
                                       +-- nms-query (execute_find, execute_route, ...)
                                       +-- nms-graph (GalaxyModel)
                                       +-- nms-core  (types, address, glyph)
```

All tools are thin wrappers: parse JSON args, build a query struct, call an `execute_*` function from nms-query, and serialize the result to JSON. No duplicated logic.

---

## New Dependencies

### Workspace `Cargo.toml`

```toml
fabryk-mcp = "0.1.3"
rmcp = { version = "0.17", features = ["server", "transport-io"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
schemars = "1.2"
```

### `crates/nms-mcp/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-graph = { workspace = true }
nms-query = { workspace = true }
nms-cache = { workspace = true }

fabryk-mcp = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
schemars = { workspace = true }

[features]
default = []
http = ["fabryk-mcp/http"]

[dev-dependencies]
nms-save = { workspace = true }
```

---

## Milestone 6.1: Server Scaffold

### New File: `crates/nms-mcp/src/main.rs`

```rust
//! NMS Copilot MCP Server -- AI integration for No Man's Sky.

use std::path::PathBuf;
use std::sync::Arc;

use fabryk_mcp::{
    CompositeRegistry, DiscoverableRegistry, FabrykMcpServer,
    HealthTools, ToolMeta,
};

mod tools;

use tools::NmsTools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load galaxy model (same pattern as CLI/REPL)
    let save_path = parse_save_arg();
    let model = load_model(save_path)?;
    let model = Arc::new(model);

    // Build tool registry
    let nms_tools = NmsTools::new(Arc::clone(&model));
    let health = HealthTools::new(
        "nms-copilot",
        env!("CARGO_PKG_VERSION"),
        nms_tools.tool_count() + 1, // +1 for health itself
    );

    let composite = CompositeRegistry::new()
        .add(nms_tools)
        .add(health);

    // Wrap with discoverability
    let discoverable = build_discoverable(composite);

    // Start server
    FabrykMcpServer::new(discoverable)
        .with_name("nms-copilot")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description(
            "Galactic copilot for No Man's Sky. Search planets, plan routes, \
             convert portal glyphs, and explore your galaxy with an AI."
        )
        .with_discoverable_instructions("nms")
        .serve_stdio()
        .await?;

    Ok(())
}

fn build_discoverable(registry: CompositeRegistry) -> DiscoverableRegistry<CompositeRegistry> {
    DiscoverableRegistry::new(registry, "nms")
        .with_tool_metas(vec![
            ("search_planets", ToolMeta {
                summary: "Search planets by biome, distance, discoverer, or name.".into(),
                when_to_use: "Looking for planets with a specific biome or property".into(),
                returns: "Ranked list of matching planets with coordinates and portal glyphs".into(),
                next: Some("Call plan_route to navigate to results".into()),
                category: Some("search".into()),
            }),
            ("plan_route", ToolMeta {
                summary: "Plan an optimal route through target systems.".into(),
                when_to_use: "Need to visit multiple systems efficiently with warp range constraints".into(),
                returns: "Step-by-step itinerary with distances and portal glyphs".into(),
                next: Some("Use convert_coordinates to get portal addresses for waypoints".into()),
                category: Some("navigation".into()),
            }),
            ("where_am_i", ToolMeta {
                summary: "Get the player's current location.".into(),
                when_to_use: "Need to know the player's current system and coordinates".into(),
                returns: "System name, coordinates, portal glyphs, galaxy".into(),
                next: Some("Call whats_nearby for situational awareness".into()),
                category: Some("location".into()),
            }),
            ("whats_nearby", ToolMeta {
                summary: "Find systems and planets near the player's current position.".into(),
                when_to_use: "Need situational awareness or looking for nearby options".into(),
                returns: "Nearby systems with distances, biomes, and portal glyphs".into(),
                next: Some("Call search_planets for filtered results or plan_route to navigate".into()),
                category: Some("search".into()),
            }),
            ("show_system", ToolMeta {
                summary: "Get detailed information about a star system.".into(),
                when_to_use: "Need details about a specific system (planets, discoverer, coordinates)".into(),
                returns: "System details with all discovered planets and their biomes".into(),
                next: None,
                category: Some("detail".into()),
            }),
            ("show_base", ToolMeta {
                summary: "Get detailed information about a player base.".into(),
                when_to_use: "Need details about a specific base (location, type, system)".into(),
                returns: "Base details with portal glyphs and system context".into(),
                next: None,
                category: Some("detail".into()),
            }),
            ("convert_coordinates", ToolMeta {
                summary: "Convert between portal glyphs, signal booster, and galactic addresses.".into(),
                when_to_use: "Need to convert coordinates between formats for in-game use".into(),
                returns: "All coordinate formats for the given address".into(),
                next: None,
                category: Some("utility".into()),
            }),
            ("galaxy_stats", ToolMeta {
                summary: "Get aggregate statistics about the explored galaxy.".into(),
                when_to_use: "Want an overview of discoveries, biome distribution, or progress".into(),
                returns: "System/planet/base counts with biome breakdown".into(),
                next: Some("Call search_planets to explore specific biomes".into()),
                category: Some("overview".into()),
            }),
        ])
        .with_query_strategy(vec![
            "1. Call nms_directory to see all available tools",
            "2. Call where_am_i to establish the player's current location",
            "3. Use search_planets or whats_nearby to find destinations",
            "4. Use plan_route to plan navigation between targets",
            "5. Use convert_coordinates to provide portal glyphs for in-game use",
        ])
}

fn parse_save_arg() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--save")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

fn load_model(
    save_path: Option<PathBuf>,
) -> Result<nms_graph::GalaxyModel, Box<dyn std::error::Error>> {
    let path = match save_path {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    Ok(nms_graph::GalaxyModel::from_save(&save))
}
```

### Tests (Milestone 6.1)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_discoverable_has_directory_tool() {
        let model = test_model();
        let nms_tools = NmsTools::new(Arc::new(model));
        let composite = CompositeRegistry::new().add(nms_tools);
        let discoverable = build_discoverable(composite);

        let tools = discoverable.tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"nms_directory"));
    }
}
```

---

## Milestones 6.2-6.7: Tool Implementations

### New File: `crates/nms-mcp/src/tools.rs`

All NMS tools in a single `ToolRegistry` implementation. Each tool handler is a method that parses JSON args, calls into nms-query, and returns structured JSON.

```rust
//! NMS Copilot MCP tools.
//!
//! Each tool wraps a function from `nms-query`, translating between
//! JSON tool arguments and typed query structs.

use std::sync::Arc;

use fabryk_mcp::{ToolRegistry, ToolResult, model::*};
use serde_json::{Map, Value, json};

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_core::glyph::Glyph;
use nms_graph::GalaxyModel;
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_query::display::{format_distance, hex_to_emoji};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::show::{ShowQuery, execute_show};
use nms_query::stats::{StatsQuery, execute_stats};

/// All NMS tools backed by a shared GalaxyModel.
pub struct NmsTools {
    model: Arc<GalaxyModel>,
}

impl NmsTools {
    pub fn new(model: Arc<GalaxyModel>) -> Self {
        Self { model }
    }

    pub fn tool_count(&self) -> usize {
        8 // search_planets, plan_route, where_am_i, whats_nearby,
          // show_system, show_base, convert_coordinates, galaxy_stats
    }
}

impl ToolRegistry for NmsTools {
    fn tools(&self) -> Vec<Tool> {
        vec![
            search_planets_tool(),
            plan_route_tool(),
            where_am_i_tool(),
            whats_nearby_tool(),
            show_system_tool(),
            show_base_tool(),
            convert_coordinates_tool(),
            galaxy_stats_tool(),
        ]
    }

    fn call(&self, name: &str, args: Value) -> Option<ToolResult> {
        let model = Arc::clone(&self.model);
        match name {
            "search_planets" => Some(Box::pin(handle_search_planets(model, args))),
            "plan_route" => Some(Box::pin(handle_plan_route(model, args))),
            "where_am_i" => Some(Box::pin(handle_where_am_i(model, args))),
            "whats_nearby" => Some(Box::pin(handle_whats_nearby(model, args))),
            "show_system" => Some(Box::pin(handle_show_system(model, args))),
            "show_base" => Some(Box::pin(handle_show_base(model, args))),
            "convert_coordinates" => Some(Box::pin(handle_convert(model, args))),
            "galaxy_stats" => Some(Box::pin(handle_galaxy_stats(model, args))),
            _ => None,
        }
    }
}
```

### Tool Definitions

Each tool definition function returns a `Tool` with name, description, and JSON schema for input validation.

```rust
fn search_planets_tool() -> Tool {
    Tool {
        name: "search_planets".into(),
        description: Some("Search planets by biome, distance, discoverer, or name.".into()),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "biome": {
                    "type": "string",
                    "description": "Biome type (Lush, Toxic, Scorched, Radioactive, Frozen, Barren, Dead, Weird, Swamp, Lava, etc.)"
                },
                "within_ly": {
                    "type": "number",
                    "description": "Maximum distance in light-years from reference point"
                },
                "nearest": {
                    "type": "integer",
                    "description": "Return only the N nearest results"
                },
                "discoverer": {
                    "type": "string",
                    "description": "Filter by discoverer username (substring match)"
                },
                "named_only": {
                    "type": "boolean",
                    "description": "Only include named planets/systems"
                },
                "from_base": {
                    "type": "string",
                    "description": "Measure distance from this base name (default: player position)"
                },
                "infested": {
                    "type": "boolean",
                    "description": "Only include infested planets"
                }
            }
        })).unwrap(),
    }
}

fn plan_route_tool() -> Tool {
    Tool {
        name: "plan_route".into(),
        description: Some("Plan an optimal route through target systems.".into()),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "biome": {
                    "type": "string",
                    "description": "Visit all systems with this biome type"
                },
                "targets": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Specific system or base names to visit"
                },
                "from_base": {
                    "type": "string",
                    "description": "Start from this base (default: player position)"
                },
                "warp_range": {
                    "type": "number",
                    "description": "Maximum warp range per hop in light-years"
                },
                "within_ly": {
                    "type": "number",
                    "description": "Only include targets within this radius"
                },
                "max_targets": {
                    "type": "integer",
                    "description": "Maximum number of targets to include"
                },
                "algorithm": {
                    "type": "string",
                    "enum": ["2opt", "nearest-neighbor"],
                    "description": "Routing algorithm (default: 2opt)"
                },
                "round_trip": {
                    "type": "boolean",
                    "description": "Return to starting system after visiting all targets"
                }
            }
        })).unwrap(),
    }
}

fn where_am_i_tool() -> Tool {
    Tool {
        name: "where_am_i".into(),
        description: Some("Get the player's current location.".into()),
        input_schema: fabryk_mcp::empty_input_schema(),
    }
}

fn whats_nearby_tool() -> Tool {
    Tool {
        name: "whats_nearby".into(),
        description: Some("Find systems and planets near the player's current position.".into()),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "description": "Number of nearby results to return (default: 10)"
                },
                "biome": {
                    "type": "string",
                    "description": "Filter by biome type"
                }
            }
        })).unwrap(),
    }
}

fn show_system_tool() -> Tool {
    Tool {
        name: "show_system".into(),
        description: Some("Get detailed information about a star system.".into()),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "System name or hex address"
                }
            },
            "required": ["name"]
        })).unwrap(),
    }
}

fn show_base_tool() -> Tool {
    Tool {
        name: "show_base".into(),
        description: Some("Get detailed information about a player base.".into()),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Base name (case-insensitive)"
                }
            },
            "required": ["name"]
        })).unwrap(),
    }
}

fn convert_coordinates_tool() -> Tool {
    Tool {
        name: "convert_coordinates".into(),
        description: Some(
            "Convert between portal glyphs, signal booster coordinates, and galactic addresses."
                .into(),
        ),
        input_schema: serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "glyphs": {
                    "type": "string",
                    "description": "Portal glyphs as 12 hex digits (e.g., 01717D8A4EA2)"
                },
                "coords": {
                    "type": "string",
                    "description": "Signal booster coordinates (XXXX:YYYY:ZZZZ:SSSS)"
                },
                "galactic_address": {
                    "type": "string",
                    "description": "Galactic address as hex (0x...)"
                }
            }
        })).unwrap(),
    }
}

fn galaxy_stats_tool() -> Tool {
    Tool {
        name: "galaxy_stats".into(),
        description: Some("Get aggregate statistics about the explored galaxy.".into()),
        input_schema: fabryk_mcp::empty_input_schema(),
    }
}
```

### Tool Handlers

Each handler is an async function that returns `Result<CallToolResult, ErrorData>`.

```rust
/// Helper: create a successful text response.
fn text_result(json: Value) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap(),
    )]))
}

/// Helper: create an error response.
fn tool_error(msg: &str) -> ErrorData {
    ErrorData::invalid_params(msg, None)
}

// ── search_planets ──────────────────────────────────────────────

async fn handle_search_planets(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let biome = args.get("biome")
        .and_then(|v| v.as_str())
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| tool_error(&format!("Invalid biome: {e}")))?;

    let reference = match args.get("from_base").and_then(|v| v.as_str()) {
        Some(name) => ReferencePoint::Base(name.into()),
        None => ReferencePoint::CurrentPosition,
    };

    let query = FindQuery {
        biome,
        infested: args.get("infested").and_then(|v| v.as_bool()).map(|b| if b { true } else { return None; /* not filtering */ }).or(None),
        within_ly: args.get("within_ly").and_then(|v| v.as_f64()),
        nearest: args.get("nearest").and_then(|v| v.as_u64()).map(|n| n as usize),
        discoverer: args.get("discoverer").and_then(|v| v.as_str()).map(String::from),
        named_only: args.get("named_only").and_then(|v| v.as_bool()).unwrap_or(false),
        name_pattern: None,
        from: reference,
    };

    let results = execute_find(&model, &query)
        .map_err(|e| tool_error(&e.to_string()))?;

    let planets: Vec<Value> = results.iter().map(|r| {
        json!({
            "planet": r.planet.name.as_deref().unwrap_or("(unnamed)"),
            "biome": r.planet.biome.map(|b| b.to_string()).unwrap_or("?".into()),
            "infested": r.planet.infested,
            "system": r.system.name.as_deref().unwrap_or("(unnamed)"),
            "distance": format_distance(r.distance_ly),
            "distance_ly": r.distance_ly,
            "portal_glyphs_hex": &r.portal_hex,
            "portal_glyphs_emoji": hex_to_emoji(&r.portal_hex),
            "discoverer": r.system.discoverer.as_deref().unwrap_or("unknown"),
        })
    }).collect();

    text_result(json!({
        "count": planets.len(),
        "results": planets,
    }))
}

// ── plan_route ──────────────────────────────────────────────────

async fn handle_plan_route(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let targets_arg = args.get("targets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>());

    let biome_arg = args.get("biome")
        .and_then(|v| v.as_str())
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| tool_error(&format!("Invalid biome: {e}")))?;

    let targets = if let Some(names) = targets_arg {
        if names.is_empty() {
            return Err(tool_error("targets array is empty"));
        }
        TargetSelection::Named(names)
    } else if let Some(biome) = biome_arg {
        TargetSelection::Biome(BiomeFilter { biome: Some(biome), ..Default::default() })
    } else {
        return Err(tool_error("Specify either 'biome' or 'targets'"));
    };

    let from = match args.get("from_base").and_then(|v| v.as_str()) {
        Some(name) => RouteFrom::Base(name.into()),
        None => RouteFrom::CurrentPosition,
    };

    let algorithm = match args.get("algorithm").and_then(|v| v.as_str()) {
        Some("nearest-neighbor") | Some("nn") => RoutingAlgorithm::NearestNeighbor,
        _ => RoutingAlgorithm::TwoOpt,
    };

    let query = RouteQuery {
        targets,
        from,
        warp_range: args.get("warp_range").and_then(|v| v.as_f64()),
        within_ly: args.get("within_ly").and_then(|v| v.as_f64()),
        max_targets: args.get("max_targets").and_then(|v| v.as_u64()).map(|n| n as usize),
        algorithm,
        return_to_start: args.get("round_trip").and_then(|v| v.as_bool()).unwrap_or(false),
    };

    let result = execute_route(&model, &query)
        .map_err(|e| tool_error(&e.to_string()))?;

    let hops: Vec<Value> = result.route.hops.iter().enumerate().map(|(i, hop)| {
        let sys_name = model.system(&hop.system_id)
            .and_then(|s| s.name.as_deref())
            .unwrap_or("(unnamed)");
        let portal_hex = model.system(&hop.system_id)
            .map(|s| format!("{:012X}", s.address.packed()))
            .unwrap_or_default();

        json!({
            "hop": i + 1,
            "system": sys_name,
            "is_waypoint": hop.is_waypoint,
            "leg_distance": format_distance(hop.leg_distance_ly),
            "leg_distance_ly": hop.leg_distance_ly,
            "cumulative": format_distance(hop.cumulative_ly),
            "cumulative_ly": hop.cumulative_ly,
            "portal_glyphs_hex": portal_hex,
            "portal_glyphs_emoji": hex_to_emoji(&portal_hex),
        })
    }).collect();

    let algo_name = match result.algorithm {
        RoutingAlgorithm::NearestNeighbor => "nearest-neighbor",
        RoutingAlgorithm::TwoOpt => "2-opt",
    };

    text_result(json!({
        "hops": hops,
        "total_distance": format_distance(result.route.total_distance_ly),
        "total_distance_ly": result.route.total_distance_ly,
        "targets_visited": result.targets_visited,
        "algorithm": algo_name,
        "warp_range": result.warp_range,
        "warp_jumps": result.warp_jumps,
    }))
}

// ── where_am_i ──────────────────────────────────────────────────

async fn handle_where_am_i(
    model: Arc<GalaxyModel>,
    _args: Value,
) -> Result<CallToolResult, ErrorData> {
    let addr = model.player_position()
        .ok_or_else(|| tool_error("Player position not available"))?;

    let portal_hex = format!("{:012X}", addr.packed());
    let galaxy = Galaxy::by_index(addr.reality_index);

    // Find nearest system to player
    let nearest = model.nearest_systems(addr, 1);
    let (system_name, system_planets) = nearest.first()
        .and_then(|(id, _)| model.system(id))
        .map(|s| (
            s.name.as_deref().unwrap_or("(unnamed)"),
            s.planets.len(),
        ))
        .unwrap_or(("(unknown)", 0));

    text_result(json!({
        "system": system_name,
        "planets_in_system": system_planets,
        "galaxy": galaxy.name,
        "voxel_x": addr.voxel_x(),
        "voxel_y": addr.voxel_y(),
        "voxel_z": addr.voxel_z(),
        "solar_system_index": addr.solar_system_index(),
        "portal_glyphs_hex": portal_hex,
        "portal_glyphs_emoji": hex_to_emoji(&portal_hex),
        "signal_booster": addr.to_signal_booster(),
    }))
}

// ── whats_nearby ────────────────────────────────────────────────

async fn handle_whats_nearby(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let count = args.get("count")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(10);

    let biome = args.get("biome")
        .and_then(|v| v.as_str())
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| tool_error(&format!("Invalid biome: {e}")))?;

    let query = FindQuery {
        biome,
        nearest: Some(count),
        from: ReferencePoint::CurrentPosition,
        ..Default::default()
    };

    let results = execute_find(&model, &query)
        .map_err(|e| tool_error(&e.to_string()))?;

    let nearby: Vec<Value> = results.iter().map(|r| {
        json!({
            "planet": r.planet.name.as_deref().unwrap_or("(unnamed)"),
            "biome": r.planet.biome.map(|b| b.to_string()),
            "system": r.system.name.as_deref().unwrap_or("(unnamed)"),
            "distance": format_distance(r.distance_ly),
            "distance_ly": r.distance_ly,
            "portal_glyphs_emoji": hex_to_emoji(&r.portal_hex),
        })
    }).collect();

    text_result(json!({
        "count": nearby.len(),
        "from": "player position",
        "results": nearby,
    }))
}

// ── show_system ─────────────────────────────────────────────────

async fn handle_show_system(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tool_error("'name' is required"))?;

    let result = execute_show(&model, &ShowQuery::System(name.into()))
        .map_err(|e| tool_error(&e.to_string()))?;

    match result {
        nms_query::ShowResult::System(s) => {
            let planets: Vec<Value> = s.system.planets.iter().map(|p| {
                json!({
                    "index": p.index,
                    "name": p.name.as_deref().unwrap_or("(unnamed)"),
                    "biome": p.biome.map(|b| b.to_string()),
                    "infested": p.infested,
                })
            }).collect();

            text_result(json!({
                "name": s.system.name.as_deref().unwrap_or("(unnamed)"),
                "galaxy": s.galaxy_name,
                "discoverer": s.system.discoverer.as_deref().unwrap_or("unknown"),
                "portal_glyphs_hex": s.portal_hex,
                "portal_glyphs_emoji": hex_to_emoji(&s.portal_hex),
                "distance_from_player": s.distance_from_player.map(format_distance),
                "voxel_x": s.system.address.voxel_x(),
                "voxel_y": s.system.address.voxel_y(),
                "voxel_z": s.system.address.voxel_z(),
                "planets": planets,
            }))
        }
        _ => Err(tool_error("unexpected result type")),
    }
}

// ── show_base ───────────────────────────────────────────────────

async fn handle_show_base(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tool_error("'name' is required"))?;

    let result = execute_show(&model, &ShowQuery::Base(name.into()))
        .map_err(|e| tool_error(&e.to_string()))?;

    match result {
        nms_query::ShowResult::Base(b) => {
            text_result(json!({
                "name": b.base.name,
                "type": b.base.base_type,
                "galaxy": b.galaxy_name,
                "portal_glyphs_hex": b.portal_hex,
                "portal_glyphs_emoji": hex_to_emoji(&b.portal_hex),
                "distance_from_player": b.distance_from_player.map(format_distance),
                "system": b.system.as_ref().and_then(|s| s.name.as_deref()),
                "system_planet_count": b.system.as_ref().map(|s| s.planets.len()),
            }))
        }
        _ => Err(tool_error("unexpected result type")),
    }
}

// ── convert_coordinates ─────────────────────────────────────────

async fn handle_convert(
    _model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let addr = if let Some(glyphs) = args.get("glyphs").and_then(|v| v.as_str()) {
        let hex = glyphs.strip_prefix("0x").or_else(|| glyphs.strip_prefix("0X")).unwrap_or(glyphs);
        if hex.len() != 12 {
            return Err(tool_error(&format!("Portal glyphs must be 12 hex digits, got {}", hex.len())));
        }
        let packed = u64::from_str_radix(hex, 16)
            .map_err(|_| tool_error(&format!("Invalid hex: {hex}")))?;
        GalacticAddress::from_packed(packed, 0)
    } else if let Some(coords) = args.get("coords").and_then(|v| v.as_str()) {
        GalacticAddress::from_signal_booster(coords, 0, 0)
            .map_err(|e| tool_error(&format!("Invalid coordinates: {e}")))?
    } else if let Some(ga) = args.get("galactic_address").and_then(|v| v.as_str()) {
        let hex = ga.strip_prefix("0x").or_else(|| ga.strip_prefix("0X")).unwrap_or(ga);
        let packed = u64::from_str_radix(hex, 16)
            .map_err(|_| tool_error(&format!("Invalid galactic address: {ga}")))?;
        GalacticAddress::from_packed(packed, 0)
    } else {
        return Err(tool_error("Specify 'glyphs', 'coords', or 'galactic_address'"));
    };

    let portal_hex = format!("{:012X}", addr.packed());
    let galaxy = Galaxy::by_index(addr.reality_index);

    text_result(json!({
        "portal_glyphs_hex": portal_hex,
        "portal_glyphs_emoji": hex_to_emoji(&portal_hex),
        "signal_booster": addr.to_signal_booster(),
        "galactic_address": format!("0x{:012X}", addr.packed()),
        "voxel_x": addr.voxel_x(),
        "voxel_y": addr.voxel_y(),
        "voxel_z": addr.voxel_z(),
        "solar_system_index": addr.solar_system_index(),
        "planet_index": addr.planet_index(),
        "galaxy": galaxy.name,
    }))
}

// ── galaxy_stats ────────────────────────────────────────────────

async fn handle_galaxy_stats(
    model: Arc<GalaxyModel>,
    _args: Value,
) -> Result<CallToolResult, ErrorData> {
    let result = execute_stats(&model, &StatsQuery { biomes: true, discoveries: true });

    let biome_breakdown: Vec<Value> = {
        let mut biomes: Vec<_> = result.biome_counts.iter().collect();
        biomes.sort_by(|a, b| b.1.cmp(a.1));
        biomes.iter().map(|(biome, count)| {
            json!({ "biome": biome.to_string(), "count": count })
        }).collect()
    };

    text_result(json!({
        "systems": result.system_count,
        "planets": result.planet_count,
        "bases": result.base_count,
        "named_systems": result.named_system_count,
        "named_planets": result.named_planet_count,
        "infested_planets": result.infested_count,
        "biome_distribution": biome_breakdown,
        "unknown_biome_count": result.unknown_biome_count,
    }))
}
```

### Tests (Milestones 6.2-6.7)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fabryk_mcp::ToolRegistry;

    fn test_model() -> Arc<GalaxyModel> {
        // Same test JSON pattern as nms-query tests
        Arc::new(/* ... */)
    }

    #[test]
    fn test_nms_tools_has_all_tools() {
        let tools = NmsTools::new(test_model());
        let names: Vec<&str> = tools.tools().iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search_planets"));
        assert!(names.contains(&"plan_route"));
        assert!(names.contains(&"where_am_i"));
        assert!(names.contains(&"whats_nearby"));
        assert!(names.contains(&"show_system"));
        assert!(names.contains(&"show_base"));
        assert!(names.contains(&"convert_coordinates"));
        assert!(names.contains(&"galaxy_stats"));
        assert_eq!(names.len(), 8);
    }

    #[test]
    fn test_nms_tools_unknown_tool_returns_none() {
        let tools = NmsTools::new(test_model());
        assert!(tools.call("nonexistent", json!({})).is_none());
    }

    #[test]
    fn test_tool_schemas_valid() {
        let tools = NmsTools::new(test_model());
        fabryk_mcp::assert_tools_valid(&tools);
    }

    #[tokio::test]
    async fn test_where_am_i() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("where_am_i", json!({})).unwrap().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_galaxy_stats() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("galaxy_stats", json!({})).unwrap().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_planets_by_biome() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("search_planets", json!({"biome": "Lush"})).unwrap().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_planets_invalid_biome() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("search_planets", json!({"biome": "NotABiome"})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convert_glyphs() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("convert_coordinates", json!({
            "glyphs": "01717D8A4EA2"
        })).unwrap().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convert_no_input_errors() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("convert_coordinates", json!({})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_show_base_not_found() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("show_base", json!({"name": "No Such Base"})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plan_route_requires_targets_or_biome() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("plan_route", json!({})).unwrap().await;
        assert!(result.is_err());
    }
}
```

---

## Implementation Notes

1. **fabryk-mcp handles all protocol concerns.** JSON-RPC framing, tool listing, request routing, error serialization -- all handled by FabrykMcpServer. NMS code only implements `ToolRegistry`.

2. **`Arc<GalaxyModel>` for thread safety.** The model is immutable after loading (Phase 6A doesn't include live updates). `Arc` enables cheap clones into async tool handlers.

3. **Structured JSON, not formatted text.** Unlike the CLI/REPL display formatters, MCP tools return structured JSON. The AI formats the response for the user. Both `format_distance` (human-readable) and raw `_ly` values are included so the AI can choose.

4. **Portal glyphs in both hex and emoji.** Every location response includes both `portal_glyphs_hex` and `portal_glyphs_emoji`. The AI can present either format depending on context.

5. **`fabryk_mcp::assert_tools_valid`** validates all tool schemas at test time. Catches issues like missing descriptions, invalid schema structure, or duplicate names before deployment.

6. **`empty_input_schema()`** for tools with no parameters (where_am_i, galaxy_stats). Returns `{"type": "object"}` which satisfies MCP's minimum schema requirement.

7. **The `infested` filter** in search_planets needs care -- `FindQuery.infested` is `Option<bool>` where `Some(true)` means "only infested" and `None` means "no filter". The tool arg is a simple boolean.

8. **No `nms-watch` dependency in 6A.** Live updates are added in 6B. The model is loaded once at startup.
