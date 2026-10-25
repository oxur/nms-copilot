//! Crossterm event handling for the interactive map.
//!
//! [`handle_input`] polls for an event and updates a borrowed
//! [`MapState`]. It does not configure or restore
//! terminal modes; [`super::run_map`] owns that lifecycle.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::state::MapState;

/// Result of polling and handling one map input event.
///
/// `None` does not imply unchanged state: an ordinary key may already have
/// changed the cursor, zoom, help, or quit flag.
pub enum MapAction {
    /// No resize action to report.
    ///
    /// Returned for timeout, ignored event, or key input; key handling may
    /// already have changed state.
    None,
    /// Terminal resized, reported as `(width, height)`.
    ///
    /// [`MapState::resize`] has already been
    /// applied before this variant is returned.
    Resized(u16, u16),
}

/// Poll for up to 100 ms and apply one available event to `state`.
///
/// Resize calls [`MapState::resize`] and
/// reports width then height. Key event kind is not filtered (press, repeat,
/// or release). While help is open, any key dismisses it and has no other key
/// effect, including `q` and Ctrl+C; resize still applies. Otherwise arrows
/// move, Enter/`+` zoom in, Esc/`-` zoom out and request exit only if the zoom
/// stack is empty, `q`/Ctrl+C request exit, unmodified lowercase `c` centers,
/// and `?` toggles help. Timeout, ignored input, and handled keys return
/// [`MapAction::None`], which may follow a state change.
///
/// This function does not enable raw mode or restore the terminal.
///
/// # Errors
///
/// Returns an I/O error if Crossterm event polling or reading fails.
///
/// # Panics
///
/// An arrow key calls [`MapState::move_cursor`], whose intermediate `i16`
/// addition can overflow with overflow checks enabled for extreme public
/// cursor values or deltas. See that method for the exact limitation.
pub fn handle_input(state: &mut MapState) -> std::io::Result<MapAction> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(MapAction::None);
    }

    match event::read()? {
        Event::Key(key) => {
            handle_key(key, state);
            Ok(MapAction::None)
        }
        Event::Resize(w, h) => {
            state.resize(w, h);
            Ok(MapAction::Resized(w, h))
        }
        _ => Ok(MapAction::None),
    }
}

/// Process a key event and update map state.
fn handle_key(key: KeyEvent, state: &mut MapState) {
    // Toggle help overlay handles its own key
    if state.show_help {
        state.show_help = false;
        return;
    }

    match key.code {
        // Navigation
        KeyCode::Up => state.move_cursor(0, -1),
        KeyCode::Down => state.move_cursor(0, 1),
        KeyCode::Left => state.move_cursor(-1, 0),
        KeyCode::Right => state.move_cursor(1, 0),

        // Zoom
        KeyCode::Enter | KeyCode::Char('+') => state.zoom_in(),
        KeyCode::Esc | KeyCode::Char('-') if !state.zoom_out() => {
            state.should_quit = true;
        }

        // Commands
        KeyCode::Char('q') => state.should_quit = true,
        KeyCode::Char('c') if key.modifiers == KeyModifiers::NONE => {
            state.center_on_player();
        }
        KeyCode::Char('?') => state.show_help = !state.show_help,

        // Ctrl+C also quits
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
        }

        _ => {}
    }
}
