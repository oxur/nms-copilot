//! Select named or biome-matched targets and print an ordered route.
//!
//! The handler ensures the player's system and delegates route planning to
//! `nms_query::route::execute_route`. Nonempty named targets suppress biome parsing and take
//! precedence. `nn` and `nearest-neighbor` select nearest-neighbor; `2opt`, `two-opt`, and
//! omission select two-opt. `--from` chooses a base or current position; other options are
//! forwarded to the query.
//!
//! # Examples
//!
//! ```text
//! nms route --save data/test/save56_multi_system.json --biome Lush --algo nn
//! ```
//!
//! Assumes repository root and built `nms` on `PATH`; fixture is synthetic and not executed.

use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_query::display::format_route;
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::theme::{Theme, should_use_colors};

/// Target, origin, and routing options for the route handler.
///
/// Named targets take precedence over biome. `nn` and `nearest-neighbor` select nearest-neighbor;
/// `2opt`, `two-opt`, or omission selects two-opt.
pub struct RouteArgs {
    /// Optional checked-save path; absent selects newest discovered save.
    pub save: Option<PathBuf>,
    /// Optional biome when named targets are absent.
    pub biome: Option<String>,
    /// Named base/system targets in caller order.
    pub targets: Vec<String>,
    /// Optional base-name origin; absent uses current position.
    pub from: Option<String>,
    /// Optional warp range forwarded to routing.
    pub warp_range: Option<f64>,
    /// Optional target radius forwarded to routing.
    pub within: Option<f64>,
    /// Optional maximum target count.
    pub max_targets: Option<usize>,
    /// Optional algorithm name; absent selects two-opt.
    pub algo: Option<String>,
    /// Whether to return to the origin.
    pub round_trip: bool,
}

/// Plan and print a route from a checked save.
///
/// Nonempty named targets suppress biome parsing and take precedence; with neither targets nor
/// biome the handler returns an input error. `nn` and `nearest-neighbor` select nearest-neighbor;
/// `2opt`, `two-opt`, and omission select two-opt. `--from` selects a base or current position.
/// Warp range, radius, target count, and round-trip are forwarded without added validation. The
/// color gate chooses dark or no theme, but formatting may still emit control characters.
///
/// # Examples
///
/// ```text
/// nms route --save data/test/save56_multi_system.json --biome Lush --algo nn
/// ```
///
/// # Errors
///
/// Returns save, validation, biome, target, origin, algorithm, or routing errors.
///
/// # Panics
///
/// Printing may panic if stdout fails. The imported [`execute_route`] may overflow jump-count
/// accumulation on checking profiles when a supplied warp range yields an unrepresentable total.
pub fn run(args: RouteArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Load save file
    let path = match args.save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_validated_save_file(&path)?;

    // Build model
    let mut model = GalaxyModel::from_validated_save(&save);
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
