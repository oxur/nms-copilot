#[path = "support/live_mcp.rs"]
pub mod live_mcp;

use nms_cache::data::CachedSystem;
use nms_cache::serialize::{deserialize, extract_cache_data, rebuild_model, serialize};
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::delta::SaveUpdate;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{DiscoverySource, Planet, System};
use nms_graph::{GalaxyModel, SystemId};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::show::{ShowQuery, ShowResult, execute_show};
use nms_save::parse_validated_save;
use nms_watch::{SaveSnapshot, compute_delta};
use serde_json::{Value, json};

const SOURCE: &str = " NMSCE α ";
const CSV: &str = "System Name,Galaxy,Portal Glyphs,Biome,Platform\nPersonal replacement,0,000100000000,Toxic,PC\n\"Catalog, \"\"Nine\"\"\",9,000100000000,Toxic,PC\nCatalog-B,0,000200000001,Frozen\nCatalog-C,0,000300000002,Barren,PC,ignored-extra\nLater,9,000100000000,Lush,PC\nBad,0,ZZZZZZZZZZZZ,Lush,PC\n";

fn address(galaxy: u8, x: i16, ssi: u16, planet: u8) -> GalacticAddress {
    GalacticAddress::new(x, 0, 0, ssi, planet, galaxy)
}

fn row(address: &str, kind: &str, vp: Value, name: Option<&str>) -> Value {
    let mut value = json!({
        "DD":{"UA":address,"DT":kind,"VP":vp},
        "DM":{},
        "OWS":{"LID":"","UID":"SourceOwnerUID","USN":"SourceOwner","PTK":"ST","TS":1700000000},
        "FL":{"U":1}
    });
    if let Some(name) = name {
        value["DM"]["CN"] = json!(name);
    }
    value
}

fn base_save() -> Value {
    let mut save: Value =
        serde_json::from_slice(include_bytes!("../../../data/test/save56_minimal.json"))
            .expect("parse pinned save56 fixture");
    let player = save["BaseContext"]["PlayerStateData"]
        .as_object_mut()
        .expect("literal player object");
    player.insert(
        "UniverseAddress".into(),
        json!({"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":0}}),
    );
    player.remove("PreviousUniverseAddress");
    player.insert("Units".into(), json!(7));
    player.insert("Nanites".into(), json!(8));
    player.insert("Specials".into(), json!(9));
    let bases = player["PersistentPlayerBases"]
        .as_array_mut()
        .expect("literal base list");
    bases.truncate(1);
    bases[0]["GalacticAddress"] = json!("0x00010000000000");
    bases[0]["Position"] = json!([1.0, 2.0, 3.0]);
    bases[0]["Name"] = json!("Anchor");
    bases[0]["BaseType"] = json!({"PersistentBaseTypes":"HomePlanetBase"});
    bases[0]["Owner"]["UID"] = json!("SourceOwner");
    save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([
        row(
            "0x00010000000000",
            "SolarSystem",
            json!([]),
            Some("Personal")
        ),
        row(
            "0x10010000000000",
            "Planet",
            json!(["0xAA", 0]),
            Some("Personal Moon")
        ),
    ]);
    assert_eq!(
        save["BaseContext"]["PlayerStateData"]["UniverseAddress"],
        json!({"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":0}})
    );
    let player_fields = save["BaseContext"]["PlayerStateData"].as_object().unwrap();
    assert!(!player_fields.contains_key("PreviousUniverseAddress"));
    assert!(!player_fields.contains_key("FreighterAddress"));
    assert_eq!(save["BaseContext"]["PlayerStateData"]["Units"], 7);
    assert_eq!(save["BaseContext"]["PlayerStateData"]["Nanites"], 8);
    assert_eq!(save["BaseContext"]["PlayerStateData"]["Specials"], 9);
    let base = &save["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0];
    assert_eq!(base["GalacticAddress"], "0x00010000000000");
    assert_eq!(base["Position"], json!([1.0, 2.0, 3.0]));
    assert_eq!(base["Name"], "Anchor");
    assert_eq!(
        base["BaseType"],
        json!({"PersistentBaseTypes":"HomePlanetBase"})
    );
    assert_eq!(base["Owner"]["UID"], "SourceOwner");
    let records = save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array()
        .unwrap();
    for (record, ua, kind, vp, name) in [
        (
            &records[0],
            "0x00010000000000",
            "SolarSystem",
            json!([]),
            "Personal",
        ),
        (
            &records[1],
            "0x10010000000000",
            "Planet",
            json!(["0xAA", 0]),
            "Personal Moon",
        ),
    ] {
        assert_eq!(record["DD"]["UA"], ua);
        assert_eq!(record["DD"]["DT"], kind);
        assert_eq!(record["DD"]["VP"], vp);
        assert!(!record["DD"].as_object().unwrap().contains_key("DM"));
        assert_eq!(record["DM"], json!({"CN": name}));
        assert_eq!(
            record["OWS"],
            json!({"LID":"","UID":"SourceOwnerUID","USN":"SourceOwner","PTK":"ST","TS":1700000000})
        );
        assert_eq!(record["FL"], json!({"U":1}));
    }
    save
}

