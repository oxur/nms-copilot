# Milestone 3.5 -- Context-Aware Prompt (nms-copilot)

Dynamic REPL prompt that displays the current galaxy, active filters, model size, and a rocket emoji: `[Euclid | Lush | 644 planets] 🚀`.

## Crate: `nms-copilot`

Path: `crates/nms-copilot/`

### Dependencies

No new dependencies -- reedline's `Prompt` trait is built-in.

---

## New File: `crates/nms-copilot/src/prompt.rs`

Custom reedline prompt that reads session state.

```rust
//! Context-aware REPL prompt.
//!
//! Displays current galaxy, active biome filter, and model size.
//! Format: `[Euclid | Lush | 644 planets] 🚀 `

use std::borrow::Cow;

use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};

use crate::session::SessionState;

/// A snapshot of session state used to render the prompt.
///
/// We take a snapshot rather than holding a reference to SessionState
/// because reedline's Prompt trait requires `&self` (not mutable),
/// and the session state changes between prompts.
#[derive(Debug, Clone)]
pub struct PromptState {
    pub galaxy_name: String,
    pub biome_filter: Option<String>,
    pub planet_count: usize,
}

impl PromptState {
    /// Build a prompt state snapshot from the current session.
    pub fn from_session(session: &SessionState) -> Self {
        Self {
            galaxy_name: session.galaxy.name.to_string(),
            biome_filter: session.biome_filter.map(|b| format!("{b:?}")),
            planet_count: session.planet_count,
        }
    }
}

/// Custom REPL prompt.
pub struct CopilotPrompt {
    state: PromptState,
}

impl CopilotPrompt {
    pub fn new(state: PromptState) -> Self {
        Self { state }
    }

    /// Update the prompt state (called before each read_line).
    pub fn update(&mut self, state: PromptState) {
        self.state = state;
    }

    fn render_left(&self) -> String {
        let mut parts = vec![self.state.galaxy_name.clone()];

        if let Some(ref biome) = self.state.biome_filter {
            parts.push(biome.clone());
        }

        parts.push(format!("{} planets", self.state.planet_count));

        format!("[{}] 🚀", parts.join(" | "))
    }
}

impl Prompt for CopilotPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(self.render_left())
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Borrowed(" ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "(failed) ",
        };
        Cow::Owned(format!("{prefix}(search: {}) ", history_search.term))
    }
}
```

---

## Modified File: `crates/nms-copilot/src/main.rs`

Replace `DefaultPrompt` with `CopilotPrompt`, updating it before each `read_line`:

```rust
mod prompt;

use prompt::{CopilotPrompt, PromptState};

// In main(), replace the DefaultPrompt with:
fn main() {
    // ... load model, create session ...

    let prompt_state = PromptState::from_session(&session);
    let mut prompt = CopilotPrompt::new(prompt_state);

    let mut editor = build_editor(/* ... */);

    loop {
        // Update prompt before each read
        prompt.update(PromptState::from_session(&session));

        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                // ... parse and dispatch ...
            }
            // ... Ctrl-C/D handling ...
        }
    }
}
```

---

## Modified File: `crates/nms-copilot/src/lib.rs`

```rust
pub mod commands;
pub mod completer;
pub mod dispatch;
pub mod paths;
pub mod prompt;
pub mod session;
```

---

## Tests

### File: `crates/nms-copilot/src/prompt.rs` (inline tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use reedline::Prompt;

    #[test]
    fn test_prompt_basic() {
        let state = PromptState {
            galaxy_name: "Euclid".into(),
            biome_filter: None,
            planet_count: 644,
        };
        let prompt = CopilotPrompt::new(state);
        let left = prompt.render_prompt_left();
        assert_eq!(left.as_ref(), "[Euclid | 644 planets] 🚀");
    }

    #[test]
    fn test_prompt_with_biome_filter() {
        let state = PromptState {
            galaxy_name: "Euclid".into(),
            biome_filter: Some("Lush".into()),
            planet_count: 42,
        };
        let prompt = CopilotPrompt::new(state);
        let left = prompt.render_prompt_left();
        assert_eq!(left.as_ref(), "[Euclid | Lush | 42 planets] 🚀");
    }

    #[test]
    fn test_prompt_different_galaxy() {
        let state = PromptState {
            galaxy_name: "Hilbert Dimension".into(),
            biome_filter: None,
            planet_count: 100,
        };
        let prompt = CopilotPrompt::new(state);
        let left = prompt.render_prompt_left();
        assert!(left.contains("Hilbert Dimension"));
    }

    #[test]
    fn test_prompt_indicator_is_space() {
        let state = PromptState {
            galaxy_name: "Euclid".into(),
            biome_filter: None,
            planet_count: 0,
        };
        let prompt = CopilotPrompt::new(state);
        assert_eq!(
            prompt.render_prompt_indicator(PromptEditMode::Default).as_ref(),
            " "
        );
    }

    #[test]
    fn test_prompt_update() {
        let state1 = PromptState {
            galaxy_name: "Euclid".into(),
            biome_filter: None,
            planet_count: 100,
        };
        let mut prompt = CopilotPrompt::new(state1);
        assert!(prompt.render_prompt_left().contains("100 planets"));

        let state2 = PromptState {
            galaxy_name: "Euclid".into(),
            biome_filter: Some("Toxic".into()),
            planet_count: 200,
        };
        prompt.update(state2);
        let left = prompt.render_prompt_left();
        assert!(left.contains("200 planets"));
        assert!(left.contains("Toxic"));
    }

    #[test]
    fn test_prompt_state_from_session() {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let model = nms_graph::GalaxyModel::from_save(&save);
        let session = crate::session::SessionState::from_model(&model);
        let ps = PromptState::from_session(&session);
        assert_eq!(ps.galaxy_name, "Euclid");
        assert!(ps.biome_filter.is_none());
    }
}
```

---

## Implementation Notes

1. **Prompt trait**: reedline's `Prompt` trait requires `render_prompt_left()`, `render_prompt_right()`, and `render_prompt_indicator()`. We use left for the context and indicator for a trailing space.

2. **PromptState snapshot**: Rather than sharing a reference to `SessionState` (which would require lifetime gymnastics with reedline), we take a snapshot `PromptState` before each `read_line()`. This is cheap to clone.

3. **Dynamic updates**: `prompt.update()` is called before each `read_line()` so changes from `set biome Lush` are reflected in the next prompt immediately.

4. **Prompt format**: `[Galaxy | Filter | N planets] 🚀 `. The pipe separators match the design doc. When no biome filter is active, that segment is omitted.

5. **Prompt indicator**: A single space after the rocket emoji. This gives `[Euclid | 644 planets] 🚀 find --biome Lush` as the full prompt + input appearance.
