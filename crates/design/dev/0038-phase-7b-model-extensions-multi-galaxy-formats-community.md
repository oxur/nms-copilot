# Phase 7B -- Model Extensions: Multi-Galaxy, Formats, Community Data

Milestones 7.4, 7.5, 7.9: Per-galaxy spatial indexes, NomNom format support, and community data import.

**Depends on:** Phases 1-4 (save parsing, galaxy model, query engine).

---

## Architecture Overview

These three milestones extend the galaxy model's scope:

1. **Multi-galaxy** (7.4) -- Separate spatial index per galaxy (reality_index 0-255)
2. **NomNom format** (7.5) -- Parse NomNom JSON export in nms-compat
3. **Community data import** (7.9) -- Import NMSCE/community CSV into the model as "external" discoveries

All three feed into `GalaxyModel` -- the downstream query engine, CLI, REPL, and MCP work unchanged.

---

## Milestone 7.4: Multi-Galaxy Support

### Goal

Currently `GalaxyModel` has a single `spatial: RTree<SystemPoint>` for all systems regardless of galaxy. Distance queries between galaxies produce meaningless results. Fix this with per-galaxy spatial indexes and cross-galaxy awareness.

### Current State

- `GalacticAddress` already stores `reality_index` (galaxy 0-255)
- `Galaxy::by_index(u8)` covers all 256 galaxies
- `GalaxyModel` uses one flat R-tree and one flat graph
- All queries assume a single galaxy

### Modified: `crates/nms-graph/src/model.rs`

Replace the single spatial index with a per-galaxy map:

```rust
use std::collections::HashMap;
use rstar::RTree;

use crate::spatial::SystemPoint;

#[derive(Debug)]
pub struct GalaxyModel {
    /// Graph topology: nodes are systems, edge weights are distance in ly.
    pub graph: StableGraph<SystemId, f64, Undirected>,

    /// 3D spatial index per galaxy (reality_index -> R-tree).
    /// Only populated galaxies have entries.
    pub spatial: HashMap<u8, RTree<SystemPoint>>,

    /// The currently active galaxy for queries (default: player's galaxy).
    pub active_galaxy: u8,

    // ... rest unchanged ...
}
```

### Spatial Query Changes

Update `nearest()` and `within_radius()` to scope to the active galaxy:

```rust
impl GalaxyModel {
    /// Get the spatial index for the active galaxy.
    pub fn active_spatial(&self) -> Option<&RTree<SystemPoint>> {
        self.spatial.get(&self.active_galaxy)
    }

    /// Get the spatial index for a specific galaxy.
    pub fn spatial_for(&self, galaxy: u8) -> Option<&RTree<SystemPoint>> {
        self.spatial.get(&galaxy)
    }

    /// Switch the active galaxy.
    pub fn set_active_galaxy(&mut self, galaxy: u8) {
        self.active_galaxy = galaxy;
    }

    /// List galaxies that have discovered systems.
    pub fn discovered_galaxies(&self) -> Vec<u8> {
        let mut galaxies: Vec<u8> = self.spatial.keys().copied().collect();
        galaxies.sort();
        galaxies
    }
}
```

### Model Construction

Update `from_save()` to partition systems by galaxy:

```rust
pub fn from_save(save: &SaveRoot) -> Self {
    let mut model = Self::new();
    let systems = extract_systems(save);

    for system in systems {
        let galaxy = system.address.reality_index();
        model.insert_system(system);
    }

    // Build per-galaxy spatial indexes
    let mut galaxy_points: HashMap<u8, Vec<SystemPoint>> = HashMap::new();
    for (&id, system) in &model.systems {
        let galaxy = system.address.reality_index();
        galaxy_points.entry(galaxy).or_default().push(
            SystemPoint::new(id, system.address.galactic_coords())
        );
    }
    for (galaxy, points) in galaxy_points {
        model.spatial.insert(galaxy, RTree::bulk_load(points));
    }

    // Set active galaxy from player position
    if let Some(state) = &model.player_state {
        model.active_galaxy = state.address.reality_index();
    }

    model
}
```

