//! Generate shell completion text from the current `Cli` definition.
//!
//! Output goes to stdout; generation returns `Ok(())` without reporting stdout write failures.
//! Pinned Bash, Zsh, and Fish generators may panic on generation errors. This command does not
//! install or source output.
//!
//! # Examples
//!
//! ```text
//! nms completions bash > nms.bash
//! ```
//!
//! The caller chooses the destination; this command-text example is not executed here.

use std::io;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::Cli;

/// Generate completions for `shell` from the current command definition.
///
/// Writes to stdout and returns `Ok(())` after generation; its `Result` does not report stdout
/// write failures.
///
/// # Examples
///
/// ```text
/// nms completions bash > nms.bash
/// ```
///
/// # Errors
///
/// The result is currently always `Ok(())` after generation returns.
///
/// # Panics
///
/// Command construction or a pinned generator may panic on failure.
pub fn run(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completions_bash_generates_output() {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();
        generate(Shell::Bash, &mut cmd, "nms", &mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("nms"));
        assert!(!s.is_empty());
    }

    #[test]
    fn test_completions_zsh_generates_output() {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();
        generate(Shell::Zsh, &mut cmd, "nms", &mut buf);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_completions_fish_generates_output() {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();
        generate(Shell::Fish, &mut cmd, "nms", &mut buf);
        assert!(!buf.is_empty());
    }
}
