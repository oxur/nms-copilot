//! MCP server integration for NMS Copilot.
//!
//! When running normally, the MCP server runs on a background thread with its
//! own tokio runtime, sharing the `GalaxyModel` with the REPL via
//! `Arc<RwLock<GalaxyModel>>`. In headless mode (`--headless`), only the MCP
//! server runs (no REPL).

pub mod config;
pub mod resources;
pub mod tools;

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use fabryk_mcp::{
    CompositeRegistry, DiscoverableRegistry, FabrykMcpServer, HealthTools, Notifier,
    ServerGuidance, ToolMeta, ToolRegistry,
};
use tokio::sync::RwLock;

use nms_graph::GalaxyModel;

use resources::{BASES_URI, GALAXY_STATS_URI, NmsResources, PLAYER_LOCATION_URI};
use tools::NmsTools;

/// MCP transport mode.
pub enum Transport {
    Stdio,
    #[cfg(feature = "http")]
    Http(SocketAddr),
}

/// Start the MCP server on a background thread with its own tokio runtime.
///
/// The server shares the given model with the REPL. When using HTTP transport,
/// the MCP server runs concurrently with the REPL. Stdio transport should only
/// be used in headless mode.
///
/// Optionally starts a file watcher that applies deltas to the shared model.
pub fn spawn_mcp_background(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    save_path: Option<std::path::PathBuf>,
) {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let watcher_rx = start_watcher(save_path.as_deref());

            if let Err(e) = run_mcp_server(model, transport, watcher_rx).await {
                log::error!("MCP server error: {e}");
            }
        });
    });
}

/// Run the MCP server in headless mode (blocking, on the current thread).
///
/// Creates a tokio runtime and blocks until the server shuts down.
pub fn run_headless(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    save_path: Option<std::path::PathBuf>,
) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let watcher_rx = start_watcher(save_path.as_deref());

        if let Err(e) = run_mcp_server(model, transport, watcher_rx).await {
            eprintln!("MCP server error: {e}");
        }
    });
}

/// Start the file watcher if a save path is provided.
///
/// Returns the delta receiver if the watcher started successfully.
fn start_watcher(
    save_path: Option<&std::path::Path>,
) -> Option<std::sync::mpsc::Receiver<nms_core::SaveDelta>> {
    let path = save_path?;
    let watch_config = nms_watch::WatchConfig {
        save_path: path.to_path_buf(),
        ..Default::default()
    };
    match nms_watch::start_watching(watch_config) {
        Ok(handle) => {
            log::info!("File watcher started for {}", path.display());
            Some(handle.receiver)
        }
        Err(e) => {
            log::warn!("File watcher failed to start: {e}");
            None
        }
    }
}

/// Run the MCP server (async, expects to be called from a tokio runtime).
///
/// If a watcher receiver is provided, starts a background task that applies
/// deltas to the model and pushes notifications to connected MCP clients.
async fn run_mcp_server(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    watcher_rx: Option<std::sync::mpsc::Receiver<nms_core::SaveDelta>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let nms_tools = NmsTools::new(Arc::clone(&model));
    let tool_count = nms_tools.tool_count() + 1; // +1 for health
    let health = HealthTools::new("nms-copilot", env!("CARGO_PKG_VERSION"), tool_count);

    let composite = CompositeRegistry::new().add(nms_tools).add(health);
    let guidance = build_guidance();
    let nms_resources = NmsResources::new(Arc::clone(&model));
    let discoverable = DiscoverableRegistry::from_guidance(composite, &guidance);

    let server = FabrykMcpServer::new(discoverable)
        .with_name("nms-copilot")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_guidance(&guidance)
        .with_resources(nms_resources);

    // Extract notifier before serving (serve consumes the server)
    let notifier = server.notifier();

    // Start delta loop with notifier if we have a watcher
    if let Some(receiver) = watcher_rx {
        let model_for_watcher = Arc::clone(&model);
        tokio::spawn(async move {
            apply_deltas_loop(receiver, model_for_watcher, notifier).await;
        });
    }

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

/// Background loop: receive deltas from watcher, apply to model, notify clients.
async fn apply_deltas_loop(
    receiver: std::sync::mpsc::Receiver<nms_core::SaveDelta>,
    model: Arc<RwLock<GalaxyModel>>,
    notifier: Notifier,
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

        // Notify connected MCP clients about changes
        if let Some(ref moved) = delta.player_moved {
            log::info!("Live update: player moved");
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "player_moved",
                        "from": format!("{}", moved.from),
                        "to": format!("{}", moved.to),
                    }),
                )
                .await;
            notifier.resource_updated(PLAYER_LOCATION_URI).await;
        }

        if !delta.new_systems.is_empty() {
            let names: Vec<&str> = delta
                .new_systems
                .iter()
                .filter_map(|s| s.name.as_deref())
                .collect();
            log::info!("Live update: {} new system(s)", delta.new_systems.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_systems",
                        "count": delta.new_systems.len(),
                        "names": names,
                    }),
                )
                .await;
            notifier.resource_updated(GALAXY_STATS_URI).await;
        }

        if !delta.new_planets.is_empty() {
            log::info!("Live update: {} new planet(s)", delta.new_planets.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_planets",
                        "count": delta.new_planets.len(),
                    }),
                )
                .await;
            notifier.resource_updated(GALAXY_STATS_URI).await;
        }

        if !delta.new_bases.is_empty() {
            let names: Vec<&str> = delta.new_bases.iter().map(|b| b.name.as_str()).collect();
            log::info!("Live update: {} new base(s)", delta.new_bases.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_bases",
                        "count": delta.new_bases.len(),
                        "names": names,
                    }),
                )
                .await;
            notifier.resource_updated(BASES_URI).await;
        }

        if !delta.modified_bases.is_empty() {
            notifier.resource_updated(BASES_URI).await;
        }
    }
}

fn build_guidance() -> ServerGuidance {
    ServerGuidance::for_domain("nms")
        .context(
            "Galactic copilot for No Man's Sky. Search planets, plan routes, \
             convert portal glyphs, and explore your galaxy with an AI.",
        )
        .subscribe(
            PLAYER_LOCATION_URI,
            "Live warp tracking \u{2014} know when the player moves",
        )
        .subscribe(
            GALAXY_STATS_URI,
            "Updated when new systems/planets are discovered",
        )
        .subscribe(BASES_URI, "Know when new bases are built or modified")
        .workflow("Subscribe to recommended resources for live updates")
        .workflow("Call where_am_i to establish the player's current location")
        .workflow("Use search_planets or whats_nearby to find destinations")
        .workflow("Use plan_route to plan navigation between targets")
        .workflow("Use convert_coordinates to provide portal glyphs for in-game use")
        .convention("Distances are in light-years")
        .convention("Portal glyphs are 12 hex digits rendered as emoji")
        .convention("System and planet names may be unnamed (shown as \"-\")")
        .tool_metas(vec![
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
}
