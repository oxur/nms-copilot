use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod info;

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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { save } => info::run(save),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
