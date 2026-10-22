use std::collections::HashMap;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::delta::{DiscoveryNames, SaveDelta, SaveUpdate};
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System, SystemId};
use nms_graph::extract::{extract_discovery_names_from_validated, extract_systems_from_validated};
use nms_graph::{BiomeFilter, EdgeStrategy, GalaxyModel};
use nms_save::parse_validated_save;
use serde_json::{Value, json};

const MINIMAL: &[u8] = include_bytes!("../../../data/test/save56_minimal.json");

fn record(hex: &str, discovery_type: &str, metadata: Option<Value>) -> Value {
    let source: Value = serde_json::from_slice(MINIMAL).unwrap();
    let mut row = source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][0].clone();
    row["DD"]["UA"] = json!(hex);
    row["DD"]["DT"] = json!(discovery_type);
    if let Some(metadata) = metadata {
        row["DM"] = metadata;
    } else {
        row.as_object_mut().unwrap().remove("DM");
    }
    row
}

fn checked_save(rows: Vec<Value>) -> (Value, nms_save::ValidatedSave) {
    let mut source: Value = serde_json::from_slice(MINIMAL).unwrap();
    source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!(rows);
    let checked = parse_validated_save(source.to_string().as_bytes()).unwrap();
    (source, checked)
}

fn id(x: i16, y: i8, z: i16, ssi: u16, galaxy: u8) -> SystemId {
    SystemId::from_address(&GalacticAddress::new(x, y, z, ssi, 0, galaxy))
}

#[test]
fn test_discovery_names_public_type_supports_empty_and_absent_values() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DiscoveryNames>();

    let a0 = id(100, 0, 0, 7, 0);
    let mut observations = DiscoveryNames::default();
    assert!(observations.is_empty());
    assert!(observations.system_names.is_empty());
    assert!(observations.planet_names.is_empty());

    observations.system_names.insert(a0, String::new());
    assert!(!observations.is_empty());
    assert_eq!(
        observations.system_names.get(&a0).map(String::as_str),
        Some("")
    );
    assert!(!observations.planet_names.contains_key(&(a0, 1)));

    let mut clone = observations.clone();
    assert_eq!(clone, observations);
    clone.planet_names.insert((a0, 1), "literal planet".into());
    assert_ne!(clone, observations);
    assert!(!observations.planet_names.contains_key(&(a0, 1)));

    let planet_only = DiscoveryNames {
        planet_names: HashMap::from([((a0, 2), "Planet only".to_owned())]),
        ..DiscoveryNames::default()
    };
    assert!(!planet_only.is_empty());
    assert!(planet_only.system_names.is_empty());
    assert_eq!(
        planet_only.planet_names,
        HashMap::from([((a0, 2), "Planet only".to_owned())])
    );
}

fn n2_rows(reverse_galaxy_groups: bool) -> Vec<Value> {
    let galaxy_zero = vec![
        record(
            "0x00080000000065",
            "UnknownType",
            Some(json!({"CN": "ignored"})),
        ),
        record(
            "0x00070000000064",
            "SolarSystem",
            Some(json!({"CN": "Kepler, \"α\"\nNorth"})),
        ),
        record(
            "0x10070000000064",
            "Planet",
            Some(json!({"CN": " Lush, \"β\"\nMoon "})),
        ),
        record("0x20070000000064", "Planet", Some(json!({"CN": ""}))),
        record("0x10080000000065", "Planet", Some(json!({"CN": "Orphan"}))),
    ];
    let galaxy_nine = vec![
        record(
            "0x00070900000064",
            "SolarSystem",
            Some(json!({"CN": "Kepler"})),
        ),
        record("0x10070900000064", "Planet", Some(json!({"CN": "Kepler"}))),
    ];
    if reverse_galaxy_groups {
        galaxy_nine.into_iter().chain(galaxy_zero).collect()
    } else {
        galaxy_zero.into_iter().chain(galaxy_nine).collect()
    }
}

