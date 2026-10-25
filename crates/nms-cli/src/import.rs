//! Import a caller-provided community CSV into a temporary checked-save model.
//!
//! The source label is forwarded as-is to `nms_graph::import::import_csv`, which owns duplicate,
//! source, and name policy. Counts go to stderr. The local model is discarded; the save is not
//! written back and no cache, export, or persistent atlas is created.
//!
//! # Examples
//!
//! ```text
//! nms import community.csv --save data/test/save56_minimal.json --source NMSCE
//! ```
//!
//! `community.csv` is caller-supplied, not shipped; this example is not executed here.

use std::path::PathBuf;

/// Import a caller-provided CSV into a temporary checked-save model.
///
/// The source label is forwarded unchanged to `nms_graph::import::import_csv`, which owns
/// duplicate, source, and name policy. Counts go to stderr as `Imported {added} systems
/// ({duplicates} duplicates, {skipped} skipped)`, followed by a newline, using the returned
/// statistics fields. The model is discarded; the save is not written back and no cache, export,
/// or persistent atlas is created. An error may follow
/// partial mutation of the local model, which is then discarded.
///
/// # Examples
///
/// ```text
/// nms import community.csv --save data/test/save56_minimal.json --source NMSCE
/// ```
///
/// # Errors
///
/// Returns save-resolution, validation, or CSV import errors.
///
/// # Panics
///
/// Printing the summary may panic if stderr fails.
pub fn run(
    file: PathBuf,
    save: Option<PathBuf>,
    source: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = crate::resolve_save(save)?;
    let save_data = nms_save::parse_validated_save_file(&path)?;
    let mut model = nms_graph::GalaxyModel::from_validated_save(&save_data);
    let stats = nms_graph::import::import_csv(&mut model, &file, &source)?;
    eprintln!(
        "Imported {} systems ({} duplicates, {} skipped)",
        stats.added, stats.duplicates, stats.skipped
    );
    Ok(())
}
