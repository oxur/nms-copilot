//! Crossterm event handling for the interactive map.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::state::MapState;

/// Map action produced by input handling.
pub enum MapAction {
    /// No action (timeout or ignored event).
    None,
    /// Terminal was resized.
    Resized(u16, u16),
}

/// Poll for input events and update map state accordingly.
///
/// Returns `MapAction::Resized` when the terminal size changes,
/// so the caller can trigger a re-render.
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
        KeyCode::Esc | KeyCode::Char('-') => {
            if !state.zoom_out() {
                state.should_quit = true;
            }
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
