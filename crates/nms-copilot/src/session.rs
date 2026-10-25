//! Mutable session context for REPL commands.
//!
//! [`SessionState`] stores a reference position, biome and warp preferences,
//! selected galaxy, and model counts. It is a caller-owned snapshot: public
//! fields can be changed independently, and values may be inconsistent with
//! the model until [`SessionState::sync_from_model`] is called. Synchronization
//! refreshes the selected galaxy and counts, follows a missing/player position,
//! and preserves base or manually specified address snapshots. These APIs do
//! not persist state, start background work, or mutate the model.
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use nms_copilot::session::{PositionContext, SessionState};
//! use nms_core::{BaseType, Biome, GalacticAddress, GalaxyType, Planet, PlayerBase, PlayerState, System};
//! use nms_graph::GalaxyModel;
//! fn view(s: &SessionState) -> (Option<(&'static str, String, (u64, u8))>, Option<Biome>, Option<f64>, u8, String, GalaxyType, usize, usize) {
//!     let position = s.position.as_ref().map(|p| {
//!         let kind = match p {
//!             PositionContext::Base { .. } => "base",
//!             PositionContext::PlayerPosition(_) => "player",
//!             PositionContext::Address(_) => "address",
//!         };
//!         let address = p.address();
//!         (kind, p.label(), (address.packed(), address.reality_index))
//!     });
//!     (position, s.biome_filter, s.warp_range, s.galaxy.index, s.galaxy.name.into(), s.galaxy.galaxy_type, s.system_count, s.planet_count)
//! }
//! let a = GalacticAddress::new(3,0,0,1,2,0);
//! let mut m = GalaxyModel::new();
//! m.insert_system(System::new(a,Some("Atlas".into()),None,None,vec![Planet::new(2,Some(Biome::Lush),None,false,Some("Garden".into()),Some(7))]));
//! m.insert_base(PlayerBase::new("Home".into(),BaseType::HomePlanetBase,a,[0.0,0.0,0.0],None));
//! m.player_state=Some(PlayerState::new(GalacticAddress::new(3,0,0,1,5,9),9,None,None,100,20,5)); m.set_active_galaxy(7);
//! let mut s=SessionState::from_model(&m);
//! assert_eq!(view(&s),(Some(("player", "player position".into(), (0x5001_0000_0003, 9))),None,None,7,"Kikolgallr".into(),GalaxyType::Norm,1,1));
//! assert_eq!(s.set_position_base("HOME",&m)?,"Position set to Home");
//! assert_eq!(view(&s),(Some(("base", "Home".into(), (0x2001_0000_0003, 0))),None,None,7,"Kikolgallr".into(),GalaxyType::Norm,1,1));
//! assert_eq!(s.set_position_base(" HOME ",&m),Err("Base not found: \" HOME \"".into()));
//! assert_eq!(view(&s),(Some(("base", "Home".into(), (0x2001_0000_0003, 0))),None,None,7,"Kikolgallr".into(),GalaxyType::Norm,1,1));
//! assert_eq!(s.set_biome_filter(Biome::Lush),"Biome filter set to Lush");
//! assert_eq!(view(&s),(Some(("base", "Home".into(), (0x2001_0000_0003, 0))),Some(Biome::Lush),None,7,"Kikolgallr".into(),GalaxyType::Norm,1,1));
//! assert_eq!(s.set_warp_range(1200.5),"Warp range set to 1200 ly");
//! assert_eq!(view(&s),(Some(("base", "Home".into(), (0x2001_0000_0003, 0))),Some(Biome::Lush),Some(1200.5),7,"Kikolgallr".into(),GalaxyType::Norm,1,1));
//! m.insert_system(System::new(GalacticAddress::new(3,0,0,1,0,9),None,None,None,vec![]));
//! m.insert_base(PlayerBase::new("HOME".into(),BaseType::HomePlanetBase,GalacticAddress::new(3,0,0,1,4,9),[1.0,2.0,3.0],None));
//! m.player_state=Some(PlayerState::new(GalacticAddress::new(4,0,0,1,1,0),0,None,None,100,20,5)); m.set_active_galaxy(9); s.sync_from_model(&m);
//! assert_eq!(view(&s),(Some(("base", "Home".into(), (0x2001_0000_0003, 0))),Some(Biome::Lush),Some(1200.5),9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! let manual=GalacticAddress::new(3,0,0,1,3,7); assert_eq!(s.set_position_address(manual),"Position set to 0x300100000003");
//! assert_eq!(view(&s),(Some(("address", "0x300100000003".into(), (0x3001_0000_0003, 7))),Some(Biome::Lush),Some(1200.5),9,"Eissentam".into(),GalaxyType::Lush,2,1)); s.sync_from_model(&m);
//! assert_eq!(view(&s),(Some(("address", "0x300100000003".into(), (0x3001_0000_0003, 7))),Some(Biome::Lush),Some(1200.5),9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! assert_eq!(s.reset_position(&m),"Position reset to player location");
//! assert_eq!(view(&s),(Some(("player", "player position".into(), (0x1001_0000_0004, 0))),Some(Biome::Lush),Some(1200.5),9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! assert_eq!(s.format_status(),"Galaxy:      Eissentam (9; Lush)\nModel:       2 systems, 1 planets\nPosition:    player position (Euclid, galaxy 0)\nBiome:       Lush\nWarp range:  1200 ly\n");
//! assert_eq!(s.reset_all(&m),"Session state reset");
//! assert_eq!(view(&s),(Some(("player", "player position".into(), (0x1001_0000_0004, 0))),None,None,9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! m.player_state=None; s.sync_from_model(&m);
//! assert_eq!(view(&s),(None,None,None,9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! m.player_state=Some(PlayerState::new(GalacticAddress::new(4,0,0,1,1,0),0,None,None,100,20,5)); s.sync_from_model(&m);
//! assert_eq!(view(&s),(Some(("player", "player position".into(), (0x1001_0000_0004, 0))),None,None,9,"Eissentam".into(),GalaxyType::Lush,2,1));
//! # Ok(())
//! # }
//! ```

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::Galaxy;
use nms_graph::GalaxyModel;

