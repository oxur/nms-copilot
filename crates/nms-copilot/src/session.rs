//! Session state for the interactive REPL.
//!
//! Tracks the user's current context: position, filters, and preferences.
//! Commands like `find` and `route` use this state as defaults when
//! explicit flags are not provided.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_graph::GalaxyModel;

/// Mutable session state maintained across REPL commands.
#[derive(Debug)]
pub struct SessionState {
    /// Current reference position (for distance calculations).
    pub position: Option<PositionContext>,

    /// Active biome filter (applied to find commands when --biome is not specified).
    pub biome_filter: Option<Biome>,

    /// Default warp range in light-years (for route planning).
    pub warp_range: Option<f64>,

    /// Current galaxy context.
    pub galaxy: Galaxy,

    /// Number of systems in the model.
    pub system_count: usize,

    /// Number of planets in the model.
    pub planet_count: usize,
}

/// Where the user's reference position is anchored.
#[derive(Debug, Clone)]
pub enum PositionContext {
    /// At a named base.
    Base {
        name: String,
        address: GalacticAddress,
    },
    /// At the player's save file position.
    PlayerPosition(GalacticAddress),
    /// At a manually specified address.
    Address(GalacticAddress),
}

impl PositionContext {
    pub fn address(&self) -> &GalacticAddress {
        match self {
            Self::Base { address, .. } => address,
            Self::PlayerPosition(a) | Self::Address(a) => a,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Base { name, .. } => name.clone(),
            Self::PlayerPosition(_) => "player position".into(),
            Self::Address(a) => format!("0x{:012X}", a.packed()),
        }
    }
}

impl SessionState {
    /// Initialize session state from the loaded model.
    pub fn from_model(model: &GalaxyModel) -> Self {
        let position = model
            .player_state
            .as_ref()
            .map(|ps| PositionContext::PlayerPosition(ps.current_address));

        let galaxy = model
            .player_state
            .as_ref()
            .map(|ps| Galaxy::by_index(ps.current_address.reality_index))
            .unwrap_or_else(|| Galaxy::by_index(0));

        Self {
            position,
            biome_filter: None,
            warp_range: None,
            galaxy,
            system_count: model.systems.len(),
            planet_count: model.planets.len(),
        }
    }

    /// Set the reference position to a named base.
    pub fn set_position_base(&mut self, name: &str, model: &GalaxyModel) -> Result<String, String> {
        let base = model
            .base(name)
            .ok_or_else(|| format!("Base not found: \"{name}\""))?;
        let address = base.address;
        let display_name = base.name.clone();
        self.position = Some(PositionContext::Base {
            name: display_name.clone(),
            address,
        });
        Ok(format!("Position set to {display_name}"))
    }

    /// Set the reference position to an explicit address.
    pub fn set_position_address(&mut self, address: GalacticAddress) -> String {
        let label = format!("0x{:012X}", address.packed());
        self.position = Some(PositionContext::Address(address));
        format!("Position set to {label}")
    }

    /// Reset position to the player's save file position.
    pub fn reset_position(&mut self, model: &GalaxyModel) -> String {
        self.position = model
            .player_state
            .as_ref()
            .map(|ps| PositionContext::PlayerPosition(ps.current_address));
        "Position reset to player location".into()
    }

    /// Set the active biome filter.
    pub fn set_biome_filter(&mut self, biome: Biome) -> String {
        let name = format!("{biome:?}");
        self.biome_filter = Some(biome);
        format!("Biome filter set to {name}")
    }

