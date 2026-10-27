use nms_copilot::session::SessionState;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::delta::{PlayerMoved, SaveDelta};
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System};
use nms_graph::GalaxyModel;
use nms_graph::edges::EdgeStrategy;
use reedline::Prompt;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": [
                    {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                ]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

fn setup() -> (GalaxyModel, SessionState) {
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);
    let session = SessionState::from_model(&model);
    (model, session)
}

fn galaxy_fixture(reverse: bool) -> (GalaxyModel, SessionState) {
    let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let b0 = GalacticAddress::new(1, 0, 0, 2, 0, 0);
    let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let b9 = GalacticAddress::new(2, 0, 0, 2, 0, 9);
    let planet =
        |name: &str| Planet::new(0, Some(Biome::Lush), None, false, Some(name.into()), None);
    let mut systems = vec![
        System::new(
            a0,
            Some("Origin-0".into()),
            None,
            None,
            vec![planet("Leaf-0")],
        ),
        System::new(b0, Some("Twin".into()), None, None, vec![planet("Bud-0")]),
        System::new(
            a9,
            Some("Origin-9".into()),
            None,
            None,
            vec![planet("Leaf-9")],
        ),
        System::new(b9, Some("Twin".into()), None, None, vec![planet("Bud-9")]),
    ];
    if reverse {
        systems.reverse();
    }

    let mut model = GalaxyModel::new();
    for system in systems {
        model.insert_system(system);
    }
    model.insert_base(PlayerBase::new(
        "Camp-0".into(),
        BaseType::HomePlanetBase,
        a0,
        [0.0; 3],
        None,
    ));
    model.insert_base(PlayerBase::new(
        "Camp-9".into(),
        BaseType::HomePlanetBase,
        a9,
        [0.0; 3],
        None,
    ));
    model.player_state = Some(PlayerState::new(
        a0,
        0,
        Some(GalacticAddress::new(4, 0, 0, 3, 0, 0)),
        Some(GalacticAddress::new(5, 0, 0, 4, 0, 0)),
        7,
        8,
        9,
    ));
    model.build_edges(EdgeStrategy::default());
    let session = SessionState::from_model(&model);
    (model, session)
}

fn dispatch_line(
    model: &mut GalaxyModel,
    session: &mut SessionState,
    line: &str,
) -> Result<String, String> {
    let action = nms_copilot::commands::parse_line(line)?.expect("non-empty command");
    nms_copilot::dispatch::dispatch(&action, model, session)
}

#[test]
fn galaxy_command_lists_selects_and_rejects_atomically() {
    let (mut model, mut session) = galaxy_fixture(false);
    let systems_before = model.systems.clone();
    let planets_before = model.planets.clone();
    let physical = model.player_state.clone();
    let listing = dispatch_line(&mut model, &mut session, "galaxy").unwrap();
    assert_eq!(
        listing,
        "Active galaxy: Euclid (0)\nDiscovered galaxies: Euclid (0), Eissentam (9)\n"
    );

    session.set_biome_filter(Biome::Lush);
    session.set_warp_range(1200.0);
    session.set_position_base("Camp-0", &model).unwrap();
    let output = dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    assert_eq!(
        output,
        "Switched to Eissentam (galaxy 9). 2 systems loaded.\n"
    );
    assert_eq!(model.active_galaxy, 9);
    assert_eq!(model.player_state, physical);
    assert_eq!(model.systems, systems_before);
    assert_eq!(model.planets, planets_before);
    assert_eq!(session.galaxy.index, 9);
    assert!(
        matches!(&session.position, Some(nms_copilot::session::PositionContext::Base { name, address }) if name == "Camp-0" && address.reality_index == 0)
    );
    assert_eq!(session.biome_filter, Some(Biome::Lush));
    assert_eq!(session.warp_range, Some(1200.0));
    let prompt_state = nms_copilot::prompt::PromptState::from_session(&session);
    assert_eq!(prompt_state.galaxy_name, "Eissentam");
    assert_eq!(prompt_state.planet_count, 4);
    let prompt = nms_copilot::prompt::CopilotPrompt::new(prompt_state);
    assert_eq!(
        prompt.render_prompt_left().as_ref(),
        "[Eissentam | Lush | 4 planets] 🚀"
    );
    let status = dispatch_line(&mut model, &mut session, "status").unwrap();
    assert!(status.contains("Galaxy:      Eissentam (9; Lush)"));
    assert!(status.contains("Position:    Camp-0 (Euclid, galaxy 0)"));

    session.system_count = 999;
    session.planet_count = 888;
    let before_galaxy = session.galaxy.clone();
    assert!(dispatch_line(&mut model, &mut session, "galaxy Unknown").is_err());
    assert!(dispatch_line(&mut model, &mut session, "galaxy 256").is_err());
    assert!(dispatch_line(&mut model, &mut session, "galaxy -1").is_err());
    assert_eq!(model.active_galaxy, 9);
    assert_eq!(session.galaxy, before_galaxy);
    assert!(
        matches!(&session.position, Some(nms_copilot::session::PositionContext::Base { name, address }) if name == "Camp-0" && address.reality_index == 0)
    );
    assert_eq!((session.system_count, session.planet_count), (999, 888));

    assert!(
        dispatch_line(&mut model, &mut session, "galaxy \"eIsSeNtAm\"")
            .unwrap()
            .contains("Eissentam")
    );
    assert!(
        dispatch_line(&mut model, &mut session, "galaxy \"Hilbert Dimension\"")
            .unwrap()
            .contains("0 systems loaded")
    );
    assert!(
        dispatch_line(&mut model, &mut session, "galaxy 255")
            .unwrap()
            .contains("galaxy 255")
    );
    assert!(
        dispatch_line(&mut model, &mut session, "galaxy 0")
            .unwrap()
            .contains("Euclid")
    );
    assert!(
        dispatch_line(&mut model, &mut session, "galaxy 9")
            .unwrap()
            .contains("Eissentam")
    );
}

