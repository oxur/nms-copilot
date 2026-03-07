//! `nms stats` command -- aggregate galaxy statistics.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::display::format_stats;
use nms_query::stats::{StatsQuery, execute_stats};

pub fn run(
    save: Option<PathBuf>,
    biomes: bool,
    discoveries: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    let model = GalaxyModel::from_save(&save);

    let query = StatsQuery {
        biomes: biomes || !discoveries, // default to all if neither specified
        discoveries: discoveries || !biomes,
    };

    let result = execute_stats(&model, &query);
    print!("{}", format_stats(&result));

    Ok(())
}
