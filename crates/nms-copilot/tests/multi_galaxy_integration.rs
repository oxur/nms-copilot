use nms_cache::serialize::{deserialize, extract_cache_data, rebuild_model, serialize};
use nms_copilot::session::{PositionContext, SessionState};
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::{Galaxy, GalaxyType};
use nms_core::player::BaseType;
use nms_graph::edges::EdgeStrategy;
use nms_graph::route::RouteError;
use nms_graph::{GalaxyModel, GraphError, SystemId};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_save::{ValidatedSave, parse_validated_save};
use reedline::Prompt;
use serde_json::{Value, json};

const SAVE56: &str = include_str!("../../../data/test/save56_minimal.json");

type PlanetRow = (
    u8,
    Option<Biome>,
    Option<nms_core::biome::BiomeSubType>,
    Option<String>,
    bool,
    Option<u64>,
);
type PlanetMapRow = (((u8, u64), u8), PlanetRow);
type BaseRow = (
    String,
    String,
    (u8, u64),
    [u32; 3],
    BaseType,
    Option<String>,
);
type BiomeRow = (Biome, Vec<((u8, u64), u8)>);
type AddressRow = ((u8, u64), (u8, u64));
type SpatialPointRow = (u64, u64, u64, (u8, u64));
type SpatialRow = (u8, Vec<SpatialPointRow>);
type PlayerRow = (
    GalacticAddress,
    u8,
    Option<GalacticAddress>,
    Option<GalacticAddress>,
    u64,
    u64,
    u64,
);

#[derive(Clone, Debug, PartialEq)]
struct SystemRow {
    id: (u8, u64),
    address: (i16, i8, i16, u16, u8, u8),
    name: Option<String>,
    discoverer: Option<String>,
    timestamp: Option<String>,
    planets: Vec<PlanetRow>,
}

