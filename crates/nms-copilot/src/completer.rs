//! Tab completion for the NMS Copilot REPL.
//!
//! Provides context-aware completions:
//! - Command names (find, show, stats, convert, info, help, exit, quit)
//! - Subcommand names (show system, show base)
//! - Flag names (--biome, --nearest, etc.)
//! - Biome names from the Biome enum
//! - Base names from the loaded model
//! - System names from the loaded model

use reedline::{Completer, Span, Suggestion};

/// Completions that depend on the loaded galaxy model.
#[derive(Clone)]
pub struct ModelCompletions {
    /// Known base names (original casing).
    pub base_names: Vec<String>,
    /// Known system names (original casing).
    pub system_names: Vec<String>,
}

/// REPL tab completer with static command knowledge and dynamic model data.
pub struct CopilotCompleter {
    model_data: ModelCompletions,
}

impl CopilotCompleter {
    pub fn new(model_data: ModelCompletions) -> Self {
        Self { model_data }
    }
}

const COMMANDS: &[&str] = &[
    "find", "show", "stats", "convert", "set", "reset", "status", "info", "help", "exit", "quit",
];

const SHOW_SUBCOMMANDS: &[&str] = &["system", "base"];

const FIND_FLAGS: &[&str] = &[
    "--biome",
    "--infested",
    "--within",
    "--nearest",
    "--named",
    "--discoverer",
    "--from",
];

const STATS_FLAGS: &[&str] = &["--biomes", "--discoveries"];

const CONVERT_FLAGS: &[&str] = &[
    "--glyphs", "--coords", "--ga", "--voxel", "--ssi", "--planet", "--galaxy",
];

const SET_SUBCOMMANDS: &[&str] = &["position", "biome", "warp-range"];

const RESET_TARGETS: &[&str] = &["position", "biome", "warp-range", "all"];

const BIOME_NAMES: &[&str] = &[
    "Lush",
    "Toxic",
    "Scorched",
    "Radioactive",
    "Frozen",
    "Barren",
    "Dead",
    "Weird",
    "Red",
    "Green",
    "Blue",
    "Swamp",
    "Lava",
    "Waterworld",
];

impl Completer for CopilotCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let line_to_pos = &line[..pos];
        let words: Vec<&str> = line_to_pos.split_whitespace().collect();
        // Lowercase words for case-insensitive context matching
        let lower: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
        let lower_refs: Vec<&str> = lower.iter().map(|s| s.as_str()).collect();
        let trailing_space = line_to_pos.ends_with(' ');

        let (partial, candidates) = match lower_refs.as_slice() {
            [] => ("", COMMANDS.to_vec()),
            [_] if !trailing_space => (words[0], COMMANDS.to_vec()),

            ["show"] if trailing_space => ("", SHOW_SUBCOMMANDS.to_vec()),
            ["show", _] if !trailing_space => (words[1], SHOW_SUBCOMMANDS.to_vec()),

            ["show", "base"] if trailing_space => {
                return self.complete_names("", &self.model_data.base_names, pos);
            }
            ["show", "base", _] if !trailing_space => {
                return self.complete_names(words[2], &self.model_data.base_names, pos);
            }

            ["show", "system"] if trailing_space => {
                return self.complete_names("", &self.model_data.system_names, pos);
            }
            ["show", "system", _] if !trailing_space => {
                return self.complete_names(words[2], &self.model_data.system_names, pos);
            }

            ["set"] if trailing_space => ("", SET_SUBCOMMANDS.to_vec()),
            ["set", _] if !trailing_space => (words[1], SET_SUBCOMMANDS.to_vec()),

            ["set", "biome"] if trailing_space => {
                return self.filter_suggestions("", BIOME_NAMES, pos);
            }
            ["set", "biome", _] if !trailing_space => {
                return self.filter_suggestions(words[2], BIOME_NAMES, pos);
            }

            ["set", "position"] if trailing_space => {
                return self.complete_names("", &self.model_data.base_names, pos);
            }
            ["set", "position", _] if !trailing_space => {
                return self.complete_names(words[2], &self.model_data.base_names, pos);
            }

            ["reset"] if trailing_space => ("", RESET_TARGETS.to_vec()),
            ["reset", _] if !trailing_space => (words[1], RESET_TARGETS.to_vec()),

            [cmd, ..] if *cmd == "find" => {
                return self.complete_find_context(line_to_pos, &words, pos);
            }

            [cmd, ..] if *cmd == "stats" => {
                let partial = if trailing_space {
                    ""
                } else {
                    words.last().copied().unwrap_or("")
                };
                (partial, STATS_FLAGS.to_vec())
            }

            [cmd, ..] if *cmd == "convert" => {
                let partial = if trailing_space {
                    ""
                } else {
                    words.last().copied().unwrap_or("")
                };
                (partial, CONVERT_FLAGS.to_vec())
            }

            _ => return vec![],
        };

        self.filter_suggestions(partial, &candidates, pos)
    }
}