#[test]
fn test_extract_discovery_names_uses_checked_addresses_and_full_literal_maps() {
    let a0 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 0));
    let a9 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 9));
    let b0 = SystemId::from_address(&GalacticAddress::new(101, 0, 0, 8, 0, 0));
    let expected = DiscoveryNames {
        system_names: HashMap::from([(a0, "Kepler, \"α\"\nNorth".into()), (a9, "Kepler".into())]),
        planet_names: HashMap::from([
            ((a0, 1), " Lush, \"β\"\nMoon ".into()),
            ((a9, 1), "Kepler".into()),
            ((a0, 2), String::new()),
            ((b0, 1), "Orphan".into()),
        ]),
    };

    for reverse_groups in [false, true] {
        let (_, checked) = checked_save(n2_rows(reverse_groups));
        let observed = extract_discovery_names_from_validated(&checked);
        assert_eq!(observed, expected);
        assert_eq!(observed.system_names.len(), 2);
        assert_eq!(observed.planet_names.len(), 4);
        assert!(!observed.system_names.contains_key(&b0));
    }
}

fn absent_variants() -> Vec<Option<Value>> {
    vec![
        None,
        Some(Value::Null),
        Some(json!("not metadata object")),
        Some(json!([])),
        Some(json!(7)),
        Some(json!({})),
        Some(json!({"CN": null})),
        Some(json!({"CN": 7})),
        Some(json!({"CN": false})),
        Some(json!({"CN": []})),
        Some(json!({"CN": {"nested": "value"}})),
    ]
}

#[test]
fn test_extract_discovery_names_first_invalid_rows_reserve_each_key() {
    const SYSTEM: &str = "0x00070000000064";
    const PLANET: &str = "0x10070000000064";
    let expected_input_addresses = vec![json!(SYSTEM), json!(SYSTEM), json!(PLANET), json!(PLANET)];
    let expected_checked_addresses = vec![
        GalacticAddress::new(100, 0, 0, 7, 0, 0),
        GalacticAddress::new(100, 0, 0, 7, 0, 0),
        GalacticAddress::new(100, 0, 0, 7, 1, 0),
        GalacticAddress::new(100, 0, 0, 7, 1, 0),
    ];

    for metadata in absent_variants() {
        let rows = vec![
            record(SYSTEM, "SolarSystem", metadata.clone()),
            record(
                SYSTEM,
                "SolarSystem",
                Some(json!({"CN": "later valid system"})),
            ),
            record(PLANET, "Planet", metadata.clone()),
            record(PLANET, "Planet", Some(json!({"CN": "later valid planet"}))),
        ];
        assert_eq!(
            rows.iter()
                .map(|row| row["DD"]["UA"].clone())
                .collect::<Vec<_>>(),
            expected_input_addresses
        );

        let original_rows = rows.clone();
        let (source, checked) = checked_save(rows);
        let before = source.clone();
        let checked_addresses = checked
            .discoveries()
            .map(|(_, address)| address)
            .collect::<Vec<_>>();
        assert_eq!(checked_addresses, expected_checked_addresses);

        let observed = extract_discovery_names_from_validated(&checked);
        assert_eq!(observed, DiscoveryNames::default());
        assert_eq!(source, before);

        let typed_rows = &checked
            .root()
            .discovery_manager_data
            .discovery_data_v1
            .store
            .record;
        assert_eq!(typed_rows.len(), 4);
        assert_eq!(typed_rows.len(), original_rows.len());
        for (typed, original) in typed_rows.iter().zip(original_rows) {
            assert_eq!(typed.dm, original.get("DM").cloned().unwrap_or(Value::Null));
        }
    }
}

