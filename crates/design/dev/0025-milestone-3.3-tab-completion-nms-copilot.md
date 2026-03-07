# Milestone 3.3 -- Tab Completion (nms-copilot)

Context-aware tab completion for the REPL. Completes command names, subcommand names, flag names, biome names, base names, and system names from the loaded model.

## Crate: `nms-copilot`

Path: `crates/nms-copilot/`

### Dependencies

No new dependencies -- reedline's `Completer` trait is built-in.

---

## New File: `crates/nms-copilot/src/completer.rs`

Implements reedline's `Completer` trait with context-aware completions.

```rust
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

/// Static command names.
const COMMANDS: &[&str] = &[
    "find", "show", "stats", "convert", "info", "help", "exit", "quit",
];

/// Show subcommands.
const SHOW_SUBCOMMANDS: &[&str] = &["system", "base"];

/// Find flags.
const FIND_FLAGS: &[&str] = &[
    "--biome",
    "--infested",
    "--within",
    "--nearest",
    "--named",
    "--discoverer",
    "--from",
];

/// Stats flags.
const STATS_FLAGS: &[&str] = &["--biomes", "--discoveries"];

/// Convert flags.
const CONVERT_FLAGS: &[&str] = &[
    "--glyphs", "--coords", "--ga", "--voxel", "--ssi", "--planet", "--galaxy",
];

/// Known biome names.
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

        // Determine what we're completing
        let (partial, candidates) = match words.as_slice() {
            // Empty or first word: complete command names
            [] => ("", COMMANDS.to_vec()),
            [partial] if !line_to_pos.ends_with(' ') => {
                (*partial, COMMANDS.to_vec())
            }

            // After "show": complete subcommands
            ["show"] if line_to_pos.ends_with(' ') => {
                ("", SHOW_SUBCOMMANDS.to_vec())
            }
            ["show", partial] if !line_to_pos.ends_with(' ') => {
                (*partial, SHOW_SUBCOMMANDS.to_vec())
            }

            // After "show base": complete base names
            ["show", "base"] if line_to_pos.ends_with(' ') => {
                return self.complete_names("", &self.model_data.base_names, pos);
            }
            ["show", "base", partial] if !line_to_pos.ends_with(' ') => {
                return self.complete_names(partial, &self.model_data.base_names, pos);
            }

            // After "show system": complete system names
            ["show", "system"] if line_to_pos.ends_with(' ') => {
                return self.complete_names("", &self.model_data.system_names, pos);
            }
            ["show", "system", partial] if !line_to_pos.ends_with(' ') => {
                return self.complete_names(partial, &self.model_data.system_names, pos);
            }

            // After "find": complete flags or biome values
            [cmd, ..] if *cmd == "find" => {
                return self.complete_find_context(line_to_pos, &words, pos);
            }

            // After "stats": complete flags
            ["stats", ..] => {
                let partial = if line_to_pos.ends_with(' ') {
                    ""
                } else {
                    words.last().copied().unwrap_or("")
                };
                (partial, STATS_FLAGS.to_vec())
            }

            // After "convert": complete flags
            ["convert", ..] => {
                let partial = if line_to_pos.ends_with(' ') {
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

        // If the previous word is --biome, complete biome names
        let prev = if line_to_pos.ends_with(' ') {
            words.last().copied()
        } else if words.len() >= 2 {
            Some(words[words.len() - 2])
        } else {
            None
        };

        if prev == Some("--biome") {
            return self.filter_suggestions(last, &BIOME_NAMES.to_vec(), pos);
        }

        // If the previous word is --from, complete base names
        if prev == Some("--from") {
            return self.complete_names(last, &self.model_data.base_names, pos);
        }

        // Otherwise, complete flags
        self.filter_suggestions(last, &FIND_FLAGS.to_vec(), pos)
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
```

---

## Modified File: `crates/nms-copilot/src/main.rs`

Wire the completer into the reedline editor.

```rust
// Add import:
use reedline::DefaultCompleter; // if using built-in, or our custom one

mod completer;

// After loading the model, build the completer:
fn build_model_completions(model: &GalaxyModel) -> completer::ModelCompletions {
    let base_names: Vec<String> = model.bases.keys().cloned().collect();
    let system_names: Vec<String> = model.systems.values()
        .filter_map(|s| s.name.clone())
        .collect();

    completer::ModelCompletions {
        base_names,
        system_names,
    }
}

// In main(), after model is loaded:
// let completions = build_model_completions(&model);
// let completer = Box::new(completer::CopilotCompleter::new(completions));

// Update build_editor to accept the completer:
fn build_editor(completer: Box<completer::CopilotCompleter>) -> Reedline {
    // ... history setup from 3.2 ...
    editor.with_completer(completer)
}
```

---

## Modified File: `crates/nms-copilot/src/lib.rs`

```rust
pub mod commands;
pub mod completer;
pub mod dispatch;
pub mod paths;
```

---

## Tests

### File: `crates/nms-copilot/src/completer.rs` (inline tests)

```rust
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
            system_names: vec![
                "Gugestor Colony".into(),
                "Esurad".into(),
            ],
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
        // Names with spaces should be quoted
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
}
```

---

## Implementation Notes

1. **Context-aware completion**: The completer examines the words before the cursor to determine what kind of completions to offer. After `find --biome`, it suggests biome names. After `show base`, it suggests base names from the model.

2. **Quoted multi-word names**: Base and system names containing spaces are wrapped in double quotes when completed, matching the `shell_words()` parser from milestone 3.1.

3. **Case-insensitive matching**: Prefix matching is case-insensitive, so typing `show base aca<TAB>` completes to `"Acadia National Park"`.

4. **Result limit**: Name completions are limited to 20 results to avoid overwhelming the display for large models.

5. **Static vs dynamic data**: Command names, flags, and biome names are static constants. Base names and system names come from the loaded `GalaxyModel`. If the model is updated (Phase 5), the completer's `ModelCompletions` would need refreshing.

6. **reedline Completer trait**: Returns `Vec<Suggestion>` with `Span` indicating which part of the input to replace. The span calculation uses `pos - partial.len()` to replace only the partial word being completed.
