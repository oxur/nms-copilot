//! Interactive ASCII galaxy map using Ratatui.
//!
//! This module joins [`input`] handling, [`render`]ing, and [`state`] for a
//! borrowed snapshot of a [`GalaxyModel`] and [`SessionState`]. Its zoom tiers
//! are [`state::ZoomLevel::Galaxy`], [`state::ZoomLevel::Region`], and
//! [`state::ZoomLevel::Local`]. The map projects voxel X/Z and does not update
//! the model or session.
//!
//! The entry point requires an interactive terminal; the state and renderer
//! can also be used separately with an in-memory backend.

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
/// Borrows the model and session as a read-only snapshot. It enables raw
/// mode, enters the alternate screen, hides the cursor, then draws before
/// each input poll; a resize updates the grid for the following draw.
///
/// Setup and teardown are sequential best-effort operations. A loop error
/// still reaches teardown. A setup error can return before cleanup starts; a
/// cleanup error can replace the loop result and prevent later restoration.
/// Panics are not protected by an RAII terminal guard.
///
/// # Examples
///
/// This terminal entry example is checked but not run because it requires an
/// interactive terminal.
/// ```no_run
/// use std::io;
/// use nms_copilot::{map::run_map, session::SessionState};
/// use nms_graph::GalaxyModel;
/// # fn main() -> io::Result<()> {
/// let model = GalaxyModel::new();
/// let session = SessionState::from_model(&model);
/// run_map(&model, &session)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns I/O errors from raw-mode setup, screen/cursor commands, terminal
/// construction or sizing, drawing, event polling or reading, raw-mode
/// teardown, or screen/cursor restoration.
///
/// # Panics
///
/// A panic in map processing unwinds without normal terminal teardown. See
/// [`state::MapState::move_cursor`] for its numeric limit.
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
