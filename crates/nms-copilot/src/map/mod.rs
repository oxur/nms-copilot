//! Interactive ASCII galaxy map using ratatui.
//!
//! Provides a full-screen TUI map with cursor navigation and zoom drill-down.
//! Three zoom levels: Galaxy → Region → Local.

pub mod input;
pub mod render;
pub mod state;

use std::io;

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use nms_graph::GalaxyModel;

use crate::session::SessionState;
use state::MapState;

/// Enter the interactive map, returning when the user exits.
///
/// Sets up an alternate screen with raw mode, runs the event loop,
/// and restores the terminal on exit (even if the loop errors).
pub fn run_map(model: &GalaxyModel, session: &SessionState) -> io::Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_map_loop(&mut terminal, model, session);

    // Teardown (always runs)
    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), Show, LeaveAlternateScreen)?;

    result
}

/// Main event loop for the interactive map.
fn run_map_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model: &GalaxyModel,
    session: &SessionState,
) -> io::Result<()> {
    let size = terminal.size()?;
    let mut state = MapState::new(model, session);
    state.resize(size.width, size.height);

    loop {
        terminal.draw(|frame| {
            render::render(frame, &state, model);
        })?;

        input::handle_input(&mut state)?;

        if state.should_quit {
            break;
        }
    }

    Ok(())
}