/// Mutable REPL state snapshot used for command defaults.
///
/// Public fields can be changed independently and may not match the model.
/// Call [`sync_from_model`](Self::sync_from_model) to refresh model-derived
/// values; base and manual address positions remain snapshots.
#[derive(Debug)]
pub struct SessionState {
    /// Reference position used for distance calculations, if one is set.
    pub position: Option<PositionContext>,

    /// Optional biome default for commands that do not specify their own filter.
    pub biome_filter: Option<Biome>,

    /// Optional warp-range default in light-years; setters store any `f64`.
    pub warp_range: Option<f64>,

    /// Selected galaxy metadata, independent of the physical position galaxy.
    pub galaxy: Galaxy,

    /// System count across the whole model, including marked systems.
    pub system_count: usize,

    /// Flattened planet-map count across the whole model.
    pub planet_count: usize,
}

/// Anchor and stored address for the session reference position.
///
/// Base and manual-address variants keep their address snapshots across model
/// synchronization; only `PlayerPosition` follows the player's current address.
#[derive(Debug, Clone)]
pub enum PositionContext {
    /// At a named base copied from the model.
    Base {
        /// Display name copied from the selected base.
        name: String,
        /// Address copied from the selected base.
        address: GalacticAddress,
    },
    /// At the player's physical save position.
    PlayerPosition(
        /// Address copied from the player state.
        GalacticAddress,
    ),
    /// At a manually specified address, independent of the selected galaxy.
    Address(
        /// Address supplied by the caller.
        GalacticAddress,
    ),
}

impl PositionContext {
    /// Borrow the address stored by this position anchor.
    pub fn address(&self) -> &GalacticAddress {
        match self {
            Self::Base { address, .. } => address,
            Self::PlayerPosition(a) | Self::Address(a) => a,
        }
    }

    /// Return the base name, `player position`, or packed hexadecimal label.
    ///
    /// Address labels omit the separate galaxy index.
    pub fn label(&self) -> String {
        match self {
            Self::Base { name, .. } => name.clone(),
            Self::PlayerPosition(_) => "player position".into(),
            Self::Address(a) => format!("0x{:012X}", a.packed()),
        }
    }
}