#[test]
fn galaxy_selection_preserves_slice09_movement_and_follow_semantics() {
    let (mut model, mut session) = galaxy_fixture(false);
    let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let b9 = GalacticAddress::new(2, 0, 0, 2, 0, 9);
    let initial = model.player_state.clone().unwrap();

    dispatch_line(&mut model, &mut session, "galaxy 7").unwrap();
    let cross = SaveDelta {
        player_moved: Some(PlayerMoved { from: a0, to: a9 }),
        ..SaveDelta::empty()
    };
    nms_copilot::watch::apply_and_notify(&mut model, &mut session, &cross);
    assert_eq!(model.active_galaxy, 9);
    assert_eq!(session.galaxy.index, 9);
    assert!(
        matches!(session.position, Some(nms_copilot::session::PositionContext::PlayerPosition(address)) if address == a9)
    );
    assert_eq!(model.player_state.as_ref().unwrap().current_address, a9);
    assert_eq!(
        model.player_state.as_ref().unwrap().previous_address,
        initial.previous_address
    );
    assert_eq!(
        model.player_state.as_ref().unwrap().freighter_address,
        initial.freighter_address
    );
    assert_eq!(
        (
            model.player_state.as_ref().unwrap().units,
            model.player_state.as_ref().unwrap().nanites,
            model.player_state.as_ref().unwrap().quicksilver
        ),
        (7, 8, 9)
    );

    dispatch_line(&mut model, &mut session, "galaxy 7").unwrap();
    let same_reality = SaveDelta {
        player_moved: Some(PlayerMoved { from: a9, to: b9 }),
        ..SaveDelta::empty()
    };
    nms_copilot::watch::apply_and_notify(&mut model, &mut session, &same_reality);
    assert_eq!(model.active_galaxy, 7);
    assert_eq!(session.galaxy.index, 7);
    assert!(
        matches!(session.position, Some(nms_copilot::session::PositionContext::PlayerPosition(address)) if address == b9)
    );
    assert_eq!(model.player_state.as_ref().unwrap().current_address, b9);

    session.set_position_base("Camp-0", &model).unwrap();
    let back_to_zero = SaveDelta {
        player_moved: Some(PlayerMoved { from: b9, to: a0 }),
        ..SaveDelta::empty()
    };
    nms_copilot::watch::apply_and_notify(&mut model, &mut session, &back_to_zero);
    assert_eq!(model.active_galaxy, 0);
    assert!(
        matches!(&session.position, Some(nms_copilot::session::PositionContext::Base { name, address }) if name == "Camp-0" && address == &a0)
    );
}

