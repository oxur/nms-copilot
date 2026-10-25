//! Detail views for systems and bases, resolved from the in-memory model.
//!
//! [`execute_show`] checks active-galaxy system names before packed addresses;
//! base names use the model-wide base index.

use nms_core::address::GalacticAddress;
use nms_core::player::PlayerBase;
use nms_core::system::System;
use nms_graph::spatial::SystemId;
use nms_graph::{GalaxyModel, GraphError};

/// What record to resolve for a detail view. See [`execute_show`] and the
/// [`crate#examples`] for resolution behavior.
#[derive(Debug, Clone)]
pub enum ShowQuery {
    /// Resolve a name first, then a packed hexadecimal address in the active galaxy.
    System(
        /// System name or packed hexadecimal address text.
        String,
    ),
    /// Resolve a base name through the model-wide base index.
    Base(
        /// Base name looked up case-insensitively without trimming.
        String,
    ),
}

/// A resolved system or base detail result. See the [`crate#examples`] for the system variant and its observed fields.
#[derive(Debug, Clone)]
pub enum ShowResult {
    /// Resolved system detail.
    System(
        /// Cloned system and its display metadata.
        ShowSystemResult,
    ),
    /// Resolved base detail.
    Base(
        /// Cloned base and optional associated system metadata.
        ShowBaseResult,
    ),
}

/// Cloned system data and display fields for a system detail view. See the
/// [`crate#examples`] for its complete system projection.
#[derive(Debug, Clone)]
pub struct ShowSystemResult {
    /// The cloned system record; its stored address may retain source planet bits.
    pub system: System,
    /// Twelve-digit uppercase hex formatted from the stored system address.
    pub portal_hex: String,
    /// Galaxy display name for the stored address.
    pub galaxy_name: String,
    /// Distance when player and system addresses share a galaxy.
    pub distance_from_player: Option<f64>,
}

/// Cloned base data and display fields for a base detail view. See
/// [`execute_show`] for resolution behavior.
#[derive(Debug, Clone)]
pub struct ShowBaseResult {
    /// The cloned base record.
    pub base: PlayerBase,
    /// Twelve-digit uppercase hex formatted from the stored base address.
    pub portal_hex: String,
    /// Galaxy display name for the base address.
    pub galaxy_name: String,
    /// Cloned associated system, when that canonical key exists in the model.
    pub system: Option<System>,
    /// Distance when player and base addresses share a galaxy.
    pub distance_from_player: Option<f64>,
}

/// Execute a show query using active-galaxy system-name and packed-address lookup.
///
/// A player distance is included only when the player and selected item share
/// a galaxy; otherwise the result has no distance. The [`crate#examples`] exercises name resolution and the missing-base error. Name
/// lookup lowercases without
/// trimming and precedes address parsing. Address lookup accepts packed hexadecimal
/// `u64` text with an optional `0x` or `0X` prefix, not a save-format decoder.
///
/// # Errors
///
/// Returns [`GraphError::SystemNotFound`] or [`GraphError::BaseNotFound`] when
/// the requested system or base cannot be resolved. A system-name miss is also
/// attempted as a packed address before `SystemNotFound` is returned.
pub fn execute_show(model: &GalaxyModel, query: &ShowQuery) -> Result<ShowResult, GraphError> {
    match query {
        ShowQuery::System(name_or_id) => show_system(model, name_or_id),
        ShowQuery::Base(name) => show_base(model, name),
    }
}

