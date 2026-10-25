//! Export filtered planet matches from a checked save as JSON or CSV.
//!
//! The handler exposes the `find` filters and does not insert a player-system record. Flat
//! records use the stored system address and galaxy, including stored planet-index bits. JSON
//! ends with a newline; empty results produce `[]` plus newline in JSON and zero bytes in CSV,
//! whose header appears with the first row.
//!
//! # Examples
//!
//! ```text
//! nms export --save data/test/save56_multi_system.json --format json --biome Lush
//! nms export --save data/test/save56_multi_system.json --format csv --biome Lush
//! ```
//!
//! These synthetic-fixture commands are illustrative text, not executed here.

use std::io::{self, Write};
use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_query::display::hex_to_emoji;
use nms_query::find::{FindQuery, FindResult, ReferencePoint, execute_find};

/// Filters, reference, and format accepted by the export handler.
pub struct ExportArgs {
    /// Optional checked-save path; absent resolves most-recent directly; CLI may pass a slot-resolved path.
    pub save: Option<PathBuf>,
    /// Optional biome filter.
    pub biome: Option<String>,
    /// Whether to require infested planets.
    pub infested: bool,
    /// Optional maximum reference distance in light-years.
    pub within: Option<f64>,
    /// Optional result limit after filtering.
    pub nearest: Option<usize>,
    /// Whether a planet name must be present.
    pub named: bool,
    /// Optional substring matched against the system discoverer.
    pub discoverer: Option<String>,
    /// Optional base reference; absent uses current position.
    pub from: Option<String>,
    /// Output format: `json` or `csv`, case-insensitively.
    pub format: String,
}

/// Supported flat-record serialization formats.
///
/// # Examples
///
/// ```text
/// nms export --save data/test/save56_minimal.json --format json
/// nms export --save data/test/save56_minimal.json --format csv
/// ```
///
/// These are command-text examples, not Rust doctests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExportFormat {
    /// Pretty JSON array with a trailing newline.
    #[default]
    Json,
    /// CSV records; a header is written when a row is serialized.
    Csv,
}

impl ExportFormat {
    /// Parse `json` or `csv` after lowercasing, without trimming whitespace.
    ///
    /// # Errors
    ///
    /// Returns an error containing the lowercased input when the format is unsupported.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            other => Err(format!("unknown format: {other} (expected json or csv)")),
        }
    }
}

/// Flat fields for one exported planet match.
///
/// Missing names, biome, or discoverer become empty strings; address-derived fields use the
/// stored system address.
#[derive(Debug, serde::Serialize)]
pub struct ExportRecord {
    /// Planet name, or empty when absent.
    pub planet_name: String,
    /// Biome label, or empty when unknown.
    pub biome: String,
    /// System name, or empty when absent.
    pub system_name: String,
    /// Distance from the selected reference in light-years.
    pub distance_ly: f64,
    /// Twelve-digit uppercase hex from the stored system address.
    pub portal_glyphs: String,
    /// Emoji rendered from `portal_glyphs`.
    pub portal_emoji: String,
    /// Decoded voxel X from the stored address.
    pub coords_x: i16,
    /// Decoded voxel Y from the stored address.
    pub coords_y: i8,
    /// Decoded voxel Z from the stored address.
    pub coords_z: i16,
    /// Display name for the stored address galaxy.
    pub galaxy: String,
    /// System discoverer name, or empty when absent.
    pub discoverer: String,
    /// Whether the matched planet is infested.
    pub infested: bool,
}

impl From<&FindResult> for ExportRecord {
    fn from(r: &FindResult) -> Self {
        Self {
            planet_name: r.planet.name.clone().unwrap_or_default(),
            biome: r.planet.biome.map(|b| b.to_string()).unwrap_or_default(),
            system_name: r.system.name.clone().unwrap_or_default(),
            distance_ly: r.distance_ly,
            portal_glyphs: r.portal_hex.clone(),
            portal_emoji: hex_to_emoji(&r.portal_hex),
            coords_x: r.system.address.voxel_x(),
            coords_y: r.system.address.voxel_y(),
            coords_z: r.system.address.voxel_z(),
            galaxy: Galaxy::by_index(r.system.address.reality_index)
                .name
                .to_string(),
            discoverer: r.system.discoverer.clone().unwrap_or_default(),
            infested: r.planet.infested,
        }
    }
}

/// Filter a checked-save model and write matching records in the selected format.
///
/// Format parsing precedes this handler's save resolution; main may resolve `--slot` first and
/// pass a path. Direct calls use an explicit path or most-recent discovery and have no slot.
/// Filters are biome, infested, distance, nearest count, named-only, system discoverer substring,
/// and base or current-position reference. False `infested` means no filter; `named` accepts any
/// present name, including empty. Records retain stored system address and galaxy. JSON writes a
/// pretty array plus newline, including `[]` when empty. Empty CSV is zero bytes; its header
/// appears with the first row. Output uses locked stdout; serialization, write, and flush errors
/// return, and a failure may leave partial output without rollback.
///
/// # Examples
///
/// ```text
/// nms export --save data/test/save56_multi_system.json --format json --biome Lush
/// nms export --save data/test/save56_multi_system.json --format csv --biome Lush
/// ```
///
/// # Errors
///
/// Returns format, save, validation, biome, query, serialization, CSV, or stdout errors.
pub fn run(args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let format = ExportFormat::parse(&args.format)?;

    let path = crate::resolve_save(args.save)?;
    let save = nms_save::parse_validated_save_file(&path)?;
    let model = nms_graph::GalaxyModel::from_validated_save(&save);

    let biome = args
        .biome
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| format!("Invalid biome: {e}"))?;

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

    let stdout = io::stdout();
    let mut out = stdout.lock();

    match format {
        ExportFormat::Json => write_json(&mut out, &results)?,
        ExportFormat::Csv => write_csv(&mut out, &results)?,
    }

    Ok(())
}

