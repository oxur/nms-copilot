//! NMS Copilot MCP tools.
//!
//! Each tool wraps a function from `nms-query`, translating between
//! JSON tool arguments and typed query structs.

use std::borrow::Cow;
use std::sync::Arc;

use fabryk_mcp::model::{CallToolResult, Content, ErrorData, Tool};
use fabryk_mcp::{ToolRegistry, ToolResult, empty_input_schema};
use serde_json::{Value, json};

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_graph::BiomeFilter;
use nms_graph::GalaxyModel;
use nms_graph::RoutingAlgorithm;
use nms_query::display::{format_distance, hex_to_emoji};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::show::{ShowQuery, ShowResult, execute_show};
use nms_query::stats::{StatsQuery, execute_stats};

/// All NMS tools backed by a shared GalaxyModel.
pub struct NmsTools {
    model: Arc<GalaxyModel>,
}

impl NmsTools {
    pub fn new(model: Arc<GalaxyModel>) -> Self {
        Self { model }
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

// ── Tool Definitions ────────────────────────────────────────────

fn schema(json: Value) -> Arc<serde_json::Map<String, Value>> {
    match json {
        Value::Object(map) => Arc::new(map),
        _ => unreachable!("schema must be a JSON object"),
    }
}

fn search_planets_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("search_planets"),
        title: None,
        description: Some(Cow::Borrowed(
            "Search planets by biome, distance, discoverer, or name.",
        )),
        input_schema: schema(json!({
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
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn plan_route_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("plan_route"),
        title: None,
        description: Some(Cow::Borrowed(
            "Plan an optimal route through target systems.",
        )),
        input_schema: schema(json!({
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
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn where_am_i_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("where_am_i"),
        title: None,
        description: Some(Cow::Borrowed("Get the player's current location.")),
        input_schema: Arc::new(empty_input_schema()),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn whats_nearby_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("whats_nearby"),
        title: None,
        description: Some(Cow::Borrowed(
            "Find systems and planets near the player's current position.",
        )),
        input_schema: schema(json!({
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
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn show_system_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("show_system"),
        title: None,
        description: Some(Cow::Borrowed(
            "Get detailed information about a star system.",
        )),
        input_schema: schema(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "System name or hex address"
                }
            },
            "required": ["name"]
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn show_base_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("show_base"),
        title: None,
        description: Some(Cow::Borrowed(
            "Get detailed information about a player base.",
        )),
        input_schema: schema(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Base name (case-insensitive)"
                }
            },
            "required": ["name"]
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn convert_coordinates_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("convert_coordinates"),
        title: None,
        description: Some(Cow::Borrowed(
            "Convert between portal glyphs, signal booster coordinates, and galactic addresses.",
        )),
        input_schema: schema(json!({
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
        })),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

fn galaxy_stats_tool() -> Tool {
    Tool {
        name: Cow::Borrowed("galaxy_stats"),
        title: None,
        description: Some(Cow::Borrowed(
            "Get aggregate statistics about the explored galaxy.",
        )),
        input_schema: Arc::new(empty_input_schema()),
        output_schema: None,
        annotations: None,
        execution: None,
        icons: None,
        meta: None,
    }
}

// ── Helpers ─────────────────────────────────────────────────────

fn text_result(json: Value) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string()),
    )]))
}

fn tool_error(msg: &str) -> ErrorData {
    ErrorData::invalid_params(msg.to_string(), None)
}

// ── Tool Handlers ───────────────────────────────────────────────

async fn handle_search_planets(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let biome = parse_biome_arg(&args, "biome")?;

    let reference = match args.get("from_base").and_then(|v| v.as_str()) {
        Some(name) => ReferencePoint::Base(name.into()),
        None => ReferencePoint::CurrentPosition,
    };

    let infested = args
        .get("infested")
        .and_then(|v| v.as_bool())
        .and_then(|b| b.then_some(true));

    let query = FindQuery {
        biome,
        infested,
        within_ly: args.get("within_ly").and_then(|v| v.as_f64()),
        nearest: args
            .get("nearest")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
        discoverer: args
            .get("discoverer")
            .and_then(|v| v.as_str())
            .map(String::from),
        named_only: args
            .get("named_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        name_pattern: None,
        from: reference,
    };

    let results = execute_find(&model, &query).map_err(|e| tool_error(&e.to_string()))?;

    let planets: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "planet": r.planet.name.as_deref().unwrap_or("(unnamed)"),
                "biome": r.planet.biome.map(|b| b.to_string()),
                "infested": r.planet.infested,
                "system": r.system.name.as_deref().unwrap_or("(unnamed)"),
                "distance": format_distance(r.distance_ly),
                "distance_ly": r.distance_ly,
                "portal_glyphs_hex": &r.portal_hex,
                "portal_glyphs_emoji": hex_to_emoji(&r.portal_hex),
                "discoverer": r.system.discoverer.as_deref().unwrap_or("unknown"),
            })
        })
        .collect();

