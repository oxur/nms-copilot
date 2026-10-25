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
/// If setup fails, the guard attempts restoration for state already acquired
/// or commands already attempted, while preserving the setup error. After the
/// loop finishes, restoration attempts every applicable operation in order.
/// A restoration error takes precedence over the loop result. On unwinding,
/// the guard makes best-effort restoration attempts and ignores their errors;
/// this cannot guarantee that the physical terminal is restored.
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
/// construction or sizing, drawing, event polling or reading, or terminal
/// restoration. Setup errors remain primary if guard cleanup also fails. Once
/// the loop returns, the first restoration error takes precedence over its
/// result; later applicable restoration operations are still attempted.
///
/// # Panics
///
/// A panic in map processing unwinds through the terminal guard, which makes
/// best-effort restoration attempts and ignores their errors. Restoration is
/// not guaranteed after operating-system failure, process abort or kill, a
/// forgotten guard, or a second panic while unwinding. See
/// [`state::MapState::move_cursor`] for its numeric limit.
pub fn run_map(model: &GalaxyModel, session: &SessionState) -> io::Result<()> {
    with_terminal_control(CrosstermControl, || {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        run_map_loop(&mut terminal, model, session)
    })
}

trait TerminalControl {
    fn enable_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn disable_raw_mode(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
    fn leave_alternate_screen(&mut self) -> io::Result<()>;
}

struct CrosstermControl;

impl TerminalControl for CrosstermControl {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Hide)
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Show)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)
    }
}

struct TerminalGuard<C: TerminalControl> {
    control: C,
    raw_enabled: bool,
    alternate_attempted: bool,
    cursor_attempted: bool,
}

impl<C: TerminalControl> TerminalGuard<C> {
    fn enter(control: C) -> io::Result<Self> {
        let mut guard = Self {
            control,
            raw_enabled: false,
            alternate_attempted: false,
            cursor_attempted: false,
        };

        guard.control.enable_raw_mode()?;
        guard.raw_enabled = true;
        guard.alternate_attempted = true;
        guard.control.enter_alternate_screen()?;
        guard.cursor_attempted = true;
        guard.control.hide_cursor()?;

        Ok(guard)
    }

    fn restore(&mut self) -> io::Result<()> {
        let disable_raw = if self.raw_enabled {
            self.raw_enabled = false;
            self.control.disable_raw_mode()
        } else {
            Ok(())
        };
        let show_cursor = if self.cursor_attempted {
            self.cursor_attempted = false;
            self.control.show_cursor()
        } else {
            Ok(())
        };
        let leave_alternate = if self.alternate_attempted {
            self.alternate_attempted = false;
            self.control.leave_alternate_screen()
        } else {
            Ok(())
        };

        disable_raw.and(show_cursor).and(leave_alternate)
    }
}