fn csv_rows() -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().expect("temporary catalog");
    file.write_all(CSV.as_bytes())
        .expect("write literal catalog");
    file.flush().expect("flush literal catalog");
    file
}

fn expected_source() -> DiscoverySource {
    DiscoverySource::Community {
        source: SOURCE.to_owned(),
    }
}

fn system(
    addr: GalacticAddress,
    name: Option<&str>,
    source: DiscoverySource,
    discoverer: Option<&str>,
    timestamp: bool,
    planets: Vec<Planet>,
) -> System {
    let mut system = System::new(
        addr,
        name.map(str::to_owned),
        discoverer.map(str::to_owned),
        timestamp.then(|| "2023-11-14T22:13:20Z".parse().expect("literal owner time")),
        planets,
    );
    system.source = source;
    system
}

fn expected_planet(index: u8, biome: Biome, name: Option<&str>, seed: Option<u64>) -> Planet {
    Planet::new(
        index,
        Some(biome),
        None,
        false,
        name.map(str::to_owned),
        seed,
    )
}

fn player_state() -> PlayerState {
    PlayerState::new(address(0, 0, 1, 0), 0, None, None, 7, 8, 9)
}

fn expected_model(b_name: Option<&str>, b_planet_name: Option<&str>) -> GalaxyModel {
    let mut model = GalaxyModel::new();
    model.insert_system(system(
        address(0, 0, 1, 0),
        Some("Personal"),
        DiscoverySource::Personal,
        Some("SourceOwner"),
        true,
        vec![expected_planet(
            1,
            Biome::Lush,
            Some("Personal Moon"),
            Some(0xAA),
        )],
    ));
    model.insert_system(system(
        address(9, 0, 1, 0),
        Some("Catalog, \"Nine\""),
        expected_source(),
        None,
        false,
        vec![expected_planet(0, Biome::Toxic, None, None)],
    ));
    model.insert_system(system(
        address(0, 1, 2, 0),
        b_name,
        expected_source(),
        None,
        false,
        vec![expected_planet(0, Biome::Frozen, b_planet_name, None)],
    ));
    model.insert_system(system(
        address(0, 2, 3, 0),
        Some("Catalog-C"),
        expected_source(),
        None,
        false,
        vec![expected_planet(0, Biome::Barren, None, None)],
    ));
    model.insert_base(PlayerBase::new(
        "Anchor".into(),
        BaseType::HomePlanetBase,
        address(0, 0, 1, 0),
        [1.0, 2.0, 3.0],
        Some("SourceOwner".into()),
    ));
    model.player_state = Some(player_state());
    model.set_active_galaxy(9);
    model
}

