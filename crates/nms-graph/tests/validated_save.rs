use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System, SystemId};
use nms_graph::{GalaxyModel, extract::extract_systems_from_validated};
use nms_save::parse_validated_save;
use serde_json::{Value, json};

const MINIMAL: &[u8] = include_bytes!("../../../data/test/save56_minimal.json");

fn source_with_galaxy_conflicts() -> Value {
    let mut source: Value = serde_json::from_slice(MINIMAL).unwrap();
    source["ActiveContext"] = json!("Main");
    source["BaseContext"]["PlayerStateData"]["UniverseAddress"] = json!({
        "RealityIndex": 0,
        "GalacticAddress": {
            "VoxelX": 1130, "VoxelY": 4, "VoxelZ": -1244,
            "SolarSystemIndex": 299, "PlanetIndex": 2
        }
    });
    source["BaseContext"]["PlayerStateData"]
        .as_object_mut()
        .unwrap()
        .remove("PreviousUniverseAddress");
    source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["GalacticAddress"] =
        json!("0x212B0904B2446A");
    source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["Name"] =
        json!("Checked Base");

    let records = source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    records[0]["DD"]["UA"] = json!("0x012B0904B2446A");
    records[0]["OWS"]["USN"] = json!("Checked");
    records[0]["DD"]["VP"] = json!([]);
    records[1]["DD"]["UA"] = json!("0x212B0904B2446A");
    records[1]["DD"]["VP"] = json!(["0xAB", 1]);
    let mut zero_system = records[0].clone();
    zero_system["DD"]["UA"] = json!("0x012B0004B2446A");
    zero_system["OWS"]["USN"] = json!("Zero");
    let mut zero_planet = records[1].clone();
    zero_planet["DD"]["UA"] = json!("0x212B0004B2446A");
    zero_planet["DD"]["VP"] = json!(["0xAC", 0]);
    let mut second_system = records[0].clone();
    second_system["DD"]["UA"] = json!("0x012C0904B2446B");
    second_system["OWS"]["USN"] = json!("Checked");
    let mut second_planet = records[1].clone();
    second_planet["DD"]["UA"] = json!("0x112C0904B2446B");
    second_planet["DD"]["VP"] = json!(["0xAD", 0]);
    records.extend([zero_system, zero_planet, second_system, second_planet]);
    source
}

