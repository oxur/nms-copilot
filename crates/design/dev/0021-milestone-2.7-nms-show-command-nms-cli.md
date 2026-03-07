# Milestone 2.7 -- `nms show` Command (nms-cli)

Detail views for systems and bases. `nms show system <name>` and `nms show base <name>`.

## Crate: `nms-cli`

Path: `crates/nms-cli/`

### Dependencies

No new dependencies beyond those added in 2.6.

---

## New File: `crates/nms-cli/src/show.rs`

```rust
//! `nms show` command -- detail views for systems and bases.

use std::path::PathBuf;

use nms_graph::GalaxyModel;
use nms_query::show::{execute_show, ShowQuery};
use nms_query::display::format_show_result;

pub fn run(
    save: Option<PathBuf>,
    target: ShowTarget,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?.path().to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    let model = GalaxyModel::from_save(&save);

    let query = match target {
        ShowTarget::System { name } => ShowQuery::System(name),
        ShowTarget::Base { name } => ShowQuery::Base(name),
    };

    let result = execute_show(&model, &query)?;
    print!("{}", format_show_result(&result));

    Ok(())
}

/// What to show -- parsed from CLI subcommand.
pub enum ShowTarget {
    System { name: String },
    Base { name: String },
}
```

### Update `crates/nms-cli/src/main.rs`

Add the Show subcommand:

```rust
mod show;

// In the Commands enum:
/// Show detailed information about a system or base.
Show {
    /// Path to save file (auto-detects if omitted).
    #[arg(long)]
    save: Option<PathBuf>,

    #[command(subcommand)]
    target: ShowTarget,
},

#[derive(Subcommand)]
enum ShowTarget {
    /// Show system details.
    System {
        /// System name or hex address.
        name: String,
    },
    /// Show base details.
    Base {
        /// Base name (case-insensitive).
        name: String,
    },
}

// In the match:
Commands::Show { save, target } => {
    let target = match target {
        ShowTarget::System { name } => show::ShowTarget::System { name },
        ShowTarget::Base { name } => show::ShowTarget::Base { name },
    };
    show::run(save, target)
}
```

---

## Tests

### File: `crates/nms-cli/tests/show_integration.rs`

```rust
use nms_graph::GalaxyModel;
use nms_query::show::{execute_show, ShowQuery, ShowResult};
use nms_query::display::format_show_result;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [100.0,200.0,300.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 1700000000, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"123","USN":"TestUser","PTK":"ST","TS":0}, "Name": "Sealab 2038", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":1700000000}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn show_base_end_to_end() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let result = execute_show(&model, &ShowQuery::Base("Sealab 2038".into())).unwrap();
    let output = format_show_result(&result);

    assert!(output.contains("Sealab 2038"));
    assert!(output.contains("HomePlanetBase"));
    assert!(output.contains("Euclid"));
}

#[test]
fn show_base_case_insensitive() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::Base("sealab 2038".into())).is_ok());
}

#[test]
fn show_system_with_planets() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    // Find the system ID from discoveries and look it up by hex
    let sys_id = model.systems.keys().next().unwrap();
    let hex = format!("{:012X}", sys_id.0);
    let result = execute_show(&model, &ShowQuery::System(hex)).unwrap();

    match result {
        ShowResult::System(s) => {
            assert!(s.system.planets.len() >= 1);
            let output = format_show_result(&ShowResult::System(s.clone()));
            assert!(output.contains("System Detail"));
        }
        _ => panic!("Expected system result"),
    }
}

#[test]
fn show_nonexistent_base_errors() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::Base("No Such Base".into())).is_err());
}

#[test]
fn show_nonexistent_system_errors() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    assert!(execute_show(&model, &ShowQuery::System("NoSystem".into())).is_err());
}
```

---

## Implementation Notes

1. **Nested subcommand** -- `nms show system "Sol"` and `nms show base "My Base"` use clap's nested `#[command(subcommand)]`. The outer Show command holds `--save`, and the inner target holds the name.

2. **System lookup** -- tries name first (case-insensitive), then hex address. This lets users do `nms show system 050003AB8C07` directly.

3. **Base lookup** -- case-insensitive substring is NOT used here (unlike find's discoverer filter). Show requires an exact name match (case-insensitive).

4. **Distance from player** -- if a player position is available, the output includes "Distance: X ly" from the current position. Helpful for gauging how far away something is.

5. **System planet list** -- the show system view lists all discovered planets with their biome and infested flag.
