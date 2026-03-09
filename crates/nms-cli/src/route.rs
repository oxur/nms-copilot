//! `nms route` command -- plan routes through discovered systems.

use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_query::display::format_route;
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::theme::{Theme, should_use_colors};

/// Arguments for the route command.
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
    // Load save file
    let path = match args.save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;

    // Build model
    let mut model = GalaxyModel::from_save(&save);
    model.ensure_player_system();

    // Determine targets
    let targets = if !args.targets.is_empty() {
        TargetSelection::Named(args.targets)
    } else if let Some(ref biome_str) = args.biome {
        let biome: Biome = biome_str
            .parse()
            .map_err(|e| format!("Invalid biome: {e}"))?;
        TargetSelection::Biome(BiomeFilter {
            biome: Some(biome),
            ..Default::default()
        })
    } else {
        return Err("Specify --target or --biome for route planning".into());
    };

    // Resolve reference point
    let from = match args.from {
        Some(name) => RouteFrom::Base(name),
        None => RouteFrom::CurrentPosition,
    };

    // Parse algorithm
    let algorithm = match args.algo.as_deref() {
        Some("nn") | Some("nearest-neighbor") => RoutingAlgorithm::NearestNeighbor,
        Some("2opt") | Some("two-opt") | None => RoutingAlgorithm::TwoOpt,
        Some(other) => {
            return Err(format!(
                "Unknown algorithm: \"{other}\". Use: nn, nearest-neighbor, 2opt, two-opt"
            )
            .into());
        }
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
    let theme = if should_use_colors(true) {
        Theme::default_dark()
    } else {
        Theme::none()
    };
    print!("{}", format_route(&result, &model, &theme));

    Ok(())
}
