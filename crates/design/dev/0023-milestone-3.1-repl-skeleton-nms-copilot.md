# Milestone 3.1 -- REPL Skeleton (nms-copilot)

Interactive REPL loop using reedline with command parsing via clap subcommands. Supports exit/quit/help and dispatches to the same query engine as the CLI.

## Crate: `nms-copilot`

Path: `crates/nms-copilot/`

### Dependencies to add to `crates/nms-copilot/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-graph = { workspace = true }
nms-query = { workspace = true }
# nms-watch and nms-cache will be added in later milestones
reedline = "0.39"
clap = { version = "4", features = ["derive"] }

[dev-dependencies]
serde_json = { workspace = true }
```

Note: Remove `nms-watch` and `nms-cache` from dependencies for now -- they are stubs. Add them back when those crates are implemented (milestones 3.6-3.7 and Phase 5).

Also add `reedline` to the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# ... existing entries ...
reedline = "0.39"
```

---

## New File: `crates/nms-copilot/src/commands.rs`

Defines the REPL command parser. Reuses clap derive but parses from split line tokens rather than process args.

```rust
//! REPL command parsing -- reuses clap derive for consistent argument handling.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Top-level REPL command parser.
///
/// This is separate from the CLI parser because:
/// - No `--save` flag (the model is already loaded)
/// - Extra REPL-only commands (exit, help, status, set, reset)
/// - Parsed from user input line, not process args
#[derive(Parser, Debug)]
#[command(
    name = "",
    no_binary_name = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct ReplCommand {
    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Search planets by biome, distance, name.
    Find {
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

    /// Show detailed information about a system or base.
    Show {
        #[command(subcommand)]
        target: ShowTarget,
    },

    /// Display aggregate galaxy statistics.
    Stats {
        /// Show biome distribution table.
        #[arg(long)]
        biomes: bool,

        /// Show discovery counts by type.
        #[arg(long)]
        discoveries: bool,
    },

    /// Convert between NMS coordinate formats.
    Convert {
        /// Portal glyphs as 12 hex digits or emoji.
        #[arg(long, group = "input")]
        glyphs: Option<String>,

        /// Signal booster coordinates (XXXX:YYYY:ZZZZ:SSSS).
        #[arg(long, group = "input")]
        coords: Option<String>,

        /// Galactic address as hex (0x...).
        #[arg(long, group = "input")]
        ga: Option<String>,

        /// Voxel position as X,Y,Z (requires --ssi).
        #[arg(long, group = "input")]
        voxel: Option<String>,

        /// Solar system index (required with --voxel).
        #[arg(long)]
        ssi: Option<u16>,

        /// Planet index (0-15, defaults to 0).
        #[arg(long, default_value = "0")]
        planet: u8,

        /// Galaxy index (0-255) or name.
        #[arg(long, default_value = "0")]
        galaxy: String,
    },

    /// Display save file summary.
    Info,

    /// Show help for REPL commands.
    Help,

    /// Exit the REPL.
    Exit,

    /// Exit the REPL.
    Quit,
}

#[derive(Subcommand, Debug)]
pub enum ShowTarget {
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

/// Parse a REPL input line into a command.
///
/// Returns `None` for empty lines.
/// Returns `Err` with clap's error message for invalid commands.
pub fn parse_line(line: &str) -> Result<Option<Action>, String> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    // Split respecting quoted strings
    let args = shell_words(line);

    match ReplCommand::try_parse_from(args) {
        Ok(cmd) => Ok(cmd.action),
        Err(e) => {
            // Clap's help/version errors are not really errors
            let rendered = e.render().to_string();
            if e.use_stderr() {
                Err(rendered)
            } else {
                // Help text -- print it and return None
                print!("{rendered}");
                Ok(None)
            }
        }
    }
}

/// Simple shell-like word splitting that respects double quotes.
fn shell_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}
```

---

## New File: `crates/nms-copilot/src/dispatch.rs`

Executes parsed commands against the loaded model.