    text_result(json!({
        "count": planets.len(),
        "results": planets,
    }))
}

async fn handle_plan_route(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let targets_arg = args.get("targets").and_then(|v| v.as_array()).map(|a| {
        a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>()
    });

    let biome_arg = parse_biome_arg(&args, "biome")?;

    let targets = if let Some(names) = targets_arg {
        if names.is_empty() {
            return Err(tool_error("targets array is empty"));
        }
        TargetSelection::Named(names)
    } else if let Some(biome) = biome_arg {
        TargetSelection::Biome(BiomeFilter {
            biome: Some(biome),
            ..Default::default()
        })
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
        max_targets: args
            .get("max_targets")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
        algorithm,
        return_to_start: args
            .get("round_trip")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    };

    let result = execute_route(&model, &query).map_err(|e| tool_error(&e.to_string()))?;

    let hops: Vec<Value> = result
        .route
        .hops
        .iter()
        .enumerate()
        .map(|(i, hop)| {
            let sys = model.system(&hop.system_id);
            let sys_name = sys.and_then(|s| s.name.as_deref()).unwrap_or("(unnamed)");
            let portal_hex = sys
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
        })
        .collect();

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

async fn handle_where_am_i(
    model: Arc<GalaxyModel>,
    _args: Value,
) -> Result<CallToolResult, ErrorData> {
    let addr = model
        .player_position()
        .ok_or_else(|| tool_error("Player position not available"))?;

    let portal_hex = format!("{:012X}", addr.packed());
    let galaxy = Galaxy::by_index(addr.reality_index);

    let nearest = model.nearest_systems(addr, 1);
    let (system_name, system_planets) = nearest
        .first()
        .and_then(|(id, _)| model.system(id))
        .map(|s| (s.name.as_deref().unwrap_or("(unnamed)"), s.planets.len()))
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

async fn handle_whats_nearby(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let count = args
        .get("count")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(10);

    let biome = parse_biome_arg(&args, "biome")?;

    let query = FindQuery {
        biome,
        nearest: Some(count),
        from: ReferencePoint::CurrentPosition,
        ..Default::default()
    };

    let results = execute_find(&model, &query).map_err(|e| tool_error(&e.to_string()))?;

    let nearby: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "planet": r.planet.name.as_deref().unwrap_or("(unnamed)"),
                "biome": r.planet.biome.map(|b| b.to_string()),
                "system": r.system.name.as_deref().unwrap_or("(unnamed)"),
                "distance": format_distance(r.distance_ly),
                "distance_ly": r.distance_ly,
                "portal_glyphs_emoji": hex_to_emoji(&r.portal_hex),
            })
        })
        .collect();

    text_result(json!({
        "count": nearby.len(),
        "from": "player position",
        "results": nearby,
    }))
}

async fn handle_show_system(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tool_error("'name' is required"))?;

    let result = execute_show(&model, &ShowQuery::System(name.into()))
        .map_err(|e| tool_error(&e.to_string()))?;

    match result {
        ShowResult::System(s) => {
            let planets: Vec<Value> = s
                .system
                .planets
                .iter()
                .map(|p| {
                    json!({
                        "index": p.index,
                        "name": p.name.as_deref().unwrap_or("(unnamed)"),
                        "biome": p.biome.map(|b| b.to_string()),
                        "infested": p.infested,
                    })
                })
                .collect();

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
        ShowResult::Base(_) => Err(tool_error("unexpected result type")),
    }
}

