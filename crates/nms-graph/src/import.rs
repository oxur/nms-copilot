//! Community data import from CSV files.
//!
//! Imports coordinate data (e.g., from NMSCE, wiki, or community spreadsheets)
//! into the galaxy model. Each row must include at minimum a system name and
//! portal glyphs; galaxy, biome, and platform columns are optional.

use std::path::Path;

use serde::Deserialize;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_core::system::{Planet, System};

use crate::model::GalaxyModel;
use crate::spatial::SystemId;

/// Statistics from a CSV import operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportStats {
    /// Number of new systems added to the model.
    pub added: usize,
    /// Number of rows that matched an existing system (skipped).
    pub duplicates: usize,
    /// Number of rows that could not be parsed (skipped).
    pub skipped: usize,
}

/// Errors that can occur during community data import.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// CSV reading/parsing error.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    /// I/O error reading the file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A single row from a community coordinate CSV.
#[derive(Debug, Deserialize)]
struct CommunityRecord {
    #[serde(alias = "System Name", alias = "system_name", alias = "Name")]
    system_name: String,

    #[serde(alias = "Galaxy", alias = "galaxy", default = "default_galaxy")]
    galaxy: String,

    #[serde(
        alias = "Portal Glyphs",
        alias = "portal_glyphs",
        alias = "Glyphs",
        alias = "glyphs"
    )]
    portal_glyphs: String,

    #[serde(alias = "Biome", alias = "biome", default)]
    biome: Option<String>,

    #[allow(dead_code)]
    #[serde(alias = "Platform", alias = "platform", default)]
    platform: Option<String>,
}

fn default_galaxy() -> String {
    "Euclid".to_string()
}

/// Resolve a galaxy name to its index (0-255).
///
/// Performs case-insensitive matching against all 256 galaxy names.
/// Falls back to 0 (Euclid) if the name is not recognized.
fn resolve_galaxy_index(name: &str) -> u8 {
    // Try parsing as a numeric index first
    if let Ok(idx) = name.parse::<u8>() {
        return idx;
    }

    let lower = name.to_lowercase();
    for i in 0..=255u8 {
        let galaxy = Galaxy::by_index(i);
        if galaxy.name.to_lowercase() == lower {
            return i;
        }
    }

    log::warn!("Unknown galaxy name '{}', defaulting to Euclid (0)", name);
    0
}

