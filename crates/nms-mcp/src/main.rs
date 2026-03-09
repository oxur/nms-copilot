//! NMS Copilot MCP Server -- AI integration for No Man's Sky.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use fabryk_mcp::{
    CompositeRegistry, DiscoverableRegistry, FabrykMcpServer, HealthTools, ToolMeta, ToolRegistry,
};
use tokio::sync::RwLock;

mod config;
mod tools;

use config::McpConfig;
use tools::NmsTools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mcp_config = McpConfig::load();
    if let Err(e) = twyg::setup(mcp_config.logging.to_twyg_opts()) {
        eprintln!("Warning: Failed to initialize logging: {e}");
    }

    let save_path = parse_save_arg();
    let transport = parse_transport();
    let model = load_model(save_path.clone())?;
    let model = Arc::new(RwLock::new(model));

    // Start file watcher if we have a save path
    if let Some(ref path) = save_path {
        let watch_config = nms_watch::WatchConfig {
            save_path: path.clone(),
            ..Default::default()
        };
        match nms_watch::start_watching(watch_config) {
            Ok(handle) => {
                log::info!("File watcher started for {}", path.display());
                let model_for_watcher = Arc::clone(&model);
                tokio::spawn(async move {
                    apply_deltas_loop(handle.receiver, model_for_watcher).await;
                });
            }
            Err(e) => {
                log::warn!("File watcher failed to start: {e}");
            }
        }
    }

    let nms_tools = NmsTools::new(Arc::clone(&model));
    let tool_count = nms_tools.tool_count() + 1; // +1 for health
    let health = HealthTools::new("nms-copilot", env!("CARGO_PKG_VERSION"), tool_count);

    let composite = CompositeRegistry::new().add(nms_tools).add(health);

    let discoverable = build_discoverable(composite);

    let server = FabrykMcpServer::new(discoverable)
        .with_name("nms-copilot")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description(
            "Galactic copilot for No Man's Sky. Search planets, plan routes, \
             convert portal glyphs, and explore your galaxy with an AI.",
        )
        .with_discoverable_instructions("nms");

    match transport {
        Transport::Stdio => {
            server.serve_stdio().await?;
        }
        #[cfg(feature = "http")]
        Transport::Http(addr) => {
            eprintln!("NMS Copilot MCP server listening on http://{addr}");
            server.serve_http(addr).await?;
        }
    }

    Ok(())
}

/// Background loop: receive deltas from watcher, apply to model.
async fn apply_deltas_loop(
    receiver: std::sync::mpsc::Receiver<nms_core::SaveDelta>,
    model: Arc<RwLock<nms_graph::GalaxyModel>>,
) {
    // Bridge std::sync::mpsc to tokio::sync::mpsc
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    thread::spawn(move || {
        while let Ok(delta) = receiver.recv() {
            if tx.send(delta).is_err() {
                break;
            }
        }
    });

    while let Some(delta) = rx.recv().await {
        let mut model = model.write().await;
        model.apply_delta(&delta);

        if !delta.new_systems.is_empty() {
            log::info!("Live update: {} new system(s)", delta.new_systems.len());
        }
        if delta.player_moved.is_some() {
            log::info!("Live update: player moved");
        }
    }
}

enum Transport {
    Stdio,
    #[cfg(feature = "http")]
    Http(std::net::SocketAddr),
}

fn parse_transport() -> Transport {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--http") {
        let addr: std::net::SocketAddr = args
            .get(pos + 1)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
        #[cfg(feature = "http")]
        return Transport::Http(addr);
        #[cfg(not(feature = "http"))]
        {
            eprintln!("HTTP transport requires the 'http' feature. Falling back to stdio.");
            let _ = addr;
            Transport::Stdio
        }
    } else {
        Transport::Stdio
    }
}

fn build_discoverable(registry: CompositeRegistry) -> DiscoverableRegistry<CompositeRegistry> {
    DiscoverableRegistry::new(registry, "nms")
        .with_tool_metas(vec![
            (
                "search_planets",
                ToolMeta {
                    summary: "Search planets by biome, distance, discoverer, or name.".into(),
                    when_to_use: "Looking for planets with a specific biome or property".into(),
                    returns: "Ranked list of matching planets with coordinates and \
                              portal glyphs"
                        .into(),
                    next: Some("Call plan_route to navigate to results".into()),
                    category: Some("search".into()),
                },
            ),
            (
                "plan_route",
                ToolMeta {
                    summary: "Plan an optimal route through target systems.".into(),
                    when_to_use: "Need to visit multiple systems efficiently with warp range \
                         constraints"
                        .into(),
                    returns: "Step-by-step itinerary with distances and portal glyphs".into(),
                    next: Some(
                        "Use convert_coordinates to get portal addresses for waypoints".into(),
                    ),
                    category: Some("navigation".into()),
                },
            ),
            (
                "where_am_i",
                ToolMeta {
                    summary: "Get the player's current location.".into(),
                    when_to_use: "Need to know the player's current system and coordinates".into(),
                    returns: "System name, coordinates, portal glyphs, galaxy".into(),
                    next: Some("Call whats_nearby for situational awareness".into()),
                    category: Some("location".into()),
                },
            ),
            (
                "whats_nearby",
                ToolMeta {
                    summary: "Find systems and planets near the player's current \
                              position."
                        .into(),
                    when_to_use: "Need situational awareness or looking for nearby options".into(),
                    returns: "Nearby systems with distances, biomes, and portal glyphs".into(),
                    next: Some(
                        "Call search_planets for filtered results or plan_route to \
                         navigate"
                            .into(),
                    ),
                    category: Some("search".into()),
                },
            ),
            (
                "show_system",
                ToolMeta {
                    summary: "Get detailed information about a star system.".into(),
                    when_to_use: "Need details about a specific system (planets, \
                                  discoverer, coordinates)"
                        .into(),
                    returns: "System details with all discovered planets and their \
                              biomes"
                        .into(),
                    next: None,
                    category: Some("detail".into()),
                },
            ),
            (
                "show_base",
                ToolMeta {
                    summary: "Get detailed information about a player base.".into(),
                    when_to_use: "Need details about a specific base (location, type, system)"
                        .into(),
                    returns: "Base details with portal glyphs and system context".into(),
                    next: None,
                    category: Some("detail".into()),
                },
            ),
            (
                "convert_coordinates",
                ToolMeta {
                    summary: "Convert between portal glyphs, signal booster, and \
                              galactic addresses."
                        .into(),
                    when_to_use: "Need to convert coordinates between formats for in-game use"
                        .into(),
                    returns: "All coordinate formats for the given address".into(),
                    next: None,
                    category: Some("utility".into()),
                },
            ),
            (
                "galaxy_stats",
                ToolMeta {
                    summary: "Get aggregate statistics about the explored galaxy.".into(),
                    when_to_use: "Want an overview of discoveries, biome distribution, \
                                  or progress"
                        .into(),
                    returns: "System/planet/base counts with biome breakdown".into(),
                    next: Some("Call search_planets to explore specific biomes".into()),
                    category: Some("overview".into()),
                },
            ),
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
    let mut model = nms_graph::GalaxyModel::from_save(&save);
    model.ensure_player_system();
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transport_default_stdio() {
        // Default (no --http) should be Stdio
        assert!(matches!(parse_transport(), Transport::Stdio));
    }
}
