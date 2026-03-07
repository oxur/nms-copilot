//! NMS Copilot -- interactive galactic REPL for No Man's Sky.

use std::path::PathBuf;

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use nms_graph::GalaxyModel;

mod commands;
mod dispatch;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let save_path = parse_save_arg(&args);

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

    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("nms".into()),
        DefaultPromptSegment::Empty,
    );

    let mut editor = Reedline::create();

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match commands::parse_line(&line) {
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
                Ok(None) => {}
                Err(e) => eprintln!("{e}"),
            },
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
