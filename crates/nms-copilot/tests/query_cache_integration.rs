use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::{GalaxyModel, GraphError, SystemId};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::show::{ShowQuery, ShowResult, execute_show};
use nms_watch::{SaveSnapshot, compute_delta};

const PREDECESSOR_ARCHIVE: &[u8] = include_bytes!("fixtures/predecessor-schema2-semantic2.rkyv");

fn system(galaxy: u8, name: &str, planet_name: &str, infested: bool) -> System {
    System::new(
        GalacticAddress::new(0, 0, 0, 1, 0, galaxy),
        Some(name.into()),
        Some("Explorer".into()),
        None,
        vec![Planet::new(
            1,
            Some(Biome::Lush),
            None,
            infested,
            Some(planet_name.into()),
            None,
        )],
    )
}

fn fresh_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    let player = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    model.player_state = Some(PlayerState::new(player, 0, None, None, 7, 8, 9));
    model.insert_system(system(0, "Shared", "Zero", false));
    model.insert_system(system(9, "Shared", "Nine", true));
    model.set_active_galaxy(9);
    model
}

fn query_projection(
    model: &GalaxyModel,
    galaxy: u8,
) -> Vec<(SystemId, Planet, System, f64, String)> {
    let address = GalacticAddress::new(0, 0, 0, 1, 0, galaxy);
    execute_find(
        model,
        &FindQuery {
            from: ReferencePoint::Address(address),
            biome: Some(Biome::Lush),
            ..Default::default()
        },
    )
    .unwrap()
    .into_iter()
    .map(|result| {
        (
            SystemId::from_address(&result.system.address),
            result.planet,
            result.system,
            result.distance_ly,
            result.portal_hex,
        )
    })
    .collect()
}

#[test]
fn schema2_semantic2_archive_is_rejected_and_current_cache_matches_fresh_queries() {
    assert_eq!(PREDECESSOR_ARCHIVE.len(), 360);
    assert_eq!(
        &PREDECESSOR_ARCHIVE[..16],
        &[
            0x4e, 0x4d, 0x53, 0x43, 0x41, 0x43, 0x48, 0x45, 2, 0, 0, 0, 2, 0, 0, 0
        ]
    );

    assert!(matches!(
        nms_cache::serialize::deserialize(PREDECESSOR_ARCHIVE),
        Err(nms_cache::CacheError::Incompatible(
            nms_cache::CacheCompatibilityError::SchemaMismatch {
                found: 2,
                expected: 3
            }
        ))
    ));

    let fresh = fresh_model();
    let current_bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    assert_eq!(&current_bytes[..16], b"NMSCACHE\x03\0\0\0\x04\0\0\0");
    let current_data = nms_cache::serialize::deserialize(&current_bytes).unwrap();
    let current = nms_cache::rebuild_model(&current_data);
    assert_eq!(current.system_count(), 2);
    assert_eq!(current.planet_count(), 2);
    assert_eq!(current.active_galaxy, 9);
    assert_eq!(current.player_position(), fresh.player_position());
    assert_eq!(current.player_state.as_ref().unwrap().units, 7);
    assert_eq!(current.player_state.as_ref().unwrap().nanites, 8);
    assert_eq!(current.player_state.as_ref().unwrap().quicksilver, 9);

    let id0 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    let id9 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 9));
    for (model, expected_id, expected_planet) in [(&current, id0, "Zero"), (&current, id9, "Nine")]
    {
        let (id, system) = model
            .system_by_name_in_galaxy("sHaReD", expected_id.reality_index())
            .unwrap();
        assert_eq!(*id, expected_id);
        assert_eq!(system.planets.len(), 1);
        assert_eq!(system.planets[0].index, 1);
        assert_eq!(system.planets[0].biome, Some(Biome::Lush));
        assert_eq!(system.planets[0].name.as_deref(), Some(expected_planet));
        assert_eq!(system.planets[0].infested, expected_planet == "Nine");
        assert_eq!(system.discoverer.as_deref(), Some("Explorer"));
    }

    for galaxy in [0, 9] {
        let fresh_projection = query_projection(&fresh, galaxy);
        let current_projection = query_projection(&current, galaxy);
        assert_eq!(current_projection, fresh_projection);
        assert_eq!(current_projection.len(), 1);
        assert_eq!(current_projection[0].0.reality_index(), galaxy);
        assert_eq!(
            current_projection[0].1.name.as_deref(),
            Some(if galaxy == 0 { "Zero" } else { "Nine" })
        );
        assert_eq!(current_projection[0].3, 0.0);
    }

    assert!(matches!(
        execute_find(&current, &FindQuery::default()),
        Err(GraphError::ReferenceGalaxyMismatch {
            reference: 0,
            active: 9
        })
    ));
    let ShowResult::System(show) =
        execute_show(&current, &ShowQuery::System("Shared".into())).unwrap()
    else {
        panic!("expected system result");
    };
    assert_eq!(show.system.name.as_deref(), Some("Shared"));
    assert_eq!(show.system.planets[0].name.as_deref(), Some("Nine"));
    assert_eq!(show.distance_from_player, None);
}

