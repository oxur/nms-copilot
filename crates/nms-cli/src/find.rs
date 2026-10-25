//! Search checked-save planets using the filters exposed by `nms find`.
//!
//! The handler ensures the player's system and delegates matching and nearest ordering to
//! `nms_query::find`. Filters are biome, infested, radius, nearest count, named-only, and system
//! discoverer substring. `--from` selects a base; otherwise current position is used. Name
//! pattern, subtype, and explicit address query inputs are not exposed by this handler.
//!
//! # Examples
//!
//! ```text
//! nms find --save data/test/save56_multi_system.json --biome Lush --nearest 3
//! ```
//!
//! Assumes repository root and built `nms` on `PATH`; fixture is synthetic and not executed.

use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::display::format_find_results;
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::theme::{Theme, should_use_colors};

/// Filters exposed by this handler.
///
/// Name-pattern, subtype, and address inputs are not part of the CLI surface.
pub struct FindArgs {
    /// Optional checked-save path; absent selects newest discovered save.
    pub save: Option<PathBuf>,
    /// Optional biome name.
    pub biome: Option<String>,
    /// Whether to require infested planets.
    pub infested: bool,
    /// Optional maximum reference distance in light-years.
    pub within: Option<f64>,
    /// Optional result count, nearest first.
    pub nearest: Option<usize>,
    /// Whether a planet name must be present.
    pub named: bool,
    /// Optional system-discoverer substring.
    pub discoverer: Option<String>,
    /// Optional base-name reference; absent uses current position.
    pub from: Option<String>,
}

/// Search a checked save and print planets matching the exposed filters.
///
/// The handler ensures the player's system. False `infested` means no filter; true requires an
/// infested planet. `named` requires `name` to be `Some`, including empty. Discoverer matching
/// is a case-insensitive substring of the SYSTEM discoverer. Filters combine before nearest
/// ordering. `--from` selects a base; otherwise current position is used. The color gate selects
/// the default dark theme or no theme, but the table backend may emit controls; piping does not
/// guarantee ANSI-free text.
///
/// # Examples
///
/// ```text
/// nms find --save data/test/save56_multi_system.json --biome Lush --nearest 3
/// ```
///
/// # Errors
///
/// Returns save, validation, biome, reference, or query errors.
///
/// # Panics
///
/// Printing results may panic if stdout fails.
pub fn run(args: FindArgs) -> Result<(), Box<dyn std::error::Error>> {
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

    // Parse biome filter
    let biome = args
        .biome
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| format!("Invalid biome: {e}"))?;

    // Resolve reference point
    let reference = match args.from {
        Some(name) => ReferencePoint::Base(name),
        None => ReferencePoint::CurrentPosition,
    };

    let query = FindQuery {
        biome,
        biome_subtype: None,
        infested: if args.infested { Some(true) } else { None },
        within_ly: args.within,
        nearest: args.nearest,
        name_pattern: None,
        discoverer: args.discoverer,
        named_only: args.named,
        from: reference,
    };

    let results = execute_find(&model, &query)?;
    let theme = if should_use_colors(true) {
        Theme::default_dark()
    } else {
        Theme::none()
    };
    print!("{}", format_find_results(&results, &theme));

    Ok(())
}
