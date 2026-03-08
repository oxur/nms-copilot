//! `nms show` command -- detail views for systems and bases.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::display::format_show_result;
use nms_query::show::{ShowQuery, execute_show};
use nms_query::theme::{Theme, should_use_colors};

/// What to show -- parsed from CLI subcommand.
pub enum ShowTarget {
    System { name: String },
    Base { name: String },
}

pub fn run(save: Option<PathBuf>, target: ShowTarget) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    let model = GalaxyModel::from_save(&save);

    let query = match target {
        ShowTarget::System { name } => ShowQuery::System(name),
        ShowTarget::Base { name } => ShowQuery::Base(name),
    };

    let result = execute_show(&model, &query)?;
    let theme = if should_use_colors(true) {
        Theme::default_dark()
    } else {
        Theme::none()
    };
    print!("{}", format_show_result(&result, &theme));

    Ok(())
}
