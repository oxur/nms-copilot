use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod convert;
mod find;
mod info;
mod route;
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

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { save } => info::run(save),
        Commands::Find {
            save,
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
        } => find::run(find::FindArgs {
            save,
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
        }),
        Commands::Show { save, target } => {
            let target = match target {
                ShowTargetCmd::System { name } => show::ShowTarget::System { name },
                ShowTargetCmd::Base { name } => show::ShowTarget::Base { name },
            };
            show::run(save, target)
        }
        Commands::Stats {
            save,
            biomes,
            discoveries,
        } => stats::run(save, biomes, discoveries),
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
        } => route::run(route::RouteArgs {
            save,
            biome,
            targets,
            from,
            warp_range,
            within,
            max_targets,
            algo,
            round_trip,
        }),
        Commands::Convert {
            glyphs,
            coords,
            ga,
            voxel,
            ssi,
            planet,
            galaxy,
        } => convert::run(glyphs, coords, ga, voxel, ssi, planet, galaxy),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