impl CopilotCompleter {
    fn complete_find_context(
        &self,
        line_to_pos: &str,
        words: &[&str],
        pos: usize,
    ) -> Vec<Suggestion> {
        let last = if line_to_pos.ends_with(' ') {
            ""
        } else {
            words.last().copied().unwrap_or("")
        };

        let prev = if line_to_pos.ends_with(' ') {
            words.last().copied()
        } else if words.len() >= 2 {
            Some(words[words.len() - 2])
        } else {
            None
        };

        if prev == Some("--biome") {
            return self.filter_suggestions(last, BIOME_NAMES, pos);
        }

        if prev == Some("--from") {
            return self.complete_names(last, &self.model_data.base_names, pos);
        }

        self.filter_suggestions(last, FIND_FLAGS, pos)
    }

    fn complete_names(&self, partial: &str, names: &[String], pos: usize) -> Vec<Suggestion> {
        let lower = partial.to_lowercase();
        names
            .iter()
            .filter(|n| n.to_lowercase().starts_with(&lower))
            .take(20)
            .map(|n| {
                let value = if n.contains(' ') {
                    format!("\"{n}\"")
                } else {
                    n.clone()
                };
                Suggestion {
                    value,
                    description: None,
                    style: None,
                    extra: None,
                    span: Span::new(pos - partial.len(), pos),
                    append_whitespace: true,
                }
            })
            .collect()
    }

    fn filter_suggestions(
        &self,
        partial: &str,
        candidates: &[&str],
        pos: usize,
    ) -> Vec<Suggestion> {
        let lower = partial.to_lowercase();
        candidates
            .iter()
            .filter(|c| c.to_lowercase().starts_with(&lower))
            .map(|c| Suggestion {
                value: c.to_string(),
                description: None,
                style: None,
                extra: None,
                span: Span::new(pos - partial.len(), pos),
                append_whitespace: true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_completer() -> CopilotCompleter {
        CopilotCompleter::new(ModelCompletions {
            base_names: vec![
                "Acadia National Park".into(),
                "Alpha Base".into(),
                "Beta Station".into(),
            ],
            system_names: vec!["Gugestor Colony".into(), "Esurad".into()],
        })
    }

    #[test]
    fn test_complete_empty_line_shows_commands() {
        let mut c = test_completer();
        let results = c.complete("", 0);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"find"));
        assert!(values.contains(&"show"));
        assert!(values.contains(&"exit"));
    }

    #[test]
    fn test_complete_partial_command() {
        let mut c = test_completer();
        let results = c.complete("fi", 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "find");
    }

    #[test]
    fn test_complete_show_subcommands() {
        let mut c = test_completer();
        let results = c.complete("show ", 5);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"system"));
        assert!(values.contains(&"base"));
    }

    #[test]
    fn test_complete_show_base_names() {
        let mut c = test_completer();
        let results = c.complete("show base A", 11);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.iter().any(|v| v.contains("Acadia")));
        assert!(values.iter().any(|v| v.contains("Alpha")));
    }

    #[test]
    fn test_complete_base_name_with_spaces_is_quoted() {
        let mut c = test_completer();
        let results = c.complete("show base Aca", 13);
        assert!(!results.is_empty());
        assert!(results[0].value.starts_with('"'));
    }

    #[test]
    fn test_complete_find_flags() {
        let mut c = test_completer();
        let results = c.complete("find --b", 8);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"--biome"));
    }

    #[test]
    fn test_complete_biome_after_flag() {
        let mut c = test_completer();
        let results = c.complete("find --biome L", 14);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"Lush"));
        assert!(values.contains(&"Lava"));
    }

    #[test]
    fn test_complete_from_base_names() {
        let mut c = test_completer();
        let results = c.complete("find --from B", 13);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.iter().any(|v| v.contains("Beta")));
    }

    #[test]
    fn test_complete_show_system_names() {
        let mut c = test_completer();
        let results = c.complete("show system G", 13);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.iter().any(|v| v.contains("Gugestor")));
    }

    #[test]
    fn test_complete_stats_flags() {
        let mut c = test_completer();
        let results = c.complete("stats --b", 9);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"--biomes"));
    }

    #[test]
    fn test_complete_convert_flags() {
        let mut c = test_completer();
        let results = c.complete("convert --g", 11);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"--glyphs"));
        assert!(values.contains(&"--ga"));
        assert!(values.contains(&"--galaxy"));
    }

    #[test]
    fn test_complete_case_insensitive_command() {
        let mut c = test_completer();
        // Typing "FI" should still match "find"
        let results = c.complete("FI", 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "find");
    }

    #[test]
    fn test_complete_case_insensitive_show_subcommand() {
        let mut c = test_completer();
        let results = c.complete("SHOW ", 5);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"system"));
        assert!(values.contains(&"base"));
    }

    #[test]
    fn test_complete_case_insensitive_show_base() {
        let mut c = test_completer();
        let results = c.complete("Show Base a", 11);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.iter().any(|v| v.contains("Acadia")));
    }

    #[test]
    fn test_complete_case_insensitive_find_flags() {
        let mut c = test_completer();
        let results = c.complete("FIND --b", 8);
        let values: Vec<&str> = results.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"--biome"));
    }
}
