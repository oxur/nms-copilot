//! NMS Copilot -- interactive galactic REPL for No Man's Sky.

use std::path::{Path, PathBuf};

use reedline::{FileBackedHistory, Reedline, Signal};

use nms_copilot::completer::{CopilotCompleter, ModelCompletions};
use nms_copilot::config::Config;
use nms_copilot::prompt::{CopilotPrompt, PromptState};
use nms_copilot::session::SessionState;
use nms_copilot::{commands, dispatch, paths};
use nms_graph::GalaxyModel;

fn main() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not load config: {e}");
            Config::default()
        }
    };

    let args: Vec<String> = std::env::args().collect();
    let save_path = parse_save_arg(&args).or_else(|| config.save_path().map(PathBuf::from));
    let no_cache = args.iter().any(|a| a == "--no-cache") || !config.cache_enabled();
    let cache_path = config.cache_path();

    let (model, was_cached) = match load_model(save_path, &cache_path, no_cache) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error loading save: {e}");
            std::process::exit(1);
        }
    };

    let source = if was_cached {
        "from cache"
    } else {
        "from save file"
    };
    println!(
        "NMS Copilot v{}\n\
         Loaded {} systems, {} planets, {} bases ({source})\n\
         Type 'help' for commands, 'exit' to quit.\n",
        env!("CARGO_PKG_VERSION"),
        model.systems.len(),
        model.planets.len(),
        model.bases.len(),
    );

    let completions = build_model_completions(&model);
    let completer = Box::new(CopilotCompleter::new(completions));
    let mut editor = build_editor(completer);
    let mut session = SessionState::from_model(&model);
    if let Some(warp_range) = config.defaults.warp_range {
        session.set_warp_range(warp_range);
    }
    let mut prompt = CopilotPrompt::new(PromptState::from_session(&session));

    loop {
        prompt.update(PromptState::from_session(&session));
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match commands::parse_line(&line) {
                Ok(Some(action)) => {
                    if matches!(action, commands::Action::Exit | commands::Action::Quit) {
                        break;
                    }
                    match dispatch::dispatch(&action, &model, &mut session) {
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

fn build_editor(completer: Box<CopilotCompleter>) -> Reedline {
    if let Err(e) = paths::ensure_data_dir() {
        eprintln!("Warning: could not create data directory: {e}");
        return Reedline::create().with_completer(completer);
    }

    let history = match FileBackedHistory::with_file(1000, paths::history_path()) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Warning: could not load history: {e}");
            return Reedline::create().with_completer(completer);
        }
    };

    Reedline::create()
        .with_history(Box::new(history))
        .with_completer(completer)
}

fn build_model_completions(model: &GalaxyModel) -> ModelCompletions {
    let base_names: Vec<String> = model.bases.values().map(|b| b.name.clone()).collect();
    let system_names: Vec<String> = model
        .systems
        .values()
        .filter_map(|s| s.name.clone())
        .collect();

    ModelCompletions {
        base_names,
        system_names,
    }
}

fn parse_save_arg(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == "--save")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

fn load_model(
    save_path: Option<PathBuf>,
    cache_path: &Path,
    no_cache: bool,
) -> Result<(GalaxyModel, bool), Box<dyn std::error::Error>> {
    let save = match save_path {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };
    nms_cache::load_or_rebuild(cache_path, &save, no_cache)
}