```rust
//! Command dispatch -- executes REPL commands against the loaded GalaxyModel.

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::display::{format_find_results, format_show_result, format_stats};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::show::{ShowQuery, execute_show};
use nms_query::stats::{StatsQuery, execute_stats};

use crate::commands::{Action, ShowTarget};

/// Execute a parsed REPL action against the model, returning output text.
pub fn dispatch(action: &Action, model: &GalaxyModel) -> Result<String, String> {
    match action {
        Action::Find {
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
        } => dispatch_find(model, biome, *infested, *within, *nearest, *named, discoverer, from),

        Action::Show { target } => dispatch_show(model, target),

        Action::Stats { biomes, discoveries } => {
            let query = StatsQuery {
                biomes: *biomes || !*discoveries,
                discoveries: *discoveries || !*biomes,
            };
            let result = execute_stats(model, &query);
            Ok(format_stats(&result))
        }

        Action::Info => {
            // In the REPL, info shows a summary of the loaded model
            let systems = model.systems.len();
            let planets = model.planets.len();
            let bases = model.bases.len();
            let pos = model.player_state.as_ref()
                .map(|ps| format!("{}", ps.current_address))
                .unwrap_or_else(|| "unknown".into());
            Ok(format!(
                "Loaded model: {systems} systems, {planets} planets, {bases} bases\n\
                 Current position: {pos}\n"
            ))
        }

        Action::Help => Ok(help_text()),

        // Exit/Quit are handled in the REPL loop, not here
        Action::Exit | Action::Quit => Ok(String::new()),

        Action::Convert { glyphs, coords, ga, voxel, ssi, planet, galaxy } => {
            // Delegate to the same converter used by nms-cli
            // For now, return a placeholder -- the full converter is already in nms-core
            dispatch_convert(glyphs, coords, ga, voxel, *ssi, *planet, galaxy)
        }
    }
}

fn dispatch_find(
    model: &GalaxyModel,
    biome: &Option<String>,
    infested: bool,
    within: Option<f64>,
    nearest: Option<usize>,
    named: bool,
    discoverer: &Option<String>,
    from: &Option<String>,
) -> Result<String, String> {
    let biome = biome
        .as_ref()
        .map(|s| s.parse::<Biome>())
        .transpose()
        .map_err(|e| format!("Invalid biome: {e}"))?;

    let reference = match from {
        Some(name) => ReferencePoint::Base(name.clone()),
        None => ReferencePoint::CurrentPosition,
    };

    let query = FindQuery {
        biome,
        infested: if infested { Some(true) } else { None },
        within_ly: within,
        nearest,
        name_pattern: None,
        discoverer: discoverer.clone(),
        named_only: named,
        from: reference,
    };

    let results = execute_find(model, &query).map_err(|e| e.to_string())?;
    Ok(format_find_results(&results))
}

fn dispatch_show(model: &GalaxyModel, target: &ShowTarget) -> Result<String, String> {
    let query = match target {
        ShowTarget::System { name } => ShowQuery::System(name.clone()),
        ShowTarget::Base { name } => ShowQuery::Base(name.clone()),
    };
    let result = execute_show(model, &query).map_err(|e| e.to_string())?;
    Ok(format_show_result(&result))
}

fn dispatch_convert(
    glyphs: &Option<String>,
    coords: &Option<String>,
    ga: &Option<String>,
    voxel: &Option<String>,
    ssi: Option<u16>,
    planet: u8,
    galaxy: &str,
) -> Result<String, String> {
    // Reuse the same conversion logic from nms-core
    use nms_core::address::GalacticAddress;
    use nms_core::glyph::PortalAddress;
    use nms_query::display::hex_to_emoji;

    if let Some(g) = glyphs {
        let portal = PortalAddress::parse(g).map_err(|e| format!("Invalid glyphs: {e}"))?;
        let addr = portal.to_galactic_address();
        let hex = portal.to_hex();
        let emoji = hex_to_emoji(&hex);
        Ok(format!(
            "Portal Glyphs:  {emoji}\n\
             Hex Glyphs:     {hex}\n\
             Galactic Addr:  0x{:012X}\n\
             Signal Booster: {}\n\
             Voxel:          {},{},{}\n\
             System Index:   {}\n\
             Planet Index:   {}\n",
            addr.to_packed(),
            addr.to_signal_booster(),
            addr.voxel_x, addr.voxel_y, addr.voxel_z,
            addr.solar_system_index,
            addr.planet_index,
        ))
    } else if let Some(c) = coords {
        let addr = GalacticAddress::from_signal_booster(c)
            .map_err(|e| format!("Invalid coordinates: {e}"))?;
        let portal = PortalAddress::from_galactic_address(&addr);
        let hex = portal.to_hex();
        let emoji = hex_to_emoji(&hex);
        Ok(format!(
            "Portal Glyphs:  {emoji}\n\
             Hex Glyphs:     {hex}\n\
             Galactic Addr:  0x{:012X}\n\
             Signal Booster: {c}\n",
            addr.to_packed(),
        ))
    } else {
        Err("Specify --glyphs, --coords, --ga, or --voxel".into())
    }
}

fn help_text() -> String {
    "\
NMS Copilot -- Interactive Galaxy Explorer

Commands:
  find       Search planets by biome, distance, name
  show       Show system or base details
  stats      Display aggregate galaxy statistics
  convert    Convert between coordinate formats
  info       Show loaded model summary
  help       Show this help message
  exit/quit  Exit the REPL

Examples:
  find --biome Lush --nearest 5
  show system 0x050003AB8C07
  show base \"Acadia National Park\"
  stats --biomes
  convert --glyphs 01717D8A4EA2
"
    .into()
}
```

