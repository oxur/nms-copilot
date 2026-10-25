//! Watch-related REPL helpers and read-only applied-update notification drains.
//!
//! [`drain_watch_events`] retains the legacy delta-only REPL path: it applies
//! queued [`SaveDelta`] values, synchronizes the caller's session, prints
//! notes, and may write one cache snapshot after the batch. The normal
//! application path instead uses [`crate::coordinator::LiveSaveCoordinator`]
//! and gives read-only subscribers [`drain_applied_notifications`]; draining
//! those notifications does not apply an update or persist a cache.
//!
//! [`notifications_for_delta`] formats notes against the model before an
//! update. [`apply_and_notify`] uses that formatter, then applies a delta and
//! synchronizes the REPL session. These helpers describe the legacy delta
//! behavior; they do not provide the coordinator's complete player state,
//! explicit discovery-name observations, or same-parse cache version.

use std::path::Path;
use std::sync::mpsc;

use nms_core::delta::SaveDelta;
use nms_graph::GalaxyModel;

use crate::coordinator::AppliedSaveUpdate;
use crate::session::SessionState;
use nms_core::system::SystemId;

/// Drain and apply all currently queued legacy watcher deltas.
///
/// For each received delta, this formats notes before application with
/// [`apply_and_notify`], applies the additive atlas changes, synchronizes
/// `session`, and prints each note to stdout. It does not apply the complete player state, source version, or
/// discovery-name observations carried by the coordinator path. An empty or
/// disconnected receiver ends the nonblocking drain.
///
/// If at least one delta was received, `cache_path` causes one cache snapshot
/// and best-effort write after the whole batch, using the caller-supplied
/// `save_version`; no received delta means no cache write. A received empty
/// delta still counts as a batch item. Cache failures are reported to stderr;
/// they do not return an error or roll back model/session updates. `None`
/// skips cache extraction and writing.
///
/// # Panics
///
/// Writing a notification or cache-failure warning to stdout or stderr can
/// panic if the output stream fails.
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

/// Drain literal notifications from already-applied updates.
///
/// Repeatedly calls `try_recv` and concatenates each event's `notifications`
/// in publication order. It stops when the channel is empty or disconnected;
/// it does not wait for future events. This read-only observer has no access
/// to the model, cache, or session, so consuming notifications cannot apply
/// state or persist it a second time. Strings are returned exactly as stored,
/// even if they were constructed independently of the coordinator's formatter.
/// This is not a cache-write receipt, replay, or model snapshot.
///
/// # Examples
///
/// ```
/// use std::sync::mpsc;
/// use nms_copilot::{coordinator::AppliedSaveUpdate, watch::drain_applied_notifications};
/// use nms_core::{GalacticAddress, PlayerState, SaveDelta, SaveUpdate};
///
/// let (sender, receiver) = mpsc::channel();
/// let first = AppliedSaveUpdate {
///     update: SaveUpdate {
///         delta: SaveDelta::empty(),
///         player_state: PlayerState::new(
///             GalacticAddress::new(0, 0, 0, 1, 0, 0), 0, None, None, 7, 8, 9,
///         ),
///         save_version: 4720,
///     },
///     names_changed: false,
///     notifications: vec!["Warped: Origin-0 -> Imported-9".to_owned()],
/// };
/// let second = AppliedSaveUpdate {
///     update: SaveUpdate {
///         delta: SaveDelta::empty(),
///         player_state: PlayerState::new(
///             GalacticAddress::new(0, 0, 0, 1, 0, 0), 0, None, None, 7, 8, 9,
///         ),
///         save_version: 4720,
///     },
///     names_changed: false,
///     notifications: vec!["New planet: Leaf-9".to_owned()],
/// };
/// sender.send(first)?;
/// sender.send(second)?;
/// assert_eq!(
///     drain_applied_notifications(&receiver),
///     ["Warped: Origin-0 -> Imported-9", "New planet: Leaf-9"]
/// );
/// assert!(drain_applied_notifications(&receiver).is_empty());
/// drop(sender);
/// assert!(drain_applied_notifications(&receiver).is_empty());
/// Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn drain_applied_notifications(receiver: &mpsc::Receiver<AppliedSaveUpdate>) -> Vec<String> {
    let mut notes = Vec::new();
    while let Ok(event) = receiver.try_recv() {
        notes.extend(event.notifications);
    }
    notes
}