fn assert_full_literal_model(model: &GalaxyModel) {
    let a9 = GalacticAddress::new(1130, 4, -1244, 299, 0, 9);
    let a0 = GalacticAddress::new(1130, 4, -1244, 299, 0, 0);
    let b9 = GalacticAddress::new(1131, 4, -1244, 300, 0, 9);
    let a9_id = SystemId::from_address(&a9);
    let a0_id = SystemId::from_address(&a0);
    let b9_id = SystemId::from_address(&b9);
    let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0);
    let expected_systems = std::collections::HashMap::from([
        (
            a9_id,
            System::new(
                a9,
                None,
                Some("Checked".into()),
                timestamp,
                vec![Planet::new(
                    2,
                    Some(Biome::Toxic),
                    None,
                    false,
                    None,
                    Some(0xAB),
                )],
            ),
        ),
        (
            a0_id,
            System::new(
                a0,
                None,
                Some("Zero".into()),
                timestamp,
                vec![Planet::new(
                    2,
                    Some(Biome::Lush),
                    None,
                    false,
                    None,
                    Some(0xAC),
                )],
            ),
        ),
        (
            b9_id,
            System::new(
                b9,
                None,
                Some("Checked".into()),
                timestamp,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    None,
                    Some(0xAD),
                )],
            ),
        ),
    ]);
    assert_eq!(model.systems, expected_systems);
    assert_eq!(
        model.planets,
        std::collections::HashMap::from([
            (
                (a9_id, 2),
                Planet::new(2, Some(Biome::Toxic), None, false, None, Some(0xAB))
            ),
            (
                (a0_id, 2),
                Planet::new(2, Some(Biome::Lush), None, false, None, Some(0xAC))
            ),
            (
                (b9_id, 1),
                Planet::new(1, Some(Biome::Lush), None, false, None, Some(0xAD))
            ),
        ])
    );
    assert_eq!(
        model.bases,
        std::collections::HashMap::from([(
            "checked base".to_owned(),
            PlayerBase::new(
                "Checked Base".into(),
                BaseType::HomePlanetBase,
                GalacticAddress::new(1130, 4, -1244, 299, 2, 9),
                [100.0, 200.0, 300.0],
                Some("76561198025707979".into())
            ),
        )])
    );
    assert_eq!(
        model.player_state,
        Some(PlayerState::new(
            GalacticAddress::new(1130, 4, -1244, 299, 2, 0),
            0,
            None,
            None,
            5_000_000,
            10_000,
            500,
        ))
    );
    assert_eq!(model.active_galaxy, 0);
    assert!(model.name_index.is_empty());
    assert_eq!(
        model.address_to_id,
        expected_systems
            .keys()
            .map(|id| (*id, *id))
            .collect::<std::collections::HashMap<_, _>>()
    );
    let mut biome_rows = model
        .biome_index
        .iter()
        .map(|(biome, rows)| {
            let mut rows = rows
                .iter()
                .map(|(id, planet)| (id.to_string(), *planet))
                .collect::<Vec<_>>();
            rows.sort();
            (*biome, rows)
        })
        .collect::<Vec<_>>();
    biome_rows.sort_by_key(|(biome, _)| format!("{biome:?}"));
    assert_eq!(
        biome_rows,
        vec![
            (
                Biome::Lush,
                vec![(a0_id.to_string(), 2), (b9_id.to_string(), 1)]
            ),
            (Biome::Toxic, vec![(a9_id.to_string(), 2)]),
        ]
    );
    assert_eq!(
        model
            .node_map
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([a9_id, a0_id, b9_id])
    );
    let mut graph_node_weights = model
        .graph
        .node_weights()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    graph_node_weights.sort();
    let mut expected_graph_node_weights =
        vec![a0_id.to_string(), a9_id.to_string(), b9_id.to_string()];
    expected_graph_node_weights.sort();
    assert_eq!(graph_node_weights, expected_graph_node_weights);
    assert_eq!(model.graph.node_count(), 3);
    assert_eq!(model.graph.edge_count(), 1);
    assert!(
        model
            .systems
            .keys()
            .all(|id| !model.is_player_placeholder(id))
    );
    for (id, node) in &model.node_map {
        assert_eq!(model.graph.node_weight(*node), Some(id));
    }
    let mut spatial_galaxies = model.spatial.keys().copied().collect::<Vec<_>>();
    spatial_galaxies.sort();
    assert_eq!(spatial_galaxies, vec![0, 9]);
    let mut points = model
        .spatial
        .iter()
        .flat_map(|(galaxy, tree)| {
            tree.iter()
                .map(move |point| (*galaxy, point.id.to_string(), point.point))
        })
        .collect::<Vec<_>>();
    points.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
    assert_eq!(
        points,
        vec![
            (0, a0_id.to_string(), [1130.0, 4.0, -1244.0]),
            (9, a9_id.to_string(), [1130.0, 4.0, -1244.0]),
            (9, b9_id.to_string(), [1131.0, 4.0, -1244.0]),
        ]
    );
    let mut edges = model
        .graph
        .edge_indices()
        .map(|edge| {
            let (left, right) = model.graph.edge_endpoints(edge).unwrap();
            let mut ids = [
                model.graph.node_weight(left).unwrap().to_string(),
                model.graph.node_weight(right).unwrap().to_string(),
            ];
            ids.sort();
            (ids, *model.graph.edge_weight(edge).unwrap())
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| left.0.cmp(&right.0));
    let mut expected_edge_ids = [a9_id.to_string(), b9_id.to_string()];
    expected_edge_ids.sort();
    assert_eq!(edges, vec![(expected_edge_ids, 400.0)]);
}