fn show_system(model: &GalaxyModel, name_or_id: &str) -> Result<ShowResult, GraphError> {
    // Try name lookup first, then try as packed hex address
    let system = if let Some((_id, sys)) = model.system_by_name(name_or_id) {
        sys
    } else {
        let hex = name_or_id
            .strip_prefix("0x")
            .or_else(|| name_or_id.strip_prefix("0X"))
            .unwrap_or(name_or_id);
        let packed = u64::from_str_radix(hex, 16)
            .map_err(|_| GraphError::SystemNotFound(name_or_id.to_string()))?;
        let id = SystemId::from_address(&GalacticAddress::from_packed(packed, model.active_galaxy));
        model
            .system(&id)
            .ok_or_else(|| GraphError::SystemNotFound(name_or_id.to_string()))?
    };

    let portal_hex = format!("{:012X}", system.address.packed());
    let galaxy = nms_core::galaxy::Galaxy::by_index(system.address.reality_index);

    let distance_from_player = model
        .player_position()
        .filter(|pos| pos.reality_index == system.address.reality_index)
        .map(|pos| pos.distance_ly(&system.address));

    Ok(ShowResult::System(ShowSystemResult {
        system: system.clone(),
        portal_hex,
        galaxy_name: galaxy.name.to_string(),
        distance_from_player,
    }))
}

