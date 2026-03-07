//! Detail view queries for systems, planets, and bases.

use nms_core::player::PlayerBase;
use nms_core::system::System;
use nms_graph::spatial::SystemId;
use nms_graph::{GalaxyModel, GraphError};

/// What to show detail for.
#[derive(Debug, Clone)]
pub enum ShowQuery {
    /// Show a system by name or packed address.
    System(String),
    /// Show a base by name.
    Base(String),
}

/// Result of a show query.
#[derive(Debug, Clone)]
pub enum ShowResult {
    System(ShowSystemResult),
    Base(ShowBaseResult),
}

#[derive(Debug, Clone)]
pub struct ShowSystemResult {
    pub system: System,
    pub portal_hex: String,
    pub galaxy_name: String,
    pub distance_from_player: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ShowBaseResult {
    pub base: PlayerBase,
    pub portal_hex: String,
    pub galaxy_name: String,
    pub system: Option<System>,
    pub distance_from_player: Option<f64>,
}

/// Execute a show query.
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
        let id = SystemId(packed & 0x0FFF_FFFF_FFFF);
        model
            .system(&id)
            .ok_or_else(|| GraphError::SystemNotFound(name_or_id.to_string()))?
    };

    let portal_hex = format!("{:012X}", system.address.packed());
    let galaxy = nms_core::galaxy::Galaxy::by_index(system.address.reality_index);

    let distance_from_player = model
        .player_position()
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
}
