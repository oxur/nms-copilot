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
use nms_core::system::{DiscoverySource, Planet, System};

use crate::model::GalaxyModel;
use crate::spatial::SystemId;

/// Statistics from a CSV import operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportStats {
    /// Number of accepted catalog contributions, including marked-placeholder
    /// promotions. Promotion does not increase [`GalaxyModel::system_count`].
    pub added: usize,
    /// Number of rows that matched an existing genuine system and were skipped.
    pub duplicates: usize,
    /// Number of rows that could not be parsed (skipped).
    pub skipped: usize,
}

/// Errors that can occur while creating or reading a community CSV reader.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// CSV reader construction or CSV parsing error.
    #[error("CSV error: {0}")]
    Csv(
        /// Underlying CSV error.
        #[from]
        csv::Error,
    ),
    /// Public conversion from a standard file I/O error.
    #[error("I/O error: {0}")]
    Io(
        /// Underlying I/O error.
        #[from]
        std::io::Error,
    ),
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
/// - `Galaxy` (optional, defaults to "Euclid"); an unrecognized or empty value
///   falls back to index 0. Numeric indexes and case-insensitive galaxy names
///   are accepted without trimming the galaxy field.
/// - `Biome` (optional); a recognized biome creates planet index 0, while an
///   absent or unrecognized biome leaves the accepted system with no planets.
/// - `Platform` (optional, currently ignored)
///
/// Accepted headers are `system_name`, `System Name`, or `Name` for the system
/// name; `portal_glyphs`, `Portal Glyphs`, `Glyphs`, or `glyphs` for glyphs;
/// `galaxy` or `Galaxy`; `biome` or `Biome`; and `platform` or `Platform` (the
/// platform column is ignored). Glyph text is trimmed and accepts an optional
/// `0x`/`0X` prefix, then must contain exactly 12 hexadecimal digits.
///
/// Returns [`ImportStats`] summarizing what was imported.
///
/// A row matching an existing genuine system is counted as a duplicate; a
/// marked player placeholder is replaced in place and counted as added. Invalid
/// rows or glyphs increment `skipped`. A recognized biome creates one planet at
/// index 0; an unrecognized biome is accepted with an empty planet list. The
/// exact `source_name` is stored as community contribution provenance, not as
/// the discoverer. The system address retains the glyph planet index, which need
/// not be zero. This method does not build graph edges or persist a cache. A
/// failure to create the initial CSV reader is returned as [`ImportError::Csv`].
///
/// # Examples
///
/// ```
/// use std::fs;
/// use std::path::Path;
///
/// use nms_core::{Biome, GalacticAddress, SystemId};
/// use nms_graph::{GalaxyModel, import::{ImportStats, import_csv}};
///
/// let temp = tempfile::tempdir()?;
/// let path = temp.path().join("catalog.csv");
/// fs::write(&path, b"System Name,Portal Glyphs,Galaxy,Biome\nAtlas,200100000003,9,Lush\nIgnored,200100000003,9,Toxic\nBad,bad,9,Lush\n")?;
/// let mut model = GalaxyModel::new();
/// let stats = import_csv(&mut model, Path::new(&path), "local catalog")?;
/// assert_eq!(stats, ImportStats { added: 1, duplicates: 1, skipped: 1 });
/// let address = GalacticAddress::new(3, 0, 0, 1, 2, 9);
/// let id = SystemId::from_address(&address);
/// let system = model.system(&id).ok_or("missing imported system")?;
/// assert_eq!(system.name.as_deref(), Some("Atlas"));
/// assert_eq!(system.source, nms_core::DiscoverySource::Community { source: "local catalog".into() });
/// assert_eq!(system.address.planet_index(), 2);
/// assert_eq!(model.planets.get(&(id, 0)).and_then(|planet| planet.biome), Some(Biome::Lush));
/// assert_eq!(model.graph.edge_count(), 0);
/// assert_eq!(id.to_string(), "9:000100000003");
/// assert_eq!(model.planet_count(), 1);
/// assert_eq!(model.system(&id).ok_or("missing system")?.planets.iter().map(|planet| (planet.index, planet.biome)).collect::<Vec<_>>(), vec![(0, Some(Biome::Lush))]);
/// assert_eq!(model.system_count(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`ImportError::Csv`] if the initial CSV reader cannot be created.
/// Per-row CSV deserialization and invalid glyph rows are counted as skipped.
/// The public [`ImportError::Io`] conversion exists, but this function does not
/// directly perform file I/O that returns that variant.
pub fn import_csv(
    model: &mut GalaxyModel,
    path: &Path,
    source_name: &str,
) -> Result<ImportStats, ImportError> {
    let mut stats = ImportStats::default();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

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
        if model.systems.contains_key(&sys_id) && !model.is_player_placeholder(&sys_id) {
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

        let mut system = System::new(addr, Some(record.system_name), None, None, planets);
        system.source = DiscoverySource::Community {
            source: source_name.to_owned(),
        };

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
        assert!(model.systems.values().all(|system| {
            system.source
                == DiscoverySource::Community {
                    source: "test".to_string(),
                }
        }));
    }

    #[test]
    fn test_import_csv_preserves_source_and_accepts_ragged_optional_columns() {
        let csv = csv_file(
            "System Name,Galaxy,Portal Glyphs,Biome,Platform\nAlpha,0,000100000000,Toxic,PC\nBeta,0,000200000001,Frozen\nGamma,0,000300000002,Barren,PC,ignored-extra\n",
        );
        let mut model = GalaxyModel::new();
        let source = " NMSCE α ";
        let stats = import_csv(&mut model, csv.path(), source).unwrap();
        assert_eq!((stats.added, stats.duplicates, stats.skipped), (3, 0, 0));
        assert_eq!(model.system_count(), 3);
        assert!(model.systems.values().all(|system| {
            system.source
                == DiscoverySource::Community {
                    source: source.to_string(),
                }
        }));
        let original = model
            .system(&SystemId::from_address(&GalacticAddress::from_packed(
                0x000100000000,
                0,
            )))
            .unwrap()
            .clone();
        let later = csv_file(
            "System Name,Galaxy,Portal Glyphs,Biome\nRenamed by later catalog,0,000100000000,Lush\n",
        );
        let duplicate = import_csv(&mut model, later.path(), "Later Catalog").unwrap();
        assert_eq!(
            (duplicate.added, duplicate.duplicates, duplicate.skipped),
            (0, 1, 0)
        );
        assert_eq!(
            model.system(&SystemId::from_address(&GalacticAddress::from_packed(
                0x000100000000,
                0
            ))),
            Some(&original)
        );
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
    fn test_import_csv_promotes_only_explicit_placeholder_and_counts_one_addition() {
        let address = GalacticAddress::from_packed(0x0001_0000_0000, 0);
        let id = SystemId::from_address(&address);
        let mut model = GalaxyModel::new();
        let player = nms_core::player::PlayerState::new(
            GalacticAddress::new(4, 5, 6, 7, 8, 9),
            9,
            None,
            None,
            11,
            12,
            13,
        );
        model.player_state = Some(player.clone());
        model.set_active_galaxy(7);
        model.insert_player_placeholder(address);
        assert!(model.is_player_placeholder(&id));

        let first = csv_file(
            "System Name,Galaxy,Portal Glyphs,Biome\nPromoted,Euclid,000100000000,Toxic\n",
        );
        let first_stats = import_csv(&mut model, first.path(), "test").unwrap();
        assert_eq!(
            (
                first_stats.added,
                first_stats.duplicates,
                first_stats.skipped
            ),
            (1, 0, 0)
        );
        assert_eq!(model.system_count(), 1);
        assert!(!model.is_player_placeholder(&id));
        let mut expected = System::new(
            address,
            Some("Promoted".into()),
            None,
            None,
            vec![Planet::new(0, Some(Biome::Toxic), None, false, None, None)],
        );
        expected.source = DiscoverySource::Community {
            source: "test".to_string(),
        };
        assert_eq!(
            model.systems,
            std::collections::HashMap::from([(id, expected.clone())])
        );
        assert_eq!(
            model.planets,
            std::collections::HashMap::from([((id, 0), expected.planets[0].clone())])
        );
        assert_eq!(
            model.name_index,
            std::collections::HashMap::from([((0, "promoted".into()), id)])
        );
        assert_eq!(
            model.biome_index,
            std::collections::HashMap::from([(Biome::Toxic, vec![(id, 0)])])
        );
        assert_eq!(
            model.address_to_id,
            std::collections::HashMap::from([(id, id)])
        );
        assert_eq!(model.graph.node_count(), 1);
        assert_eq!(model.node_map.len(), 1);
        assert_eq!(model.graph.node_weight(model.node_map[&id]), Some(&id));
        assert_eq!(
            model
                .spatial_for(0)
                .unwrap()
                .iter()
                .map(|point| (point.id, point.point))
                .collect::<Vec<_>>(),
            vec![(id, [0.0, 0.0, 0.0])]
        );
        assert_eq!(model.player_state, Some(player.clone()));
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(model.system_count(), 1);

        let duplicate =
            csv_file("System Name,Galaxy,Portal Glyphs,Biome\nLater,Euclid,000100000000,Lush\n");
        let duplicate_stats = import_csv(&mut model, duplicate.path(), "test").unwrap();
        assert_eq!(
            (
                duplicate_stats.added,
                duplicate_stats.duplicates,
                duplicate_stats.skipped
            ),
            (0, 1, 0)
        );
        assert_eq!(
            model.systems,
            std::collections::HashMap::from([(id, expected.clone())])
        );
        assert_eq!(
            model.planets,
            std::collections::HashMap::from([((id, 0), expected.planets[0].clone())])
        );
        assert_eq!(
            model.name_index,
            std::collections::HashMap::from([((0, "promoted".into()), id)])
        );
        assert_eq!(
            model.biome_index,
            std::collections::HashMap::from([(Biome::Toxic, vec![(id, 0)])])
        );
        assert_eq!(
            model.address_to_id,
            std::collections::HashMap::from([(id, id)])
        );
        assert_eq!(model.player_state, Some(player.clone()));
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(model.system_count(), 1);

        let mut genuine_empty = GalaxyModel::new();
        let genuine_system = System::new(address, None, None, None, vec![]);
        genuine_empty.insert_system(genuine_system.clone());
        let genuine_player = nms_core::player::PlayerState::new(address, 0, None, None, 21, 22, 23);
        genuine_empty.player_state = Some(genuine_player.clone());
        genuine_empty.set_active_galaxy(9);
        let empty_row =
            csv_file("System Name,Galaxy,Portal Glyphs,Biome\nIgnored,Euclid,000100000000,Toxic\n");
        let empty_stats = import_csv(&mut genuine_empty, empty_row.path(), "test").unwrap();
        assert_eq!(
            (
                empty_stats.added,
                empty_stats.duplicates,
                empty_stats.skipped
            ),
            (0, 1, 0)
        );
        assert_eq!(
            genuine_empty.systems,
            std::collections::HashMap::from([(id, genuine_system)])
        );
        assert!(genuine_empty.planets.is_empty());
        assert!(genuine_empty.name_index.is_empty());
        assert!(genuine_empty.biome_index.is_empty());
        assert_eq!(
            genuine_empty.address_to_id,
            std::collections::HashMap::from([(id, id)])
        );
        assert_eq!(genuine_empty.graph.node_count(), 1);
        assert_eq!(
            genuine_empty.graph.node_weight(genuine_empty.node_map[&id]),
            Some(&id)
        );
        assert_eq!(
            genuine_empty
                .spatial_for(0)
                .unwrap()
                .iter()
                .map(|point| (point.id, point.point))
                .collect::<Vec<_>>(),
            vec![(id, [0.0, 0.0, 0.0])]
        );
        assert_eq!(genuine_empty.player_state, Some(genuine_player));
        assert_eq!(genuine_empty.active_galaxy, 9);
        assert_eq!(genuine_empty.system_count(), 1);
        assert!(!genuine_empty.is_player_placeholder(&id));
    }

    #[test]
    fn test_import_csv_same_coordinates_in_two_galaxies_is_order_independent() {
        for reverse in [false, true] {
            let rows = if reverse {
                "Nine,9,050003AB8C07,Toxic\nZero,0,050003AB8C07,Lush\n"
            } else {
                "Zero,0,050003AB8C07,Lush\nNine,9,050003AB8C07,Toxic\n"
            };
            let csv = format!("System Name,Galaxy,Portal Glyphs,Biome\n{rows}");
            let file = csv_file(&csv);
            let mut model = GalaxyModel::new();
            let stats = import_csv(&mut model, file.path(), "test").unwrap();
            assert_eq!((stats.added, stats.duplicates, stats.skipped), (2, 0, 0));
            assert_eq!(model.system_count(), 2);

            let replay = csv_file(&csv);
            let replay_stats = import_csv(&mut model, replay.path(), "test").unwrap();
            assert_eq!(
                (
                    replay_stats.added,
                    replay_stats.duplicates,
                    replay_stats.skipped
                ),
                (0, 2, 0)
            );
            let id0 = SystemId::from_address(&GalacticAddress::from_packed(0x050003AB8C07, 0));
            let id9 = SystemId::from_address(&GalacticAddress::from_packed(0x050003AB8C07, 9));
            assert_eq!(model.system(&id0).unwrap().name.as_deref(), Some("Zero"));
            assert_eq!(model.system(&id9).unwrap().name.as_deref(), Some("Nine"));
            assert_eq!(
                model.system(&id0).unwrap().planets[0].biome,
                Some(Biome::Lush)
            );
            assert_eq!(
                model.system(&id9).unwrap().planets[0].biome,
                Some(Biome::Toxic)
            );
        }
    }

    #[test]
    fn test_import_csv_different_planet_glyphs_same_galaxy_remain_duplicate() {
        let csv = "System Name,Galaxy,Portal Glyphs\nFirst,0,050003AB8C07\nSecond,0,F50003AB8C07\n";
        let file = csv_file(csv);
        let mut model = GalaxyModel::new();
        let stats = import_csv(&mut model, file.path(), "test").unwrap();
        assert_eq!((stats.added, stats.duplicates, stats.skipped), (1, 1, 0));
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
