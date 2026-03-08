//! `nms find` command -- search planets by biome, distance, name, discoverer.

use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::display::format_find_results;
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::theme::{Theme, should_use_colors};

/// Arguments for the find command.
pub struct FindArgs {
    pub save: Option<PathBuf>,
    pub biome: Option<String>,
    pub infested: bool,
    pub within: Option<f64>,
    pub nearest: Option<usize>,
    pub named: bool,
    pub discoverer: Option<String>,
    pub from: Option<String>,
}

pub fn run(args: FindArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Load save file
    let path = match args.save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;

    // Build model
    let model = GalaxyModel::from_save(&save);

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