impl SessionState {
    /// Initialize a session snapshot with counts from the whole model.
    ///
    /// Counts span all galaxies. System counts include marked systems; planet
    /// counts use the entire flat planet map. Position and selected galaxy are
    /// copied independently. A missing player system is not ensured; filters
    /// and range start unset.
    pub fn from_model(model: &GalaxyModel) -> Self {
        let position = model
            .player_state
            .as_ref()
            .map(|ps| PositionContext::PlayerPosition(ps.current_address));

        let galaxy = Galaxy::by_index(model.active_galaxy);

        Self {
            position,
            biome_filter: None,
            warp_range: None,
            galaxy,
            system_count: model.systems.len(),
            planet_count: model.planets.len(),
        }
    }

    /// Refresh model-derived fields while preserving explicit anchors.
    ///
    /// The selected galaxy and whole-model system and planet counts are
    /// refreshed. A missing or player-following position follows the current
    /// physical player address; base and manual address anchors remain snapshots.
    /// Biome and warp values are preserved. No placeholder is inserted and the
    /// model is not otherwise mutated.
    pub fn sync_from_model(&mut self, model: &GalaxyModel) {
        self.galaxy = Galaxy::by_index(model.active_galaxy);
        self.system_count = model.system_count();
        self.planet_count = model.planet_count();

        if matches!(
            &self.position,
            None | Some(PositionContext::PlayerPosition(_))
        ) {
            self.position = model
                .player_state
                .as_ref()
                .map(|player| PositionContext::PlayerPosition(player.current_address));
        }
    }

    /// Snapshot a base found by lowercase, untrimmed model name lookup.
    ///
    /// The current display name and address are copied into the position.
    ///
    /// # Errors
    ///
    /// Returns `Base not found: "{name}"` and leaves state unchanged when the
    /// name does not match a base.
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

    /// Set a manual address snapshot without changing selected galaxy.
    ///
    /// The confirmation formats the packed address as 12 hexadecimal digits.
    pub fn set_position_address(&mut self, address: GalacticAddress) -> String {
        let label = format!("0x{:012X}", address.packed());
        self.position = Some(PositionContext::Address(address));
        format!("Position set to {label}")
    }

    /// Clear the anchor and synchronize to the current player, if available.
    ///
    /// Filters and warp range are preserved; without player state the anchor
    /// remains `None`.
    pub fn reset_position(&mut self, model: &GalaxyModel) -> String {
        self.position = None;
        self.sync_from_model(model);
        "Position reset to player location".into()
    }

    /// Store a biome filter and return its Debug-formatted confirmation.
    pub fn set_biome_filter(&mut self, biome: Biome) -> String {
        let name = format!("{biome:?}");
        self.biome_filter = Some(biome);
        format!("Biome filter set to {name}")
    }

