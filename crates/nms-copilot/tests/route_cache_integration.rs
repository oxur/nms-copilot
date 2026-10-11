//! Cache compatibility and route reconstruction regressions.

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::PlayerState;
use nms_core::system::{Planet, System};
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_graph::{GalaxyModel, SystemId};
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};

const PREDECESSOR_ARCHIVE: &[u8] = include_bytes!("fixtures/predecessor-schema2-semantic2.rkyv");
const PREDECESSOR_SHA256: &str = "291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7";

fn route_query(start: GalacticAddress) -> RouteQuery {
    RouteQuery {
        targets: TargetSelection::Biome(BiomeFilter {
            biome: Some(Biome::Lush),
            ..Default::default()
        }),
        from: RouteFrom::Address(start),
        warp_range: None,
        within_ly: None,
        max_targets: None,
        algorithm: RoutingAlgorithm::NearestNeighbor,
        return_to_start: false,
    }
}

fn rich_route_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    let player = GalacticAddress::new(0, 0, 0, 1, 0, 0);
    model.player_state = Some(PlayerState::new(player, 0, None, None, 7, 8, 9));
    model.insert_system(System::new(
        player,
        Some("Twin".into()),
        None,
        None,
        vec![Planet::new(0, Some(Biome::Barren), None, false, None, None)],
    ));
    for x in [1, 2] {
        let address = GalacticAddress::new(x, 0, 0, 1, 0, 9);
        model.insert_system(System::new(
            address,
            Some(if x == 1 { "Twin" } else { "Twin Two" }.into()),
            Some("Explorer".into()),
            None,
            vec![Planet::new(
                1,
                Some(Biome::Lush),
                None,
                false,
                Some(format!("Lush {x}")),
                None,
            )],
        ));
    }
    model.set_active_galaxy(0);
    model
}

fn route_projection(model: &GalaxyModel, start: GalacticAddress) -> Vec<(SystemId, f64)> {
    execute_route(model, &route_query(start))
        .unwrap()
        .route
        .hops
        .into_iter()
        .map(|hop| (hop.system_id, hop.leg_distance_ly))
        .collect()
}

#[test]
fn predecessor_fixture_stays_byte_identical_and_checked_rebuild_matches_typed_expectations() {
    assert_eq!(PREDECESSOR_ARCHIVE.len(), 360);
    assert_eq!(
        PREDECESSOR_SHA256,
        "291b283d06db222bf355c259269328d23fef34d0d020c89012fb922bdf4d31c7"
    );
    assert_eq!(&PREDECESSOR_ARCHIVE[..16], b"NMSCACHE\x02\0\0\0\x02\0\0\0");
    let data = nms_cache::serialize::deserialize(PREDECESSOR_ARCHIVE).unwrap();
    assert_eq!(data.systems.len(), 2);
    assert_eq!(data.active_galaxy, 9);
    let rebuilt = nms_cache::rebuild_model(&data);
    assert_eq!(rebuilt.system_count(), 2);
    assert_eq!(rebuilt.planet_count(), 2);
    let id0 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    let id9 = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 9));
    assert_eq!(
        rebuilt.system_by_name_in_galaxy("shared", 0).unwrap().0,
        &id0
    );
    assert_eq!(
        rebuilt.system_by_name_in_galaxy("shared", 9).unwrap().0,
        &id9
    );
    assert!(
        rebuilt
            .nearest_systems_from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 9), 5)
            .iter()
            .all(|(id, _)| id.reality_index() == 9)
    );
}

#[test]
fn current_cache_round_trip_preserves_rich_galaxy_route_and_qualified_names() {
    let fresh = rich_route_model();
    let start = GalacticAddress::new(0, 0, 0, 1, 0, 9);
    let expected = vec![
        (
            SystemId::from_address(&GalacticAddress::new(1, 0, 0, 1, 0, 9)),
            0.0,
        ),
        (
            SystemId::from_address(&GalacticAddress::new(2, 0, 0, 1, 0, 9)),
            400.0,
        ),
    ];
    let fresh_projection = route_projection(&fresh, start);
    assert_eq!(fresh_projection, expected);

    let bytes =
        nms_cache::serialize::serialize(&nms_cache::extract_cache_data(&fresh, 4720)).unwrap();
    assert_eq!(&bytes[..16], b"NMSCACHE\x02\0\0\0\x02\0\0\0");
    let checked = nms_cache::serialize::deserialize(&bytes).unwrap();
    let rebuilt = nms_cache::rebuild_model(&checked);
    assert_eq!(rebuilt.active_galaxy, 0);
    assert_eq!(route_projection(&rebuilt, start), expected);
    let twin_zero = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
    assert_eq!(
        rebuilt.system_by_name_in_galaxy("twin", 0).unwrap().0,
        &twin_zero
    );
    assert_eq!(
        rebuilt.system_by_name_in_galaxy("twin", 9).unwrap().0,
        &expected[0].0
    );
}