impl<C: TerminalControl> Drop for TerminalGuard<C> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn with_terminal_control<C, F>(control: C, body: F) -> io::Result<()>
where
    C: TerminalControl,
    F: FnOnce() -> io::Result<()>,
{
    let mut guard = TerminalGuard::enter(control)?;
    let body_result = body();
    let cleanup_result = guard.restore();
    cleanup_result.and(body_result)
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;

    use super::{TerminalControl, with_terminal_control};
    use std::io;

    struct MockControl {
        trace: Rc<RefCell<Vec<&'static str>>>,
        setup_failure: Option<&'static str>,
        cleanup_failures: &'static [&'static str],
    }

    impl MockControl {
        fn operation(&mut self, name: &'static str, setup: bool) -> io::Result<()> {
            self.trace.borrow_mut().push(name);
            let fails = if setup {
                self.setup_failure == Some(name)
            } else {
                self.cleanup_failures.contains(&name)
            };
            if fails {
                let kind = if setup {
                    io::ErrorKind::PermissionDenied
                } else {
                    io::ErrorKind::BrokenPipe
                };
                let phase = if setup { "setup" } else { "cleanup" };
                Err(io::Error::new(kind, format!("{phase} {name}")))
            } else {
                Ok(())
            }
        }
    }

    impl TerminalControl for MockControl {
        fn enable_raw_mode(&mut self) -> io::Result<()> {
            self.operation("enable_raw_mode", true)
        }

        fn enter_alternate_screen(&mut self) -> io::Result<()> {
            self.operation("enter_alternate_screen", true)
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            self.operation("hide_cursor", true)
        }

        fn disable_raw_mode(&mut self) -> io::Result<()> {
            self.operation("disable_raw_mode", false)
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            self.operation("show_cursor", false)
        }

        fn leave_alternate_screen(&mut self) -> io::Result<()> {
            self.operation("leave_alternate_screen", false)
        }
    }

    impl Drop for MockControl {
        fn drop(&mut self) {
            self.trace.borrow_mut().push("control_drop");
        }
    }

    fn mock(
        trace: &Rc<RefCell<Vec<&'static str>>>,
        setup_failure: Option<&'static str>,
        cleanup_failures: &'static [&'static str],
    ) -> MockControl {
        MockControl {
            trace: Rc::clone(trace),
            setup_failure,
            cleanup_failures,
        }
    }

    fn assert_error(error: io::Error, kind: io::ErrorKind, message: &str) {
        assert_eq!(error.kind(), kind);
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn test_with_terminal_control_success_restores_in_order_and_drops_owner() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let result = with_terminal_control(mock(&trace, None, &[]), || {
            trace.borrow_mut().push("body");
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(
            *trace.borrow(),
            [
                "enable_raw_mode",
                "enter_alternate_screen",
                "hide_cursor",
                "body",
                "disable_raw_mode",
                "show_cursor",
                "leave_alternate_screen",
                "control_drop",
            ]
        );
        assert_eq!(Rc::strong_count(&trace), 1);
    }

    #[test]
    fn test_terminal_guard_setup_failures_preserve_error_and_restore_attempts() {
        let cases: [(&'static str, &'static [&'static str]); 3] = [
            ("enable_raw_mode", &["enable_raw_mode", "control_drop"]),
            (
                "enter_alternate_screen",
                &[
                    "enable_raw_mode",
                    "enter_alternate_screen",
                    "disable_raw_mode",
                    "leave_alternate_screen",
                    "control_drop",
                ],
            ),
            (
                "hide_cursor",
                &[
                    "enable_raw_mode",
                    "enter_alternate_screen",
                    "hide_cursor",
                    "disable_raw_mode",
                    "show_cursor",
                    "leave_alternate_screen",
                    "control_drop",
                ],
            ),
        ];

        for (setup_failure, expected_trace) in cases {
            for cleanup_failures in [
                &[][..],
                &["disable_raw_mode", "show_cursor", "leave_alternate_screen"][..],
            ] {
                let trace = Rc::new(RefCell::new(Vec::new()));
                let result = with_terminal_control(
                    mock(&trace, Some(setup_failure), cleanup_failures),
                    || {
                        trace.borrow_mut().push("body");
                        Ok(())
                    },
                );

                let error = result.expect_err("setup failure must be returned");
                assert_error(
                    error,
                    io::ErrorKind::PermissionDenied,
                    &format!("setup {setup_failure}"),
                );
                assert_eq!(*trace.borrow(), *expected_trace);
                assert_eq!(Rc::strong_count(&trace), 1);
            }
        }
    }

    #[test]
    fn test_terminal_guard_cleanup_failures_precede_body_and_all_run() {
        let cases: [(&'static [&'static str], Option<&'static str>); 8] = [
            (&[], None),
            (&["disable_raw_mode"], Some("disable_raw_mode")),
            (&["show_cursor"], Some("show_cursor")),
            (&["leave_alternate_screen"], Some("leave_alternate_screen")),
            (
                &["disable_raw_mode", "show_cursor"],
                Some("disable_raw_mode"),
            ),
            (
                &["disable_raw_mode", "leave_alternate_screen"],
                Some("disable_raw_mode"),
            ),
            (
                &["show_cursor", "leave_alternate_screen"],
                Some("show_cursor"),
            ),
            (
                &["disable_raw_mode", "show_cursor", "leave_alternate_screen"],
                Some("disable_raw_mode"),
            ),
        ];

        for (cleanup_failures, first_cleanup_failure) in cases {
            for body_fails in [false, true] {
                let trace = Rc::new(RefCell::new(Vec::new()));
                let result = with_terminal_control(mock(&trace, None, cleanup_failures), || {
                    trace.borrow_mut().push("body");
                    if body_fails {
                        Err(io::Error::new(io::ErrorKind::Interrupted, "body failed"))
                    } else {
                        Ok(())
                    }
                });

                assert_eq!(
                    *trace.borrow(),
                    [
                        "enable_raw_mode",
                        "enter_alternate_screen",
                        "hide_cursor",
                        "body",
                        "disable_raw_mode",
                        "show_cursor",
                        "leave_alternate_screen",
                        "control_drop",
                    ]
                );
                assert_eq!(Rc::strong_count(&trace), 1);
                match first_cleanup_failure {
                    Some(operation) => assert_error(
                        result.expect_err("cleanup error must take precedence"),
                        io::ErrorKind::BrokenPipe,
                        &format!("cleanup {operation}"),
                    ),
                    None if body_fails => assert_error(
                        result.expect_err("body error must survive successful cleanup"),
                        io::ErrorKind::Interrupted,
                        "body failed",
                    ),
                    None => assert!(result.is_ok()),
                }
            }
        }
    }

    #[test]
    fn test_terminal_guard_unwind_restores_and_preserves_panic_payload() {
        for cleanup_failures in [
            &[][..],
            &["disable_raw_mode", "show_cursor", "leave_alternate_screen"][..],
        ] {
            let trace = Rc::new(RefCell::new(Vec::new()));
            let panic = catch_unwind(AssertUnwindSafe(|| {
                let _ = with_terminal_control(mock(&trace, None, cleanup_failures), || {
                    trace.borrow_mut().push("body");
                    std::panic::panic_any("map body panic");
                });
            }));

            let payload = panic.expect_err("body panic must continue unwinding");
            assert_eq!(
                payload.downcast_ref::<&'static str>(),
                Some(&"map body panic")
            );
            assert_eq!(
                *trace.borrow(),
                [
                    "enable_raw_mode",
                    "enter_alternate_screen",
                    "hide_cursor",
                    "body",
                    "disable_raw_mode",
                    "show_cursor",
                    "leave_alternate_screen",
                    "control_drop",
                ]
            );
            assert_eq!(Rc::strong_count(&trace), 1);
        }
    }
}