fn expected_cache(b_name: Option<&str>, b_planet_name: Option<&str>) -> nms_cache::CacheData {
    nms_cache::CacheData {
        systems: vec![
            CachedSystem {
                address: address(0, 0, 1, 0),
                source: DiscoverySource::Personal,
                name: Some("Personal".into()),
                discoverer: Some("SourceOwner".into()),
                timestamp_secs: Some(1_700_000_000),
                planets: vec![expected_planet(
                    1,
                    Biome::Lush,
                    Some("Personal Moon"),
                    Some(0xAA),
                )],
                is_player_placeholder: false,
            },
            CachedSystem {
                address: address(9, 0, 1, 0),
                source: expected_source(),
                name: Some("Catalog, \"Nine\"".into()),
                discoverer: None,
                timestamp_secs: None,
                planets: vec![expected_planet(0, Biome::Toxic, None, None)],
                is_player_placeholder: false,
            },
            CachedSystem {
                address: address(0, 1, 2, 0),
                source: expected_source(),
                name: b_name.map(str::to_owned),
                discoverer: None,
                timestamp_secs: None,
                planets: vec![expected_planet(0, Biome::Frozen, b_planet_name, None)],
                is_player_placeholder: false,
            },
            CachedSystem {
                address: address(0, 2, 3, 0),
                source: expected_source(),
                name: Some("Catalog-C".into()),
                discoverer: None,
                timestamp_secs: None,
                planets: vec![expected_planet(0, Biome::Barren, None, None)],
                is_player_placeholder: false,
            },
        ],
        bases: vec![PlayerBase::new(
            "Anchor".into(),
            BaseType::HomePlanetBase,
            address(0, 0, 1, 0),
            [1.0, 2.0, 3.0],
            Some("SourceOwner".into()),
        )],
        player_state: Some(player_state()),
        active_galaxy: 9,
        save_version: 4720,
        cached_at: 0,
    }
}

fn apply_checked_names(model: &mut GalaxyModel, bytes: &[u8]) -> bool {
    let validated = parse_validated_save(bytes).expect("validate complete source snapshot");
    let names = nms_graph::extract::extract_discovery_names_from_validated(&validated);
    model.apply_discovery_names(&names)
}

fn changed_b_save(name: Option<&str>, planet_name: Option<&str>) -> Vec<u8> {
    let mut save = base_save();
    let mut rows = save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array()
        .unwrap()
        .clone();
    rows.push(row("0x00020000000001", "SolarSystem", json!([]), name));
    rows.push(row(
        "0x00020000000001",
        "Planet",
        json!(["0xAB", 4]),
        planet_name,
    ));
    for (record, kind, vp, name) in [
        (&rows[2], "SolarSystem", json!([]), name),
        (&rows[3], "Planet", json!(["0xAB", 4]), planet_name),
    ] {
        assert_eq!(record["DD"]["UA"], "0x00020000000001");
        assert_eq!(record["DD"]["DT"], kind);
        assert_eq!(record["DD"]["VP"], vp);
        assert!(!record["DD"].as_object().unwrap().contains_key("DM"));
        assert_eq!(
            record["DM"],
            name.map(|name| json!({"CN":name}))
                .unwrap_or_else(|| json!({}))
        );
        assert_eq!(
            record["OWS"],
            json!({"LID":"","UID":"SourceOwnerUID","USN":"SourceOwner","PTK":"ST","TS":1700000000})
        );
        assert_eq!(record["FL"], json!({"U":1}));
    }
    save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!(rows);
    serde_json::to_vec(&save).expect("serialize changed validated source")
}

fn update_from_checked_source(model: &mut GalaxyModel, before_bytes: &[u8], after_bytes: &[u8]) {
    let before_save = parse_validated_save(before_bytes).expect("validate before snapshot");
    let after_save = parse_validated_save(after_bytes).expect("validate after snapshot");
    let before = SaveSnapshot::from_validated_save(&before_save);
    let after = SaveSnapshot::from_validated_save(&after_save);
    let delta = compute_delta(&before, &after);
    let update = SaveUpdate {
        delta,
        player_state: after_save.player_state().clone(),
        save_version: after_save.root().version,
    };
    model.apply_save_update(&update);
    let names = nms_graph::extract::extract_discovery_names_from_validated(&after_save);
    model.apply_discovery_names(&names);
}

