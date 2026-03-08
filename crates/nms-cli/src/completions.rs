//! `nms completions` command -- generate shell completions.

use std::io;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::Cli;

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