### Query Engine Updates

`nms-query` functions that use the spatial index need the galaxy context:

```rust
// In execute_find:
let spatial = model.spatial_for(model.active_galaxy)
    .ok_or(GraphError::NoData("no systems in active galaxy".into()))?;
```

### REPL Galaxy Switching

Add a `galaxy` command to nms-copilot REPL:

```
nms> galaxy
Active galaxy: Euclid (0)
Discovered galaxies: Euclid (0), Hilbert (1), Eissentam (9)

nms> galaxy Eissentam
Switched to Eissentam (galaxy 9). 47 systems loaded.
```

### Tests

```rust
#[test]
fn test_multi_galaxy_separate_spatial() {
    let mut model = GalaxyModel::new();
    // Add system in Euclid (galaxy 0)
    let euclid_sys = test_system_in_galaxy(0);
    model.insert_system(euclid_sys);
    // Add system in Hilbert (galaxy 1)
    let hilbert_sys = test_system_in_galaxy(1);
    model.insert_system(hilbert_sys);

    model.rebuild_spatial();

    assert_eq!(model.discovered_galaxies(), vec![0, 1]);
    assert_eq!(model.spatial_for(0).unwrap().size(), 1);
    assert_eq!(model.spatial_for(1).unwrap().size(), 1);
    assert!(model.spatial_for(2).is_none());
}

#[test]
fn test_active_galaxy_default() {
    let model = GalaxyModel::new();
    assert_eq!(model.active_galaxy, 0); // Euclid
}

#[test]
fn test_set_active_galaxy() {
    let mut model = GalaxyModel::new();
    model.set_active_galaxy(9);
    assert_eq!(model.active_galaxy, 9);
}

#[test]
fn test_nearest_scoped_to_active_galaxy() {
    let mut model = test_multi_galaxy_model();
    model.set_active_galaxy(0);
    let nearby = model.nearest_systems(5);
    // Should only return Euclid systems
    for sys in &nearby {
        assert_eq!(model.systems[&sys.id].address.reality_index(), 0);
    }
}
```

---

## Milestone 7.5: NomNom Format Support

### Goal

Parse NomNom's export format (JSON with slightly different key names/structure) in `nms-compat`, producing the same `SaveRoot` that the standard parser outputs.

### Current State

`nms-compat` is a placeholder crate (`src/lib.rs` has only module docs). It already has a goatfungus JSON fixer concept.

### Architecture

```
NomNom JSON  -->  nms-compat::nomnom::parse()  -->  SaveRoot
                  (key mapping + structure normalization)
```

### New File: `crates/nms-compat/src/nomnom.rs`

```rust
//! Parser for NomNom JSON export format.
//!
//! NomNom uses mostly-deobfuscated keys but with some differences
//! from the standard format. This module normalizes the structure.

use nms_save::model::SaveRoot;

/// Detect whether a JSON string is likely NomNom format.
pub fn is_nomnom_format(json: &str) -> bool {
    // NomNom exports typically have a "Version" key at top level
    // and use "PlayerStateData" instead of the obfuscated key
    json.contains("\"Version\"") && json.contains("\"PlayerStateData\"")
}

/// Parse a NomNom JSON export into a SaveRoot.
pub fn parse_nomnom(json: &str) -> Result<SaveRoot, NomNomError> {
    // NomNom's format is close to deobfuscated standard JSON
    // Key differences:
    // 1. Some nested keys have different casing
    // 2. Coordinates may use decimal instead of hex
    // 3. Some fields are absent or renamed
    //
    // Strategy: normalize keys, then delegate to nms-save's deserializer
    let normalized = normalize_keys(json)?;
    let save: SaveRoot = serde_json::from_str(&normalized)?;
    Ok(save)
}

#[derive(Debug, thiserror::Error)]
pub enum NomNomError {
    #[error("not a NomNom format file")]
    NotNomNom,
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("key normalization failed: {0}")]
    Normalization(String),
}
```

