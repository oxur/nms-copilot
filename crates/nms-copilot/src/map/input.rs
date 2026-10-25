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
    handle_input_with(state, event::poll, event::read)
}

// Keep event routing testable without accessing the terminal.
fn handle_input_with<P, R>(state: &mut MapState, poll: P, read: R) -> std::io::Result<MapAction>
where
    P: FnOnce(Duration) -> std::io::Result<bool>,
    R: FnOnce() -> std::io::Result<Event>,
{
    if !poll(Duration::from_millis(100))? {
        return Ok(MapAction::None);
    }

    match read()? {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
    use std::cell::RefCell;

    #[derive(Debug, PartialEq)]
    struct StateSnapshot {
        zoom: super::super::state::ZoomLevel,
        center: (f64, f64),
        cursor: (u16, u16),
        grid_size: (u16, u16),
        galaxy: u8,
        galaxy_name: String,
        base_labels: Vec<(char, String, i16, i16)>,
        player_pos: Option<(i16, i16)>,
        zoom_stack: Vec<(super::super::state::ZoomLevel, f64, f64)>,
        show_help: bool,
        should_quit: bool,
    }

    fn seed_state() -> MapState {
        MapState {
            zoom: super::super::state::ZoomLevel::Galaxy,
            center: (16.0, -8.0),
            cursor: (2, 1),
            grid_size: (8, 4),
            galaxy: 9,
            galaxy_name: "Eissentam".to_owned(),
            base_labels: vec![super::super::state::BaseLabel {
                letter: 'A',
                name: "Preserved é".to_owned(),
                voxel_x: 30,
                voxel_z: -40,
            }],
            player_pos: Some((100, 200)),
            zoom_stack: Vec::new(),
            show_help: false,
            should_quit: false,
        }
    }

    fn snapshot(state: &MapState) -> StateSnapshot {
        StateSnapshot {
            zoom: state.zoom,
            center: state.center,
            cursor: state.cursor,
            grid_size: state.grid_size,
            galaxy: state.galaxy,
            galaxy_name: state.galaxy_name.clone(),
            base_labels: state
                .base_labels
                .iter()
                .map(|label| {
                    (
                        label.letter,
                        label.name.clone(),
                        label.voxel_x,
                        label.voxel_z,
                    )
                })
                .collect(),
            player_pos: state.player_pos,
            zoom_stack: state.zoom_stack.clone(),
            show_help: state.show_help,
            should_quit: state.should_quit,
        }
    }

    fn route_event(state: &mut MapState, event: Event) -> MapAction {
        let calls = RefCell::new(Vec::new());
        let result = handle_input_with(
            state,
            |timeout| {
                calls.borrow_mut().push("poll");
                assert_eq!(timeout, Duration::from_millis(100));
                Ok(true)
            },
            || {
                calls.borrow_mut().push("read");
                Ok(event)
            },
        );
        assert_eq!(*calls.borrow(), ["poll", "read"]);
        match result {
            Ok(action) => action,
            Err(error) => panic!("unexpected input error: {error}"),
        }
    }

    fn assert_none(action: MapAction) {
        assert!(matches!(action, MapAction::None));
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn assert_snapshot(state: &MapState, expected: &StateSnapshot) {
        assert_eq!(&snapshot(state), expected);
    }

    #[test]
    fn test_handle_input_poll_timeout_and_error_preserve_state() {
        let mut state = seed_state();
        let before = snapshot(&state);
        let calls = RefCell::new(Vec::new());
        let result = handle_input_with(
            &mut state,
            |timeout| {
                calls.borrow_mut().push("poll");
                assert_eq!(timeout, Duration::from_millis(100));
                Ok(false)
            },
            || -> std::io::Result<Event> { panic!("read must not run after a poll timeout") },
        );
        assert!(matches!(result, Ok(MapAction::None)));
        assert_eq!(*calls.borrow(), ["poll"]);
        assert_snapshot(&state, &before);

        let calls = RefCell::new(Vec::new());
        let result = handle_input_with(
            &mut state,
            |timeout| {
                calls.borrow_mut().push("poll");
                assert_eq!(timeout, Duration::from_millis(100));
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "poll denied",
                ))
            },
            || -> std::io::Result<Event> { panic!("read must not run after a poll error") },
        );
        match result {
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
                assert_eq!(error.to_string(), "poll denied");
            }
            Ok(_) => panic!("poll error must be returned"),
        }
        assert_eq!(*calls.borrow(), ["poll"]);
        assert_snapshot(&state, &before);
    }

    #[test]
    fn test_handle_input_read_error_preserves_state() {
        let mut state = seed_state();
        let before = snapshot(&state);
        let calls = RefCell::new(Vec::new());
        let result = handle_input_with(
            &mut state,
            |timeout| {
                calls.borrow_mut().push("poll");
                assert_eq!(timeout, Duration::from_millis(100));
                Ok(true)
            },
            || {
                calls.borrow_mut().push("read");
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "read interrupted",
                ))
            },
        );
        match result {
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
                assert_eq!(error.to_string(), "read interrupted");
            }
            Ok(_) => panic!("read error must be returned"),
        }
        assert_eq!(*calls.borrow(), ["poll", "read"]);
        assert_snapshot(&state, &before);
    }

    #[test]
    fn test_handle_input_resize_and_ignored_events_update_expected_state() {
        let resize_cases = [
            ((5, 7), (3, 2), (2, 1)),
            ((1, 3), (0, 0), (6, 3)),
            ((1, 9), (0, 4), (6, 3)),
            ((10, 3), (8, 0), (6, 3)),
            ((14, 12), (12, 7), (6, 3)),
        ];
        for (dimensions, grid_size, cursor) in resize_cases {
            let mut state = seed_state();
            state.cursor = (6, 3);
            let mut expected = snapshot(&state);
            expected.grid_size = grid_size;
            expected.cursor = cursor;
            assert!(matches!(
                route_event(&mut state, Event::Resize(dimensions.0, dimensions.1)),
                MapAction::Resized(width, height) if (width, height) == dimensions
            ));
            assert_snapshot(&state, &expected);
        }

        let ignored = [
            Event::FocusGained,
            Event::FocusLost,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 2,
                row: 3,
                modifiers: KeyModifiers::NONE,
            }),
        ];
        for event in ignored {
            let mut state = seed_state();
            let before = snapshot(&state);
            assert_none(route_event(&mut state, event));
            assert_snapshot(&state, &before);
        }
    }

    #[test]
    fn test_handle_input_navigation_edges_and_key_kinds_move_cursor() {
        let navigation = [
            (KeyCode::Up, (2, 0)),
            (KeyCode::Down, (2, 2)),
            (KeyCode::Left, (1, 1)),
            (KeyCode::Right, (3, 1)),
        ];
        for (code, cursor) in navigation {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.cursor = cursor;
            assert_none(route_event(&mut state, key(code, KeyModifiers::NONE)));
            assert_snapshot(&state, &expected);
        }

        for (initial_cursor, code, expected_cursor) in [
            ((0, 0), KeyCode::Up, (0, 0)),
            ((0, 0), KeyCode::Left, (0, 0)),
            ((7, 3), KeyCode::Down, (7, 3)),
            ((7, 3), KeyCode::Right, (7, 3)),
        ] {
            let mut state = seed_state();
            state.cursor = initial_cursor;
            let mut expected = snapshot(&state);
            expected.cursor = expected_cursor;
            assert_none(route_event(&mut state, key(code, KeyModifiers::NONE)));
            assert_snapshot(&state, &expected);
        }

        for kind in [
            KeyEventKind::Press,
            KeyEventKind::Repeat,
            KeyEventKind::Release,
        ] {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.cursor = (3, 1);
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Right,
                KeyModifiers::NONE,
                kind,
            ));
            assert_none(route_event(&mut state, event));
            assert_snapshot(&state, &expected);
        }

        let mut state = seed_state();
        let mut expected = snapshot(&state);
        expected.cursor = (3, 1);
        assert_none(route_event(
            &mut state,
            key(KeyCode::Right, KeyModifiers::ALT),
        ));
        assert_snapshot(&state, &expected);
    }

    #[test]
    fn test_handle_input_zoom_aliases_restore_literal_states() {
        for zoom_key in [KeyCode::Enter, KeyCode::Char('+')] {
            let mut state = seed_state();

            let mut expected = snapshot(&state);
            expected.zoom = super::super::state::ZoomLevel::Region;
            expected.center = (-1008.0, -1032.0);
            expected.cursor = (4, 2);
            expected.zoom_stack = vec![(super::super::state::ZoomLevel::Galaxy, 16.0, -8.0)];
            assert_none(route_event(&mut state, key(zoom_key, KeyModifiers::NONE)));
            assert_snapshot(&state, &expected);

            expected.cursor = (5, 2);
            assert_none(route_event(
                &mut state,
                key(KeyCode::Right, KeyModifiers::NONE),
            ));
            assert_snapshot(&state, &expected);

            expected.zoom = super::super::state::ZoomLevel::Local;
            expected.center = (-944.0, -1032.0);
            expected.cursor = (4, 2);
            expected.zoom_stack = vec![
                (super::super::state::ZoomLevel::Galaxy, 16.0, -8.0),
                (super::super::state::ZoomLevel::Region, -1008.0, -1032.0),
            ];
            assert_none(route_event(
                &mut state,
                key(KeyCode::Char('+'), KeyModifiers::NONE),
            ));
            assert_snapshot(&state, &expected);

            assert_none(route_event(
                &mut state,
                key(KeyCode::Enter, KeyModifiers::NONE),
            ));
            assert_snapshot(&state, &expected);

            expected.zoom = super::super::state::ZoomLevel::Region;
            expected.center = (-1008.0, -1032.0);
            expected.cursor = (4, 2);
            expected.zoom_stack = vec![(super::super::state::ZoomLevel::Galaxy, 16.0, -8.0)];
            assert_none(route_event(
                &mut state,
                key(KeyCode::Esc, KeyModifiers::NONE),
            ));
            assert_snapshot(&state, &expected);

            expected.zoom = super::super::state::ZoomLevel::Galaxy;
            expected.center = (16.0, -8.0);
            expected.cursor = (4, 2);
            expected.zoom_stack.clear();
            assert_none(route_event(
                &mut state,
                key(KeyCode::Char('-'), KeyModifiers::NONE),
            ));
            assert_snapshot(&state, &expected);
        }

        for exit_key in [KeyCode::Esc, KeyCode::Char('-')] {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.should_quit = true;
            assert_none(route_event(&mut state, key(exit_key, KeyModifiers::NONE)));
            assert_snapshot(&state, &expected);
        }
    }

    #[test]
    fn test_handle_input_command_modifiers_and_quit_persistence() {
        let mut state = seed_state();
        let mut expected = snapshot(&state);
        expected.center = (100.0, 200.0);
        expected.cursor = (4, 2);
        assert_none(route_event(
            &mut state,
            key(KeyCode::Char('c'), KeyModifiers::NONE),
        ));
        assert_snapshot(&state, &expected);

        let mut state = seed_state();
        state.player_pos = None;
        let before = snapshot(&state);
        assert_none(route_event(
            &mut state,
            key(KeyCode::Char('c'), KeyModifiers::NONE),
        ));
        assert_snapshot(&state, &before);

        for event in [
            key(KeyCode::Char('c'), KeyModifiers::SHIFT),
            key(KeyCode::Char('C'), KeyModifiers::NONE),
            key(KeyCode::Char('x'), KeyModifiers::NONE),
        ] {
            let mut state = seed_state();
            let before = snapshot(&state);
            assert_none(route_event(&mut state, event));
            assert_snapshot(&state, &before);
        }

        for modifiers in [
            KeyModifiers::CONTROL,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ] {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.should_quit = true;
            assert_none(route_event(&mut state, key(KeyCode::Char('c'), modifiers)));
            assert_snapshot(&state, &expected);
        }

        for modifiers in [KeyModifiers::NONE, KeyModifiers::SHIFT] {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.should_quit = true;
            assert_none(route_event(&mut state, key(KeyCode::Char('q'), modifiers)));
            assert_snapshot(&state, &expected);
        }

        for modifiers in [KeyModifiers::NONE, KeyModifiers::ALT] {
            let mut state = seed_state();
            let mut expected = snapshot(&state);
            expected.show_help = true;
            assert_none(route_event(&mut state, key(KeyCode::Char('?'), modifiers)));
            assert_snapshot(&state, &expected);
        }

        let mut state = seed_state();
        state.should_quit = true;
        let before = snapshot(&state);
        assert_none(route_event(
            &mut state,
            key(KeyCode::Char('x'), KeyModifiers::NONE),
        ));
        assert_snapshot(&state, &before);
    }

    #[test]
    fn test_handle_input_help_consumes_keys_but_not_resize() {
        let help_keys = [
            (KeyCode::Up, KeyModifiers::NONE),
            (KeyCode::Enter, KeyModifiers::NONE),
            (KeyCode::Char('+'), KeyModifiers::NONE),
            (KeyCode::Esc, KeyModifiers::NONE),
            (KeyCode::Char('-'), KeyModifiers::NONE),
            (KeyCode::Char('q'), KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
            (KeyCode::Char('?'), KeyModifiers::NONE),
        ];
        for (code, modifiers) in help_keys {
            let mut state = seed_state();
            state.show_help = true;
            let mut expected = snapshot(&state);
            expected.show_help = false;
            assert_none(route_event(&mut state, key(code, modifiers)));
            assert_snapshot(&state, &expected);
        }

        let mut state = seed_state();
        state.show_help = true;
        state.cursor = (6, 3);
        let mut expected = snapshot(&state);
        expected.grid_size = (3, 2);
        expected.cursor = (2, 1);
        assert!(matches!(
            route_event(&mut state, Event::Resize(5, 7)),
            MapAction::Resized(5, 7)
        ));
        assert_snapshot(&state, &expected);
    }
}