#[test]
fn test_extract_discovery_names_keeps_first_valid_empty_and_unknown_type_rules() {
    let rows = vec![
        record(
            "0x00070000000064",
            "SolarSystem",
            Some(json!({"CN": "first system"})),
        ),
        record(
            "0x00070000000064",
            "SolarSystem",
            Some(json!({"CN": "contradictory system"})),
        ),
        record(
            "0x20070000000064",
            "SolarSystem",
            Some(json!({"CN": "planet nibble does not change system identity"})),
        ),
        record("0x00080000000065", "SolarSystem", Some(json!({"CN": ""}))),
        record(
            "0x00080000000065",
            "SolarSystem",
            Some(json!({"CN": "later system"})),
        ),
        record(
            "0x10070000000064",
            "Planet",
            Some(json!({"CN": "first planet"})),
        ),
        record(
            "0x10070000000064",
            "Planet",
            Some(json!({"CN": "contradictory planet"})),
        ),
        record("0x20070000000064", "Planet", Some(json!({"CN": ""}))),
        record(
            "0x20070000000064",
            "Planet",
            Some(json!({"CN": "later planet"})),
        ),
        record(
            "0x00070900000064",
            "UnknownType",
            Some(json!({"CN": "does not reserve"})),
        ),
        record(
            "0x00070900000064",
            "SolarSystem",
            Some(json!({"CN": "recognized"})),
        ),
        record(
            "0x10070900000064",
            "UnknownType",
            Some(json!({"CN": "does not reserve"})),
        ),
        record(
            "0x10070900000064",
            "Planet",
            Some(json!({"CN": "recognized planet"})),
        ),
    ];
    let original = rows.clone();
    let (source, checked) = checked_save(rows);
    let original_source = source.clone();
    let names = extract_discovery_names_from_validated(&checked);
    let a0 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 0));
    let b0 = SystemId::from_address(&GalacticAddress::new(101, 0, 0, 8, 0, 0));
    let a9 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 9));
    assert_eq!(
        names,
        DiscoveryNames {
            system_names: HashMap::from([
                (a0, "first system".into()),
                (b0, String::new()),
                (a9, "recognized".into()),
            ]),
            planet_names: HashMap::from([
                ((a0, 1), "first planet".into()),
                ((a0, 2), String::new()),
                ((a9, 1), "recognized planet".into()),
            ]),
        }
    );
    assert_eq!(source, original_source);
    for (typed, original) in checked
        .root()
        .discovery_manager_data
        .discovery_data_v1
        .store
        .record
        .iter()
        .zip(original)
    {
        assert_eq!(typed.dm, original.get("DM").cloned().unwrap_or(Value::Null));
    }
}

#[derive(Debug, PartialEq)]
struct NonNameState {
    bases: HashMap<String, PlayerBase>,
    player_state: Option<PlayerState>,
    active_galaxy: u8,
    address_to_id: HashMap<SystemId, SystemId>,
    node_map: HashMap<SystemId, petgraph::stable_graph::NodeIndex>,
    graph_nodes: Vec<String>,
    graph_edges: Vec<(String, String, u64)>,
    spatial: HashMap<u8, Vec<(String, [u64; 3])>>,
    biome_index: HashMap<Biome, HashMap<(SystemId, u8), usize>>,
    placeholders: std::collections::HashSet<SystemId>,
    system_count: usize,
    planet_count: usize,
    base_count: usize,
}

fn non_name_state(model: &GalaxyModel) -> NonNameState {
    let mut graph_nodes = model
        .graph
        .node_weights()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    graph_nodes.sort();

    let mut graph_edges = model
        .graph
        .edge_indices()
        .map(|edge| {
            let (left, right) = model.graph.edge_endpoints(edge).unwrap();
            let mut left = model.graph.node_weight(left).unwrap().to_string();
            let mut right = model.graph.node_weight(right).unwrap().to_string();
            if left > right {
                std::mem::swap(&mut left, &mut right);
            }
            (
                left,
                right,
                model.graph.edge_weight(edge).unwrap().to_bits(),
            )
        })
        .collect::<Vec<_>>();
    graph_edges.sort();

    let spatial = model
        .spatial
        .iter()
        .map(|(galaxy, tree)| {
            let mut points = tree
                .iter()
                .map(|point| {
                    (
                        point.id.to_string(),
                        [
                            point.point[0].to_bits(),
                            point.point[1].to_bits(),
                            point.point[2].to_bits(),
                        ],
                    )
                })
                .collect::<Vec<_>>();
            points.sort();
            (*galaxy, points)
        })
        .collect();

    let mut biome_index: HashMap<Biome, HashMap<(SystemId, u8), usize>> = HashMap::new();
    for (biome, keys) in &model.biome_index {
        let counts = biome_index.entry(*biome).or_default();
        for key in keys {
            *counts.entry(*key).or_default() += 1;
        }
    }

    let placeholders = model
        .systems
        .keys()
        .copied()
        .filter(|system_id| model.is_player_placeholder(system_id))
        .collect();

    NonNameState {
        bases: model.bases.clone(),
        player_state: model.player_state.clone(),
        active_galaxy: model.active_galaxy,
        address_to_id: model.address_to_id.clone(),
        node_map: model.node_map.clone(),
        graph_nodes,
        graph_edges,
        spatial,
        biome_index,
        placeholders,
        system_count: model.system_count(),
        planet_count: model.planet_count(),
        base_count: model.base_count(),
    }
}