### Integration with Save Pipeline

Update `nms-save` or add to `nms-compat` a unified loader:

```rust
/// Try to parse a file as any supported format.
pub fn parse_any(path: &Path) -> Result<SaveRoot, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    // Try NomNom format
    if nomnom::is_nomnom_format(&content) {
        return Ok(nomnom::parse_nomnom(&content)?);
    }

    // Try goatfungus format (has \xNN escapes)
    if content.contains("\\x") {
        let fixed = goatfungus::fix_json(&content);
        return Ok(serde_json::from_str(&fixed)?);
    }

    // Standard deobfuscated JSON
    Ok(serde_json::from_str(&content)?)
}
```

### Tests

```rust
#[test]
fn test_is_nomnom_format_positive() {
    let json = r#"{"Version": 2002, "PlayerStateData": {}}"#;
    assert!(is_nomnom_format(json));
}

#[test]
fn test_is_nomnom_format_negative() {
    let json = r#"{"F2P": "data"}"#;
    assert!(!is_nomnom_format(json));
}

#[test]
fn test_parse_nomnom_minimal() {
    let json = include_str!("../test_data/nomnom_minimal.json");
    let save = parse_nomnom(json).unwrap();
    assert!(save.discovery_manager.is_some());
}
```

### Implementation Notes

NomNom format specifics will need to be verified against actual NomNom exports. The design here provides the skeleton -- the key mapping table (`normalize_keys`) will be filled in during implementation when we have reference files. If NomNom's format turns out to be identical to deobfuscated standard JSON, this milestone simplifies to a format detection check.

---

## Milestone 7.9: Community Data Import

### Goal

Import external coordinate lists (NMSCE spreadsheets, community CSVs) into the galaxy model as "external" discoveries, enriching the queryable dataset beyond personal save data.

### Data Source

The No Man's Sky Coordinate Exchange (NMSCE) provides community-submitted coordinates. Common format:

```csv
System Name,Galaxy,Portal Glyphs,Biome,Platform
Paradise Planet,Euclid,01717D8A4EA2,Lush,PC
Toxic World,Eissentam,0234AB12CD56,Toxic,PS5
```

### Source Tag

Tag imported data so it can be distinguished from personal discoveries:

```rust
// In nms-core::system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverySource {
    /// From the player's save file.
    Personal,
    /// From a community data import.
    Community { source: String },
}

// Add to System struct:
pub struct System {
    // ... existing fields ...
    pub source: DiscoverySource,
}
```

### Import Module: `crates/nms-graph/src/import.rs`

```rust
use std::path::Path;
use csv::ReaderBuilder;
use nms_core::address::GalacticAddress;
use nms_core::glyph::PortalAddress;

use crate::GalaxyModel;

/// Import a community CSV file into the galaxy model.
///
/// Expected columns: System Name, Galaxy, Portal Glyphs, Biome, Platform
/// Additional columns are ignored.
pub fn import_csv(
    model: &mut GalaxyModel,
    path: &Path,
    source_name: &str,
) -> Result<ImportStats, ImportError> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let mut stats = ImportStats::default();

    for result in reader.deserialize::<CommunityRecord>() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                stats.skipped += 1;
                log::debug!("Skipped row: {e}");
                continue;
            }
        };

        match import_record(model, &record, source_name) {
            Ok(true) => stats.added += 1,
            Ok(false) => stats.duplicates += 1,
            Err(e) => {
                stats.skipped += 1;
                log::debug!("Failed to import: {e}");
            }
        }
    }

    Ok(stats)
}

#[derive(Debug, Default)]
pub struct ImportStats {
    pub added: usize,
    pub duplicates: usize,
    pub skipped: usize,
}

#[derive(Debug, serde::Deserialize)]
struct CommunityRecord {
    #[serde(rename = "System Name")]
    system_name: String,
    #[serde(rename = "Galaxy")]
    galaxy: String,
    #[serde(rename = "Portal Glyphs")]
    portal_glyphs: String,
    #[serde(rename = "Biome")]
    biome: Option<String>,
    #[serde(rename = "Platform")]
    platform: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("invalid portal glyphs: {0}")]
    InvalidGlyphs(String),
}
```