---

## Modified File: `crates/nms-copilot/src/main.rs`

Replace the placeholder with the reedline REPL loop.

```rust
//! NMS Copilot -- interactive galactic REPL for No Man's Sky.

use std::path::PathBuf;

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use nms_graph::GalaxyModel;

mod commands;
mod dispatch;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Optional --save flag for specifying save file path
    let save_path = parse_save_arg(&args);

    // Load the galaxy model
    let model = match load_model(save_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading save: {e}");
            std::process::exit(1);
        }
    };

    println!(
        "NMS Copilot v{}\n\
         Loaded {} systems, {} planets, {} bases\n\
         Type 'help' for commands, 'exit' to quit.\n",
        env!("CARGO_PKG_VERSION"),
        model.systems.len(),
        model.planets.len(),
        model.bases.len(),
    );

    // Build reedline editor
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("nms".into()),
        DefaultPromptSegment::Empty,
    );

    let mut editor = Reedline::create();

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                match commands::parse_line(&line) {
                    Ok(Some(action)) => {
                        if matches!(action, commands::Action::Exit | commands::Action::Quit) {
                            break;
                        }
                        match dispatch::dispatch(&action, &model) {
                            Ok(output) => {
                                if !output.is_empty() {
                                    print!("{output}");
                                }
                            }
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }
                    Ok(None) => {} // empty line or help printed
                    Err(e) => eprintln!("{e}"),
                }
            }
            Ok(Signal::CtrlD | Signal::CtrlC) => {
                break;
            }
            Err(e) => {
                eprintln!("Input error: {e}");
                break;
            }
        }
    }

    println!("Goodbye!");
}

fn parse_save_arg(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == "--save")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

fn load_model(save_path: Option<PathBuf>) -> Result<GalaxyModel, Box<dyn std::error::Error>> {
    let path = match save_path {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    let save = nms_save::parse_save_file(&path)?;
    Ok(GalaxyModel::from_save(&save))
}
```

---

## Tests

