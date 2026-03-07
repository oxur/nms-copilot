//! Watch integration for the REPL.
//!
//! Drains pending file-watcher deltas between prompts, applies them to the
//! galaxy model, updates session state, and optionally writes through to the
//! cache. Notifications are returned as strings for the REPL to print.

use std::path::Path;
use std::sync::mpsc;

use nms_core::delta::SaveDelta;
use nms_graph::GalaxyModel;

use crate::session::{PositionContext, SessionState};

/// Drain all pending deltas from the watcher and apply them to the model.
///
/// Prints notifications for each delta. If any deltas were applied and
/// `cache_path` is provided, writes an updated cache.
pub fn drain_watch_events(
    receiver: &mpsc::Receiver<SaveDelta>,
    model: &mut GalaxyModel,
    session: &mut SessionState,
    cache_path: Option<&Path>,
    save_version: u32,
) {
    let mut any_delta = false;

    while let Ok(delta) = receiver.try_recv() {
        let notifications = apply_and_notify(model, session, &delta);
        for note in &notifications {
            println!("{note}");
        }
        any_delta = true;
    }

    // Write updated cache if any deltas were applied
    if any_delta {
        if let Some(path) = cache_path {
            let data = nms_cache::extract_cache_data(model, save_version);
            if let Err(e) = nms_cache::write_cache(&data, path) {
                eprintln!("Warning: could not update cache: {e}");
            }
        }
    }
}

/// Apply a delta to the model and generate human-readable notifications.
///
/// Notifications are generated *before* the delta is applied so that
/// system name lookups for the "from" position still resolve correctly.
pub fn apply_and_notify(
    model: &mut GalaxyModel,
    session: &mut SessionState,
    delta: &SaveDelta,
) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(ref moved) = delta.player_moved {
        let from_name = system_name_near(model, &moved.from);
        let to_name = system_name_near(model, &moved.to);
        notes.push(format!("  Warped: {from_name} -> {to_name}"));
    }

    for system in &delta.new_systems {
        let name = system.name.as_deref().unwrap_or("(unnamed)");
        let planets = system.planets.len();
        notes.push(format!(
            "  New system: {name} ({planets} planet{})",
            if planets == 1 { "" } else { "s" }
        ));
    }

    for (sys_id, planet) in &delta.new_planets {
        let sys_name = model
            .system(sys_id)
            .and_then(|s| s.name.as_deref())
            .unwrap_or("(unnamed)");
        let biome = planet
            .biome
            .map(|b| b.to_string())
            .unwrap_or_else(|| "?".into());
        let planet_name = planet.name.as_deref().unwrap_or("(unnamed)");
        notes.push(format!(
            "  New scan: \"{planet_name}\" ({biome}) in {sys_name}"
        ));
    }

    for base in &delta.new_bases {
        notes.push(format!("  New base: {}", base.name));
    }

    // Apply delta to model
    model.apply_delta(delta);

    // Update session counts
    session.system_count = model.system_count();
    session.planet_count = model.planet_count();

    // Update position if player moved
    if let Some(ref moved) = delta.player_moved {
        session.position = Some(PositionContext::PlayerPosition(moved.to));
    }

    notes
}

/// Look up the name of the nearest system to an address.
fn system_name_near(model: &GalaxyModel, addr: &nms_core::address::GalacticAddress) -> String {
    model
        .nearest_systems(addr, 1)
        .first()
        .and_then(|(id, _)| model.system(id))
        .and_then(|s| s.name.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::delta::PlayerMoved;
    use nms_core::system::System;

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
    fn test_apply_and_notify_empty_delta() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let delta = SaveDelta::empty();

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_apply_and_notify_new_system() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("New System".into()),
                None,
                None,
                vec![],
            )],
            ..SaveDelta::empty()
        };

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.iter().any(|n| n.contains("New system")));
        assert!(notes.iter().any(|n| n.contains("New System")));
    }

    #[test]
    fn test_apply_and_notify_player_moved() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let from = *model.player_position().unwrap();
        let to = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = SaveDelta {
            player_moved: Some(PlayerMoved { from, to }),
            ..SaveDelta::empty()
        };

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.iter().any(|n| n.contains("Warped")));
        assert_eq!(model.player_position().unwrap().voxel_x(), 999);
    }

    #[test]
    fn test_apply_and_notify_updates_session_counts() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let sys_before = session.system_count;

        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                None,
                None,
                None,
                vec![],
            )],
            ..SaveDelta::empty()
        };

        apply_and_notify(&mut model, &mut session, &delta);
        assert_eq!(session.system_count, sys_before + 1);
    }

    #[test]
    fn test_apply_and_notify_updates_session_position() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let from = *model.player_position().unwrap();
        let to = GalacticAddress::new(42, 0, 0, 1, 0, 0);

        let delta = SaveDelta {
            player_moved: Some(PlayerMoved { from, to }),
            ..SaveDelta::empty()
        };

        apply_and_notify(&mut model, &mut session, &delta);
        match &session.position {
            Some(PositionContext::PlayerPosition(addr)) => {
                assert_eq!(addr.voxel_x(), 42);
            }
            _ => panic!("Expected PlayerPosition after warp"),
        }
    }

    #[test]
    fn test_apply_and_notify_new_base() {
        use nms_core::player::{BaseType, PlayerBase};

        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let addr = GalacticAddress::new(100, 50, -200, 42, 0, 0);
        let base = PlayerBase::new(
            "New Outpost".into(),
            BaseType::HomePlanetBase,
            addr,
            [0.0, 0.0, 0.0],
            None,
        );

        let delta = SaveDelta {
            new_bases: vec![base],
            ..SaveDelta::empty()
        };

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.iter().any(|n| n.contains("New base")));
        assert!(notes.iter().any(|n| n.contains("New Outpost")));
    }

    #[test]
    fn test_drain_watch_events_no_pending() {
        let (_tx, rx) = mpsc::channel::<SaveDelta>();
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);

        // No events pending, should not panic or change anything
        let count_before = session.system_count;
        drain_watch_events(&rx, &mut model, &mut session, None, 0);
        assert_eq!(session.system_count, count_before);
    }

    #[test]
    fn test_drain_watch_events_applies_delta() {
        let (tx, rx) = mpsc::channel();
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let count_before = session.system_count;

        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("Drain Test".into()),
                None,
                None,
                vec![],
            )],
            ..SaveDelta::empty()
        };
        tx.send(delta).unwrap();
        drop(tx);

        drain_watch_events(&rx, &mut model, &mut session, None, 0);
        assert_eq!(session.system_count, count_before + 1);
    }
}