#[test]
fn parser_derived_discovery_promotes_placeholder_and_preserves_imported_atlas() {
    fn save(records: serde_json::Value, version: u32, units: u64) -> nms_save::model::SaveRoot {
        let value = serde_json::json!({
            "Version": version,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "PlaceholderPromotion", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {
                    "VoxelX": 0, "VoxelY": 0, "VoxelZ": 0,
                    "SolarSystemIndex": 1, "PlanetIndex": 0
                }},
                "PreviousUniverseAddress": {"RealityIndex": 9, "GalacticAddress": {
                    "VoxelX": 8, "VoxelY": 7, "VoxelZ": 6,
                    "SolarSystemIndex": 5, "PlanetIndex": 0
                }},
                "Units": units, "Nanites": 82, "Specials": 93,
                "PersistentPlayerBases": []
            }},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {
                "ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": records}
            }}
        });
        nms_save::parse_save(&serde_json::to_vec(&value).unwrap()).unwrap()
    }

    let before_save = save(serde_json::json!([]), 4724, 7);
    let after_save = save(
        serde_json::json!([
            {"DD": {"UA": "0x000100000000", "DT": "SolarSystem", "VP": []},
             "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
            {"DD": {"UA": "0x200100000000", "DT": "Planet", "VP": ["0xAB", 1]},
             "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
        ]),
        4725,
        71,
    );
    let before = SaveSnapshot::from_save(&before_save);
    let after = SaveSnapshot::from_save(&after_save);
    let delta = compute_delta(&before, &after);
    assert_eq!(delta.new_systems.len(), 1);
    assert_eq!(delta.new_planets.len(), 1);

    let physical = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    let imported = GalacticAddress::new(20, 1, -2, 2, 0, 9);
    let placeholder_id = SystemId::from_address(&physical);
    let imported_id = SystemId::from_address(&imported);
    let expected_planet = Planet::new(2, Some(Biome::Toxic), None, false, None, Some(171));
    let expected_promoted = System::new(
        physical,
        None,
        Some("Explorer".into()),
        Some("2023-11-14T22:13:20Z".parse().unwrap()),
        vec![expected_planet.clone()],
    );
    assert_eq!(delta.new_systems, vec![expected_promoted.clone()]);
    assert_eq!(
        delta.new_planets,
        vec![(placeholder_id, expected_planet.clone())]
    );
    let mut model = GalaxyModel::new();
    model.insert_player_placeholder(physical);
    model.insert_system(System::new(
        imported,
        Some("Imported atlas".into()),
        Some("Import owner".into()),
        None,
        vec![],
    ));
    model.player_state = Some(PlayerState::new(
        physical,
        0,
        Some(GalacticAddress::new(8, 7, 6, 5, 0, 9)),
        Some(imported),
        7,
        8,
        9,
    ));
    model.set_active_galaxy(7);
    assert!(model.is_player_placeholder(&placeholder_id));

    let update = nms_core::SaveUpdate {
        delta,
        player_state: after_save.to_core_player_state(),
        save_version: after_save.version,
    };
    assert_eq!(update.save_version, 4725);
    let expected_player = PlayerState::new(
        GalacticAddress::new(0, 0, 0, 1, 0, 0),
        0,
        Some(GalacticAddress::new(8, 7, 6, 5, 0, 9)),
        None,
        71,
        82,
        93,
    );
    assert_eq!(update.player_state, expected_player);
    model.apply_save_update(&update);

    assert_eq!(model.system_count(), 2);
    assert_eq!(model.planet_count(), 1);
    assert!(!model.is_player_placeholder(&placeholder_id));
    assert_eq!(model.systems[&placeholder_id].address, physical);
    assert_eq!(
        model.systems[&placeholder_id].discoverer.as_deref(),
        Some("Explorer")
    );
    assert_eq!(
        model.systems[&placeholder_id]
            .timestamp
            .unwrap()
            .timestamp(),
        1_700_000_000
    );
    assert_eq!(model.systems[&placeholder_id].planets[0].index, 2);
    assert_eq!(
        model.systems[&placeholder_id].planets[0].biome,
        Some(Biome::Toxic)
    );
    assert!(!model.systems[&placeholder_id].planets[0].infested);
    assert_eq!(
        model.systems[&placeholder_id].planets[0].seed_hash,
        Some(0xAB)
    );
    assert_eq!(
        model.planets[&(placeholder_id, 2)],
        model.systems[&placeholder_id].planets[0]
    );
    assert_eq!(model.biome_index[&Biome::Toxic], vec![(placeholder_id, 2)]);
    assert_eq!(
        model.system(&imported_id).unwrap().name.as_deref(),
        Some("Imported atlas")
    );
    assert_eq!(model.player_state, Some(expected_player));
    assert_eq!(model.active_galaxy, 7);
    assert_eq!(
        model.systems,
        std::collections::HashMap::from([
            (placeholder_id, expected_promoted,),
            (
                imported_id,
                System::new(
                    imported,
                    Some("Imported atlas".into()),
                    Some("Import owner".into()),
                    None,
                    vec![],
                ),
            ),
        ])
    );
    assert_eq!(
        model.name_index,
        std::collections::HashMap::from([((9, "imported atlas".into()), imported_id)])
    );
    assert_eq!(
        model.biome_index,
        std::collections::HashMap::from([(Biome::Toxic, vec![(placeholder_id, 2)])])
    );
    assert_eq!(
        model.planets,
        std::collections::HashMap::from([((placeholder_id, 2), expected_planet)])
    );
    assert_eq!(
        model.address_to_id,
        std::collections::HashMap::from([
            (placeholder_id, placeholder_id),
            (imported_id, imported_id)
        ])
    );
    assert!(
        model
            .systems
            .keys()
            .all(|id| !model.is_player_placeholder(id))
    );
    assert!(model.bases.is_empty());
    assert_eq!(model.node_map.len(), 2);
    for (id, index) in &model.node_map {
        assert_eq!(model.graph.node_weight(*index), Some(id));
    }
    assert_eq!(model.graph.node_count(), 2);
    assert_eq!(model.graph.edge_count(), 0);
    assert_eq!(
        model
            .spatial
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([0, 9])
    );
    assert_eq!(
        model
            .spatial_for(0)
            .unwrap()
            .iter()
            .map(|point| (point.id, point.point))
            .collect::<Vec<_>>(),
        vec![(placeholder_id, [0.0, 0.0, 0.0])]
    );
    assert_eq!(
        model
            .spatial_for(9)
            .unwrap()
            .iter()
            .map(|point| (point.id, point.point))
            .collect::<Vec<_>>(),
        vec![(imported_id, [20.0, 1.0, -2.0])]
    );
}