### File: `crates/nms-copilot/src/commands.rs` (inline tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_line("").unwrap().is_none());
        assert!(parse_line("   ").unwrap().is_none());
    }

    #[test]
    fn test_parse_exit() {
        let action = parse_line("exit").unwrap().unwrap();
        assert!(matches!(action, Action::Exit));
    }

    #[test]
    fn test_parse_quit() {
        let action = parse_line("quit").unwrap().unwrap();
        assert!(matches!(action, Action::Quit));
    }

    #[test]
    fn test_parse_help() {
        let action = parse_line("help").unwrap().unwrap();
        assert!(matches!(action, Action::Help));
    }

    #[test]
    fn test_parse_find_with_biome() {
        let action = parse_line("find --biome Lush --nearest 5").unwrap().unwrap();
        match action {
            Action::Find { biome, nearest, .. } => {
                assert_eq!(biome.as_deref(), Some("Lush"));
                assert_eq!(nearest, Some(5));
            }
            _ => panic!("Expected Find"),
        }
    }

    #[test]
    fn test_parse_show_base_quoted() {
        let action = parse_line("show base \"Acadia National Park\"").unwrap().unwrap();
        match action {
            Action::Show { target: ShowTarget::Base { name } } => {
                assert_eq!(name, "Acadia National Park");
            }
            _ => panic!("Expected Show Base"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        assert!(parse_line("foobar").is_err());
    }

    #[test]
    fn test_shell_words_basic() {
        let words = shell_words("find --biome Lush");
        assert_eq!(words, vec!["find", "--biome", "Lush"]);
    }

    #[test]
    fn test_shell_words_quoted() {
        let words = shell_words("show base \"My Base Name\"");
        assert_eq!(words, vec!["show", "base", "My Base Name"]);
    }

    #[test]
    fn test_parse_stats_flags() {
        let action = parse_line("stats --biomes").unwrap().unwrap();
        match action {
            Action::Stats { biomes, discoveries } => {
                assert!(biomes);
                assert!(!discoveries);
            }
            _ => panic!("Expected Stats"),
        }
    }

    #[test]
    fn test_parse_info() {
        let action = parse_line("info").unwrap().unwrap();
        assert!(matches!(action, Action::Info));
    }
}
```

### File: `crates/nms-copilot/tests/dispatch_integration.rs`

```rust
use nms_graph::GalaxyModel;

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
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn dispatch_find_returns_results() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    // Parse and dispatch a find command
    let action = nms_copilot::commands::parse_line("find").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn dispatch_stats_returns_output() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("stats").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("Galaxy Statistics") || output.contains("system"));
}

#[test]
fn dispatch_info_returns_summary() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("info").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("systems"));
    assert!(output.contains("planets"));
}

#[test]
fn dispatch_help_returns_text() {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);

    let action = nms_copilot::commands::parse_line("help").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &model).unwrap();
    assert!(output.contains("Commands:"));
}
```

Note: For integration tests to access `commands` and `dispatch` modules, the crate needs a `lib.rs`:

### New File: `crates/nms-copilot/src/lib.rs`

```rust
//! NMS Copilot library -- shared modules for the interactive REPL.

pub mod commands;
pub mod dispatch;
```

---

## Implementation Notes

1. **reedline integration**: The REPL uses `Reedline::create()` for the simplest possible setup. No history, completion, or custom prompt yet -- those come in milestones 3.2, 3.3, and 3.5.

2. **Command parsing via clap**: `ReplCommand` uses `no_binary_name = true` and `disable_help_flag = true` so clap doesn't expect argv[0] or inject `--help`. The REPL handles `help` as an explicit subcommand instead.

3. **Shell word splitting**: The `shell_words()` function handles double-quoted strings so `show base "Acadia National Park"` works. Single quotes and escape characters are not supported (can be added if needed).

4. **Model is immutable**: In this milestone, the model is loaded once at startup and never modified. Live updates (Phase 5) will require `Arc<RwLock<GalaxyModel>>` or similar.

5. **No --save in REPL commands**: Unlike the CLI, the REPL loads the save once at startup. Individual commands operate on the in-memory model, not the save file.

6. **Convert command**: Reuses `nms_core::glyph::PortalAddress` and `nms_core::address::GalacticAddress` directly. Only `--glyphs` and `--coords` are wired up initially; `--ga` and `--voxel` can be added straightforwardly.

7. **lib.rs + main.rs split**: The `lib.rs` exposes `commands` and `dispatch` for integration testing. The `main.rs` handles the REPL loop and startup.