fn expected_systems(
    a0_name: Option<&str>,
    a0_p1: Option<&str>,
    a0_p2: Option<&str>,
    a9_name: Option<&str>,
    a9_p1: Option<&str>,
) -> HashMap<SystemId, System> {
    let a0 = GalacticAddress::new(100, 0, 0, 7, 0, 0);
    let a9 = GalacticAddress::new(100, 0, 0, 7, 0, 9);
    let b0 = GalacticAddress::new(101, 0, 0, 8, 0, 0);
    let c0 = GalacticAddress::new(102, 0, 0, 9, 0, 0);
    let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0);
    HashMap::from([
        (
            SystemId::from_address(&a0),
            System::new(
                a0,
                a0_name.map(str::to_owned),
                Some("Owner".into()),
                timestamp,
                vec![
                    Planet::new(
                        1,
                        Some(Biome::Toxic),
                        None,
                        true,
                        a0_p1.map(str::to_owned),
                        Some(0xAB),
                    ),
                    Planet::new(
                        2,
                        Some(Biome::Lush),
                        None,
                        false,
                        a0_p2.map(str::to_owned),
                        Some(0xAC),
                    ),
                ],
            ),
        ),
        (
            SystemId::from_address(&a9),
            System::new(
                a9,
                a9_name.map(str::to_owned),
                Some("Owner".into()),
                timestamp,
                vec![Planet::new(
                    1,
                    Some(Biome::Frozen),
                    None,
                    false,
                    a9_p1.map(str::to_owned),
                    Some(0xAD),
                )],
            ),
        ),
        (
            SystemId::from_address(&b0),
            System::new(
                b0,
                Some("Absent Import".into()),
                Some("Owner".into()),
                timestamp,
                vec![Planet::new(
                    2,
                    Some(Biome::Barren),
                    None,
                    false,
                    Some("Imported P2".into()),
                    Some(0xAE),
                )],
            ),
        ),
        (
            SystemId::from_address(&c0),
            System::new(c0, None, None, None, vec![]),
        ),
    ])
}

fn expected_planets(
    a0_p1: Option<&str>,
    a0_p2: Option<&str>,
    a9_p1: Option<&str>,
) -> HashMap<(SystemId, u8), Planet> {
    let a0 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 0));
    let a9 = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 9));
    let b0 = SystemId::from_address(&GalacticAddress::new(101, 0, 0, 8, 0, 0));
    HashMap::from([
        (
            (a0, 1),
            Planet::new(
                1,
                Some(Biome::Toxic),
                None,
                true,
                a0_p1.map(str::to_owned),
                Some(0xAB),
            ),
        ),
        (
            (a0, 2),
            Planet::new(
                2,
                Some(Biome::Lush),
                None,
                false,
                a0_p2.map(str::to_owned),
                Some(0xAC),
            ),
        ),
        (
            (a9, 1),
            Planet::new(
                1,
                Some(Biome::Frozen),
                None,
                false,
                a9_p1.map(str::to_owned),
                Some(0xAD),
            ),
        ),
        (
            (b0, 2),
            Planet::new(
                2,
                Some(Biome::Barren),
                None,
                false,
                Some("Imported P2".into()),
                Some(0xAE),
            ),
        ),
    ])
}

