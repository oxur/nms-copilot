//! Watch integration for the REPL.
//!
//! Drains pending file-watcher deltas between prompts, applies them to the
//! galaxy model, updates session state, and optionally writes through to the
//! cache. Notifications are returned as strings for the REPL to print.

use std::path::Path;
use std::sync::mpsc;

use nms_core::delta::SaveDelta;
use nms_graph::GalaxyModel;

use crate::session::SessionState;
use nms_core::system::SystemId;

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
    if any_delta && let Some(path) = cache_path {
        let data = nms_cache::extract_cache_data(model, save_version);
        if let Err(e) = nms_cache::write_cache(&data, path) {
            eprintln!("Warning: could not update cache: {e}");
        }
    }
}

/// Format user-facing notes for an update before applying it to the model.
///
/// This is pure with respect to the model and is shared by the coordinator so
/// both live paths preserve the same name-resolution behavior.
pub fn notifications_for_delta(model: &GalaxyModel, delta: &SaveDelta) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(ref moved) = delta.player_moved {
        let from_name = system_name_near(model, &moved.from);
        let to_id = SystemId::from_address(&moved.to);
        let incoming_destination = delta
            .new_systems
            .iter()
            .find(|system| SystemId::from_address(&system.address) == to_id);
        let to_name = incoming_destination.map_or_else(
            || system_name_near(model, &moved.to),
            |system| system.name.as_deref().unwrap_or("unknown").to_owned(),
        );
        notes.push(format!("  Warped: {from_name} -> {to_name}"));
    }

    for system in &delta.new_systems {
        let name = system.name.as_deref().unwrap_or("-");
        let planets = system.planets.len();
        notes.push(format!(
            "  New system: {name} ({planets} planet{})",
            if planets == 1 { "" } else { "s" }
        ));
    }

    for (sys_id, planet) in &delta.new_planets {
        let incoming_system = delta
            .new_systems
            .iter()
            .find(|system| SystemId::from_address(&system.address) == *sys_id);
        let sys_name = incoming_system
            .map(|system| system.name.as_deref().unwrap_or("-"))
            .or_else(|| {
                model
                    .system(sys_id)
                    .and_then(|system| system.name.as_deref())
            })
            .unwrap_or("-");
        let biome = planet
            .biome
            .map(|b| b.to_string())
            .unwrap_or_else(|| "?".into());
        let planet_name = planet.name.as_deref().unwrap_or("-");
        notes.push(format!(
            "  New scan: \"{planet_name}\" ({biome}) in {sys_name}"
        ));
    }

    for base in &delta.new_bases {
        notes.push(format!("  New base: {}", base.name));
    }

    notes
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
    let notes = notifications_for_delta(model, delta);

    // Apply delta to model
    model.apply_delta(delta);

    session.sync_from_model(model);

    notes
}