/// Import community coordinate data from a CSV file into the galaxy model.
///
/// Expected CSV columns (flexible naming via aliases):
/// - `System Name` (required)
/// - `Portal Glyphs` (required) -- 12 hex digits
/// - `Galaxy` (optional, defaults to "Euclid")
/// - `Biome` (optional)
/// - `Platform` (optional, currently ignored)
///
/// Returns [`ImportStats`] summarizing what was imported.
pub fn import_csv(
    model: &mut GalaxyModel,
    path: &Path,
    _source_name: &str,
) -> Result<ImportStats, ImportError> {
    let mut stats = ImportStats::default();
    let mut reader = csv::Reader::from_path(path)?;

    for result in reader.deserialize() {
        let record: CommunityRecord = match result {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Skipping malformed CSV row: {}", e);
                stats.skipped += 1;
                continue;
            }
        };

        // Parse portal glyphs (12 hex digits) into a packed u64
        let glyphs = record.portal_glyphs.trim();
        let hex_str = glyphs
            .strip_prefix("0x")
            .or_else(|| glyphs.strip_prefix("0X"))
            .unwrap_or(glyphs);

        if hex_str.len() != 12 {
            log::warn!(
                "Skipping '{}': portal glyphs '{}' must be exactly 12 hex digits",
                record.system_name,
                record.portal_glyphs
            );
            stats.skipped += 1;
            continue;
        }

        let packed = match u64::from_str_radix(hex_str, 16) {
            Ok(v) => v,
            Err(_) => {
                log::warn!(
                    "Skipping '{}': invalid hex in portal glyphs '{}'",
                    record.system_name,
                    record.portal_glyphs
                );
                stats.skipped += 1;
                continue;
            }
        };

        let galaxy_index = resolve_galaxy_index(&record.galaxy);
        let addr = GalacticAddress::from_packed(packed, galaxy_index);
        let sys_id = SystemId::from_address(&addr);

        // Check for duplicate
        if model.systems.contains_key(&sys_id) {
            stats.duplicates += 1;
            continue;
        }

        // Parse optional biome
        let biome = record
            .biome
            .as_deref()
            .and_then(|b| b.parse::<Biome>().ok());

        // Build planet list (a single planet if biome is provided)
        let planets = if let Some(b) = biome {
            vec![Planet::new(0, Some(b), None, false, None, None)]
        } else {
            vec![]
        };

        let system = System::new(addr, Some(record.system_name), None, None, planets);

        model.insert_system(system);
        stats.added += 1;
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a temporary CSV file with the given content.
    fn csv_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_import_csv_basic() {
        let csv = "\
System Name,Galaxy,Portal Glyphs,Biome
Alpha System,Euclid,01717D8A4EA2,Lush
Beta System,Euclid,0A0002001234,Toxic
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 2);
        assert_eq!(stats.duplicates, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(model.system_count(), 2);
    }

    #[test]
    fn test_import_csv_duplicate_detection() {
        let csv = "\
System Name,Galaxy,Portal Glyphs,Biome
Alpha System,Euclid,01717D8A4EA2,Lush
Alpha Duplicate,Euclid,01717D8A4EA2,Toxic
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        assert_eq!(stats.duplicates, 1);
        assert_eq!(model.system_count(), 1);
    }

    #[test]
    fn test_import_csv_bad_glyphs_skipped() {
        let csv = "\
System Name,Galaxy,Portal Glyphs,Biome
Good System,Euclid,01717D8A4EA2,Lush
Bad Glyphs,Euclid,ZZZZZZZZZZZZ,Lush
Short Glyphs,Euclid,0171,Lush
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        assert_eq!(stats.skipped, 2);
        assert_eq!(model.system_count(), 1);
    }

    #[test]
    fn test_import_csv_no_biome() {
        let csv = "\
System Name,Galaxy,Portal Glyphs
No Biome System,Euclid,01717D8A4EA2
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        let sys_id = *model.systems.keys().next().unwrap();
        let system = model.system(&sys_id).unwrap();
        assert!(system.planets.is_empty());
    }

    #[test]
    fn test_import_csv_default_galaxy() {
        let csv = "\
System Name,Portal Glyphs
Default Galaxy,01717D8A4EA2
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        let sys_id = *model.systems.keys().next().unwrap();
        let system = model.system(&sys_id).unwrap();
        assert_eq!(system.address.reality_index, 0);
    }

    #[test]
    fn test_import_csv_eissentam_galaxy() {
        let csv = "\
System Name,Galaxy,Portal Glyphs
Eissentam System,Eissentam,01717D8A4EA2
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        let sys_id = *model.systems.keys().next().unwrap();
        let system = model.system(&sys_id).unwrap();
        assert_eq!(system.address.reality_index, 9);
    }

    #[test]
    fn test_import_csv_galaxy_numeric() {
        let csv = "\
System Name,Galaxy,Portal Glyphs
Numeric Galaxy,9,01717D8A4EA2
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        let sys_id = *model.systems.keys().next().unwrap();
        let system = model.system(&sys_id).unwrap();
        assert_eq!(system.address.reality_index, 9);
    }

    #[test]
    fn test_import_csv_into_populated_model() {
        let mut model = GalaxyModel::new();

        // Pre-populate with a system
        let addr = GalacticAddress::new(100, 0, 0, 0x100, 0, 0);
        let existing = System::new(addr, Some("Existing".into()), None, None, vec![]);
        model.insert_system(existing);
        assert_eq!(model.system_count(), 1);

        let csv = "\
System Name,Galaxy,Portal Glyphs
New System,Euclid,01717D8A4EA2
";
        let file = csv_file(csv);
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
        assert_eq!(model.system_count(), 2);
    }

    #[test]
    fn test_import_csv_empty_file() {
        let csv = "System Name,Galaxy,Portal Glyphs\n"; // headers only
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 0);
        assert_eq!(stats.duplicates, 0);
        assert_eq!(stats.skipped, 0);
    }

    #[test]
    fn test_import_csv_multi_galaxy() {
        let csv = "\
System Name,Galaxy,Portal Glyphs
Euclid Sys,Euclid,01717D8A4EA2
Hilbert Sys,Hilbert Dimension,0A0002001234
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 2);

        // Should have separate spatial indexes per galaxy
        let galaxies = model.discovered_galaxies();
        assert_eq!(galaxies.len(), 2);
        assert!(galaxies.contains(&0));
        assert!(galaxies.contains(&1));
    }

    #[test]
    fn test_import_csv_with_biome_creates_planet() {
        let csv = "\
System Name,Galaxy,Portal Glyphs,Biome
Lush World,Euclid,01717D8A4EA2,Lush
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        import_csv(&mut model, file.path(), "test").unwrap();

        let sys_id = *model.systems.keys().next().unwrap();
        let system = model.system(&sys_id).unwrap();
        assert_eq!(system.planets.len(), 1);
        assert_eq!(system.planets[0].biome, Some(Biome::Lush));
    }

    #[test]
    fn test_import_csv_file_not_found() {
        let mut model = GalaxyModel::new();
        let result = import_csv(&mut model, Path::new("/no/such/file.csv"), "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_galaxy_index_euclid() {
        assert_eq!(resolve_galaxy_index("Euclid"), 0);
    }

    #[test]
    fn test_resolve_galaxy_index_case_insensitive() {
        assert_eq!(resolve_galaxy_index("euclid"), 0);
        assert_eq!(resolve_galaxy_index("EUCLID"), 0);
        assert_eq!(resolve_galaxy_index("eissentam"), 9);
    }

    #[test]
    fn test_resolve_galaxy_index_numeric() {
        assert_eq!(resolve_galaxy_index("0"), 0);
        assert_eq!(resolve_galaxy_index("9"), 9);
        assert_eq!(resolve_galaxy_index("255"), 255);
    }

    #[test]
    fn test_resolve_galaxy_index_unknown_defaults_to_zero() {
        assert_eq!(resolve_galaxy_index("NotAGalaxy"), 0);
    }

    #[test]
    fn test_import_csv_glyphs_with_0x_prefix() {
        let csv = "\
System Name,Galaxy,Portal Glyphs
Prefixed,Euclid,0x01717D8A4EA2
";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();

        assert_eq!(stats.added, 1);
    }

    #[test]
    fn test_import_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.added, 0);
        assert_eq!(stats.duplicates, 0);
        assert_eq!(stats.skipped, 0);
    }
}
