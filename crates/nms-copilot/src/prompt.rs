//! Context-aware REPL prompt.
//!
//! The prompt renders a copied session snapshot: selected galaxy name, the
//! `Debug` spelling of the optional biome filter, and planet count. It does not
//! look up the physical position galaxy or retain a live session reference.
//! Public snapshot fields can be constructed or changed independently. A
//! `CopilotPrompt` owns the snapshot passed to it; [`CopilotPrompt::update`]
//! replaces it, and the REPL currently refreshes session state and updates the
//! prompt before each `read_line` call.
//!
//! The left prompt is `[galaxy | optional biome | N planets] 🚀` (without a
//! trailing space); an empty-string biome still produces a segment, and the
//! count is not singularized. The right prompt is empty, the edit-mode
//! indicator is one ASCII space for every mode, and the multiline indicator is
//! `... `. History search shows `(search: TERM) ` or `(failed) (search: TERM) `.
//! Reedline's default trait colors remain in effect.
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use nms_copilot::prompt::{CopilotPrompt, PromptState};
//! use nms_copilot::session::SessionState;
//! use nms_core::{galaxy::Galaxy, Biome};
//! use nms_graph::GalaxyModel;
//! use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
//!
//! let mut session = SessionState::from_model(&GalaxyModel::new());
//! session.galaxy = Galaxy::by_index(9);
//! session.biome_filter = Some(Biome::Toxic);
//! session.planet_count = 2;
//! let state = PromptState::from_session(&session);
//! assert_eq!((state.galaxy_name.as_str(), state.biome_filter.as_deref(), state.planet_count), ("Eissentam", Some("Toxic"), 2));
//! let mut prompt = CopilotPrompt::new(state);
//! assert_eq!(prompt.render_prompt_left().as_ref(), "[Eissentam | Toxic | 2 planets] 🚀");
//! assert_eq!(prompt.render_prompt_right().as_ref(), "");
//! assert_eq!(prompt.render_prompt_indicator(PromptEditMode::Default).as_ref(), " ");
//! assert_eq!(prompt.render_prompt_multiline_indicator().as_ref(), "... ");
//! assert_eq!(prompt.render_prompt_history_search_indicator(PromptHistorySearch::new(PromptHistorySearchStatus::Passing, "find".to_owned())).as_ref(), "(search: find) ");
//! assert_eq!(prompt.render_prompt_history_search_indicator(PromptHistorySearch::new(PromptHistorySearchStatus::Failing, "find".to_owned())).as_ref(), "(failed) (search: find) ");
//!
//! session.galaxy = Galaxy::by_index(0);
//! session.biome_filter = None;
//! session.planet_count = 1;
//! assert_eq!(prompt.render_prompt_left().as_ref(), "[Eissentam | Toxic | 2 planets] 🚀");
//! prompt.update(PromptState::from_session(&session));
//! assert_eq!(prompt.render_prompt_left().as_ref(), "[Euclid | 1 planets] 🚀");
//! # Ok(())
//! # }
//! ```

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
    /// Selected session galaxy name copied for display.
    pub galaxy_name: String,
    /// Optional biome's display spelling copied from its `Debug` form.
    pub biome_filter: Option<String>,
    /// Planet count copied from the session snapshot.
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
    /// Create a prompt that owns the supplied display snapshot.
    ///
    /// Later session changes do not affect this prompt until [`Self::update`]
    /// receives another [`PromptState`].
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
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(self.render_left())
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed(" ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "(failed) ",
        };
        Cow::Owned(format!("{prefix}(search: {}) ", history_search.term))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            prompt
                .render_prompt_indicator(PromptEditMode::Default)
                .as_ref(),
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