#[test]
fn test_import_csv_checked_source_cache_query_and_name_lifecycle_preserves_full_state() {
    use nms_graph::edges::EdgeStrategy;

    let initial_save = base_save();
    let initial_bytes = serde_json::to_vec(&initial_save).unwrap();
    let validated = parse_validated_save(&initial_bytes).expect("checked Personal fixture");
    let mut model = GalaxyModel::from_validated_save(&validated);
    let placeholder = address(0, 2, 3, 0);
    model.insert_player_placeholder(placeholder);
    model.set_active_galaxy(9);
    assert!(model.is_player_placeholder(&SystemId::from_address(&placeholder)));
    assert_eq!(
        model.graph.edge_count(),
        0,
        "fixture starts without derived edges"
    );

    let catalog = csv_rows();
    let stats = nms_graph::import::import_csv(&mut model, catalog.path(), SOURCE)
        .expect("import literal ragged community CSV");
    assert_eq!((stats.added, stats.duplicates, stats.skipped), (3, 2, 1));
    assert!(!model.is_player_placeholder(&SystemId::from_address(&placeholder)));
    let expected = expected_model(Some("Catalog-B"), None);
    live_mcp::assert_model_matches(&model, &expected).expect("complete post-import model");
    assert_eq!(
        model.graph.edge_count(),
        0,
        "import does not add graph edges"
    );
    let replay = nms_graph::import::import_csv(&mut model, catalog.path(), SOURCE)
        .expect("replay literal ragged community CSV");
    assert_eq!((replay.added, replay.duplicates, replay.skipped), (0, 5, 1));
    live_mcp::assert_model_matches(&model, &expected).expect("complete model after exact replay");

    let a0 = address(0, 0, 1, 0);
    let a9 = address(9, 0, 1, 0);
    for (
        from,
        biome,
        destination,
        destination_planet,
        expected_system,
        expected_planet,
        distance,
        portal,
    ) in [
        (
            a0,
            Biome::Lush,
            address(0, 0, 1, 1),
            1,
            "Personal",
            Some("Personal Moon"),
            0.0,
            "000100000000",
        ),
        (
            a0,
            Biome::Frozen,
            address(0, 1, 2, 0),
            0,
            "Catalog-B",
            None,
            400.0,
            "000200000001",
        ),
        (
            a0,
            Biome::Barren,
            address(0, 2, 3, 0),
            0,
            "Catalog-C",
            None,
            800.0,
            "000300000002",
        ),
        (
            a9,
            Biome::Toxic,
            address(9, 0, 1, 0),
            0,
            "Catalog, \"Nine\"",
            None,
            0.0,
            "000100000000",
        ),
    ] {
        let found = execute_find(
            &model,
            &FindQuery {
                from: ReferencePoint::Address(from),
                biome: Some(biome),
                ..Default::default()
            },
        )
        .expect("explicit galaxy find");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].distance_ly, distance);
        let expected_id = SystemId::from_address(&destination);
        assert_eq!(found[0].system, expected.systems[&expected_id]);
        assert_eq!(
            found[0].planet,
            expected.planets[&(expected_id, destination_planet)]
        );
        assert_eq!(found[0].system.name.as_deref(), Some(expected_system));
        assert_eq!(found[0].planet.name.as_deref(), expected_planet);
        assert_eq!(
            found[0].system.source,
            if expected_system == "Personal" {
                DiscoverySource::Personal
            } else {
                expected_source()
            }
        );
        assert_eq!(found[0].planet.biome, Some(biome));
        assert_eq!(found[0].portal_hex, portal);
    }
    model.set_active_galaxy(0);
    let opposed = execute_find(
        &model,
        &FindQuery {
            from: ReferencePoint::Address(a9),
            biome: Some(Biome::Toxic),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(opposed.len(), 1);
    let a9_id = SystemId::from_address(&a9);
    assert_eq!(opposed[0].system, expected.systems[&a9_id]);
    assert_eq!(opposed[0].planet, expected.planets[&(a9_id, 0)]);
    assert_eq!(opposed[0].distance_ly, 0.0);
    assert_eq!(opposed[0].portal_hex, "000100000000");
    let opposed_exclusion = execute_find(
        &model,
        &FindQuery {
            from: ReferencePoint::Address(a9),
            biome: Some(Biome::Lush),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        opposed_exclusion.is_empty(),
        "GG0 Personal Moon is excluded from the explicit GG9 Lush query"
    );
    assert_eq!(model.active_galaxy, 0);
    let mut expected_selected0 = expected_model(Some("Catalog-B"), None);
    expected_selected0.set_active_galaxy(0);
    live_mcp::assert_model_matches(&model, &expected_selected0)
        .expect("query preserves full atlas and intentional selected galaxy");
    model.set_active_galaxy(9);
    let foreign = execute_find(
        &model,
        &FindQuery {
            from: ReferencePoint::Address(a0),
            biome: Some(Biome::Toxic),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        foreign.is_empty(),
        "A9 is excluded from explicit GG0 search"
    );
    let ShowResult::System(shown) =
        execute_show(&model, &ShowQuery::System("Catalog, \"Nine\"".into())).unwrap()
    else {
        panic!("show returns imported system");
    };
    assert_eq!(shown.system, expected.systems[&a9_id]);
    assert_eq!(shown.system.source, expected_source());

    let before_edges = model.graph.edge_count();
    model.build_edges(EdgeStrategy::default());
    let mut expected_with_edges = expected_model(Some("Catalog-B"), None);
    expected_with_edges.build_edges(EdgeStrategy::default());
    live_mcp::assert_model_matches(&model, &expected_with_edges)
        .expect("whole default edge rebuild");
    assert_eq!(model.graph.edge_count() - before_edges, 3);
    let mut observed_edges = model
        .graph
        .edge_indices()
        .map(|edge| {
            let (left, right) = model.graph.edge_endpoints(edge).unwrap();
            let left = *model.graph.node_weight(left).unwrap();
            let right = *model.graph.node_weight(right).unwrap();
            let left = (left.reality_index(), left.system_address());
            let right = (right.reality_index(), right.system_address());
            let pair = if left <= right {
                (left, right)
            } else {
                (right, left)
            };
            (pair.0, pair.1, *model.graph.edge_weight(edge).unwrap())
        })
        .collect::<Vec<_>>();
    observed_edges.sort_by_key(|edge| (edge.0, edge.1));
    assert_eq!(
        observed_edges,
        vec![
            ((0, 0x000100000000), (0, 0x000200000001), 400.0),
            ((0, 0x000100000000), (0, 0x000300000002), 800.0),
            ((0, 0x000200000001), (0, 0x000300000002), 400.0),
        ]
    );

    let b0 = address(0, 1, 2, 0);
    let original_b = model.system(&SystemId::from_address(&b0)).unwrap().clone();
    model.insert_system(System::new(
        b0,
        Some("Personal duplicate must not replace".into()),
        None,
        None,
        vec![expected_planet(0, Biome::Toxic, Some("wrong"), Some(99))],
    ));
    assert_eq!(
        model.system(&SystemId::from_address(&b0)),
        Some(&original_b)
    );
    live_mcp::assert_model_matches(&model, &expected_with_edges)
        .expect("later genuine Personal duplicate preserves the full Community record");

    let renamed = changed_b_save(Some("Renamed B"), Some("Renamed Moon"));
    update_from_checked_source(&mut model, &initial_bytes, &renamed);
    let renamed_expected = expected_model(Some("Renamed B"), Some("Renamed Moon"));
    let mut renamed_expected = renamed_expected;
    renamed_expected.build_edges(EdgeStrategy::default());
    live_mcp::assert_model_matches(&model, &renamed_expected)
        .expect("checked parsed update and name extraction preserve source and full graph");

    let cleared = changed_b_save(Some(""), Some(""));
    assert!(apply_checked_names(&mut model, &cleared));
    let cleared_expected = expected_model(None, None);
    let mut cleared_expected = cleared_expected;
    cleared_expected.build_edges(EdgeStrategy::default());
    live_mcp::assert_model_matches(&model, &cleared_expected)
        .expect("empty CN clears names but keeps Community source");

    let omitted = changed_b_save(None, None);
    assert!(!apply_checked_names(&mut model, &omitted));
    live_mcp::assert_model_matches(&model, &cleared_expected)
        .expect("missing CN preserves absent names and Community source");

    let data = extract_cache_data(&model, 4720);
    assert!(data.cached_at > 0, "cache records a real cache timestamp");
    let extracted_cached_at = data.cached_at;
    let encoded = serialize(&data).expect("serialize full mixed-origin atlas");
    assert_eq!(&encoded[..16], b"NMSCACHE\x04\0\0\0\x06\0\0\0");
    let decoded = deserialize(&encoded).expect("checked current cache roundtrip");
    assert_eq!(decoded.cached_at, extracted_cached_at);
    let rebuilt = rebuild_model(&decoded);
    let expected_data = expected_cache(None, None);
    live_mcp::compare_cache_model(&decoded, &rebuilt, &expected_data, &cleared_expected)
        .expect("full cache tags, atlas, indices and graph survive archive rebuild");

    let a9_id = SystemId::from_address(&a9);
    assert_eq!(model.system(&a9_id).map(|system| system.address), Some(a9));
    let baseline = deserialize(&encoded).unwrap();
    let mut wrong_variant = deserialize(&encoded).unwrap();
    let a9_row = wrong_variant
        .systems
        .iter_mut()
        .find(|row| row.address == a9)
        .unwrap();
    assert_eq!(a9_row.source, expected_source());
    a9_row.source = DiscoverySource::Personal;
    let wrong_variant_model = rebuild_model(&wrong_variant);
    assert!(
        live_mcp::compare_cache_model(
            &wrong_variant,
            &wrong_variant_model,
            &expected_data,
            &cleared_expected
        )
        .is_err()
    );
    let baseline_a9 = baseline
        .systems
        .iter()
        .find(|row| row.address == a9)
        .unwrap();
    let wrong_a9 = wrong_variant
        .systems
        .iter()
        .find(|row| row.address == a9)
        .unwrap();
    assert_eq!(wrong_a9.address, baseline_a9.address);
    assert_eq!(wrong_a9.name, baseline_a9.name);
    assert_eq!(wrong_a9.discoverer, baseline_a9.discoverer);
    assert_eq!(wrong_a9.timestamp_secs, baseline_a9.timestamp_secs);
    assert_eq!(wrong_a9.planets, baseline_a9.planets);
    assert_eq!(
        wrong_a9.is_player_placeholder,
        baseline_a9.is_player_placeholder
    );
    wrong_variant
        .systems
        .iter_mut()
        .find(|row| row.address == a9)
        .unwrap()
        .source = baseline_a9.source.clone();
    live_mcp::compare_cache(&wrong_variant, &baseline)
        .expect("restoring only source restores every complete cache field");
    assert_eq!(wrong_variant.cached_at, baseline.cached_at);
    assert_eq!(a9_id, SystemId::from_address(&a9));

    let mut wrong_label = deserialize(&encoded).unwrap();
    let a9_row = wrong_label
        .systems
        .iter_mut()
        .find(|row| row.address == a9)
        .unwrap();
    if let DiscoverySource::Community { source } = &mut a9_row.source {
        source.replace_range(1..2, "X");
    } else {
        panic!("literal A9 origin is Community");
    }
    let wrong_label_model = rebuild_model(&wrong_label);
    assert!(
        live_mcp::compare_cache_model(
            &wrong_label,
            &wrong_label_model,
            &expected_data,
            &cleared_expected
        )
        .is_err()
    );
    let wrong_label_a9 = wrong_label
        .systems
        .iter()
        .find(|row| row.address == a9)
        .unwrap();
    assert_eq!(wrong_label_a9.address, baseline_a9.address);
    assert_eq!(wrong_label_a9.name, baseline_a9.name);
    assert_eq!(wrong_label_a9.discoverer, baseline_a9.discoverer);
    assert_eq!(wrong_label_a9.timestamp_secs, baseline_a9.timestamp_secs);
    assert_eq!(wrong_label_a9.planets, baseline_a9.planets);
    assert_eq!(
        wrong_label_a9.is_player_placeholder,
        baseline_a9.is_player_placeholder
    );
    assert_eq!(
        wrong_label_a9.source,
        DiscoverySource::Community {
            source: " XMSCE α ".into()
        }
    );
    wrong_label
        .systems
        .iter_mut()
        .find(|row| row.address == a9)
        .unwrap()
        .source = baseline_a9.source.clone();
    live_mcp::compare_cache(&wrong_label, &baseline)
        .expect("restoring only source restores every complete cache field");
    assert_eq!(wrong_label.cached_at, baseline.cached_at);
    assert_eq!(a9_id, SystemId::from_address(&a9));
}
