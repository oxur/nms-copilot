//! Resolve and print a system or base detail view from a checked save.
//!
//! System names are resolved before packed addresses in the active galaxy. Base lookup uses the
//! model-wide case-insensitive index and does not trim the requested name. The handler does not
//! require a player-system record.
//!
//! # Examples
//!
//! ```text
//! nms show --save data/test/save56_minimal.json base "Homestead Alpha"
//! ```
//!
//! Synthetic-fixture command text only; not executed here.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::display::format_show_result;
use nms_query::show::{ShowQuery, execute_show};
use nms_query::theme::{Theme, should_use_colors};

/// Detail target selected by the `show` command.
pub enum ShowTarget {
    /// Resolve a system name or packed address through the shared query.
    System {
        /// System name or packed hexadecimal address text.
        name: String,
    },
    /// Resolve a base through the model-wide base index.
    Base {
        /// Base name, matched case-insensitively by the query.
        name: String,
    },
}

/// Load a checked save, resolve one detail target, and print its view.
///
/// System lookup tries a name before a packed address in the active galaxy. Base lookup is
/// model-wide, case-insensitive, and does not trim the supplied name. The handler does not
/// ensure a player-system record.
///
/// # Examples
///
/// ```text
/// nms show --save data/test/save56_minimal.json base "Homestead Alpha"
/// ```
///
/// # Errors
///
/// Returns save, validation, or unresolved system/base errors.
///
/// # Panics
///
/// Printing the detail view may panic if stdout fails.
pub fn run(save: Option<PathBuf>, target: ShowTarget) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_validated_save_file(&path)?;
    let model = GalaxyModel::from_validated_save(&save);

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