async fn handle_show_base(
    model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tool_error("'name' is required"))?;

    let result = execute_show(&model, &ShowQuery::Base(name.into()))
        .map_err(|e| tool_error(&e.to_string()))?;

    match result {
        ShowResult::Base(b) => text_result(json!({
            "name": b.base.name,
            "type": format!("{}", b.base.base_type),
            "galaxy": b.galaxy_name,
            "portal_glyphs_hex": b.portal_hex,
            "portal_glyphs_emoji": hex_to_emoji(&b.portal_hex),
            "distance_from_player": b.distance_from_player.map(format_distance),
            "system": b.system.as_ref().and_then(|s| s.name.as_deref()),
            "system_planet_count": b.system.as_ref().map(|s| s.planets.len()),
        })),
        ShowResult::System(_) => Err(tool_error("unexpected result type")),
    }
}

async fn handle_convert(
    _model: Arc<GalaxyModel>,
    args: Value,
) -> Result<CallToolResult, ErrorData> {
    let addr = if let Some(glyphs) = args.get("glyphs").and_then(|v| v.as_str()) {
        let hex = glyphs
            .strip_prefix("0x")
            .or_else(|| glyphs.strip_prefix("0X"))
            .unwrap_or(glyphs);
        if hex.len() != 12 {
            return Err(tool_error(&format!(
                "Portal glyphs must be 12 hex digits, got {}",
                hex.len()
            )));
        }
        let packed =
            u64::from_str_radix(hex, 16).map_err(|_| tool_error(&format!("Invalid hex: {hex}")))?;
        GalacticAddress::from_packed(packed, 0)
    } else if let Some(coords) = args.get("coords").and_then(|v| v.as_str()) {
        GalacticAddress::from_signal_booster(coords, 0, 0)
            .map_err(|e| tool_error(&format!("Invalid coordinates: {e}")))?
    } else if let Some(ga) = args.get("galactic_address").and_then(|v| v.as_str()) {
        let hex = ga
            .strip_prefix("0x")
            .or_else(|| ga.strip_prefix("0X"))
            .unwrap_or(ga);
        let packed = u64::from_str_radix(hex, 16)
            .map_err(|_| tool_error(&format!("Invalid galactic address: {ga}")))?;
        GalacticAddress::from_packed(packed, 0)
    } else {
        return Err(tool_error(
            "Specify 'glyphs', 'coords', or 'galactic_address'",
        ));
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

async fn handle_galaxy_stats(
    model: Arc<GalaxyModel>,
    _args: Value,
) -> Result<CallToolResult, ErrorData> {
    let result = execute_stats(
        &model,
        &StatsQuery {
            biomes: true,
            discoveries: true,
        },
    );

    let biome_breakdown: Vec<Value> = {
        let mut biomes: Vec<_> = result.biome_counts.iter().collect();
        biomes.sort_by(|a, b| b.1.cmp(a.1));
        biomes
            .iter()
            .map(|(biome, count)| json!({ "biome": biome.to_string(), "count": count }))
            .collect()
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

fn parse_biome_arg(args: &Value, key: &str) -> Result<Option<Biome>, ErrorData> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| {
            s.parse::<Biome>()
                .map_err(|e| tool_error(&format!("Invalid biome: {e}")))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_graph::GalaxyModel;

    fn test_model() -> Arc<GalaxyModel> {
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
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x003000001900", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x103000001900", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        Arc::new(
            nms_save::parse_save(json.as_bytes())
                .map(|save| GalaxyModel::from_save(&save))
                .expect("test model JSON is valid"),
        )
    }

    #[test]
    fn test_tools_has_all_eight() {
        let tools = NmsTools::new(test_model());
        let tool_list = tools.tools();
        let names: Vec<&str> = tool_list.iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(names.len(), 8);
        assert!(names.contains(&"search_planets"));
        assert!(names.contains(&"plan_route"));
        assert!(names.contains(&"where_am_i"));
        assert!(names.contains(&"whats_nearby"));
        assert!(names.contains(&"show_system"));
        assert!(names.contains(&"show_base"));
        assert!(names.contains(&"convert_coordinates"));
        assert!(names.contains(&"galaxy_stats"));
    }

    #[test]
    fn test_tools_unknown_returns_none() {
        let tools = NmsTools::new(test_model());
        assert!(tools.call("nonexistent", json!({})).is_none());
    }

    #[test]
    fn test_tools_tool_count() {
        let tools = NmsTools::new(test_model());
        assert_eq!(tools.tool_count(), 8);
    }

    #[test]
    fn test_tools_schemas_valid() {
        let tools = NmsTools::new(test_model());
        fabryk_mcp::assert_tools_valid(&tools);
    }

    #[tokio::test]
    async fn test_where_am_i_returns_position() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("where_am_i", json!({})).unwrap().await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert!(v.get("system").is_some());
        assert!(v.get("portal_glyphs_hex").is_some());
        assert!(v.get("galaxy").is_some());
    }

    #[tokio::test]
    async fn test_galaxy_stats_returns_counts() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("galaxy_stats", json!({})).unwrap().await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert!(v["systems"].as_u64().unwrap() >= 3);
        assert!(v["planets"].as_u64().unwrap() >= 3);
    }

    #[tokio::test]
    async fn test_search_planets_all() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("search_planets", json!({})).unwrap().await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert!(v["count"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_search_planets_invalid_biome() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("search_planets", json!({"biome": "NotABiome"}))
            .unwrap()
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whats_nearby_default() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("whats_nearby", json!({})).unwrap().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_whats_nearby_with_count() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("whats_nearby", json!({"count": 1}))
            .unwrap()
            .await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert!(v["count"].as_u64().unwrap() <= 1);
    }

    #[tokio::test]
    async fn test_show_base_existing() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("show_base", json!({"name": "Alpha Base"}))
            .unwrap()
            .await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["name"], "Alpha Base");
    }

    #[tokio::test]
    async fn test_show_base_not_found() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("show_base", json!({"name": "No Such Base"}))
            .unwrap()
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_show_base_missing_name() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("show_base", json!({})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_show_system_missing_name() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("show_system", json!({})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convert_glyphs() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("convert_coordinates", json!({"glyphs": "01717D8A4EA2"}))
            .unwrap()
            .await;
        assert!(result.is_ok());
        let ctr = result.unwrap();
        let text = extract_text(&ctr);
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["portal_glyphs_hex"], "01717D8A4EA2");
        assert!(v.get("signal_booster").is_some());
    }

    #[tokio::test]
    async fn test_convert_galactic_address() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call(
                "convert_coordinates",
                json!({"galactic_address": "0x01717D8A4EA2"}),
            )
            .unwrap()
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convert_no_input_errors() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("convert_coordinates", json!({})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convert_bad_glyphs_length() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("convert_coordinates", json!({"glyphs": "ABC"}))
            .unwrap()
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convert_bad_hex() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("convert_coordinates", json!({"glyphs": "ZZZZZZZZZZZZ"}))
            .unwrap()
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plan_route_requires_targets_or_biome() {
        let tools = NmsTools::new(test_model());
        let result = tools.call("plan_route", json!({})).unwrap().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plan_route_empty_targets_errors() {
        let tools = NmsTools::new(test_model());
        let result = tools
            .call("plan_route", json!({"targets": []}))
            .unwrap()
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_biome_arg_valid() {
        let args = json!({"biome": "Lush"});
        assert_eq!(parse_biome_arg(&args, "biome").unwrap(), Some(Biome::Lush));
    }

    #[tokio::test]
    async fn test_parse_biome_arg_invalid() {
        let args = json!({"biome": "NotReal"});
        assert!(parse_biome_arg(&args, "biome").is_err());
    }

    #[tokio::test]
    async fn test_parse_biome_arg_missing() {
        let args = json!({});
        assert_eq!(parse_biome_arg(&args, "biome").unwrap(), None);
    }

    fn extract_text(ctr: &CallToolResult) -> String {
        ctr.content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("")
    }
}
