//! Handler for the `nms import` command.

use std::path::PathBuf;

pub fn run(
    file: PathBuf,
    save: Option<PathBuf>,
    source: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = crate::resolve_save(save)?;
    let save_data = nms_save::parse_save_file(&path)?;
    let mut model = nms_graph::GalaxyModel::from_save(&save_data);
    let stats = nms_graph::import::import_csv(&mut model, &file, &source)?;
    eprintln!(
        "Imported {} systems ({} duplicates, {} skipped)",
        stats.added, stats.duplicates, stats.skipped
    );
    Ok(())
}