#[test]
fn galaxy_selection_and_find_preserve_reference_scope() {
    let (mut model, mut session) = galaxy_fixture(true);
    dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    assert_eq!(
        dispatch_line(&mut model, &mut session, "find --nearest 1").unwrap_err(),
        "reference galaxy 0 differs from active galaxy 9; use an explicit address or base"
    );

    dispatch_line(&mut model, &mut session, "set position Camp-9").unwrap();
    let leaf9 = dispatch_line(&mut model, &mut session, "find --nearest 1").unwrap();
    assert!(leaf9.contains("Leaf-9"));
    assert!(!leaf9.contains("Leaf-0"));
    dispatch_line(&mut model, &mut session, "galaxy 0").unwrap();
    assert!(
        dispatch_line(&mut model, &mut session, "find --nearest 1")
            .unwrap()
            .contains("Leaf-9")
    );
    assert!(
        dispatch_line(&mut model, &mut session, "find --from Camp-0 --nearest 1")
            .unwrap()
            .contains("Leaf-0")
    );

    session.set_position_address(GalacticAddress::new(0, 0, 0, 1, 0, 9));
    dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    assert!(
        matches!(session.position, Some(nms_copilot::session::PositionContext::Address(address)) if address.reality_index == 9)
    );
    dispatch_line(&mut model, &mut session, "galaxy 0").unwrap();
    assert!(
        dispatch_line(&mut model, &mut session, "find --nearest 1")
            .unwrap()
            .contains("Leaf-9")
    );
    session.position = Some(nms_copilot::session::PositionContext::PlayerPosition(
        GalacticAddress::new(2, 0, 0, 2, 0, 9),
    ));
    assert!(
        dispatch_line(&mut model, &mut session, "find --nearest 1")
            .unwrap()
            .contains("Leaf-0")
    );
}

#[test]
fn route_reference_precedence_is_consistent_for_both_algorithms() {
    for algo in ["nn", "2opt"] {
        let (mut model, mut session) = galaxy_fixture(false);
        dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
        assert_eq!(
            dispatch_line(
                &mut model,
                &mut session,
                &format!("route --target Twin --algo {algo}")
            )
            .unwrap_err(),
            "reference galaxy 0 differs from active galaxy 9; use an explicit address or base"
        );

        dispatch_line(&mut model, &mut session, "set position Camp-9").unwrap();
        let from9 = dispatch_line(
            &mut model,
            &mut session,
            &format!("route --target Twin --algo {algo}"),
        )
        .unwrap();
        assert!(from9.contains("Origin-9"));
        assert!(from9.contains("Twin"));
        assert!(from9.contains("~800 ly"));
        assert!(!from9.contains("Origin-0"));

        dispatch_line(&mut model, &mut session, "galaxy 0").unwrap();
        let explicit9 = dispatch_line(
            &mut model,
            &mut session,
            &format!("route --target Twin --algo {algo}"),
        )
        .unwrap();
        assert!(explicit9.contains("Origin-9"));
        assert!(explicit9.contains("~800 ly"));

        let from0 = dispatch_line(
            &mut model,
            &mut session,
            &format!("route --target Twin --from Camp-0 --algo {algo}"),
        )
        .unwrap();
        assert!(from0.contains("Origin-0"));
        assert!(from0.contains("~400 ly"));
        assert!(!from0.contains("Origin-9"));
    }
}

#[test]
fn empty_model_galaxy_listing_and_selection_are_supported() {
    let mut model = GalaxyModel::new();
    let mut session = SessionState::from_model(&model);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "galaxy").unwrap(),
        "Active galaxy: Euclid (0)\nDiscovered galaxies: (none)\n"
    );
    assert_eq!(
        dispatch_line(&mut model, &mut session, "galaxy 7").unwrap(),
        "Switched to Kikolgallr (galaxy 7). 0 systems loaded.\n"
    );
}

#[test]
fn dispatch_find_returns_results() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("find").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn dispatch_stats_returns_output() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("stats").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("GALAXY STATISTICS") || output.contains("system"));
}

#[test]
fn dispatch_info_returns_summary() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("info").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("systems"));
    assert!(output.contains("planets"));
}

#[test]
fn dispatch_help_returns_text() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("help").unwrap().unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("Commands:"));
    assert!(output.contains("galaxy"));
    assert!(output.contains("last"));
    assert!(output.contains("snapshot"));
}

#[test]
fn dispatch_convert_glyphs() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("convert --glyphs 01717D8A4EA2")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("01717D8A4EA2"));
    assert!(output.contains("Signal Booster"));
}