### CLI Command: `nms import`

```rust
/// Import community coordinate data.
Import {
    /// Path to CSV file.
    file: PathBuf,

    /// Path to save file (auto-detects if omitted).
    #[arg(long)]
    save: Option<PathBuf>,

    /// Source name for imported data (e.g., "NMSCE").
    #[arg(long, default_value = "community")]
    source: String,
},
```

```
$ nms import nmsce_lush_planets.csv --source NMSCE
Imported 1,247 systems (42 duplicates, 3 skipped)
```

### Tests

```rust
#[test]
fn test_import_csv_basic() {
    let csv = "System Name,Galaxy,Portal Glyphs,Biome,Platform\n\
               Test System,Euclid,01717D8A4EA2,Lush,PC\n";
    let tmp = write_temp_csv(csv);
    let mut model = GalaxyModel::new();
    let stats = import_csv(&mut model, tmp.path(), "test").unwrap();
    assert_eq!(stats.added, 1);
    assert_eq!(stats.duplicates, 0);
}

#[test]
fn test_import_csv_duplicate_skipped() {
    let csv = "System Name,Galaxy,Portal Glyphs,Biome,Platform\n\
               Test System,Euclid,01717D8A4EA2,Lush,PC\n\
               Test System,Euclid,01717D8A4EA2,Lush,PC\n";
    let tmp = write_temp_csv(csv);
    let mut model = GalaxyModel::new();
    let stats = import_csv(&mut model, tmp.path(), "test").unwrap();
    assert_eq!(stats.added, 1);
    assert_eq!(stats.duplicates, 1);
}

#[test]
fn test_import_csv_bad_row_skipped() {
    let csv = "System Name,Galaxy,Portal Glyphs,Biome,Platform\n\
               Test System,Euclid,01717D8A4EA2,Lush,PC\n\
               ,,invalid,,\n";
    let tmp = write_temp_csv(csv);
    let mut model = GalaxyModel::new();
    let stats = import_csv(&mut model, tmp.path(), "test").unwrap();
    assert_eq!(stats.added, 1);
    assert_eq!(stats.skipped, 1);
}

#[test]
fn test_imported_systems_tagged_community() {
    let csv = "System Name,Galaxy,Portal Glyphs,Biome,Platform\n\
               Test System,Euclid,01717D8A4EA2,Lush,PC\n";
    let tmp = write_temp_csv(csv);
    let mut model = GalaxyModel::new();
    import_csv(&mut model, tmp.path(), "NMSCE").unwrap();

    let system = model.systems.values().next().unwrap();
    assert!(matches!(system.source, DiscoverySource::Community { .. }));
}
```

---

## Implementation Notes

### Multi-Galaxy Impact on Routing

Route planning must stay within a single galaxy -- cross-galaxy warps aren't possible in NMS. The `execute_route` function should validate that all targets are in the same galaxy and return an error otherwise.

### Backward Compatibility

The `spatial` field type change from `RTree<SystemPoint>` to `HashMap<u8, RTree<SystemPoint>>` is a breaking internal change. All direct references to `model.spatial` must be updated. Since this is internal to the workspace, this is fine -- no public API stability concern.

### Community Import Persistence

Imported community data lives only in the in-memory model for the current session. For persistence across sessions, milestone 3.6 (rkyv cache) could be extended to include imported data. This is deferred -- the import command re-runs quickly enough for the expected data sizes.
