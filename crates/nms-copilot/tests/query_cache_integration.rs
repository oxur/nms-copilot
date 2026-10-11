use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::{GalaxyModel, GraphError, SystemId};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::show::{ShowQuery, ShowResult, execute_show};

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
fn schema2_semantic2_archive_rebuilds_corrected_indexes_and_matches_fresh_queries() {
    assert_eq!(PREDECESSOR_ARCHIVE.len(), 360);
    assert_eq!(
        &PREDECESSOR_ARCHIVE[..16],
        &[
            0x4e, 0x4d, 0x53, 0x43, 0x41, 0x43, 0x48, 0x45, 2, 0, 0, 0, 2, 0, 0, 0
        ]
    );

    let prior_data = nms_cache::serialize::deserialize(PREDECESSOR_ARCHIVE).unwrap();
    assert_eq!(prior_data.systems.len(), 2);
    assert_eq!(
        prior_data
            .systems
            .iter()
            .map(|system| system.planets.len())
            .sum::<usize>(),
        2
    );
    assert_eq!(prior_data.bases.len(), 0);
    assert_eq!(prior_data.save_version, 4720);
    assert_eq!(prior_data.active_galaxy, 9);
    let prior_player = prior_data.player_state.as_ref().unwrap();
    assert_eq!(prior_player.current_address.reality_index, 0);
    assert_eq!(prior_player.current_reality, 0);
    assert_eq!(
        (
            prior_player.units,
            prior_player.nanites,
            prior_player.quicksilver
        ),
        (7, 8, 9)
    );

    let prior = nms_cache::rebuild_model(&prior_data);
    let fresh = fresh_model();
    assert_eq!(prior.system_count(), 2);
    assert_eq!(prior.planet_count(), 2);
    assert_eq!(prior.active_galaxy, 9);
    assert_eq!(prior.player_position(), fresh.player_position());
    assert_eq!(prior.player_state.as_ref().unwrap().units, 7);
    assert_eq!(prior.player_state.as_ref().unwrap().nanites, 8);
    assert_eq!(prior.player_state.as_ref().unwrap().quicksilver, 9);

    let id0 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    let id9 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 9));
    for (model, expected_id, expected_planet) in [(&prior, id0, "Zero"), (&prior, id9, "Nine")] {
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
        let prior_projection = query_projection(&prior, galaxy);
        let fresh_projection = query_projection(&fresh, galaxy);
        assert_eq!(prior_projection, fresh_projection);
        assert_eq!(prior_projection.len(), 1);
        assert_eq!(prior_projection[0].0.reality_index(), galaxy);
        assert_eq!(
            prior_projection[0].1.name.as_deref(),
            Some(if galaxy == 0 { "Zero" } else { "Nine" })
        );
        assert_eq!(prior_projection[0].3, 0.0);
    }

    assert!(matches!(
        execute_find(&prior, &FindQuery::default()),
        Err(GraphError::ReferenceGalaxyMismatch {
            reference: 0,
            active: 9
        })
    ));
    let ShowResult::System(prior_show) =
        execute_show(&prior, &ShowQuery::System("Shared".into())).unwrap()
    else {
        panic!("expected system result");
    };
    assert_eq!(prior_show.system.name.as_deref(), Some("Shared"));
    assert_eq!(prior_show.system.planets[0].name.as_deref(), Some("Nine"));
    assert_eq!(prior_show.distance_from_player, None);

    let current_bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    assert_eq!(&current_bytes[..16], &PREDECESSOR_ARCHIVE[..16]);
    let current_data = nms_cache::serialize::deserialize(&current_bytes).unwrap();
    let current = nms_cache::rebuild_model(&current_data);
    assert_eq!(current.system_count(), 2);
    assert_eq!(current.planet_count(), 2);
    assert_eq!(current.active_galaxy, 9);
    assert_eq!(query_projection(&prior, 0), query_projection(&current, 0));
    assert_eq!(query_projection(&prior, 9), query_projection(&current, 9));
    let ShowResult::System(current_show) =
        execute_show(&current, &ShowQuery::System("Shared".into())).unwrap()
    else {
        panic!("expected system result");
    };
    assert_eq!(current_show.system, prior_show.system);
    assert_eq!(current_show.distance_from_player, None);
}
