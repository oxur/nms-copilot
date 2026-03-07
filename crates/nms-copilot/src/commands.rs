//! REPL command parsing -- reuses clap derive for consistent argument handling.

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
    disable_help_subcommand = true,
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

    /// Plan a route through discovered systems.
    Route {
        /// Filter targets by biome (e.g., Lush, Toxic).
        #[arg(long)]
        biome: Option<String>,

        /// Named targets (bases or systems) to visit.
        #[arg(long = "target", num_args = 1)]
        targets: Vec<String>,

        /// Start from this base name (default: current position).
        #[arg(long)]
        from: Option<String>,

        /// Ship warp range in light-years (for hop constraints).
        #[arg(long)]
        warp_range: Option<f64>,

        /// Only consider targets within this radius in light-years.
        #[arg(long)]
        within: Option<f64>,

        /// Maximum number of targets to visit.
        #[arg(long)]
        max_targets: Option<usize>,

        /// Routing algorithm: nn, nearest-neighbor, 2opt, two-opt.
        #[arg(long)]
        algo: Option<String>,

        /// Return to starting system at the end.
        #[arg(long)]
        round_trip: bool,
    },

    /// Set session context (position, biome filter, warp range).
    Set {
        #[command(subcommand)]
        target: SetTarget,
    },

    /// Reset session state.
    Reset {
        /// What to reset (position, biome, warp-range, all).
        #[arg(default_value = "all")]
        target: String,
    },

    /// Show current session state.
    Status,

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
pub enum SetTarget {
    /// Set reference position to a base name.
    Position {
        /// Base name or address.
        name: String,
    },
    /// Set active biome filter.
    Biome {
        /// Biome name (e.g., Lush, Toxic).
        name: String,
    },
    /// Set default warp range.
    #[command(name = "warp-range")]
    WarpRange {
        /// Range in light-years.
        ly: f64,
    },
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

    let args = shell_words(line);

    match ReplCommand::try_parse_from(args) {
        Ok(cmd) => Ok(cmd.action),
        Err(e) => {
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
        let action = parse_line("find --biome Lush --nearest 5")
            .unwrap()
            .unwrap();
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
        let action = parse_line("show base \"Acadia National Park\"")
            .unwrap()
            .unwrap();
        match action {
            Action::Show {
                target: ShowTarget::Base { name },
            } => {
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
            Action::Stats {
                biomes,
                discoveries,
            } => {
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

    #[test]
    fn test_parse_status() {
        let action = parse_line("status").unwrap().unwrap();
        assert!(matches!(action, Action::Status));
    }

    #[test]
    fn test_parse_set_biome() {
        let action = parse_line("set biome Lush").unwrap().unwrap();
        match action {
            Action::Set {
                target: SetTarget::Biome { name },
            } => assert_eq!(name, "Lush"),
            _ => panic!("Expected Set Biome"),
        }
    }

    #[test]
    fn test_parse_set_position() {
        let action = parse_line("set position \"Home Base\"").unwrap().unwrap();
        match action {
            Action::Set {
                target: SetTarget::Position { name },
            } => assert_eq!(name, "Home Base"),
            _ => panic!("Expected Set Position"),
        }
    }

    #[test]
    fn test_parse_set_warp_range() {
        let action = parse_line("set warp-range 2500").unwrap().unwrap();
        match action {
            Action::Set {
                target: SetTarget::WarpRange { ly },
            } => assert_eq!(ly, 2500.0),
            _ => panic!("Expected Set WarpRange"),
        }
    }

    #[test]
    fn test_parse_reset_default() {
        let action = parse_line("reset").unwrap().unwrap();
        match action {
            Action::Reset { target } => assert_eq!(target, "all"),
            _ => panic!("Expected Reset"),
        }
    }

    #[test]
    fn test_parse_route_with_biome_and_warp_range() {
        let action = parse_line("route --biome Lush --warp-range 2500")
            .unwrap()
            .unwrap();
        match action {
            Action::Route {
                biome, warp_range, ..
            } => {
                assert_eq!(biome.as_deref(), Some("Lush"));
                assert_eq!(warp_range, Some(2500.0));
            }
            _ => panic!("Expected Route"),
        }
    }

    #[test]
    fn test_parse_route_with_targets() {
        let action = parse_line("route --target \"Alpha Base\" --target \"Beta Base\"")
            .unwrap()
            .unwrap();
        match action {
            Action::Route { targets, .. } => {
                assert_eq!(targets.len(), 2);
                assert_eq!(targets[0], "Alpha Base");
                assert_eq!(targets[1], "Beta Base");
            }
            _ => panic!("Expected Route"),
        }
    }

    #[test]
    fn test_parse_route_round_trip() {
        let action = parse_line("route --biome Lush --round-trip")
            .unwrap()
            .unwrap();
        match action {
            Action::Route { round_trip, .. } => {
                assert!(round_trip);
            }
            _ => panic!("Expected Route"),
        }
    }

    #[test]
    fn test_parse_reset_biome() {
        let action = parse_line("reset biome").unwrap().unwrap();
        match action {
            Action::Reset { target } => assert_eq!(target, "biome"),
            _ => panic!("Expected Reset"),
        }
    }
}