    /// Clear the biome filter and return `Biome filter cleared`.
    pub fn clear_biome_filter(&mut self) -> &'static str {
        self.biome_filter = None;
        "Biome filter cleared"
    }

    /// Store `ly` without validating range or finiteness.
    ///
    /// The message casts it to `u64`; the stored value remains the original `f64`.
    pub fn set_warp_range(&mut self, ly: f64) -> String {
        self.warp_range = Some(ly);
        format!("Warp range set to {} ly", ly as u64)
    }

    /// Clear the warp range and return `Warp range cleared`.
    pub fn clear_warp_range(&mut self) -> &'static str {
        self.warp_range = None;
        "Warp range cleared"
    }

    /// Refresh model-derived fields and clear biome and warp defaults.
    ///
    /// Position follows the current player when available; the model is not
    /// changed.
    pub fn reset_all(&mut self, model: &GalaxyModel) -> &'static str {
        self.reset_position(model);
        self.biome_filter = None;
        self.warp_range = None;
        "Session state reset"
    }

    /// Format stored state as five lines followed by a final LF.
    ///
    /// The selected galaxy and position's physical galaxy are shown separately;
    /// this reads no model and performs no output operation.
    pub fn format_status(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Galaxy:      {} ({}; {})",
            self.galaxy.name, self.galaxy.index, self.galaxy.galaxy_type
        ));
        lines.push(format!(
            "Model:       {} systems, {} planets",
            self.system_count, self.planet_count
        ));

        match &self.position {
            Some(pos) => {
                let address = pos.address();
                let reference_galaxy = Galaxy::by_index(address.reality_index);
                lines.push(format!(
                    "Position:    {} ({}, galaxy {})",
                    pos.label(),
                    reference_galaxy.name,
                    reference_galaxy.index
                ));
            }
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
        let mut model = test_model();
        model.set_active_galaxy(9);
        let session = SessionState::from_model(&model);
        assert!(session.position.is_some());
        assert_eq!(session.galaxy.index, 9);
        assert_eq!(session.position.unwrap().address().reality_index, 0);
        assert!(session.system_count > 0);
    }

    #[test]
    fn test_session_from_model_without_player_uses_selected_galaxy() {
        let mut model = test_model();
        model.player_state = None;
        model.set_active_galaxy(7);

        let session = SessionState::from_model(&model);
        assert_eq!(session.galaxy.index, 7);
        assert!(session.position.is_none());
    }

    #[test]
    fn test_sync_from_model_updates_follow_position_and_preserves_explicit_anchor() {
        let mut model = test_model();
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let mut session = SessionState::from_model(&model);
        session.set_biome_filter(Biome::Lush);
        session.set_warp_range(400.0);

        model.set_active_galaxy(9);
        let player = model.player_state.as_mut().unwrap();
        player.current_address = a9;
        player.current_reality = 9;
        let physical_state = player.clone();
        session.sync_from_model(&model);
        assert_eq!(session.galaxy.index, 9);
        assert!(
            matches!(session.position, Some(PositionContext::PlayerPosition(address)) if address == a9)
        );
        assert_eq!(session.biome_filter, Some(Biome::Lush));
        assert_eq!(session.warp_range, Some(400.0));

        let explicit = GalacticAddress::new(12, 4, 3, 2, 0, 7);
        session.position = Some(PositionContext::Address(explicit));
        model.set_active_galaxy(0);
        session.system_count = 0;
        session.planet_count = 0;
        session.sync_from_model(&model);
        assert_eq!(session.galaxy.index, 0);
        assert_eq!(session.system_count, model.system_count());
        assert_eq!(session.planet_count, model.planet_count());
        assert!(
            matches!(session.position, Some(PositionContext::Address(address)) if address == explicit)
        );
        assert_eq!(model.player_state.as_ref(), Some(&physical_state));

        session.position = Some(PositionContext::Base {
            name: "Home Base".into(),
            address: explicit,
        });
        session.sync_from_model(&model);
        assert!(
            matches!(&session.position, Some(PositionContext::Base { name, address }) if name == "Home Base" && *address == explicit)
        );
    }

    #[test]
    fn test_sync_from_model_absent_player_clears_follow_position_only() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        model.player_state = None;
        model.set_active_galaxy(7);
        session.sync_from_model(&model);
        assert!(session.position.is_none());
        assert_eq!(session.galaxy.index, 7);

        let anchor = GalacticAddress::new(1, 2, 3, 4, 0, 8);
        session.position = Some(PositionContext::Address(anchor));
        session.sync_from_model(&model);
        assert!(
            matches!(session.position, Some(PositionContext::Address(address)) if address == anchor)
        );
        session.reset_position(&model);
        assert!(session.position.is_none());
        assert_eq!(session.galaxy.index, 7);
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
        let mut model = test_model();
        model.set_active_galaxy(9);
        let mut player = model.player_state.as_ref().unwrap().clone();
        player.units = 7;
        player.nanites = 8;
        player.quicksilver = 9;
        model.player_state = Some(player.clone());
        let mut session = SessionState::from_model(&model);
        session.set_biome_filter(Biome::Toxic);
        session.set_warp_range(1000.0);
        session.reset_all(&model);
        assert!(session.biome_filter.is_none());
        assert!(session.warp_range.is_none());
        assert_eq!(session.galaxy.index, 9);
        assert_eq!(session.system_count, model.system_count());
        assert_eq!(session.planet_count, model.planet_count());
        assert_eq!(model.active_galaxy, 9);
        assert_eq!(model.player_state, Some(player));
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
        let mut model = test_model();
        model.set_active_galaxy(9);
        let mut session = SessionState::from_model(&model);
        session.set_position_base("Home Base", &model).unwrap();
        session.set_biome_filter(Biome::Lush);
        session.set_warp_range(400.0);
        session.reset_position(&model);
        assert!(matches!(
            &session.position,
            Some(PositionContext::PlayerPosition(_))
        ));
        assert_eq!(session.galaxy.index, 9);
        assert_eq!(session.biome_filter, Some(Biome::Lush));
        assert_eq!(session.warp_range, Some(400.0));
        assert_eq!(model.active_galaxy, 9);
    }
}