#[derive(Clone, Debug, PartialEq)]
struct EdgeRow {
    left: (u8, u64),
    right: (u8, u64),
    weight: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ModelView {
    systems: Vec<SystemRow>,
    planets: Vec<PlanetMapRow>,
    bases: Vec<BaseRow>,
    biome_members: Vec<BiomeRow>,
    name_members: Vec<(u8, String, (u8, u64))>,
    address_members: Vec<AddressRow>,
    node_payloads: Vec<(u8, u64)>,
    spatial_points: Vec<SpatialRow>,
    edges: Vec<EdgeRow>,
    placeholders: Vec<(u8, u64)>,
    player: Option<PlayerRow>,
    active: u8,
}

fn sys_id(galaxy: u8, x: i16, y: i8, z: i16, ssi: u16) -> SystemId {
    SystemId::from_address(&GalacticAddress::new(x, y, z, ssi, 0, galaxy))
}

fn id_key(id: SystemId) -> (u8, u64) {
    (id.reality_index(), id.system_address())
}

fn addr(galaxy: u8, x: i16, y: i8, z: i16, ssi: u16, planet: u8) -> GalacticAddress {
    GalacticAddress::new(x, y, z, ssi, planet, galaxy)
}

fn discovery_records(reverse: bool) -> (Value, Vec<Value>) {
    let mut root: Value = serde_json::from_str(SAVE56).unwrap();
    let records = root["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array()
        .unwrap();
    let system_template = records
        .iter()
        .find(|row| row["DD"]["DT"] == "SolarSystem")
        .unwrap()
        .clone();
    let planet_template = records
        .iter()
        .find(|row| row["DD"]["DT"] == "Planet")
        .unwrap()
        .clone();

    let mut generated = Vec::with_capacity(1024);
    for g in 0u16..=255 {
        for (template, hex, name) in [
            (
                &system_template,
                format!("0x0001{g:02X}00000000"),
                "Shared".to_owned(),
            ),
            (
                &planet_template,
                format!("0x1001{g:02X}00000000"),
                format!("Leaf-{g}-A"),
            ),
            (
                &system_template,
                format!("0x0002{g:02X}00000001"),
                "Target".to_owned(),
            ),
            (
                &planet_template,
                format!("0x1002{g:02X}00000001"),
                format!("Leaf-{g}-B"),
            ),
        ] {
            let mut row = template.clone();
            row["DD"]["UA"] = Value::String(hex);
            row["DM"]["CN"] = Value::String(name);
            generated.push(row);
        }
    }
    assert_eq!(generated.len(), 1024);

    for (index, row) in generated.iter().enumerate() {
        let kind = row["DD"]["DT"].as_str().unwrap();
        let name = row["DM"]["CN"].as_str().unwrap();
        assert!(row["DD"].get("DM").is_none());
        let galaxy = (index / 4) as u8;
        match index % 4 {
            0 => {
                assert_eq!(kind, "SolarSystem");
                assert_eq!(name, "Shared");
                assert_eq!(row["DD"]["UA"], format!("0x0001{galaxy:02X}00000000"));
            }
            1 => {
                assert_eq!(kind, "Planet");
                assert_eq!(name, format!("Leaf-{galaxy}-A"));
                assert_eq!(row["DD"]["UA"], format!("0x1001{galaxy:02X}00000000"));
            }
            2 => {
                assert_eq!(kind, "SolarSystem");
                assert_eq!(name, "Target");
                assert_eq!(row["DD"]["UA"], format!("0x0002{galaxy:02X}00000001"));
            }
            _ => {
                assert_eq!(kind, "Planet");
                assert_eq!(name, format!("Leaf-{galaxy}-B"));
                assert_eq!(row["DD"]["UA"], format!("0x1002{galaxy:02X}00000001"));
            }
        }
    }
    assert_eq!(generated[0]["DD"]["UA"], "0x00010000000000");
    assert_eq!(generated[36]["DD"]["UA"], "0x00010900000000");
    assert_eq!(generated[1020]["DD"]["UA"], "0x0001FF00000000");
    assert_eq!(generated[1023]["DD"]["UA"], "0x1002FF00000001");

    if reverse {
        generated.reverse();
    }
    root["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] =
        Value::Array(generated.clone());

    assert_eq!(root["Version"], 4720);
    let player = &mut root["BaseContext"]["PlayerStateData"];
    player["UniverseAddress"]["RealityIndex"] = json!(0);
    player["UniverseAddress"]["GalacticAddress"] = json!({
        "VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0
    });
    assert_eq!(
        player["PreviousUniverseAddress"]["GalacticAddress"],
        json!({"VoxelX": 80, "VoxelY": 30, "VoxelZ": -150, "SolarSystemIndex": 38, "PlanetIndex": 0})
    );
    player["Units"] = json!(5_000_000);
    player["Nanites"] = json!(10_000);
    player["Specials"] = json!(500);
    let bases = player["PersistentPlayerBases"].as_array_mut().unwrap();
    let mut base = bases[0].clone();
    base["Name"] = json!("Camp-0");
    base["GalacticAddress"] = json!("0x00010000000000");
    let mut second = base.clone();
    second["Name"] = json!("Camp-255");
    second["GalacticAddress"] = json!("0x0001FF00000000");
    *bases = vec![base, second];
    assert!(bases.iter().all(|base| base["Name"] != "Homestead Alpha"));
    (root, generated)
}

fn checked(reverse: bool) -> (ValidatedSave, GalaxyModel, Vec<Value>) {
    let (root, records) = discovery_records(reverse);
    let bytes = serde_json::to_vec(&root).unwrap();
    let save = parse_validated_save(&bytes).unwrap();
    assert_eq!(save.discoveries().len(), 1024);
    let model = GalaxyModel::from_validated_save(&save);
    (save, model, records)
}

fn view(model: &GalaxyModel) -> ModelView {
    let mut systems = model
        .systems
        .iter()
        .map(|(&id, system)| SystemRow {
            id: id_key(id),
            address: (
                system.address.voxel_x(),
                system.address.voxel_y(),
                system.address.voxel_z(),
                system.address.solar_system_index(),
                system.address.planet_index(),
                system.address.reality_index,
            ),
            name: system.name.clone(),
            discoverer: system.discoverer.clone(),
            timestamp: system.timestamp.as_ref().map(|time| time.to_rfc3339()),
            planets: system
                .planets
                .iter()
                .map(|planet| {
                    (
                        planet.index,
                        planet.biome,
                        planet.biome_subtype,
                        planet.name.clone(),
                        planet.infested,
                        planet.seed_hash,
                    )
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    systems.sort_by_key(|row| row.id);

    let mut bases = model
        .bases
        .iter()
        .map(|(key, base)| {
            (
                key.clone(),
                base.name.clone(),
                (base.address.reality_index, base.address.packed()),
                base.position.map(f32::to_bits),
                base.base_type,
                base.owner_uid.clone(),
            )
        })
        .collect::<Vec<_>>();
    bases.sort_by_key(|row| (row.0.clone(), row.2));

    let mut planets = model
        .planets
        .iter()
        .map(|(&(id, index), planet)| {
            (
                (id_key(id), index),
                (
                    planet.index,
                    planet.biome,
                    planet.biome_subtype,
                    planet.name.clone(),
                    planet.infested,
                    planet.seed_hash,
                ),
            )
        })
        .collect::<Vec<_>>();
    planets.sort_by_key(|row| row.0);

    let mut biome_members = model
        .biome_index
        .iter()
        .map(|(&biome, members)| {
            let mut members = members
                .iter()
                .map(|&(id, planet)| (id_key(id), planet))
                .collect::<Vec<_>>();
            members.sort();
            (biome, members)
        })
        .collect::<Vec<_>>();
    biome_members.sort_by_key(|row| format!("{:?}", row.0));

    let mut name_members = model
        .name_index
        .iter()
        .map(|((galaxy, name), &id)| (*galaxy, name.clone(), id_key(id)))
        .collect::<Vec<_>>();
    name_members.sort();

    let mut address_members = model
        .address_to_id
        .iter()
        .map(|(&address, &id)| (id_key(address), id_key(id)))
        .collect::<Vec<_>>();
    address_members.sort();

    let mut node_payloads = Vec::new();
    for (&id, &index) in &model.node_map {
        assert_eq!(model.graph.node_weight(index), Some(&id));
        node_payloads.push(id_key(id));
    }
    node_payloads.sort();
    let mut graph_nodes = model
        .graph
        .node_weights()
        .copied()
        .map(id_key)
        .collect::<Vec<_>>();
    graph_nodes.sort();
    assert_eq!(node_payloads, graph_nodes);

    let mut spatial_points = model
        .spatial
        .iter()
        .map(|(&galaxy, tree)| {
            let mut points = tree
                .iter()
                .map(|point| {
                    (
                        point.point[0].to_bits(),
                        point.point[1].to_bits(),
                        point.point[2].to_bits(),
                        id_key(point.id),
                    )
                })
                .collect::<Vec<_>>();
            points.sort();
            (galaxy, points)
        })
        .collect::<Vec<_>>();
    spatial_points.sort_by_key(|row| row.0);

    let mut edges = model
        .graph
        .edge_indices()
        .map(|edge| {
            let (left, right) = model.graph.edge_endpoints(edge).unwrap();
            let mut left = id_key(*model.graph.node_weight(left).unwrap());
            let mut right = id_key(*model.graph.node_weight(right).unwrap());
            if right < left {
                std::mem::swap(&mut left, &mut right);
            }
            EdgeRow {
                left,
                right,
                weight: model.graph.edge_weight(edge).unwrap().to_bits(),
            }
        })
        .collect::<Vec<_>>();
    edges.sort_by_key(|edge| (edge.left, edge.right, edge.weight));

    let mut placeholders = model
        .systems
        .keys()
        .copied()
        .filter(|id| model.is_player_placeholder(id))
        .map(id_key)
        .collect::<Vec<_>>();
    placeholders.sort();
    let player = model.player_state.as_ref().map(|player| {
        (
            player.current_address,
            player.current_reality,
            player.previous_address,
            player.freighter_address,
            player.units,
            player.nanites,
            player.quicksilver,
        )
    });
    ModelView {
        systems,
        planets,
        bases,
        biome_members,
        name_members,
        address_members,
        node_payloads,
        spatial_points,
        edges,
        placeholders,
        player,
        active: model.active_galaxy,
    }
}

fn assert_literal_model_for_active(model: &GalaxyModel, active: u8) {
    assert_eq!(model.system_count(), 512);
    assert_eq!(model.planet_count(), 512);
    assert_eq!(model.bases.len(), 2);
    assert_eq!(model.spatial.len(), 256);
    assert_eq!(model.node_map.len(), 512);
    assert_eq!(model.address_to_id.len(), 512);
    assert_eq!(model.name_index.len(), 512);
    assert!(
        model
            .systems
            .keys()
            .all(|id| !model.is_player_placeholder(id))
    );
    let discovered = model.discovered_galaxies();
    assert_eq!(
        discovered,
        (0u16..=255).map(|g| g as u8).collect::<Vec<_>>()
    );
    assert_eq!(model.graph.node_count(), 512);
    assert_eq!(model.graph.edge_count(), 256);
    assert_eq!(model.planets.len(), 512);
    assert_eq!(model.biome_index.len(), 1);
    assert_eq!(
        model.biome_index.keys().copied().collect::<Vec<_>>(),
        vec![Biome::Lush]
    );
    for g in 0u16..=255 {
        let g = g as u8;
        let a = addr(g, 0, 0, 0, 1, 0);
        let b = addr(g, 1, 0, 0, 2, 0);
        let a_id = sys_id(g, 0, 0, 0, 1);
        let b_id = sys_id(g, 1, 0, 0, 2);
        let a_key = (g, 0x000100000000u64);
        let b_key = (g, 0x000200000001u64);
        assert_eq!(id_key(a_id), a_key);
        assert_eq!(id_key(b_id), b_key);
        let a_system = model.system(&a_id).unwrap();
        let b_system = model.system(&b_id).unwrap();
        assert_eq!(a_system.address, a);
        assert_eq!(b_system.address, b);
        assert_eq!(a_system.name.as_deref(), Some("Shared"));
        assert_eq!(b_system.name.as_deref(), Some("Target"));
        assert_eq!(a_system.planets.len(), 1);
        assert_eq!(b_system.planets.len(), 1);
        assert_eq!(a_system.planets[0].index, 1);
        assert_eq!(b_system.planets[0].index, 1);
        assert_eq!(
            a_system.planets[0].name.as_deref(),
            Some(format!("Leaf-{g}-A").as_str())
        );
        assert_eq!(
            b_system.planets[0].name.as_deref(),
            Some(format!("Leaf-{g}-B").as_str())
        );
        assert_eq!(a_system.planets[0].biome, Some(Biome::Lush));
        assert_eq!(b_system.planets[0].biome, Some(Biome::Lush));
        assert_eq!(model.planets.get(&(a_id, 1)), Some(&a_system.planets[0]));
        assert_eq!(model.planets.get(&(b_id, 1)), Some(&b_system.planets[0]));
        assert_eq!(
            model.system_by_name_in_galaxy("SHARED", g).unwrap().0,
            &a_id
        );
        assert_eq!(
            model.system_by_name_in_galaxy("target", g).unwrap().0,
            &b_id
        );
        assert_eq!(model.address_to_id.get(&a_id), Some(&a_id));
        assert_eq!(model.address_to_id.get(&b_id), Some(&b_id));
        assert_eq!(
            model
                .node_map
                .get(&a_id)
                .and_then(|&n| model.graph.node_weight(n)),
            Some(&a_id)
        );
        assert_eq!(
            model
                .node_map
                .get(&b_id)
                .and_then(|&n| model.graph.node_weight(n)),
            Some(&b_id)
        );
        let mut points = model
            .spatial
            .get(&g)
            .unwrap()
            .iter()
            .map(|p| (id_key(p.id), p.point.map(f64::to_bits)))
            .collect::<Vec<_>>();
        points.sort();
        assert_eq!(
            points,
            vec![
                (
                    a_key,
                    [0.0f64.to_bits(), 0.0f64.to_bits(), 0.0f64.to_bits()]
                ),
                (
                    b_key,
                    [1.0f64.to_bits(), 0.0f64.to_bits(), 0.0f64.to_bits()]
                ),
            ]
        );
        let lush = model.biome_index.get(&Biome::Lush).unwrap();
        assert_eq!(
            lush.iter()
                .filter(|&&(id, index)| (id_key(id), index) == (a_key, 1))
                .count(),
            1
        );
        assert_eq!(
            lush.iter()
                .filter(|&&(id, index)| (id_key(id), index) == (b_key, 1))
                .count(),
            1
        );
    }
    let mut expected_biome = (0u16..=255)
        .flat_map(|g| {
            let g = g as u8;
            [(sys_id(g, 0, 0, 0, 1), 1), (sys_id(g, 1, 0, 0, 2), 1)]
        })
        .collect::<Vec<_>>();
    expected_biome.sort_by_key(|&(id, index)| (id_key(id), index));
    let mut actual_biome = model.biome_index.get(&Biome::Lush).unwrap().clone();
    actual_biome.sort_by_key(|&(id, index)| (id_key(id), index));
    assert_eq!(actual_biome, expected_biome);
    assert!(model.base("Camp-0").is_some());
    assert!(model.base("Camp-255").is_some());
    assert_eq!(model.bases.len(), 2);
    assert_eq!(
        model.base("Camp-0").unwrap().address,
        addr(0, 0, 0, 0, 1, 0)
    );
    assert_eq!(
        model.base("Camp-255").unwrap().address,
        addr(255, 0, 0, 0, 1, 0)
    );
    for name in ["Camp-0", "Camp-255"] {
        let base = model.base(name).unwrap();
        assert_eq!(base.position, [100.0, 200.0, 300.0]);
        assert_eq!(base.base_type, BaseType::HomePlanetBase);
        assert_eq!(base.owner_uid.as_deref(), Some("76561198025707979"));
    }
    let player = model.player_state.as_ref().unwrap();
    assert_eq!(
        model.player_position().copied(),
        Some(addr(0, 0, 0, 0, 1, 0))
    );
    assert_eq!(player.current_reality, 0);
    assert_eq!(player.previous_address, Some(addr(0, 80, 30, -150, 38, 0)));
    assert_eq!(player.freighter_address, None);
    assert_eq!(
        (player.units, player.nanites, player.quicksilver),
        (5_000_000, 10_000, 500)
    );
    assert_eq!(model.active_galaxy, active);
}

fn expected_edges() -> Vec<EdgeRow> {
    let mut expected = (0u16..=255)
        .map(|g| {
            let g = g as u8;
            let mut left = (g, 0x000100000000u64);
            let mut right = (g, 0x000200000001u64);
            if right < left {
                std::mem::swap(&mut left, &mut right);
            }
            EdgeRow {
                left,
                right,
                weight: 400.0f64.to_bits(),
            }
        })
        .collect::<Vec<_>>();
    expected.sort_by_key(|edge| (edge.left, edge.right, edge.weight));
    expected
}

fn assert_literal_model(model: &GalaxyModel) {
    assert_literal_model_for_active(model, 0);
    assert_eq!(view(model).edges, expected_edges());
}

type QueryRow = (SystemId, u8, String, String, Option<Biome>, String, f64);

fn planet_rows(
    model: &GalaxyModel,
    rows: Vec<(nms_graph::model::PlanetKey, &nms_core::system::Planet, f64)>,
) -> Vec<QueryRow> {
    rows.into_iter()
        .map(|((id, key_index), planet, distance)| {
            assert_eq!(planet.index, key_index);
            let system = model.system(&id).unwrap();
            (
                id,
                key_index,
                system.name.clone().unwrap(),
                planet.name.clone().unwrap(),
                planet.biome,
                format!("{:012X}", system.address.packed()),
                distance,
            )
        })
        .collect()
}

fn find_rows(rows: Vec<nms_query::find::FindResult>) -> Vec<QueryRow> {
    rows.into_iter()
        .map(|result| {
            let id = SystemId::from_address(&result.system.address);
            assert_eq!(
                result.portal_hex,
                format!("{:012X}", result.system.address.packed())
            );
            (
                id,
                result.planet.index,
                result.system.name.unwrap(),
                result.planet.name.unwrap(),
                result.planet.biome,
                result.portal_hex,
                result.distance_ly,
            )
        })
        .collect()
}

fn query_rows(model: &GalaxyModel, galaxy: u8, explicit: bool) -> Vec<QueryRow> {
    let from = addr(galaxy, 0, 0, 0, 1, 0);
    let filter = nms_graph::query::BiomeFilter {
        biome: Some(Biome::Lush),
        ..Default::default()
    };
    let nearest = if explicit {
        model.nearest_planets_from_address(&from, 2, &filter)
    } else {
        model.nearest_planets(&from, 2, &filter)
    };
    planet_rows(model, nearest)
}

fn expected_query_rows(galaxy: u8) -> Vec<QueryRow> {
    vec![
        (
            sys_id(galaxy, 0, 0, 0, 1),
            1,
            "Shared".into(),
            format!("Leaf-{galaxy}-A"),
            Some(Biome::Lush),
            "000100000000".into(),
            0.0,
        ),
        (
            sys_id(galaxy, 1, 0, 0, 2),
            1,
            "Target".into(),
            format!("Leaf-{galaxy}-B"),
            Some(Biome::Lush),
            "000200000001".into(),
            400.0,
        ),
    ]
}

fn query_rows_match(actual: &[QueryRow], galaxy: u8) -> bool {
    actual == expected_query_rows(galaxy)
}

fn assert_two_query_rows(model: &GalaxyModel, galaxy: u8, explicit: bool) {
    let a = sys_id(galaxy, 0, 0, 0, 1);
    let b = sys_id(galaxy, 1, 0, 0, 2);
    let actual = query_rows(model, galaxy, explicit);
    assert!(query_rows_match(&actual, galaxy));
    let mut foreign = actual.clone();
    foreign[0].0 = sys_id(if galaxy == 255 { 0 } else { galaxy + 1 }, 0, 0, 0, 1);
    assert_eq!(foreign[0].1, actual[0].1);
    assert_eq!(foreign[0].2, actual[0].2);
    assert_eq!(foreign[0].3, actual[0].3);
    assert_eq!(foreign[0].4, actual[0].4);
    assert_eq!(foreign[0].5, actual[0].5);
    assert_eq!(foreign[0].6, actual[0].6);
    assert_eq!(foreign[1], actual[1]);
    assert!(!query_rows_match(&foreign, galaxy));
    let mut missing_b = actual.clone();
    missing_b.pop();
    assert_eq!(missing_b, vec![actual[0].clone()]);
    assert!(!query_rows_match(&missing_b, galaxy));
    let mut duplicate_a = actual.clone();
    duplicate_a[1] = duplicate_a[0].clone();
    assert_eq!(duplicate_a[0], actual[0]);
    assert_eq!(duplicate_a[1], actual[0]);
    assert!(!query_rows_match(&duplicate_a, galaxy));
    let mut changed_biome = actual.clone();
    changed_biome[0].4 = Some(Biome::Toxic);
    let mut expected_changed_biome = actual[0].clone();
    expected_changed_biome.4 = Some(Biome::Toxic);
    assert_eq!(changed_biome[0], expected_changed_biome);
    assert_eq!(changed_biome[1], actual[1]);
    assert!(!query_rows_match(&changed_biome, galaxy));
    let mut changed_name = actual.clone();
    changed_name[0].3.push_str("-mutant");
    let mut expected_changed_name = actual[0].clone();
    expected_changed_name.3.push_str("-mutant");
    assert_eq!(changed_name[0], expected_changed_name);
    assert_eq!(changed_name[1], actual[1]);
    assert!(!query_rows_match(&changed_name, galaxy));

    let find = execute_find(
        model,
        &FindQuery {
            biome: Some(Biome::Lush),
            within_ly: Some(400.0),
            nearest: Some(2),
            from: ReferencePoint::Address(addr(galaxy, 0, 0, 0, 1, 0)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(find_rows(find), expected_query_rows(galaxy));

    let nearest_systems = if explicit {
        model.nearest_systems_from_address(&addr(galaxy, 0, 0, 0, 1, 0), 2)
    } else {
        model.nearest_systems(&addr(galaxy, 0, 0, 0, 1, 0), 2)
    };
    assert_eq!(nearest_systems, vec![(a, 0.0), (b, 400.0)]);
    let radius399 = model.systems_within_radius_from_address(&addr(galaxy, 0, 0, 0, 1, 0), 399.0);
    let radius400 = model.systems_within_radius_from_address(&addr(galaxy, 0, 0, 0, 1, 0), 400.0);
    assert_eq!(radius399, vec![(a, 0.0)]);
    assert_eq!(radius400, vec![(a, 0.0), (b, 400.0)]);
    if !explicit {
        assert_eq!(
            model.systems_within_radius(&addr(galaxy, 0, 0, 0, 1, 0), 399.0),
            radius399
        );
        assert_eq!(
            model.systems_within_radius(&addr(galaxy, 0, 0, 0, 1, 0), 400.0),
            radius400
        );
    }
    let planet399 = model.planets_within_radius_from_address(
        &addr(galaxy, 0, 0, 0, 1, 0),
        399.0,
        &nms_graph::query::BiomeFilter {
            biome: Some(Biome::Lush),
            ..Default::default()
        },
    );
    let planet400 = model.planets_within_radius_from_address(
        &addr(galaxy, 0, 0, 0, 1, 0),
        400.0,
        &nms_graph::query::BiomeFilter {
            biome: Some(Biome::Lush),
            ..Default::default()
        },
    );
    assert_eq!(
        planet_rows(model, planet399),
        expected_query_rows(galaxy)[..1]
    );
    assert_eq!(planet_rows(model, planet400), expected_query_rows(galaxy));
    if !explicit {
        let active399 = model.planets_within_radius(
            &addr(galaxy, 0, 0, 0, 1, 0),
            399.0,
            &nms_graph::query::BiomeFilter {
                biome: Some(Biome::Lush),
                ..Default::default()
            },
        );
        let active400 = model.planets_within_radius(
            &addr(galaxy, 0, 0, 0, 1, 0),
            400.0,
            &nms_graph::query::BiomeFilter {
                biome: Some(Biome::Lush),
                ..Default::default()
            },
        );
        assert_eq!(
            planet_rows(model, active399),
            expected_query_rows(galaxy)[..1]
        );
        assert_eq!(planet_rows(model, active400), expected_query_rows(galaxy));
    }

    let route = model.shortest_path(a, b).unwrap();
    assert_eq!(route.hops.len(), 2);
    assert_eq!(route.hops[0].system_id, a);
    assert_eq!(route.hops[0].leg_distance_ly, 0.0);
    assert_eq!(route.hops[0].cumulative_ly, 0.0);
    assert_eq!(route.hops[1].system_id, b);
    assert_eq!(route.hops[1].leg_distance_ly, 400.0);
    assert_eq!(route.hops[1].cumulative_ly, 400.0);
    assert_eq!(route.total_distance_ly, 400.0);
    let next = if galaxy == 255 { 0 } else { galaxy + 1 };
    assert!(matches!(
        model.shortest_path(a, sys_id(next, 0, 0, 0, 1)),
        Err(RouteError::NoPath { from, to }) if from == a && to == sys_id(next, 0, 0, 0, 1)
    ));
}

fn assert_all_galaxy_query_batches(model: &mut GalaxyModel) {
    let initial_active = model.active_galaxy;
    let baseline = view(model);
    let filter = nms_graph::query::BiomeFilter {
        biome: Some(Biome::Lush),
        ..Default::default()
    };

    for raw_galaxy in 0u16..=255 {
        let galaxy = raw_galaxy as u8;
        model.set_active_galaxy(galaxy);
        assert_two_query_rows(model, galaxy, false);
        assert_two_query_rows(model, galaxy, true);

        if galaxy == 0 {
            let current = execute_find(
                model,
                &FindQuery {
                    biome: Some(Biome::Lush),
                    within_ly: Some(400.0),
                    nearest: Some(2),
                    from: ReferencePoint::CurrentPosition,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(find_rows(current), expected_query_rows(0));
        } else {
            assert!(matches!(
                execute_find(
                    model,
                    &FindQuery {
                        from: ReferencePoint::CurrentPosition,
                        ..Default::default()
                    }
                ),
                Err(GraphError::ReferenceGalaxyMismatch { reference: 0, active }) if active == galaxy
            ));
        }

        let mut aligned = baseline.clone();
        aligned.active = galaxy;
        assert_eq!(view(model), aligned);

        let other = if galaxy == 255 { 0 } else { galaxy + 1 };
        model.set_active_galaxy(other);
        let reference = addr(galaxy, 0, 0, 0, 1, 0);
        assert!(model.nearest_systems(&reference, 2).is_empty());
        assert!(model.systems_within_radius(&reference, 399.0).is_empty());
        assert!(model.systems_within_radius(&reference, 400.0).is_empty());
        assert!(model.nearest_planets(&reference, 2, &filter).is_empty());
        assert!(
            model
                .planets_within_radius(&reference, 399.0, &filter)
                .is_empty()
        );
        assert!(
            model
                .planets_within_radius(&reference, 400.0, &filter)
                .is_empty()
        );

        // The explicit-address case uses the same complete oracle while a
        // deliberately different galaxy is selected.
        assert_two_query_rows(model, galaxy, true);
        assert_eq!(
            model.nearest_systems_from_address(&reference, 2),
            vec![
                (sys_id(galaxy, 0, 0, 0, 1), 0.0),
                (sys_id(galaxy, 1, 0, 0, 2), 400.0),
            ]
        );
        assert_eq!(
            model.systems_within_radius_from_address(&reference, 399.0),
            vec![(sys_id(galaxy, 0, 0, 0, 1), 0.0)]
        );
        assert_eq!(
            model.systems_within_radius_from_address(&reference, 400.0),
            vec![
                (sys_id(galaxy, 0, 0, 0, 1), 0.0),
                (sys_id(galaxy, 1, 0, 0, 2), 400.0),
            ]
        );
        assert_eq!(query_rows(model, galaxy, true), expected_query_rows(galaxy));
        assert_eq!(
            planet_rows(
                model,
                model.planets_within_radius_from_address(&reference, 399.0, &filter),
            ),
            expected_query_rows(galaxy)[..1]
        );
        assert_eq!(
            planet_rows(
                model,
                model.planets_within_radius_from_address(&reference, 400.0, &filter),
            ),
            expected_query_rows(galaxy)
        );
        let explicit_find = execute_find(
            model,
            &FindQuery {
                biome: Some(Biome::Lush),
                within_ly: Some(400.0),
                nearest: Some(2),
                from: ReferencePoint::Address(reference),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(find_rows(explicit_find), expected_query_rows(galaxy));

        let mut opposed = baseline.clone();
        opposed.active = other;
        assert_eq!(view(model), opposed);
    }

    model.set_active_galaxy(initial_active);
    assert_eq!(view(model), baseline);
}

#[test]
fn test_checked_save_all_galaxies_queries_routes_and_rebuild_orders() {
    let empty = GalaxyModel::new();
    assert_eq!(empty.active_galaxy, 0);
    assert!(empty.spatial.is_empty());
    assert!(empty.active_spatial().is_none());
    assert!(empty.discovered_galaxies().is_empty());

    let mut order_zero = None;
    for reverse in [false, true] {
        let (_save, mut model, _records) = checked(reverse);
        assert_literal_model(&model);
        let baseline = view(&model);
        if let Some(first) = &order_zero {
            assert_eq!(&baseline, first);
        } else {
            order_zero = Some(baseline.clone());
        }

        assert_all_galaxy_query_batches(&mut model);
        assert_eq!(view(&model), baseline);

        let expected = expected_edges();
        model.build_edges(EdgeStrategy::Knn { k: 1 });
        let mut expected_view = baseline.clone();
        expected_view.edges = expected.clone();
        assert_eq!(view(&model), expected_view);

        model.build_edges(EdgeStrategy::WarpRange { max_ly: 399.0 });
        expected_view.edges.clear();
        assert_eq!(view(&model), expected_view);

        model.build_edges(EdgeStrategy::WarpRange { max_ly: 400.0 });
        expected_view.edges = expected.clone();
        assert_eq!(view(&model), expected_view);

        model.build_edges(EdgeStrategy::default());
        assert_eq!(view(&model), expected_view);
        model.rebuild_spatial();
        assert_eq!(view(&model), expected_view);
        assert_literal_model(&model);
    }
}

#[test]
fn test_current_cache_full_256_galaxy_round_trips_both_orders() {
    for reverse in [false, true] {
        let (_save, mut fresh, _records) = checked(reverse);
        assert_literal_model(&fresh);
        fresh.set_active_galaxy(255);
        fresh.build_edges(EdgeStrategy::default());
        let expected = view(&fresh);
        let data = extract_cache_data(&fresh, 4720);
        assert_eq!(data.save_version, 4720);
        assert_eq!(data.active_galaxy, 255);
        assert_eq!(data.systems.len(), 512);
        assert_eq!(data.bases.len(), 2);
        let bytes = serialize(&data).unwrap();
        assert_eq!(&bytes[..16], b"NMSCACHE\x03\0\0\0\x05\0\0\0");
        let decoded = deserialize(&bytes).unwrap();
        assert_eq!(decoded.save_version, 4720);
        assert_eq!(decoded.active_galaxy, 255);
        let mut rebuilt = rebuild_model(&decoded);
        assert_eq!(rebuilt.active_galaxy, 255);
        assert_eq!(rebuilt.player_position().unwrap().reality_index, 0);
        assert_eq!(rebuilt.system_count(), 512);
        assert_eq!(rebuilt.planet_count(), 512);
        assert_literal_model_for_active(&rebuilt, 255);
        assert_eq!(view(&rebuilt), expected);
        assert_all_galaxy_query_batches(&mut rebuilt);
        assert_eq!(view(&rebuilt), expected);
    }
}

#[derive(Clone, Debug, PartialEq)]
enum PositionView {
    Base(String, GalacticAddress),
    Player(GalacticAddress),
    Address(GalacticAddress),
}

#[derive(Clone, Debug, PartialEq)]
struct SessionView {
    position: Option<PositionView>,
    biome: Option<Biome>,
    warp: Option<f64>,
    galaxy: (u8, String, GalaxyType),
    systems: usize,
    planets: usize,
}

fn session_view(session: &SessionState) -> SessionView {
    SessionView {
        position: session.position.as_ref().map(|position| match position {
            PositionContext::Base { name, address } => PositionView::Base(name.clone(), *address),
            PositionContext::PlayerPosition(address) => PositionView::Player(*address),
            PositionContext::Address(address) => PositionView::Address(*address),
        }),
        biome: session.biome_filter,
        warp: session.warp_range,
        galaxy: (
            session.galaxy.index,
            session.galaxy.name.to_owned(),
            session.galaxy.galaxy_type,
        ),
        systems: session.system_count,
        planets: session.planet_count,
    }
}

fn dispatch_line(
    model: &mut GalaxyModel,
    session: &mut SessionState,
    line: &str,
) -> Result<String, String> {
    let action = nms_copilot::commands::parse_line(line)?.expect("nonempty REPL command");
    nms_copilot::dispatch::dispatch(&action, model, session)
}

#[test]
fn test_actual_repl_selects_all_galaxies_and_preserves_session() {
    let (_save, mut data_model, _records) = checked(true);
    data_model.set_active_galaxy(255);
    let data = extract_cache_data(&data_model, 4720);
    assert_eq!(data.save_version, 4720);
    assert_eq!(data.active_galaxy, 255);
    assert_eq!(data.systems.len(), 512);
    assert_eq!(data.bases.len(), 2);
    let bytes = serialize(&data).unwrap();
    assert_eq!(&bytes[..16], b"NMSCACHE\x03\0\0\0\x05\0\0\0");
    let decoded = deserialize(&bytes).unwrap();
    let mut model = rebuild_model(&decoded);
    assert_eq!(view(&model), view(&data_model));
    assert_literal_model_for_active(&model, 255);
    assert_literal_model_for_active(&data_model, 255);
    assert_eq!(view(&model).edges, expected_edges());
    let mut session = SessionState::from_model(&model);
    assert_eq!(session.galaxy.index, 255);
    assert_eq!(
        session.galaxy.galaxy_type,
        Galaxy::by_index(255).galaxy_type
    );
    assert_eq!(
        session_view(&session).position,
        Some(PositionView::Player(addr(0, 0, 0, 0, 1, 0)))
    );
    assert_eq!((session.system_count, session.planet_count), (512, 512));
    let anchor = addr(255, 0, 0, 0, 1, 0);
    session.set_position_address(anchor);
    session.set_biome_filter(Biome::Lush);
    session.set_warp_range(1200.0);

    let complete = view(&model);
    for g in 0u16..=255 {
        let g = g as u8;
        let output = dispatch_line(&mut model, &mut session, &format!("galaxy {g}")).unwrap();
        let galaxy = Galaxy::by_index(g);
        assert_eq!(
            output,
            format!(
                "Switched to {} (galaxy {}). 2 systems loaded.\n",
                galaxy.name, galaxy.index
            )
        );
        assert_eq!(model.active_galaxy, g);
        let actual_session = session_view(&session);
        assert_eq!(
            actual_session.galaxy,
            (g, galaxy.name.to_owned(), galaxy.galaxy_type)
        );
        assert_eq!(actual_session.position, Some(PositionView::Address(anchor)));
        assert_eq!(actual_session.biome, Some(Biome::Lush));
        assert_eq!(actual_session.warp, Some(1200.0));
        assert_eq!((actual_session.systems, actual_session.planets), (512, 512));
        let mut expected = complete.clone();
        expected.active = g;
        assert_eq!(view(&model), expected);

        if [0, 1, 9, 255].contains(&g) {
            let prompt_state = nms_copilot::prompt::PromptState::from_session(&session);
            let prompt = nms_copilot::prompt::CopilotPrompt::new(prompt_state);
            assert_eq!(
                prompt.render_prompt_left().as_ref(),
                format!("[{} | Lush | 512 planets] 🚀", galaxy.name)
            );
        }
    }

    let list = dispatch_line(&mut model, &mut session, "galaxy").unwrap();
    let expected_names = (0u16..=255)
        .map(|index| {
            let galaxy = Galaxy::by_index(index as u8);
            format!("{} ({})", galaxy.name, galaxy.index)
        })
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        list,
        format!("Active galaxy: Odyalutai (255)\nDiscovered galaxies: {expected_names}\n")
    );

    for (command, index, name, exact) in [
        (
            "galaxy Euclid",
            0,
            "Euclid",
            "Switched to Euclid (galaxy 0). 2 systems loaded.\n",
        ),
        (
            "galaxy \"hIlBeRt DiMeNsIoN\"",
            1,
            "Hilbert Dimension",
            "Switched to Hilbert Dimension (galaxy 1). 2 systems loaded.\n",
        ),
        (
            "galaxy \"eIsSeNtAm\"",
            9,
            "Eissentam",
            "Switched to Eissentam (galaxy 9). 2 systems loaded.\n",
        ),
        (
            "galaxy Odyalutai",
            255,
            "Odyalutai",
            "Switched to Odyalutai (galaxy 255). 2 systems loaded.\n",
        ),
    ] {
        assert_eq!(
            dispatch_line(&mut model, &mut session, command).unwrap(),
            exact
        );
        let mut expected_model = complete.clone();
        expected_model.active = index;
        assert_eq!(view(&model), expected_model);
        let expected_galaxy = Galaxy::by_index(index);
        assert_eq!(expected_galaxy.name, name);
        assert_eq!(
            session_view(&session),
            SessionView {
                position: Some(PositionView::Address(anchor)),
                biome: Some(Biome::Lush),
                warp: Some(1200.0),
                galaxy: (index, name.to_owned(), expected_galaxy.galaxy_type),
                systems: 512,
                planets: 512,
            }
        );
        let prompt_state = nms_copilot::prompt::PromptState::from_session(&session);
        let prompt = nms_copilot::prompt::CopilotPrompt::new(prompt_state);
        assert_eq!(
            prompt.render_prompt_left().as_ref(),
            format!("[{name} | Lush | 512 planets] 🚀")
        );
    }

    dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    let before_model = view(&model);
    let before_session = session_view(&session);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "galaxy 256").unwrap_err(),
        "Galaxy index out of range: 256 (must be 0-255)"
    );
    assert_eq!(view(&model), before_model);
    assert_eq!(session_view(&session), before_session);
    assert_eq!(
        dispatch_line(&mut model, &mut session, "galaxy Unknown").unwrap_err(),
        "Unknown galaxy: \"Unknown\". Use a number 0-255 or a name like \"Euclid\"."
    );
    assert_eq!(view(&model), before_model);
    assert_eq!(session_view(&session), before_session);
    let parser_error = nms_copilot::commands::parse_line("galaxy -1").unwrap_err();
    assert!(parser_error.contains("unexpected argument"));
    assert!(parser_error.contains("-1"));
    assert_eq!(view(&model), before_model);
    assert_eq!(session_view(&session), before_session);

    session.set_position_base("Camp-0", &model).unwrap();
    let base_session = session_view(&session);
    let base_model = view(&model);
    dispatch_line(&mut model, &mut session, "galaxy 0").unwrap();
    let mut expected_model = base_model.clone();
    expected_model.active = 0;
    assert_eq!(view(&model), expected_model);
    let mut expected_session = base_session.clone();
    expected_session.galaxy = (0, "Euclid".into(), Galaxy::by_index(0).galaxy_type);
    assert_eq!(session_view(&session), expected_session);
    dispatch_line(&mut model, &mut session, "galaxy 255").unwrap();
    expected_model.active = 255;
    assert_eq!(view(&model), expected_model);
    expected_session.galaxy = (255, "Odyalutai".into(), Galaxy::by_index(255).galaxy_type);
    assert_eq!(session_view(&session), expected_session);
    dispatch_line(&mut model, &mut session, "galaxy 9").unwrap();
    session.reset_position(&model);
    assert_eq!(
        session_view(&session).position,
        Some(PositionView::Player(addr(0, 0, 0, 0, 1, 0)))
    );
    let following_model = view(&model);
    let following_session = session_view(&session);
    dispatch_line(&mut model, &mut session, "galaxy 255").unwrap();
    let mut expected_following_model = following_model;
    expected_following_model.active = 255;
    assert_eq!(view(&model), expected_following_model);
    let mut expected_following_session = following_session;
    expected_following_session.galaxy =
        (255, "Odyalutai".into(), Galaxy::by_index(255).galaxy_type);
    assert_eq!(session_view(&session), expected_following_session);
}