#[test]
fn test_validated_extraction_and_model_preserve_record_galaxies_and_whole_indexes() {
    let source = source_with_galaxy_conflicts();
    let save = parse_validated_save(source.to_string().as_bytes()).unwrap();
    let extracted = extract_systems_from_validated(&save);
    let model = GalaxyModel::from_validated_save(&save);
    assert_full_literal_model(&model);

    let a9 = GalacticAddress::new(1130, 4, -1244, 299, 0, 9);
    let a0 = GalacticAddress::new(1130, 4, -1244, 299, 0, 0);
    let b9 = GalacticAddress::new(1131, 4, -1244, 300, 0, 9);
    let a9_id = SystemId::from_address(&a9);
    let a0_id = SystemId::from_address(&a0);
    let b9_id = SystemId::from_address(&b9);
    let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0);
    let expected_systems = std::collections::HashMap::from([
        (
            a9_id,
            System::new(
                a9,
                None,
                Some("Checked".into()),
                timestamp,
                vec![Planet::new(
                    2,
                    Some(Biome::Toxic),
                    None,
                    false,
                    None,
                    Some(0xAB),
                )],
            ),
        ),
        (
            a0_id,
            System::new(
                a0,
                None,
                Some("Zero".into()),
                timestamp,
                vec![Planet::new(
                    2,
                    Some(Biome::Lush),
                    None,
                    false,
                    None,
                    Some(0xAC),
                )],
            ),
        ),
        (
            b9_id,
            System::new(
                b9,
                None,
                Some("Checked".into()),
                timestamp,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    None,
                    Some(0xAD),
                )],
            ),
        ),
    ]);
    assert_eq!(extracted.len(), 3);
    assert_eq!(model.systems.len(), 3);
    assert_eq!(model.planets.len(), 3);
    assert_eq!(extracted, expected_systems);
    assert_eq!(model.systems, expected_systems);
    assert_eq!(model.active_galaxy, 0);
    assert_eq!(
        model.player_state.as_ref().unwrap().current_address,
        GalacticAddress::new(1130, 4, -1244, 299, 2, 0)
    );
    assert_eq!(
        model.player_state,
        Some(PlayerState::new(
            GalacticAddress::new(1130, 4, -1244, 299, 2, 0),
            0,
            None,
            None,
            5_000_000,
            10_000,
            500,
        ))
    );

    let mut expected_systems = std::collections::HashMap::new();
    expected_systems.insert(a9_id, a9);
    expected_systems.insert(a0_id, a0);
    expected_systems.insert(b9_id, b9);
    let actual_systems = model
        .systems
        .iter()
        .map(|(id, system)| (*id, system.address))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(actual_systems, expected_systems);
    assert_eq!(
        model.address_to_id,
        expected_systems
            .keys()
            .map(|id| (*id, *id))
            .collect::<std::collections::HashMap<_, _>>()
    );
    assert!(model.name_index.is_empty());
    assert_eq!(
        model.planets,
        std::collections::HashMap::from([
            (
                (a9_id, 2),
                Planet::new(2, Some(Biome::Toxic), None, false, None, Some(0xAB))
            ),
            (
                (a0_id, 2),
                Planet::new(2, Some(Biome::Lush), None, false, None, Some(0xAC))
            ),
            (
                (b9_id, 1),
                Planet::new(1, Some(Biome::Lush), None, false, None, Some(0xAD))
            ),
        ])
    );
    let mut toxic = model.biome_index[&Biome::Toxic]
        .iter()
        .map(|(id, planet)| (id.to_string(), *planet))
        .collect::<Vec<_>>();
    toxic.sort();
    assert_eq!(toxic, vec![(a9_id.to_string(), 2)]);
    let mut lush = model.biome_index[&Biome::Lush]
        .iter()
        .map(|(id, planet)| (id.to_string(), *planet))
        .collect::<Vec<_>>();
    lush.sort();
    assert_eq!(lush, vec![(a0_id.to_string(), 2), (b9_id.to_string(), 1)]);
    assert_eq!(model.biome_index.len(), 2);
    assert_eq!(
        model
            .node_map
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        std::collections::HashSet::from([a9_id, a0_id, b9_id])
    );
    for (id, node) in &model.node_map {
        assert_eq!(model.graph.node_weight(*node), Some(id));
    }
    let mut spatial = model
        .spatial
        .iter()
        .flat_map(|(galaxy, tree)| {
            tree.iter()
                .map(move |point| (*galaxy, point.id.to_string(), point.point))
        })
        .collect::<Vec<_>>();
    spatial.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
    assert_eq!(
        spatial,
        vec![
            (0, a0_id.to_string(), [1130.0, 4.0, -1244.0]),
            (9, a9_id.to_string(), [1130.0, 4.0, -1244.0]),
            (9, b9_id.to_string(), [1131.0, 4.0, -1244.0]),
        ]
    );
    let base = model.bases.get("checked base").unwrap();
    assert_eq!(
        model.bases,
        std::collections::HashMap::from([(
            "checked base".to_owned(),
            PlayerBase::new(
                "Checked Base".into(),
                BaseType::HomePlanetBase,
                GalacticAddress::new(1130, 4, -1244, 299, 2, 9),
                [100.0, 200.0, 300.0],
                Some("76561198025707979".into()),
            ),
        )])
    );
    assert_eq!(
        base.address,
        GalacticAddress::new(1130, 4, -1244, 299, 2, 9)
    );
    assert_eq!(model.graph.node_count(), 3);
    assert_eq!(model.graph.edge_count(), 1);
    let edge = model.graph.edge_indices().next().unwrap();
    let (left, right) = model.graph.edge_endpoints(edge).unwrap();
    let endpoints = std::collections::HashSet::from([
        *model.graph.node_weight(left).unwrap(),
        *model.graph.node_weight(right).unwrap(),
    ]);
    assert_eq!(endpoints, std::collections::HashSet::from([a9_id, b9_id]));
    assert_eq!(*model.graph.edge_weight(edge).unwrap(), 400.0);
}