fn expected_name_index(
    a0_name: Option<&str>,
    a9_name: Option<&str>,
) -> HashMap<(u8, String), SystemId> {
    let mut index = HashMap::new();
    if let Some(name) = a0_name {
        index.insert(
            (0, name.to_lowercase()),
            SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 0)),
        );
    }
    if let Some(name) = a9_name {
        index.insert(
            (9, name.to_lowercase()),
            SystemId::from_address(&GalacticAddress::new(100, 0, 0, 7, 0, 9)),
        );
    }
    index.insert(
        (0, "absent import".into()),
        SystemId::from_address(&GalacticAddress::new(101, 0, 0, 8, 0, 0)),
    );
    index
}

fn assert_name_stage(
    model: &GalaxyModel,
    structure: &NonNameState,
    systems: HashMap<SystemId, System>,
    planets: HashMap<(SystemId, u8), Planet>,
    names: HashMap<(u8, String), SystemId>,
) {
    assert_eq!(non_name_state(model), *structure);
    assert_eq!(model.systems, systems);
    assert_eq!(model.planets, planets);
    assert_eq!(model.name_index, names);
    for ((galaxy, indexed_name), system_id) in &model.name_index {
        let system = model
            .system(system_id)
            .expect("name index points to a system");
        assert_eq!(system.address.reality_index, *galaxy);
        assert_eq!(
            system.name.as_deref().map(str::to_lowercase).as_deref(),
            Some(indexed_name.as_str())
        );
        assert!(!model.is_player_placeholder(system_id));
    }
}

fn model_with_name_fixture() -> (GalaxyModel, SystemId, SystemId, SystemId, SystemId) {
    let a0 = GalacticAddress::new(100, 0, 0, 7, 0, 0);
    let a9 = GalacticAddress::new(100, 0, 0, 7, 0, 9);
    let b0 = GalacticAddress::new(101, 0, 0, 8, 0, 0);
    let c0 = GalacticAddress::new(102, 0, 0, 9, 0, 0);
    let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0);
    let mut model = GalaxyModel::new();
    model.insert_system(System::new(
        a0,
        Some("Community Rest".into()),
        Some("Owner".into()),
        timestamp,
        vec![
            Planet::new(
                1,
                Some(Biome::Toxic),
                None,
                true,
                Some("Community Moon".into()),
                Some(0xAB),
            ),
            Planet::new(2, Some(Biome::Lush), None, false, None, Some(0xAC)),
        ],
    ));
    model.insert_system(System::new(
        a9,
        Some("Community Nine".into()),
        Some("Owner".into()),
        timestamp,
        vec![Planet::new(
            1,
            Some(Biome::Frozen),
            None,
            false,
            Some("Nine Moon".into()),
            Some(0xAD),
        )],
    ));
    model.insert_system(System::new(
        b0,
        Some("Absent Import".into()),
        Some("Owner".into()),
        timestamp,
        vec![Planet::new(
            2,
            Some(Biome::Barren),
            None,
            false,
            Some("Imported P2".into()),
            Some(0xAE),
        )],
    ));
    model.insert_player_placeholder(c0);
    model.player_state = Some(PlayerState::new(
        GalacticAddress::new(100, 0, 0, 7, 1, 0),
        0,
        Some(GalacticAddress::new(99, 0, 0, 6, 0, 0)),
        None,
        123,
        45,
        6,
    ));
    model.insert_base(PlayerBase::new(
        "Anchor".into(),
        BaseType::HomePlanetBase,
        GalacticAddress::new(100, 0, 0, 7, 1, 0),
        [1.0, 2.0, 3.0],
        Some("uid".into()),
    ));
    model.set_active_galaxy(9);
    model.build_edges(EdgeStrategy::WarpRange { max_ly: 500.0 });
    (
        model,
        SystemId::from_address(&a0),
        SystemId::from_address(&a9),
        SystemId::from_address(&b0),
        SystemId::from_address(&c0),
    )
}