fn write_json(
    out: &mut impl Write,
    results: &[FindResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let records: Vec<ExportRecord> = results.iter().map(ExportRecord::from).collect();
    serde_json::to_writer_pretty(&mut *out, &records)?;
    writeln!(out)?;
    Ok(())
}

fn write_csv(
    out: &mut impl Write,
    results: &[FindResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(&mut *out);
    for result in results {
        wtr.serialize(ExportRecord::from(result))?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};
    use std::io;

    fn test_find_result() -> FindResult {
        let addr = GalacticAddress::new(100, 50, -200, 0x123, 0, 0);
        FindResult {
            planet: Planet::new(0, Some(Biome::Lush), None, false, Some("Eden".into()), None),
            system: System::new(
                addr,
                Some("Sol".into()),
                Some("Explorer".into()),
                None,
                vec![],
            ),
            distance_ly: 42_000.0,
            portal_hex: format!("{:012X}", addr.packed()),
        }
    }

    #[test]
    fn test_export_format_parse_json() {
        assert_eq!(ExportFormat::parse("json").unwrap(), ExportFormat::Json);
        assert_eq!(ExportFormat::parse("JSON").unwrap(), ExportFormat::Json);
    }

    #[test]
    fn test_export_format_parse_csv() {
        assert_eq!(ExportFormat::parse("csv").unwrap(), ExportFormat::Csv);
        assert_eq!(ExportFormat::parse("CSV").unwrap(), ExportFormat::Csv);
    }

    #[test]
    fn test_export_format_parse_unknown() {
        assert!(ExportFormat::parse("xml").is_err());
    }

    #[test]
    fn test_write_json_empty() {
        let mut buf = Vec::new();
        write_json(&mut buf, &[]).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s.trim(), "[]");
    }

    #[test]
    fn test_write_json_with_data() {
        let results = vec![test_find_result()];
        let mut buf = Vec::new();
        write_json(&mut buf, &results).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("Eden"));
        assert!(s.contains("Lush"));
        assert!(s.contains("Sol"));
        assert!(s.contains("Explorer"));
        assert!(s.contains("Euclid"));
    }

    #[test]
    fn test_write_csv_empty() {
        let mut buf = Vec::new();
        write_csv(&mut buf, &[]).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn test_write_csv_with_data() {
        let results = vec![test_find_result()];
        let mut buf = Vec::new();
        write_csv(&mut buf, &results).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("planet_name"));
        assert!(s.contains("Eden"));
        assert!(s.contains("Lush"));
        assert!(s.contains("Sol"));
    }

    #[test]
    fn test_export_record_from_find_result() {
        let result = test_find_result();
        let record = ExportRecord::from(&result);
        assert_eq!(record.planet_name, "Eden");
        assert_eq!(record.biome, "Lush");
        assert_eq!(record.system_name, "Sol");
        assert_eq!(record.distance_ly, 42_000.0);
        assert!(!record.portal_glyphs.is_empty());
        assert!(!record.portal_emoji.is_empty());
        assert_eq!(record.galaxy, "Euclid");
        assert_eq!(record.discoverer, "Explorer");
        assert!(!record.infested);
    }

    #[test]
    fn test_export_record_missing_names() {
        let addr = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let result = FindResult {
            planet: Planet::new(0, None, None, true, None, None),
            system: System::new(addr, None, None, None, vec![]),
            distance_ly: 0.0,
            portal_hex: "000000000001".into(),
        };
        let record = ExportRecord::from(&result);
        assert!(record.planet_name.is_empty());
        assert!(record.system_name.is_empty());
        assert!(record.discoverer.is_empty());
        assert!(record.infested);
    }

    #[test]
    fn test_json_output_is_valid_json() {
        let results = vec![test_find_result()];
        let mut buf = Vec::new();
        write_json(&mut buf, &results).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_csv_output_has_header_and_row() {
        let results = vec![test_find_result()];
        let mut buf = Vec::new();
        write_csv(&mut buf, &results).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 row
        assert!(lines[0].starts_with("planet_name,"));
    }

    struct BrokenPipeWriter {
        writes: usize,
    }

    impl Write for BrokenPipeWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "closed test pipe",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn assert_broken_pipe(result: Result<(), Box<dyn std::error::Error>>, writes: usize) {
        assert!(writes > 0, "writer was never called");
        let error = result.unwrap_err();
        let kind = if let Some(json_error) = error.downcast_ref::<serde_json::Error>() {
            json_error.io_error_kind()
        } else if let Some(io_error) = error.downcast_ref::<io::Error>() {
            Some(io_error.kind())
        } else if let Some(csv_error) = error.downcast_ref::<csv::Error>() {
            match csv_error.kind() {
                csv::ErrorKind::Io(io_error) => Some(io_error.kind()),
                _ => None,
            }
        } else {
            None
        };
        assert_eq!(kind, Some(io::ErrorKind::BrokenPipe), "{error:#}");
        assert!(
            format!("{error:#}").contains("closed test pipe"),
            "{error:#}"
        );
    }

    #[test]
    fn test_write_json_propagates_broken_pipe() {
        let mut writer = BrokenPipeWriter { writes: 0 };
        let result = write_json(&mut writer, &[test_find_result()]);
        assert_broken_pipe(result, writer.writes);
    }

    #[test]
    fn test_write_csv_propagates_broken_pipe() {
        let mut writer = BrokenPipeWriter { writes: 0 };
        let result = write_csv(&mut writer, &[test_find_result()]);
        assert_broken_pipe(result, writer.writes);
    }
}
