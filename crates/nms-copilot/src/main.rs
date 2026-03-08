//! NMS Copilot -- interactive galactic REPL for No Man's Sky.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use reedline::{FileBackedHistory, Reedline, Signal};

use nms_copilot::banner;
use nms_copilot::completer::{CopilotCompleter, ModelCompletions};
use nms_copilot::config::Config;
use nms_copilot::prompt::{CopilotPrompt, PromptState};
use nms_copilot::session::SessionState;
use nms_copilot::watch::drain_watch_events;
use nms_copilot::{commands, dispatch, paths};
use nms_graph::GalaxyModel;
use nms_watch::{WatchConfig, WatchHandle, start_watching};

fn main() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not load config: {e}");
            Config::default()
        }
    };

    let args: Vec<String> = std::env::args().collect();
    let save_path = resolve_save_path(&args, &config);
    let no_cache = args.iter().any(|a| a == "--no-cache") || !config.cache_enabled();
    let cache_path = config.cache_path();

    let (model, was_cached, save_version) =
        match load_model(save_path.clone(), &cache_path, no_cache) {
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

    // Art banner (ASCII art, configurable)
    banner::print_banner(
        config.display.banner.as_deref(),
        config.display.show_banner,
        config.display.color,
    );

    // System banner (model stats + help hint)
    banner::print_system_banner(
        config.display.show_system_banner,
        model.systems.len(),
        model.planets.len(),
        model.bases.len(),
        source,
    );

    // Start file watcher (optional -- don't fail startup if watcher can't start)
    let watch_handle = if config.watch_enabled() {
        match start_watcher(&config, save_path) {
            Ok(handle) => {
                println!("Watching save file for live updates.\n");
                Some(handle)
            }
            Err(e) => {
                eprintln!("Warning: could not start file watcher: {e}\n");
                None
            }
        }
    } else {
        println!();
        None
    };

    let cache_for_watcher = if no_cache {
        None
    } else {
        Some(cache_path.as_path())
    };
    let mut model = model;

    let completions = build_model_completions(&model);
    let completer = Box::new(CopilotCompleter::new(completions));
    let mut editor = build_editor(completer);
    let mut session = SessionState::from_model(&model);
    if let Some(warp_range) = config.defaults.warp_range {
        session.set_warp_range(warp_range);
    }
    let mut prompt = CopilotPrompt::new(PromptState::from_session(&session));

    loop {
        // Drain any pending watch events before showing prompt
        if let Some(ref handle) = watch_handle {
            drain_watch_events(
                &handle.receiver,
                &mut model,
                &mut session,
                cache_for_watcher,
                save_version,
            );
        }

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

                    // Also drain after command execution
                    if let Some(ref handle) = watch_handle {
                        drain_watch_events(
                            &handle.receiver,
                            &mut model,
                            &mut session,
                            cache_for_watcher,
                            save_version,
                        );
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

fn resolve_save_path(args: &[String], config: &Config) -> Option<PathBuf> {
    // 1. CLI arg
    if let Some(p) = parse_save_arg(args) {
        return Some(p);
    }
    // 2+3. ENV vars + config (env already applied in Config::load())
    if let Some(p) = config.effective_save_file() {
        return Some(p);
    }
    // 4. Auto-detect
    if let Ok(save) = nms_save::locate::find_most_recent_save() {
        return Some(save.path().to_path_buf());
    }
    // 5. Wizard (TTY only)
    if std::io::stdin().is_terminal() {
        match nms_copilot::setup::run_setup_wizard() {
            Ok(path) => return Some(path),
            Err(e) => {
                eprintln!("Setup failed: {e}");
                eprintln!(
                    "Configure manually in ~/.nms-copilot/config.toml \
                     or use: nms-copilot --save /path/to/save.hg"
                );
                std::process::exit(1);
            }
        }
    }
    None
}

fn load_model(
    save_path: Option<PathBuf>,
    cache_path: &Path,
    no_cache: bool,
) -> Result<(GalaxyModel, bool, u32), Box<dyn std::error::Error>> {
    let save = save_path.ok_or("no save file path resolved")?;
    let result = nms_cache::load_or_rebuild(cache_path, &save, no_cache)?;
    Ok((result.model, result.was_cached, result.save_version))
}

fn start_watcher(
    config: &Config,
    save_path: Option<PathBuf>,
) -> Result<WatchHandle, Box<dyn std::error::Error>> {
    let path = match save_path {
        Some(p) => p,
        None => match config.save_path() {
            Some(p) => p,
            None => nms_save::locate::find_most_recent_save()?
                .path()
                .to_path_buf(),
        },
    };

    let watch_config = WatchConfig {
        save_path: path,
        debounce: config.watch_debounce(),
    };

    Ok(start_watching(watch_config)?)
}