#[test]
fn test_apply_discovery_names_updates_only_existing_entities_and_preserves_model_state() {
    let (mut model, a0, a9, b0, c0) = model_with_name_fixture();
    let structure = non_name_state(&model);
    assert_eq!(
        structure.placeholders,
        std::collections::HashSet::from([c0])
    );
    assert_eq!(structure.graph_edges.len(), 2);
    assert_eq!(model.graph.edge_count(), 2);
    assert_name_stage(
        &model,
        &structure,
        expected_systems(
            Some("Community Rest"),
            Some("Community Moon"),
            None,
            Some("Community Nine"),
            Some("Nine Moon"),
        ),
        expected_planets(Some("Community Moon"), None, Some("Nine Moon")),
        expected_name_index(Some("Community Rest"), Some("Community Nine")),
    );

    let mut prior_planet_only = DiscoveryNames::default();
    prior_planet_only
        .planet_names
        .insert((a0, 2), "Old P2".into());
    assert!(model.apply_discovery_names(&prior_planet_only));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(
            Some("Community Rest"),
            Some("Community Moon"),
            Some("Old P2"),
            Some("Community Nine"),
            Some("Nine Moon"),
        ),
        expected_planets(Some("Community Moon"), Some("Old P2"), Some("Nine Moon")),
        expected_name_index(Some("Community Rest"), Some("Community Nine")),
    );

    let observations = extract_discovery_names_from_validated(&checked_save(n2_rows(false)).1);
    let missing = SystemId::from_address(&GalacticAddress::new(103, 0, 0, 10, 0, 0));
    let mut with_missing = observations.clone();
    with_missing
        .system_names
        .insert(missing, "Must not insert".into());
    with_missing
        .system_names
        .insert(c0, "Must not promote".into());
    with_missing
        .planet_names
        .insert((a0, 7), "Must not insert".into());

    assert!(model.apply_discovery_names(&with_missing));
    assert_eq!(model.system_count(), 4);
    assert_eq!(model.planet_count(), 4);
    assert!(model.system(&missing).is_none());
    assert!(model.is_player_placeholder(&c0));
    assert_eq!(
        model.system(&c0).and_then(|system| system.name.as_deref()),
        None
    );
    assert_name_stage(
        &model,
        &structure,
        expected_systems(
            Some("Kepler, \"α\"\nNorth"),
            Some(" Lush, \"β\"\nMoon "),
            None,
            Some("Kepler"),
            Some("Kepler"),
        ),
        expected_planets(Some(" Lush, \"β\"\nMoon "), None, Some("Kepler")),
        expected_name_index(Some("Kepler, \"α\"\nNorth"), Some("Kepler")),
    );
    assert_eq!(
        model
            .system_by_name_in_galaxy("kepler, \"α\"\nnorth", 0)
            .map(|(id, _)| *id),
        Some(a0)
    );
    assert!(model.system_by_name_in_galaxy("kepler", 0).is_none());
    assert_eq!(
        model
            .system_by_name_in_galaxy("kepler", 9)
            .map(|(id, _)| *id),
        Some(a9)
    );
    assert_eq!(
        model
            .system_by_name_in_galaxy("absent import", 0)
            .map(|(id, _)| *id),
        Some(b0)
    );
    assert_eq!(
        model.system(&b0).and_then(|system| system.name.as_deref()),
        Some("Absent Import")
    );
    assert_eq!(
        model
            .planets
            .get(&(b0, 2))
            .and_then(|planet| planet.name.as_deref()),
        Some("Imported P2")
    );
    assert!(!model.apply_discovery_names(&with_missing));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(
            Some("Kepler, \"α\"\nNorth"),
            Some(" Lush, \"β\"\nMoon "),
            None,
            Some("Kepler"),
            Some("Kepler"),
        ),
        expected_planets(Some(" Lush, \"β\"\nMoon "), None, Some("Kepler")),
        expected_name_index(Some("Kepler, \"α\"\nNorth"), Some("Kepler")),
    );

    let mut clear = DiscoveryNames::default();
    clear.system_names.insert(a0, String::new());
    clear.planet_names.insert((a0, 1), String::new());
    assert!(model.apply_discovery_names(&clear));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(None, None, None, Some("Kepler"), Some("Kepler")),
        expected_planets(None, None, Some("Kepler")),
        expected_name_index(None, Some("Kepler")),
    );
    assert!(model.system_by_name_in_galaxy("kepler", 9).is_some());
    assert!(model.system_by_name_in_galaxy("absent import", 0).is_some());
    assert!(!model.apply_discovery_names(&clear));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(None, None, None, Some("Kepler"), Some("Kepler")),
        expected_planets(None, None, Some("Kepler")),
        expected_name_index(None, Some("Kepler")),
    );

    assert!(!model.apply_discovery_names(&DiscoveryNames::default()));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(None, None, None, Some("Kepler"), Some("Kepler")),
        expected_planets(None, None, Some("Kepler")),
        expected_name_index(None, Some("Kepler")),
    );

    let mut system_only = DiscoveryNames::default();
    system_only.system_names.insert(a0, "Kepler".into());
    assert!(model.apply_discovery_names(&system_only));
    assert_name_stage(
        &model,
        &structure,
        expected_systems(Some("Kepler"), None, None, Some("Kepler"), Some("Kepler")),
        expected_planets(None, None, Some("Kepler")),
        expected_name_index(Some("Kepler"), Some("Kepler")),
    );
    assert_eq!(
        model
            .system_by_name_in_galaxy("kepler", 0)
            .map(|(id, _)| *id),
        Some(a0)
    );
    assert_eq!(
        model
            .system_by_name_in_galaxy("kepler", 9)
            .map(|(id, _)| *id),
        Some(a9)
    );

    let unnamed_planet = model.system(&a0).unwrap().planets[0].clone();
    assert!(
        !BiomeFilter {
            biome: Some(Biome::Toxic),
            named_only: true,
            ..Default::default()
        }
        .matches(&unnamed_planet)
    );
    let named_planet = model.system(&a9).unwrap().planets[0].clone();
    assert!(
        BiomeFilter {
            biome: Some(Biome::Frozen),
            named_only: true,
            ..Default::default()
        }
        .matches(&named_planet)
    );
}

