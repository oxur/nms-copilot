use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod completions;
mod convert;
mod export;
mod find;
mod info;
mod route;
mod saves;
mod show;
mod stats;

#[derive(Parser)]
#[command(
    name = "nms",
    about = "NMS Copilot CLI -- search planets, plan routes, convert glyphs",
    version
)]
pub(crate) struct Cli {
    /// Use a specific save slot (1-15) instead of most recent.
    #[arg(long, global = true)]
    slot: Option<u8>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display save file summary.
    Info {
        /// Path to a decompressed/deobfuscated save file (JSON).
        /// If omitted, auto-detects the most recent save.
        #[arg(long)]
        save: Option<PathBuf>,
    },

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

    /// Show detailed information about a system or base.
    Show {
        /// Path to save file (auto-detects if omitted).
        #[arg(long)]
        save: Option<PathBuf>,

        #[command(subcommand)]
        target: ShowTargetCmd,
    },

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

    /// Plan a route through discovered systems.
    Route {
        /// Path to save file (auto-detects if omitted).
        #[arg(long)]
        save: Option<PathBuf>,

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

    /// Convert between NMS coordinate formats.
    Convert {
        /// Portal glyphs as 12 hex digits (e.g., 01717D8A4EA2).
        #[arg(long, group = "input")]
        glyphs: Option<String>,

        /// Signal booster coordinates (XXXX:YYYY:ZZZZ:SSSS).
        #[arg(long, group = "input")]
        coords: Option<String>,

        /// Galactic address as hex (0x01717D8A4EA2).
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

        /// Galaxy index (0-255) or name (e.g., "Euclid").
        #[arg(long, default_value = "0")]
        galaxy: String,
    },

    /// Export filtered planets as JSON or CSV.
    Export {
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

        /// Output format: json, csv (default: json).
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for: bash, zsh, fish, powershell, elvish.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// List all save slots.
    Saves,
}

#[derive(Subcommand)]
enum ShowTargetCmd {
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

/// Resolve a save file path from --save, --slot, or auto-detect.
fn resolve_save(save: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = save {
        return Ok(path);
    }
    Ok(nms_save::locate::find_most_recent_save()?
        .path()
        .to_path_buf())
}

/// Resolve a save file path, checking --slot from the global CLI arg.
fn resolve_save_with_slot(
    save: Option<PathBuf>,
    slot: Option<u8>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = save {
        return Ok(path);
    }
    if let Some(slot_num) = slot {
        let save_dir = nms_save::locate::nms_save_dir_checked()?;
        let accounts = nms_save::locate::list_accounts(&save_dir)?;
        let saves = nms_save::locate::list_saves(accounts[0].path())?;
        let slots = nms_save::locate::group_into_slots(&saves);
        let target = slots
            .iter()
            .find(|s| s.slot() == slot_num)
            .ok_or_else(|| format!("save slot {slot_num} not found"))?;
        let file = target
            .most_recent()
            .ok_or_else(|| format!("save slot {slot_num} is empty"))?;
        return Ok(file.path().to_path_buf());
    }
    Ok(nms_save::locate::find_most_recent_save()?
        .path()
        .to_path_buf())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let slot = cli.slot;

    match cli.command {
        Commands::Info { save } => {
            let path = resolve_save_with_slot(save, slot)?;
            info::run(Some(path))
        }
        Commands::Find {
            save,
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
        } => {
            let path = resolve_save_with_slot(save, slot)?;
            find::run(find::FindArgs {
                save: Some(path),
                biome,
                infested,
                within,
                nearest,
                named,
                discoverer,
                from,
            })
        }
        Commands::Show { save, target } => {
            let path = resolve_save_with_slot(save, slot)?;
            let target = match target {
                ShowTargetCmd::System { name } => show::ShowTarget::System { name },
                ShowTargetCmd::Base { name } => show::ShowTarget::Base { name },
            };
            show::run(Some(path), target)
        }
        Commands::Stats {
            save,
            biomes,
            discoveries,
        } => {
            let path = resolve_save_with_slot(save, slot)?;
            stats::run(Some(path), biomes, discoveries)
        }
        Commands::Route {
            save,
            biome,
            targets,
            from,
            warp_range,
            within,
            max_targets,
            algo,
            round_trip,
        } => {
            let path = resolve_save_with_slot(save, slot)?;
            route::run(route::RouteArgs {
                save: Some(path),
                biome,
                targets,
                from,
                warp_range,
                within,
                max_targets,
                algo,
                round_trip,
            })
        }
        Commands::Convert {
            glyphs,
            coords,
            ga,
            voxel,
            ssi,
            planet,
            galaxy,
        } => convert::run(glyphs, coords, ga, voxel, ssi, planet, galaxy),
        Commands::Export {
            save,
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
            format,
        } => {
            let path = resolve_save_with_slot(save, slot)?;
            export::run(export::ExportArgs {
                save: Some(path),
                biome,
                infested,
                within,
                nearest,
                named,
                discoverer,
                from,
                format,
            })
        }
        Commands::Completions { shell } => completions::run(shell),
        Commands::Saves => saves::run(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_save_explicit_path() {
        let path = PathBuf::from("/tmp/save.hg");
        let result = resolve_save(Some(path.clone())).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn test_resolve_save_with_slot_explicit_path_takes_priority() {
        let path = PathBuf::from("/tmp/save.hg");
        let result = resolve_save_with_slot(Some(path.clone()), Some(2)).unwrap();
        assert_eq!(result, path);
    }
}
