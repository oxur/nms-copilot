# Milestone 2.6 -- `nms find` Command (nms-cli)

Search planets by biome, distance, name, discoverer. Sorted results with emoji portal glyphs.

## Crate: `nms-cli`

Path: `crates/nms-cli/`

### Dependencies to add to `crates/nms-cli/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-graph = { workspace = true }
nms-query = { workspace = true }
clap = { version = "4", features = ["derive"] }
```

---

## New File: `crates/nms-cli/src/find.rs`

```rust
//! `nms find` command -- search planets by biome, distance, name, discoverer.

use std::path::PathBuf;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::find::{execute_find, FindQuery, ReferencePoint};
use nms_query::display::format_find_results;

pub fn run(
    save: Option<PathBuf>,
    biome: Option<String>,
    infested: bool,
    within: Option<f64>,
    nearest: Option<usize>,
    named: bool,
    discoverer: Option<String>,
    from: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load save file
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?.path().to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;

    // Build model
    let model = GalaxyModel::from_save(&save);

    // Parse biome filter
    let biome = biome
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| format!("Invalid biome: {e}"))?;

    // Resolve reference point
    let reference = match from {
        Some(name) => ReferencePoint::Base(name),
        None => ReferencePoint::CurrentPosition,
    };

    let query = FindQuery {
        biome,
        infested: if infested { Some(true) } else { None },
        within_ly: within,
        nearest,
        name_pattern: None,
        discoverer,
        named_only: named,
        from: reference,
    };

    let results = execute_find(&model, &query)?;
    print!("{}", format_find_results(&results));

    Ok(())
}
```

### Update `crates/nms-cli/src/main.rs`

Add the Find subcommand and module:

```rust
mod find;

// In the Commands enum:
/// Search planets by biome, distance, name.
Find {
    /// Path to save file (auto-detects if omitted).
    #[arg(long)]
    save: Option<PathBuf>,

    /// Filter by biome (e.g., Lush, Toxic, Scorched).
    #[arg(long)]
    biome: Option<String>,

    /// Only show infested planets.
    #[arg(long)]
    infested: bool,

    /// Only within this radius in light-years.
    #[arg(long)]
    within: Option<f64>,

    /// Show only the N nearest results.
    #[arg(long)]
    nearest: Option<usize>,

    /// Only show named planets/systems.
    #[arg(long)]
    named: bool,

    /// Filter by discoverer username (substring match).
    #[arg(long)]
    discoverer: Option<String>,

    /// Distance from this base name (default: current position).
    #[arg(long)]
    from: Option<String>,
},

// In the match:
Commands::Find { save, biome, infested, within, nearest, named, discoverer, from } => {
    find::run(save, biome, infested, within, nearest, named, discoverer, from)
}
```

---

## Tests

### File: `crates/nms-cli/src/find.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    // Integration tests for the find command are in tests/find_integration.rs
    // because they need a full save JSON to test the pipeline end-to-end.
    // Unit tests for the query logic are in nms-query.
}
```

### File: `crates/nms-cli/tests/find_integration.rs`

```rust
use nms_graph::GalaxyModel;
use nms_query::find::{execute_find, FindQuery, ReferencePoint};
use nms_query::display::format_find_results;
use nms_core::biome::Biome;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Home Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Traveler","PTK":"ST","TS":1700000000}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn find_pipeline_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery::default();
    let results = execute_find(&model, &query).unwrap();
    let output = format_find_results(&results);

    assert!(!results.is_empty());
    assert!(output.contains("Portal Glyphs"));
}

#[test]
fn find_with_biome_filter() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        biome: Some(Biome::Lush),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    for r in &results {
        assert_eq!(r.planet.biome, Some(Biome::Lush));
    }
}

#[test]
fn find_from_base_reference() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        from: ReferencePoint::Base("Home Base".into()),
        nearest: Some(1),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 1);
}

#[test]
fn find_nearest_respects_limit() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        nearest: Some(1),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(results.len() <= 1);
}
```

---

## Implementation Notes

1. **Pipeline:** load save -> build GalaxyModel -> execute FindQuery -> format results -> print. This is the same pipeline all three CLI commands use.

2. **`--from` base name** -- if specified, distances are measured from the named base. Otherwise, from the player's current position in the save file.

3. **`--infested` is a flag, not Option<bool>** -- when present, filters to infested planets. When absent, shows all. There's no "not infested" filter (add `--no-infested` if needed later).

4. **`--biome` accepts the Biome::FromStr** -- case-insensitive, e.g., "lush", "LUSH", "Lush" all work. Invalid biome names produce a clear error.

5. **No `--name` flag in this milestone** -- name pattern search is defined in the query type but not exposed as a CLI flag yet. Can be added easily.