/// Look up the name of the nearest system to an address.
fn system_name_near(model: &GalaxyModel, addr: &nms_core::address::GalacticAddress) -> String {
    model
        .nearest_systems_from_address(addr, 1)
        .first()
        .and_then(|(id, _)| model.system(id))
        .and_then(|s| s.name.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::PositionContext;
    use nms_core::address::GalacticAddress;
    use nms_core::delta::PlayerMoved;
    use nms_core::player::PlayerState;
    use nms_core::system::{Planet, System};

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

    fn collision_model(player_address: GalacticAddress, selected: u8) -> GalaxyModel {
        let mut model = GalaxyModel::new();
        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        model.insert_system(System::new(
            a0,
            Some("Origin-0".into()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(nms_core::biome::Biome::Lush),
                None,
                false,
                Some("Leaf-0".into()),
                None,
            )],
        ));
        model.insert_system(System::new(
            a9,
            Some("Destination-9".into()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(nms_core::biome::Biome::Toxic),
                None,
                false,
                Some("Leaf-9".into()),
                None,
            )],
        ));
        model.player_state = Some(PlayerState::new(
            player_address,
            player_address.reality_index,
            None,
            None,
            7,
            8,
            9,
        ));
        model.set_active_galaxy(selected);
        model
    }

    #[test]
    fn test_apply_and_notify_empty_delta() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        session.system_count = 0;
        session.planet_count = 0;
        model.set_active_galaxy(9);
        let delta = SaveDelta::empty();

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.is_empty());
        assert_eq!(session.galaxy.index, 9);
        assert_eq!(session.system_count, model.system_count());
        assert_eq!(session.planet_count, model.planet_count());
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
    fn test_apply_and_notify_cross_reality_uses_qualified_system_names() {
        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let mut model = collision_model(a0, 9);
        let mut session = SessionState::from_model(&model);
        let delta = SaveDelta {
            player_moved: Some(PlayerMoved { from: a0, to: a9 }),
            ..SaveDelta::empty()
        };
        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert_eq!(notes[0], "  Warped: Origin-0 -> Destination-9");
        assert_eq!(session.galaxy.index, 9);
        assert!(
            matches!(session.position, Some(PositionContext::PlayerPosition(address)) if address == a9)
        );

        let delta = SaveDelta {
            new_systems: vec![System::new(a0, Some("Origin-0".into()), None, None, vec![])],
            player_moved: Some(PlayerMoved { from: a9, to: a0 }),
            ..SaveDelta::empty()
        };
        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert_eq!(notes[0], "  Warped: Destination-9 -> Origin-0");
        assert_eq!(session.galaxy.index, 0);
        assert!(
            matches!(session.position, Some(PositionContext::PlayerPosition(address)) if address == a0)
        );

        for use_base_anchor in [false, true] {
            let mut model = collision_model(a0, 0);
            let mut session = SessionState::from_model(&model);
            let anchor = GalacticAddress::new(12, 4, 3, 2, 0, 7);
            session.position = Some(if use_base_anchor {
                PositionContext::Base {
                    name: "Fixed Base".into(),
                    address: anchor,
                }
            } else {
                PositionContext::Address(anchor)
            });
            session.set_biome_filter(nms_core::biome::Biome::Lush);
            session.set_warp_range(400.0);
            apply_and_notify(
                &mut model,
                &mut session,
                &SaveDelta {
                    player_moved: Some(PlayerMoved { from: a0, to: a9 }),
                    ..SaveDelta::empty()
                },
            );
            assert_eq!(session.galaxy.index, 9);
            assert_eq!(session.biome_filter, Some(nms_core::biome::Biome::Lush));
            assert_eq!(session.warp_range, Some(400.0));
            if use_base_anchor {
                assert!(
                    matches!(&session.position, Some(PositionContext::Base { name, address }) if name == "Fixed Base" && *address == anchor)
                );
            } else {
                assert!(
                    matches!(session.position, Some(PositionContext::Address(address)) if address == anchor)
                );
            }
            apply_and_notify(
                &mut model,
                &mut session,
                &SaveDelta {
                    player_moved: Some(PlayerMoved { from: a9, to: a0 }),
                    ..SaveDelta::empty()
                },
            );
            assert_eq!(session.galaxy.index, 0);
            assert_eq!(session.biome_filter, Some(nms_core::biome::Biome::Lush));
            assert_eq!(session.warp_range, Some(400.0));
            if use_base_anchor {
                assert!(
                    matches!(&session.position, Some(PositionContext::Base { name, address }) if name == "Fixed Base" && *address == anchor)
                );
            } else {
                assert!(
                    matches!(session.position, Some(PositionContext::Address(address)) if address == anchor)
                );
            }
        }
    }

    #[test]
    fn test_apply_and_notify_incoming_destination_and_scan_use_exact_identity() {
        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let mut model = GalaxyModel::new();
        model.insert_system(System::new(a0, Some("Origin-0".into()), None, None, vec![]));
        model.insert_system(System::new(
            a9,
            Some("Old-Destination-Name".into()),
            None,
            None,
            vec![],
        ));
        let mut session = SessionState::from_model(&model);
        let new_system = System::new(
            a9,
            Some("Destination-9".into()),
            None,
            None,
            vec![Planet::new(
                0,
                Some(nms_core::biome::Biome::Toxic),
                None,
                false,
                Some("Leaf-9".into()),
                None,
            )],
        );
        let delta = SaveDelta {
            new_systems: vec![new_system],
            new_planets: vec![(
                SystemId::from_address(&a9),
                Planet::new(
                    0,
                    Some(nms_core::biome::Biome::Toxic),
                    None,
                    false,
                    Some("Leaf-9".into()),
                    None,
                ),
            )],
            player_moved: Some(PlayerMoved { from: a0, to: a9 }),
            ..SaveDelta::empty()
        };
        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert_eq!(notes[0], "  Warped: Origin-0 -> Destination-9");
        assert_eq!(notes[1], "  New system: Destination-9 (1 planet)");
        assert_eq!(notes[2], "  New scan: \"Leaf-9\" (Toxic) in Destination-9");

        let mut foreign_only = GalaxyModel::new();
        foreign_only.insert_system(System::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 0),
            Some("Foreign-0".into()),
            None,
            None,
            vec![],
        ));
        let mut session = SessionState::from_model(&foreign_only);
        let notes = apply_and_notify(
            &mut foreign_only,
            &mut session,
            &SaveDelta {
                player_moved: Some(PlayerMoved { from: a9, to: a9 }),
                ..SaveDelta::empty()
            },
        );
        assert_eq!(notes[0], "  Warped: unknown -> unknown");
        assert!(!notes[0].contains("Foreign-0"));
        assert!(foreign_only.player_state.is_none());
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

    fn cache_test_model(player_address: GalacticAddress, active_galaxy: u8) -> GalaxyModel {
        use nms_core::player::PlayerState;

        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let previous = GalacticAddress::new(5, 2, 1, 3, 0, 4);
        let freighter = GalacticAddress::new(-3, 1, 7, 8, 0, 6);
        let mut model = GalaxyModel::new();
        for (address, name, leaf, biome) in [
            (a0, "Origin-0", "Leaf-0", nms_core::biome::Biome::Lush),
            (a9, "Destination-9", "Leaf-9", nms_core::biome::Biome::Toxic),
        ] {
            model.insert_system(System::new(
                address,
                Some(name.into()),
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(biome),
                    None,
                    false,
                    Some(leaf.into()),
                    None,
                )],
            ));
        }
        model.player_state = Some(PlayerState::new(
            player_address,
            player_address.reality_index,
            Some(previous),
            Some(freighter),
            7,
            8,
            9,
        ));
        model.set_active_galaxy(active_galaxy);
        model
    }

    #[test]
    fn test_drain_watch_events_cross_and_same_reality_movements_roundtrip_cache() {
        use nms_core::address::GalacticAddress;
        use nms_core::delta::PlayerMoved;
        use nms_core::player::PlayerState;
        use nms_core::system::{Planet, System, SystemId};

        let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let final_address = GalacticAddress::new(1, 0, 0, 2, 0, 9);
        let next_id = SystemId::from_address(&final_address);
        let directory = tempfile::tempdir().unwrap();
        let cache_path = directory.path().join("live.rkyv");

        let mut model = cache_test_model(a0, 0);
        let mut session = SessionState::from_model(&model);
        let anchor = GalacticAddress::new(9, 8, 7, 6, 0, 3);
        session.position = Some(PositionContext::Address(anchor));
        let (sender, receiver) = mpsc::channel();
        sender
            .send(SaveDelta {
                player_moved: Some(PlayerMoved { from: a0, to: a9 }),
                ..SaveDelta::empty()
            })
            .unwrap();
        sender
            .send(SaveDelta {
                new_systems: vec![System::new(
                    final_address,
                    Some("Next-9".into()),
                    None,
                    None,
                    vec![Planet::new(
                        0,
                        Some(nms_core::biome::Biome::Frozen),
                        None,
                        false,
                        Some("Next-leaf".into()),
                        None,
                    )],
                )],
                new_planets: vec![(
                    next_id,
                    Planet::new(
                        0,
                        Some(nms_core::biome::Biome::Frozen),
                        None,
                        false,
                        Some("Next-leaf".into()),
                        None,
                    ),
                )],
                player_moved: Some(PlayerMoved {
                    from: a9,
                    to: final_address,
                }),
                ..SaveDelta::empty()
            })
            .unwrap();
        drop(sender);

        drain_watch_events(&receiver, &mut model, &mut session, Some(&cache_path), 4720);
        assert!(
            matches!(session.position, Some(PositionContext::Address(address)) if address == anchor)
        );
        assert_eq!(session.galaxy.index, 9);
        let bytes_before_empty = std::fs::read(&cache_path).unwrap();
        let data = nms_cache::read_cache(&cache_path).unwrap();
        let rebuilt = nms_cache::rebuild_model(&data);
        let expected_state = PlayerState::new(
            final_address,
            9,
            Some(GalacticAddress::new(5, 2, 1, 3, 0, 4)),
            Some(GalacticAddress::new(-3, 1, 7, 8, 0, 6)),
            7,
            8,
            9,
        );
        assert_eq!(rebuilt.player_state, Some(expected_state));
        assert_eq!(rebuilt.active_galaxy, 9);
        let id0 = SystemId::from_address(&a0);
        let id9 = SystemId::from_address(&a9);
        assert_eq!(
            rebuilt.system(&id0).unwrap().name.as_deref(),
            Some("Origin-0")
        );
        assert_eq!(
            rebuilt.system(&id9).unwrap().name.as_deref(),
            Some("Destination-9")
        );
        assert_eq!(
            rebuilt.system(&next_id).unwrap().name.as_deref(),
            Some("Next-9")
        );
        assert_eq!(
            rebuilt.planets.get(&(id0, 0)).unwrap().name.as_deref(),
            Some("Leaf-0")
        );
        assert_eq!(
            rebuilt.planets.get(&(id9, 0)).unwrap().name.as_deref(),
            Some("Leaf-9")
        );
        assert_eq!(
            rebuilt.planets.get(&(next_id, 0)).unwrap().name.as_deref(),
            Some("Next-leaf")
        );
        let rebuilt_session = SessionState::from_model(&rebuilt);
        assert_eq!(rebuilt_session.galaxy.index, 9);
        assert!(
            matches!(rebuilt_session.position, Some(PositionContext::PlayerPosition(address)) if address == final_address)
        );
        assert_eq!(rebuilt_session.system_count, rebuilt.system_count());
        assert_eq!(rebuilt_session.planet_count, rebuilt.planet_count());

        let (_empty_sender, empty_receiver) = mpsc::channel::<SaveDelta>();
        drain_watch_events(
            &empty_receiver,
            &mut model,
            &mut session,
            Some(&cache_path),
            4720,
        );
        assert_eq!(std::fs::read(&cache_path).unwrap(), bytes_before_empty);
    }

    #[test]
    fn test_drain_watch_events_same_reality_preserves_selected_galaxy_in_cache() {
        use nms_core::address::GalacticAddress;
        use nms_core::delta::PlayerMoved;
        use nms_core::system::{System, SystemId};

        let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let moved_to = GalacticAddress::new(2, 0, 0, 3, 0, 9);
        let mut model = cache_test_model(a9, 7);
        let mut session = SessionState::from_model(&model);
        let (sender, receiver) = mpsc::channel();
        sender
            .send(SaveDelta {
                new_systems: vec![System::new(
                    moved_to,
                    Some("Same-Reality-9".into()),
                    None,
                    None,
                    vec![],
                )],
                player_moved: Some(PlayerMoved {
                    from: a9,
                    to: moved_to,
                }),
                ..SaveDelta::empty()
            })
            .unwrap();
        drop(sender);
        let directory = tempfile::tempdir().unwrap();
        let cache_path = directory.path().join("selected.rkyv");

        drain_watch_events(&receiver, &mut model, &mut session, Some(&cache_path), 4720);
        let rebuilt = nms_cache::rebuild_model(&nms_cache::read_cache(&cache_path).unwrap());
        assert_eq!(rebuilt.active_galaxy, 7);
        assert_eq!(rebuilt.player_state.as_ref().unwrap().current_reality, 9);
        assert_eq!(rebuilt.player_position(), Some(&moved_to));
        assert!(rebuilt.system(&SystemId::from_address(&moved_to)).is_some());
    }
}