#[test]
fn test_apply_discovery_names_preserves_valid_duplicate_alias_representatives() {
    for keep_first_representative in [false, true] {
        for clear_representative in [false, true] {
            let first_address = GalacticAddress::new(10, 0, 0, 1, 0, 0);
            let second_address = GalacticAddress::new(20, 0, 0, 2, 0, 0);
            let first = SystemId::from_address(&first_address);
            let second = SystemId::from_address(&second_address);
            let mut model = GalaxyModel::new();
            let (insert_first, insert_second) = if keep_first_representative {
                (second_address, first_address)
            } else {
                (first_address, second_address)
            };
            model.insert_system(System::new(
                insert_first,
                Some("Shared".into()),
                None,
                None,
                vec![],
            ));
            model.insert_system(System::new(
                insert_second,
                Some("Shared".into()),
                None,
                None,
                vec![],
            ));
            let representative = model.name_index[&(0, "shared".into())];
            let cleared = if clear_representative {
                representative
            } else if representative == first {
                second
            } else {
                first
            };
            let remaining = if cleared == first { second } else { first };
            let structure = non_name_state(&model);
            let mut names = DiscoveryNames::default();
            names.system_names.insert(cleared, String::new());
            assert!(model.apply_discovery_names(&names));

            let expected_systems = HashMap::from([
                (
                    first,
                    System::new(
                        first_address,
                        (cleared != first).then(|| "Shared".to_owned()),
                        None,
                        None,
                        vec![],
                    ),
                ),
                (
                    second,
                    System::new(
                        second_address,
                        (cleared != second).then(|| "Shared".to_owned()),
                        None,
                        None,
                        vec![],
                    ),
                ),
            ]);
            assert_eq!(model.systems, expected_systems);
            assert_eq!(model.planets, HashMap::new());
            assert_eq!(
                model.name_index,
                HashMap::from([((0, "shared".into()), remaining)])
            );
            assert_eq!(
                model
                    .system_by_name_in_galaxy("shared", 0)
                    .map(|(id, _)| *id),
                Some(remaining)
            );
            for ((galaxy, indexed_name), system_id) in &model.name_index {
                let system = model.system(system_id).unwrap();
                assert_eq!(system.address.reality_index, *galaxy);
                assert_eq!(
                    system.name.as_deref().map(str::to_lowercase).as_deref(),
                    Some(indexed_name.as_str())
                );
                assert!(!model.is_player_placeholder(system_id));
            }
            assert_eq!(non_name_state(&model), structure);
        }
    }
}