fn show_base(model: &GalaxyModel, name: &str) -> Result<ShowResult, GraphError> {
    let base = model
        .base(name)
        .ok_or_else(|| GraphError::BaseNotFound(name.to_string()))?;

    let portal_hex = format!("{:012X}", base.address.packed());
    let galaxy = nms_core::galaxy::Galaxy::by_index(base.address.reality_index);

    // Try to find the system this base is in
    let sys_id = SystemId::from_address(&base.address);
    let system = model.system(&sys_id).cloned();

    let distance_from_player = model
        .player_position()
        .filter(|pos| pos.reality_index == base.address.reality_index)
        .map(|pos| pos.distance_ly(&base.address));

    Ok(ShowResult::Base(ShowBaseResult {
        base: base.clone(),
        portal_hex,
        galaxy_name: galaxy.name.to_string(),
        system,
        distance_from_player,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{format_show_base, format_show_system};
    use crate::theme::Theme;
    use nms_core::player::BaseType;
    use nms_core::player::PlayerBase;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn test_show_base_by_name() {
        let model = test_model();
        let result = execute_show(&model, &ShowQuery::Base("Alpha Base".into())).unwrap();
        match result {
            ShowResult::Base(b) => {
                assert_eq!(b.base.name, "Alpha Base");
                assert_eq!(b.galaxy_name, "Euclid");
                assert_eq!(b.portal_hex.len(), 12);
            }
            _ => panic!("Expected Base result"),
        }
    }

    #[test]
    fn test_show_base_case_insensitive() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::Base("alpha base".into())).is_ok());
    }

    #[test]
    fn test_show_base_not_found() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::Base("No Base".into())).is_err());
    }

    #[test]
    fn test_show_system_not_found() {
        let model = test_model();
        assert!(execute_show(&model, &ShowQuery::System("No System".into())).is_err());
    }

    #[test]
    fn test_show_packed_address_uses_active_galaxy_and_clears_planet_bits() {
        let address0 = GalacticAddress::from_packed(0x050003AB8C07, 0);
        let address9 = GalacticAddress::from_packed(0x050003AB8C07, 9);
        let mut model = GalaxyModel::new();
        model.insert_system(System::new(
            address0,
            Some("Zero".into()),
            None,
            None,
            vec![],
        ));
        model.insert_system(System::new(
            address9,
            Some("Nine".into()),
            None,
            None,
            vec![],
        ));

        model.set_active_galaxy(0);
        let result = execute_show(&model, &ShowQuery::System("150003AB8C07".into())).unwrap();
        assert!(
            matches!(result, ShowResult::System(s) if s.system.name.as_deref() == Some("Zero"))
        );

        model.set_active_galaxy(9);
        let result = execute_show(&model, &ShowQuery::System("150003AB8C07".into())).unwrap();
        assert!(
            matches!(result, ShowResult::System(s) if s.system.name.as_deref() == Some("Nine"))
        );

        model.set_active_galaxy(1);
        assert!(matches!(
            execute_show(&model, &ShowQuery::System("050003AB8C07".into())),
            Err(GraphError::SystemNotFound(_))
        ));
    }

    #[test]
    fn test_show_named_bases_attach_to_their_qualified_systems() {
        let address0 = GalacticAddress::from_packed(0x050003AB8C07, 0);
        let address9 = GalacticAddress::from_packed(0x050003AB8C07, 9);
        let mut model = GalaxyModel::new();
        model.insert_system(System::new(
            address0,
            Some("Zero".into()),
            None,
            None,
            vec![],
        ));
        model.insert_system(System::new(
            address9,
            Some("Nine".into()),
            None,
            None,
            vec![],
        ));
        for (name, address) in [("Zero Base", address0), ("Nine Base", address9)] {
            model.insert_base(nms_core::player::PlayerBase::new(
                name.into(),
                nms_core::player::BaseType::HomePlanetBase,
                address,
                [0.0; 3],
                None,
            ));
        }

        for (base_name, system_name) in [("Zero Base", "Zero"), ("Nine Base", "Nine")] {
            let result = execute_show(&model, &ShowQuery::Base(base_name.into())).unwrap();
            match result {
                ShowResult::Base(base) => assert_eq!(
                    base.system
                        .as_ref()
                        .and_then(|system| system.name.as_deref()),
                    Some(system_name)
                ),
                _ => panic!("expected base result"),
            }
        }
    }

    #[test]
    fn test_show_omits_cross_galaxy_distance_and_formats_only_comparable_values() {
        let zero0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let zero1 = GalacticAddress::new(1, 0, 0, 1, 0, 0);
        let nine0 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let mut model = GalaxyModel::new();
        model.player_state = Some(nms_core::player::PlayerState::new(
            zero0, 0, None, None, 0, 0, 0,
        ));
        model.insert_system(System::new(
            zero0,
            Some("Shared".into()),
            None,
            None,
            vec![],
        ));
        model.insert_system(System::new(
            zero1,
            Some("Zero One".into()),
            None,
            None,
            vec![],
        ));
        model.insert_system(System::new(
            nine0,
            Some("Shared".into()),
            None,
            None,
            vec![],
        ));
        model.insert_base(PlayerBase::new(
            "Nine Base".into(),
            BaseType::HomePlanetBase,
            nine0,
            [0.0; 3],
            None,
        ));

        model.set_active_galaxy(0);
        let ShowResult::System(same_system) =
            execute_show(&model, &ShowQuery::System("Shared".into())).unwrap()
        else {
            panic!("expected system result");
        };
        assert_eq!(same_system.system.name.as_deref(), Some("Shared"));
        assert_eq!(same_system.distance_from_player, Some(0.0));
        assert!(format_show_system(&same_system, &Theme::none()).contains("Distance"));

        let ShowResult::System(same_galaxy) =
            execute_show(&model, &ShowQuery::System("Zero One".into())).unwrap()
        else {
            panic!("expected system result");
        };
        assert_eq!(same_galaxy.distance_from_player, Some(400.0));

        model.set_active_galaxy(9);
        let ShowResult::System(other_galaxy) =
            execute_show(&model, &ShowQuery::System("Shared".into())).unwrap()
        else {
            panic!("expected system result");
        };
        assert_eq!(other_galaxy.system.name.as_deref(), Some("Shared"));
        assert_eq!(other_galaxy.distance_from_player, None);
        assert!(!format_show_system(&other_galaxy, &Theme::none()).contains("Distance"));

        let ShowResult::Base(base) =
            execute_show(&model, &ShowQuery::Base("Nine Base".into())).unwrap()
        else {
            panic!("expected base result");
        };
        assert_eq!(base.distance_from_player, None);
        assert_eq!(
            base.system
                .as_ref()
                .and_then(|system| system.name.as_deref()),
            Some("Shared")
        );
        assert!(!format_show_base(&base, &Theme::none()).contains("Distance"));

        model.player_state = None;
        let ShowResult::System(no_player) =
            execute_show(&model, &ShowQuery::System("Shared".into())).unwrap()
        else {
            panic!("expected system result");
        };
        assert_eq!(no_player.distance_from_player, None);
    }
}