#[test]
fn test_validated_extraction_is_independent_of_original_discovery_row_order() {
    let source = source_with_galaxy_conflicts();
    let mut reversed = source.clone();
    reversed["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let original = GalaxyModel::from_validated_save(
        &parse_validated_save(source.to_string().as_bytes()).unwrap(),
    );
    let reordered = GalaxyModel::from_validated_save(
        &parse_validated_save(reversed.to_string().as_bytes()).unwrap(),
    );
    assert_full_literal_model(&original);
    assert_full_literal_model(&reordered);
    assert_eq!(reordered.systems, original.systems);
    assert_eq!(reordered.planets, original.planets);
    assert_eq!(reordered.bases, original.bases);
    assert_eq!(reordered.player_state, original.player_state);
    assert_eq!(reordered.active_galaxy, original.active_galaxy);
    assert_eq!(reordered.name_index, original.name_index);
    assert_eq!(reordered.address_to_id, original.address_to_id);
    assert_eq!(
        reordered
            .node_map
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        original
            .node_map
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    let original_biomes = original
        .biome_index
        .iter()
        .map(|(biome, values)| {
            let mut values = values
                .iter()
                .map(|(id, planet)| (id.to_string(), *planet))
                .collect::<Vec<_>>();
            values.sort();
            (*biome, values)
        })
        .collect::<std::collections::HashMap<_, _>>();
    let reordered_biomes = reordered
        .biome_index
        .iter()
        .map(|(biome, values)| {
            let mut values = values
                .iter()
                .map(|(id, planet)| (id.to_string(), *planet))
                .collect::<Vec<_>>();
            values.sort();
            (*biome, values)
        })
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(reordered_biomes, original_biomes);
}

#[test]
fn test_validated_graph_rejects_unknown_discovery_rows_even_when_the_builder_would_ignore_them() {
    let mut source = source_with_galaxy_conflicts();
    let records = source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    let mut unknown = records[0].clone();
    unknown["DD"]["DT"] = json!("UnmodeledDiscoveryKind");
    unknown["DD"]["UA"] = json!("0x100000000000000");
    records.push(unknown);
    assert!(matches!(
        parse_validated_save(source.to_string().as_bytes()),
        Err(nms_save::SaveError::Address(
            nms_save::address::AddressDecodeError::PackedWidth { path }
        )) if path == "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/6/DD/UA"
    ));
}