#[test]
fn dispatch_show_base() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("show base \"Test Base\"")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("Test Base"));
}

#[test]
fn dispatch_set_biome_filter() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("set biome Lush")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("Lush"));
    assert!(session.biome_filter.is_some());
}

#[test]
fn dispatch_status_shows_state() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("status")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("Euclid"));
    assert!(output.contains("Position:"));
}

#[test]
fn dispatch_reset_clears_state() {
    let (mut model, mut session) = setup();

    // Set a biome filter first
    let set_action = nms_copilot::commands::parse_line("set biome Toxic")
        .unwrap()
        .unwrap();
    nms_copilot::dispatch::dispatch(&set_action, &mut model, &mut session).unwrap();
    assert!(session.biome_filter.is_some());

    // Reset it
    let reset_action = nms_copilot::commands::parse_line("reset biome")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&reset_action, &mut model, &mut session).unwrap();
    assert!(output.contains("cleared"));
    assert!(session.biome_filter.is_none());
}

#[test]
fn dispatch_set_position_to_base() {
    let (mut model, mut session) = setup();
    let action = nms_copilot::commands::parse_line("set position \"Test Base\"")
        .unwrap()
        .unwrap();
    let output = nms_copilot::dispatch::dispatch(&action, &mut model, &mut session).unwrap();
    assert!(output.contains("Test Base"));
}

#[test]
fn test_last_recalls_only_successful_query_responses() {
    const EMPTY: &str = "No successful query result in this session yet.";
    const HEADER: &str = "Last successful query result (snapshot; not refreshed):\n";

    let (mut model, mut session) = galaxy_fixture(false);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        EMPTY
    );

    let mut new_session = SessionState::from_model(&model);
    assert_eq!(
        dispatch_line(&mut model, &mut new_session, "last").unwrap(),
        EMPTY
    );

    let mut previous = None;
    for command in [
        "find --nearest 1",
        "show base Camp-0",
        "stats",
        "route --target Twin",
    ] {
        let response = dispatch_line(&mut model, &mut session, command).unwrap();
        assert!(!response.is_empty(), "{command}");
        if let Some(previous) = &previous {
            assert_ne!(
                &response, previous,
                "{command} should replace the prior result"
            );
        }
        let expected = format!("{HEADER}{response}");
        assert_eq!(
            dispatch_line(&mut model, &mut session, "last").unwrap(),
            expected
        );
        assert_eq!(
            dispatch_line(&mut model, &mut session, "last").unwrap(),
            expected
        );
        previous = Some(response);
    }

    let prior = previous.unwrap();
    let empty_result = dispatch_line(
        &mut model,
        &mut session,
        "find --discoverer NoSuchDiscoverer",
    )
    .unwrap();
    assert_ne!(empty_result, prior);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{empty_result}")
    );

    assert!(dispatch_line(&mut model, &mut session, "find --biome NotABiome").is_err());
    assert!(dispatch_line(&mut model, &mut session, "not-a-command").is_err());
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{empty_result}")
    );
}

#[test]
fn test_last_is_a_snapshot_across_controls_and_live_model_sync() {
    const HEADER: &str = "Last successful query result (snapshot; not refreshed):\n";

    let (mut model, mut session) = galaxy_fixture(false);
    let original = dispatch_line(&mut model, &mut session, "find --nearest 1").unwrap();
    assert!(original.contains("Leaf-0"));

    dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{original}")
    );

    let a0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let a9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let movement = SaveDelta {
        player_moved: Some(PlayerMoved { from: a0, to: a9 }),
        ..SaveDelta::empty()
    };
    nms_copilot::watch::apply_and_notify(&mut model, &mut session, &movement);
    assert_eq!(model.active_galaxy, 9);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{original}")
    );

    dispatch_line(&mut model, &mut session, "reset all").unwrap();
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{original}")
    );

    dispatch_line(&mut model, &mut session, "set position Camp-9").unwrap();
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{original}")
    );
    let refreshed = dispatch_line(&mut model, &mut session, "find --nearest 1").unwrap();
    assert!(refreshed.contains("Leaf-9"));
    assert!(!refreshed.contains("Leaf-0"));
    assert_ne!(refreshed, original);

    dispatch_line(&mut model, &mut session, "reset all").unwrap();
    assert_eq!(
        dispatch_line(&mut model, &mut session, "last").unwrap(),
        format!("{HEADER}{refreshed}")
    );
}
