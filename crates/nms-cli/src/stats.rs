//! Print aggregate statistics for the complete checked-save model.
//!
//! The handler builds `biomes || !discoveries` and `discoveries || !biomes` query inputs. The
//! shared query currently ignores both and computes all aggregates across every galaxy. The
//! handler does not ensure a player-system record.
//!
//! # Examples
//!
//! ```text
//! nms stats --save data/test/save56_minimal.json --biomes
//! ```
//!
//! `--biomes` is currently ignored; this synthetic-fixture command is not executed.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::display::format_stats;
use nms_query::stats::{StatsQuery, execute_stats};
use nms_query::theme::{Theme, should_use_colors};

/// Print aggregate statistics for the complete checked-save model.
///
/// The handler builds `biomes || !discoveries` and `discoveries || !biomes` inputs. The shared
/// query currently ignores both flags and computes all aggregates across every galaxy. No
/// player-system record is ensured.
///
/// # Examples
///
/// ```text
/// nms stats --save data/test/save56_minimal.json --biomes
/// ```
///
/// # Errors
///
/// Returns save-discovery or validation errors.
///
/// # Panics
///
/// Printing may panic if stdout fails.
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
    let save = nms_save::parse_validated_save_file(&path)?;
    let model = GalaxyModel::from_validated_save(&save);

    let query = StatsQuery {
        biomes: biomes || !discoveries, // default to all if neither specified
        discoveries: discoveries || !biomes,
    };

    let result = execute_stats(&model, &query);
    let theme = if should_use_colors(true) {
        Theme::default_dark()
    } else {
        Theme::none()
    };
    print!("{}", format_stats(&result, &theme));

    Ok(())
}