#[cfg(test)]
mod applied_notification_tests {
    use super::*;
    use nms_core::{PlayerState, SaveDelta, SaveUpdate};

    fn event(note: &str) -> AppliedSaveUpdate {
        AppliedSaveUpdate {
            update: SaveUpdate {
                delta: SaveDelta::empty(),
                player_state: PlayerState::new(
                    nms_core::GalacticAddress::new(0, 0, 0, 1, 0, 0),
                    0,
                    None,
                    None,
                    7,
                    8,
                    9,
                ),
                save_version: 4720,
            },
            names_changed: false,
            notifications: vec![note.to_owned()],
        }
    }

    #[test]
    fn test_drain_applied_notifications_preserves_order_then_empty_and_disconnect() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(event("Warped: Origin-0 -> Imported-9"))
            .unwrap();
        sender.send(event("New planet: Leaf-9")).unwrap();

        assert_eq!(
            drain_applied_notifications(&receiver),
            ["Warped: Origin-0 -> Imported-9", "New planet: Leaf-9"]
        );
        assert!(drain_applied_notifications(&receiver).is_empty());
        drop(sender);
        assert!(drain_applied_notifications(&receiver).is_empty());
    }
}

/// Format user-facing notes from a delta against the current model.
///
/// This function only reads the model and returns strings; it does not apply
/// the delta, print output, or report cache status. Notes are ordered as
/// movement, each new system, each new planet, then each new base. Modified
/// bases, removals, and name-only changes do not produce notes. System lookup
/// uses the galaxy-qualified, planet-zeroed system key. A movement destination
/// uses a matching incoming system only when the model has no genuine system
/// there or has a player placeholder; otherwise the existing system name is
/// retained. For each scan, the system name comes from the first exact-key
/// matching incoming `new_system`, before a stored-system lookup. Missing
/// movement names render as `unknown`, unnamed new systems and planets as `-`,
/// and planets without a biome as `?`. Every generated note begins with two
/// ASCII spaces. These notes describe the incoming delta, not a receipt that
/// every duplicate record was inserted.
///
/// [`apply_and_notify`] uses this formatter before applying a legacy delta.
pub fn notifications_for_delta(model: &GalaxyModel, delta: &SaveDelta) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(ref moved) = delta.player_moved {
        let from_name = system_name_at_key(model, &moved.from);
        let to_id = SystemId::from_address(&moved.to);
        let destination_can_be_inserted =
            model.system(&to_id).is_none() || model.is_player_placeholder(&to_id);
        let incoming_destination = destination_can_be_inserted
            .then(|| {
                delta
                    .new_systems
                    .iter()
                    .find(|system| SystemId::from_address(&system.address) == to_id)
            })
            .flatten();
        let to_name = incoming_destination.map_or_else(
            || system_name_at_key(model, &moved.to),
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

/// Format notes against the current model, then apply a legacy delta.
///
/// Notes are generated before `model.apply_delta`, preserving lookups for the
/// movement origin. A movement changes the physical player address only when
/// player state already exists; only a cross-galaxy physical move changes the
/// selected galaxy. The helper then synchronizes model-derived session fields:
/// it refreshes the selected galaxy and counts, follows a missing or
/// player-following position, and preserves base or manual-address snapshots,
/// biome filter, and warp preference. It does not print, apply complete player
/// fields or explicit discovery names, create player state, start a watcher,
/// ensure a player placeholder, or write a cache.
///
/// # Examples
///
/// This in-memory example formats and applies a cross-galaxy update without
/// reading a save file, starting a watcher, or writing a cache. The expected
/// notifications are literal user-facing strings, including their two-space
/// prefixes.
///
/// ```
/// use nms_copilot::session::{PositionContext, SessionState};
/// use nms_copilot::watch::{apply_and_notify, notifications_for_delta};
/// use nms_core::{BaseType, Biome, GalacticAddress, PlayerBase, PlayerMoved, PlayerState, Planet, SaveDelta, System, SystemId};
/// use nms_graph::GalaxyModel;
///
/// let origin = GalacticAddress::new(3, 0, 0, 1, 2, 0);
/// let destination = GalacticAddress::new(3, 0, 0, 1, 4, 9);
/// let origin_id = SystemId::from_address(&origin);
/// let destination_id = SystemId::from_address(&destination);
/// let mut model = GalaxyModel::new();
/// model.set_active_galaxy(7);
/// model.player_state = Some(PlayerState::new(origin, 0, None, None, 7, 8, 9));
/// model.insert_system(System::new(
///     origin,
///     Some("Origin-0".into()),
///     None,
///     None,
///     vec![],
/// ));
/// let mut session = SessionState::from_model(&model);
/// session.position = Some(PositionContext::Address(origin));
/// session.biome_filter = Some(Biome::Lush);
/// session.warp_range = Some(1200.0);
///
/// let delta = SaveDelta {
///     new_systems: vec![System::new(
///         destination,
///         Some("Destination-9".into()),
///         None,
///         None,
///         vec![Planet::new(4, Some(Biome::Lush), None, false, Some("Garden-9".into()), None)],
///     )],
///     new_planets: vec![(
///         destination_id,
///         Planet::new(5, Some(Biome::Toxic), None, false, Some("Leaf-9".into()), None),
///     )],
///     player_moved: Some(PlayerMoved { from: origin, to: destination }),
///     new_bases: vec![PlayerBase::new(
///         "Outpost-9".into(),
///         BaseType::HomePlanetBase,
///         destination,
///         [0.0, 0.0, 0.0],
///         None,
///     )],
///     modified_bases: vec![],
/// };
/// let expected_before = vec![
///     "  Warped: Origin-0 -> Destination-9".to_owned(),
///     "  New system: Destination-9 (1 planet)".to_owned(),
///     "  New scan: \"Leaf-9\" (Toxic) in Destination-9".to_owned(),
///     "  New base: Outpost-9".to_owned(),
/// ];
/// assert_eq!(notifications_for_delta(&model, &delta), expected_before);
/// assert_eq!((model.system_count(), model.planet_count(), model.bases.len()), (1, 0, 0));
/// assert_eq!(model.active_galaxy, 7);
/// let before = model.player_state.as_ref().map(|player| (
///     player.current_address.packed(), player.current_address.reality_index,
///     player.current_reality, player.previous_address, player.freighter_address,
///     player.units, player.nanites, player.quicksilver,
/// ));
/// assert_eq!(before, Some((0x2001_0000_0003, 0, 0, None, None, 7, 8, 9)));
///
/// let notes = apply_and_notify(&mut model, &mut session, &delta);
/// assert_eq!(notes, vec![
///     "  Warped: Origin-0 -> Destination-9".to_owned(),
///     "  New system: Destination-9 (1 planet)".to_owned(),
///     "  New scan: \"Leaf-9\" (Toxic) in Destination-9".to_owned(),
///     "  New base: Outpost-9".to_owned(),
/// ]);
/// assert_eq!((model.system_count(), model.planet_count(), model.bases.len()), (2, 2, 1));
/// assert_eq!(model.active_galaxy, 9);
/// let after = model.player_state.as_ref().map(|player| (
///     player.current_address.packed(), player.current_address.reality_index,
///     player.current_reality, player.previous_address, player.freighter_address,
///     player.units, player.nanites, player.quicksilver,
/// ));
/// assert_eq!(after, Some((0x4001_0000_0003, 9, 9, None, None, 7, 8, 9)));
/// assert_eq!(model.system(&origin_id).map(|system| system.name.as_deref()), Some(Some("Origin-0")));
/// assert_eq!(model.system(&destination_id).map(|system| system.name.as_deref()), Some(Some("Destination-9")));
/// assert_eq!(model.planets.get(&(destination_id, 4)).map(|planet| (planet.name.as_deref(), planet.biome)), Some((Some("Garden-9"), Some(Biome::Lush))));
/// assert_eq!(model.planets.get(&(destination_id, 5)).map(|planet| (planet.name.as_deref(), planet.biome)), Some((Some("Leaf-9"), Some(Biome::Toxic))));
/// let mut system_keys: Vec<_> = model.systems.keys()
///     .map(|id| (id.reality_index(), id.system_address()))
///     .collect();
/// system_keys.sort_unstable();
/// assert_eq!(system_keys, [(0, 0x0001_0000_0003), (9, 0x0001_0000_0003)]);
/// let mut planet_keys: Vec<_> = model.planets.keys()
///     .map(|(id, index)| (id.reality_index(), id.system_address(), *index))
///     .collect();
/// planet_keys.sort_unstable();
/// assert_eq!(planet_keys, [(9, 0x0001_0000_0003, 4), (9, 0x0001_0000_0003, 5)]);
/// let mut base_keys: Vec<_> = model.bases.keys().map(String::as_str).collect();
/// base_keys.sort_unstable();
/// assert_eq!(base_keys, ["outpost-9"]);
/// assert_eq!(
///     model.bases.get("outpost-9").map(|base| (
///         base.name.as_str(), base.address.packed(), base.address.reality_index,
///     )),
///     Some(("Outpost-9", 0x4001_0000_0003, 9)),
/// );
/// assert_eq!(
///     (session.galaxy.index, session.galaxy.name, session.galaxy.galaxy_type,
///      session.system_count, session.planet_count,
///      session.position.as_ref().map(|position| {
///          let kind = match position {
///              PositionContext::Address(_) => "address",
///              PositionContext::Base { .. } => "base",
///              PositionContext::PlayerPosition(_) => "player",
///          };
///          (
///              kind,
///              position.label(),
///              position.address().packed(),
///              position.address().reality_index,
///          )
///      }),
///      session.biome_filter, session.warp_range),
///     (9, "Eissentam", nms_core::GalaxyType::Lush, 2, 2,
///      Some(("address", "0x200100000003".to_owned(), 0x2001_0000_0003, 0)),
///      Some(Biome::Lush), Some(1200.0)),
/// );
/// ```
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

/// Look up the name registered for the address's system key.
fn system_name_at_key(model: &GalaxyModel, addr: &nms_core::address::GalacticAddress) -> String {
    let id = SystemId::from_address(addr);
    model
        .system(&id)
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

        let mut duplicate = GalaxyModel::new();
        duplicate.insert_system(System::new(a0, Some("Origin-0".into()), None, None, vec![]));
        duplicate.insert_system(System::new(
            a9,
            Some("Existing-Destination".into()),
            None,
            None,
            vec![],
        ));
        let mut duplicate_session = SessionState::from_model(&duplicate);
        let duplicate_notes = apply_and_notify(
            &mut duplicate,
            &mut duplicate_session,
            &SaveDelta {
                new_systems: vec![System::new(
                    a9,
                    Some("Ignored-Incoming-Name".into()),
                    None,
                    None,
                    vec![],
                )],
                player_moved: Some(PlayerMoved { from: a0, to: a9 }),
                ..SaveDelta::empty()
            },
        );
        assert_eq!(
            duplicate_notes[0],
            "  Warped: Origin-0 -> Existing-Destination"
        );
        assert_eq!(
            duplicate
                .system(&SystemId::from_address(&a9))
                .unwrap()
                .name
                .as_deref(),
            Some("Existing-Destination")
        );

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