#[test]
fn test_checked_cold_extraction_applies_names_and_legacy_extraction_stays_unnamed() {
    let (source, checked) = checked_save(vec![record(
        "0x00070000000064",
        "SolarSystem",
        Some(json!({"CN": "opt-in name"})),
    )]);
    let cold_systems = extract_systems_from_validated(&checked);
    assert_eq!(cold_systems.len(), 1);
    assert_eq!(
        cold_systems.values().next().unwrap().name.as_deref(),
        Some("opt-in name")
    );
    let mut model = GalaxyModel::from_validated_save(&checked);
    assert_eq!(
        model.systems.values().next().unwrap().name.as_deref(),
        Some("opt-in name")
    );

    let names = extract_discovery_names_from_validated(&checked);
    assert_eq!(names.system_names.len(), 1);
    assert!(!model.apply_discovery_names(&names));

    let legacy = nms_save::parse_save(source.to_string().as_bytes()).unwrap();
    let legacy_systems = nms_graph::extract::extract_systems(&legacy);
    assert_eq!(legacy_systems.len(), 1);
    assert_eq!(legacy_systems.values().next().unwrap().name, None);
    let mut legacy_model = GalaxyModel::from_save(&legacy);
    assert_eq!(legacy_model.systems.values().next().unwrap().name, None);
    let legacy_id = SystemId::from_address(&GalacticAddress::new(100, 0, 0, 0x700, 0, 0));
    assert!(legacy_model.systems.contains_key(&legacy_id));
    let mut legacy_names = DiscoveryNames::default();
    legacy_names
        .system_names
        .insert(legacy_id, "opt-in name".to_owned());
    assert!(legacy_model.apply_discovery_names(&legacy_names));
    assert_eq!(
        legacy_model
            .systems
            .values()
            .next()
            .unwrap()
            .name
            .as_deref(),
        Some("opt-in name")
    );
    assert!(SaveDelta::empty().is_empty());
    let delta = SaveDelta::empty();
    assert_eq!(delta.change_count(), 0);
    let player_state = PlayerState::new(
        GalacticAddress::new(100, 0, 0, 7, 0, 0),
        0,
        None,
        None,
        1,
        2,
        3,
    );
    let update = SaveUpdate {
        delta,
        player_state: player_state.clone(),
        save_version: 56,
    };
    assert_eq!(update.player_state, player_state);
    assert_eq!(update.save_version, 56);
    assert!(update.delta.is_empty());
    assert_eq!(
        source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][0]["DM"]["CN"],
        "opt-in name"
    );

    let (_, empty_checked) = checked_save(vec![
        record("0x00070000000064", "SolarSystem", Some(json!({"CN": ""}))),
        record("0x10070000000064", "Planet", Some(json!({"CN": ""}))),
    ]);
    let empty_model = GalaxyModel::from_validated_save(&empty_checked);
    let empty_system_id = id(100, 0, 0, 7, 0);
    assert_eq!(empty_model.system(&empty_system_id).unwrap().name, None);
    assert_eq!(
        empty_model.planets.get(&(empty_system_id, 1)).unwrap().name,
        None
    );
}