    /// Clear the active biome filter.
    pub fn clear_biome_filter(&mut self) -> &'static str {
        self.biome_filter = None;
        "Biome filter cleared"
    }

    /// Set the default warp range.
    pub fn set_warp_range(&mut self, ly: f64) -> String {
        self.warp_range = Some(ly);
        format!("Warp range set to {} ly", ly as u64)
    }

    /// Clear the warp range.
    pub fn clear_warp_range(&mut self) -> &'static str {
        self.warp_range = None;
        "Warp range cleared"
    }

    /// Reset all session state to defaults.
    pub fn reset_all(&mut self, model: &GalaxyModel) -> &'static str {
        self.reset_position(model);
        self.biome_filter = None;
        self.warp_range = None;
        "Session state reset"
    }

    /// Format the current session state for display.
    pub fn format_status(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Galaxy:      {} ({})",
            self.galaxy.name, self.galaxy.galaxy_type
        ));
        lines.push(format!(
            "Model:       {} systems, {} planets",
            self.system_count, self.planet_count
        ));

        match &self.position {
            Some(pos) => lines.push(format!("Position:    {}", pos.label())),
            None => lines.push("Position:    unknown".into()),
        }

        match &self.biome_filter {
            Some(b) => lines.push(format!("Biome:       {b:?}")),
            None => lines.push("Biome:       (none)".into()),
        }

        match self.warp_range {
            Some(r) => lines.push(format!("Warp range:  {} ly", r as u64)),
            None => lines.push("Warp range:  (none)".into()),
        }

        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [
                        {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Home Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                    ]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        GalaxyModel::from_save(&save)
    }

    #[test]
    fn test_session_from_model() {
        let model = test_model();
        let session = SessionState::from_model(&model);
        assert!(session.position.is_some());
        assert_eq!(session.galaxy.name, "Euclid");
        assert!(session.system_count > 0);
    }

    #[test]
    fn test_set_position_base() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        let result = session.set_position_base("Home Base", &model);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Home Base"));
        match &session.position {
            Some(PositionContext::Base { name, .. }) => assert_eq!(name, "Home Base"),
            _ => panic!("Expected Base position"),
        }
    }

    #[test]
    fn test_set_position_unknown_base_errors() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        assert!(session.set_position_base("No Such Base", &model).is_err());
    }

    #[test]
    fn test_set_biome_filter() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        session.set_biome_filter(Biome::Lush);
        assert_eq!(session.biome_filter, Some(Biome::Lush));
    }

    #[test]
    fn test_clear_biome_filter() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        session.set_biome_filter(Biome::Lush);
        session.clear_biome_filter();
        assert!(session.biome_filter.is_none());
    }

    #[test]
    fn test_set_warp_range() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        session.set_warp_range(2500.0);
        assert_eq!(session.warp_range, Some(2500.0));
    }

    #[test]
    fn test_reset_all() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        session.set_biome_filter(Biome::Toxic);
        session.set_warp_range(1000.0);
        session.reset_all(&model);
        assert!(session.biome_filter.is_none());
        assert!(session.warp_range.is_none());
    }

    #[test]
    fn test_format_status() {
        let model = test_model();
        let session = SessionState::from_model(&model);
        let output = session.format_status();
        assert!(output.contains("Euclid"));
        assert!(output.contains("systems"));
    }

    #[test]
    fn test_position_context_label() {
        let addr = GalacticAddress::new(0, 0, 0, 0, 0, 0);
        let base = PositionContext::Base {
            name: "Test".into(),
            address: addr,
        };
        assert_eq!(base.label(), "Test");

        let player = PositionContext::PlayerPosition(addr);
        assert_eq!(player.label(), "player position");
    }

    #[test]
    fn test_set_position_address() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        let addr = GalacticAddress::new(100, 50, -200, 42, 0, 0);
        let msg = session.set_position_address(addr);
        assert!(msg.contains("Position set to"));
        assert!(matches!(
            &session.position,
            Some(PositionContext::Address(_))
        ));
    }

    #[test]
    fn test_reset_position() {
        let model = test_model();
        let mut session = SessionState::from_model(&model);
        session.set_position_base("Home Base", &model).unwrap();
        session.reset_position(&model);
        assert!(matches!(
            &session.position,
            Some(PositionContext::PlayerPosition(_))
        ));
    }
}
