# Milestone 2.8 -- `nms stats` Command (nms-cli)

Aggregate statistics: biome distribution, discovery counts, named vs unnamed breakdown.

## Crate: `nms-cli`

Path: `crates/nms-cli/`

### Dependencies

No new dependencies beyond those added in 2.6.

---

## New File: `crates/nms-cli/src/stats.rs`

```rust
//! `nms stats` command -- aggregate galaxy statistics.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::stats::{execute_stats, StatsQuery};
use nms_query::display::format_stats;

pub fn run(
    save: Option<PathBuf>,
    biomes: bool,
    discoveries: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?.path().to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    let model = GalaxyModel::from_save(&save);

    let query = StatsQuery {
        biomes: biomes || (!biomes && !discoveries), // default to all
        discoveries: discoveries || (!biomes && !discoveries),
    };

    let result = execute_stats(&model, &query);
    print!("{}", format_stats(&result));

    Ok(())
}
```

### Update `crates/nms-cli/src/main.rs`

Add the Stats subcommand:

```rust
mod stats;

// In the Commands enum:
/// Display aggregate galaxy statistics.
Stats {
    /// Path to save file (auto-detects if omitted).
    #[arg(long)]
    save: Option<PathBuf>,

    /// Show biome distribution table.
    #[arg(long)]
    biomes: bool,

    /// Show discovery counts by type.
    #[arg(long)]
    discoveries: bool,
},

// In the match:
Commands::Stats { save, biomes, discoveries } => {
    stats::run(save, biomes, discoveries)
}
```

---

## Complete `main.rs` After Phase 2

For reference, the full `main.rs` after milestones 2.6-2.8:

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod convert;
mod find;
mod info;
mod show;
mod stats;

#[derive(Parser)]
#[command(
    name = "nms",
    about = "NMS Copilot CLI -- search planets, plan routes, convert glyphs",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display save file summary.
    Info {
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Convert between NMS coordinate formats.
    Convert {
        #[arg(long, group = "input")]
        glyphs: Option<String>,
        #[arg(long, group = "input")]
        coords: Option<String>,
        #[arg(long, group = "input")]
        ga: Option<String>,
        #[arg(long, group = "input")]
        voxel: Option<String>,
        #[arg(long)]
        ssi: Option<u16>,
        #[arg(long, default_value = "0")]
        planet: u8,
        #[arg(long, default_value = "0")]
        galaxy: String,
    },

    /// Search planets by biome, distance, name.
    Find {
        #[arg(long)]
        save: Option<PathBuf>,
        #[arg(long)]
        biome: Option<String>,
        #[arg(long)]
        infested: bool,
        #[arg(long)]
        within: Option<f64>,
        #[arg(long)]
        nearest: Option<usize>,
        #[arg(long)]
        named: bool,
        #[arg(long)]
        discoverer: Option<String>,
        #[arg(long)]
        from: Option<String>,
    },

    /// Show detailed information about a system or base.
    Show {
        #[arg(long)]
        save: Option<PathBuf>,
        #[command(subcommand)]
        target: ShowTarget,
    },

    /// Display aggregate galaxy statistics.
    Stats {
        #[arg(long)]
        save: Option<PathBuf>,
        #[arg(long)]
        biomes: bool,
        #[arg(long)]
        discoveries: bool,
    },
}

#[derive(Subcommand)]
enum ShowTarget {
    /// Show system details.
    System { name: String },
    /// Show base details.
    Base { name: String },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { save } => info::run(save),
        Commands::Convert {
            glyphs, coords, ga, voxel, ssi, planet, galaxy,
        } => convert::run(glyphs, coords, ga, voxel, ssi, planet, galaxy),
        Commands::Find {
            save, biome, infested, within, nearest, named, discoverer, from,
        } => find::run(save, biome, infested, within, nearest, named, discoverer, from),
        Commands::Show { save, target } => {
            let t = match target {
                ShowTarget::System { name } => show::ShowTarget::System { name },
                ShowTarget::Base { name } => show::ShowTarget::Base { name },
            };
            show::run(save, t)
        }
        Commands::Stats { save, biomes, discoveries } => {
            stats::run(save, biomes, discoveries)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
```

---

## Tests

### File: `crates/nms-cli/tests/stats_integration.rs`

```rust
use nms_graph::GalaxyModel;
use nms_query::stats::{execute_stats, StatsQuery};
use nms_query::display::format_stats;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 5000000, "Nanites": 10000, "Specials": 500,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Base Alpha", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""},
                    {"BaseVersion": 8, "GalacticAddress": "0x0A0002001234", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Base Beta", "BaseType": {"PersistentBaseTypes": "ExternalPlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x250003AB8C07", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"A","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x0A0002001234", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"B","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x1A0002001234", "DT": "Planet", "VP": ["0xEF", 4]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"B","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn stats_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = StatsQuery { biomes: true, discoveries: true };
    let result = execute_stats(&model, &query);
    let output = format_stats(&result);

    assert_eq!(result.system_count, 2);
    assert_eq!(result.planet_count, 3);
    assert_eq!(result.base_count, 2);
    assert!(output.contains("Galaxy Statistics"));
    assert!(output.contains("Biome Distribution"));
}

#[test]
fn stats_biome_distribution_consistent() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_stats(&model, &StatsQuery::default());
    let counted: usize = result.biome_counts.values().sum::<usize>() + result.unknown_biome_count;
    assert_eq!(counted, result.planet_count);
}

#[test]
fn stats_base_count_matches_save() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_stats(&model, &StatsQuery::default());
    assert_eq!(result.base_count, 2);
}
```

---

## Implementation Notes

1. **Default behavior** -- if neither `--biomes` nor `--discoveries` is specified, show everything. The `run()` function handles this by defaulting both to true.

2. **Stats are cheap** -- iterating over a few hundred systems/planets takes microseconds. No optimization needed.

3. **Biome distribution sorted by count** -- the display layer sorts biomes descending by count, making the most common biomes appear first.

4. **Future extensions** -- Phase 7 can add `--distances` (distance histogram), `--format csv` (machine-readable output), and ANSI color for biome names.

5. **This milestone completes Phase 2** -- after 2.8, the CLI has 5 working commands: `info`, `convert`, `find`, `show`, `stats`. The galaxy model, spatial index, query engine, and display layer are all functional.
