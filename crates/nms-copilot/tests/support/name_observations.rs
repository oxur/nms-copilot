//! Independent literal fixtures and complete model/cache oracles for checked
//! discovery-name observation tests.

use nms_cache::CacheData;
use nms_cache::data::CachedSystem;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System};
use nms_graph::GalaxyModel;
use serde_json::{Value, json};

pub fn a0() -> GalacticAddress {
    GalacticAddress::new(100, 0, 0, 7, 0, 0)
}
pub fn a0_p1() -> GalacticAddress {
    GalacticAddress::new(100, 0, 0, 7, 1, 0)
}
pub fn a9() -> GalacticAddress {
    GalacticAddress::new(100, 0, 0, 7, 0, 9)
}
pub fn a9_p1() -> GalacticAddress {
    GalacticAddress::new(100, 0, 0, 7, 1, 9)
}
pub fn b0() -> GalacticAddress {
    GalacticAddress::new(101, 0, 0, 8, 0, 0)
}
pub fn b0_p2() -> GalacticAddress {
    GalacticAddress::new(101, 0, 0, 8, 2, 0)
}
pub fn c0() -> GalacticAddress {
    GalacticAddress::new(102, 0, 0, 9, 0, 0)
}
pub fn c0_p3() -> GalacticAddress {
    GalacticAddress::new(102, 0, 0, 9, 3, 0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourcePhase {
    Initial,
    Overwrite,
    Promote,
    Rename,
    Clear,
    Omit,
    Null,
    NonString,
    NonObject,
    NonObjectDm,
    Recover,
    CounterOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelStage {
    Initial,
    Overwrite,
    Promote,
    Rename,
    Clear,
    Ignore,
    Recover,
}

fn discovery_record(address: &str, kind: &str, vp: Value, name: Option<Value>) -> Value {
    let mut row = json!({
        "DD": {"UA": address, "DT": kind, "VP": vp},
        "DM": {},
        "OWS": {"LID": "", "UID": "owner", "USN": "Explorer", "PTK": "ST", "TS": 1700000000},
        "FL": {"U": 1}
    });
    if let Some(name) = name {
        row["DM"]["CN"] = name;
    }
    row
}

fn source_name_values(
    phase: SourcePhase,
) -> (Option<Value>, Option<Value>, Option<Value>, Option<Value>) {
    let b0_system = (phase != SourcePhase::Initial).then(|| json!("Kepler, \"α\"\nNorth"));
    let b0_planet = (phase != SourcePhase::Initial).then(|| json!(" Lush, \"β\"\nMoon "));
    let (a0_system, a0_planet) = match phase {
        SourcePhase::Initial | SourcePhase::Overwrite | SourcePhase::CounterOnly => (None, None),
        SourcePhase::Promote => (Some(json!("Current")), Some(json!("Current Moon"))),
        SourcePhase::Rename => (Some(json!("Current Renamed")), Some(json!("Renamed Moon"))),
        SourcePhase::Clear => (Some(json!("")), Some(json!(""))),
        SourcePhase::Omit => (None, None),
        SourcePhase::Null => (Some(Value::Null), Some(Value::Null)),
        SourcePhase::NonString => (Some(json!(7)), Some(json!(false))),
        SourcePhase::NonObject => (Some(json!({"invalid": "name"})), Some(json!(["invalid"]))),
        SourcePhase::NonObjectDm => (None, None),
        SourcePhase::Recover => (
            Some(json!("Current Recovered")),
            Some(json!("Recovered Moon")),
        ),
    };
    (b0_system, b0_planet, a0_system, a0_planet)
}

/// Construct a fixed-order save with independent literal 56-bit address text.
pub fn source_bytes(phase: SourcePhase) -> Vec<u8> {
    let (b0_system, b0_planet, a0_system, a0_planet) = source_name_values(phase);
    let mut rows = vec![
        discovery_record("0x00080000000065", "SolarSystem", json!([]), b0_system),
        discovery_record("0x20080000000065", "Planet", json!(["0xAC", 0]), b0_planet),
        discovery_record("0x00070900000064", "SolarSystem", json!([]), None),
        discovery_record("0x10070900000064", "Planet", json!(["0xAD", 4]), None),
    ];
    if !matches!(
        phase,
        SourcePhase::Initial | SourcePhase::Overwrite | SourcePhase::CounterOnly
    ) {
        rows.push(discovery_record(
            "0x00070000000064",
            "SolarSystem",
            json!([]),
            a0_system,
        ));
        rows.push(discovery_record(
            "0x10070000000064",
            "Planet",
            json!(["0xAB", 65_537]),
            a0_planet,
        ));
    }
    if phase == SourcePhase::NonObjectDm {
        let a0_system = rows
            .iter_mut()
            .find(|row| {
                row.pointer("/DD/DT").and_then(Value::as_str) == Some("SolarSystem")
                    && row.pointer("/DD/UA").and_then(Value::as_str) == Some("0x00070000000064")
            })
            .expect("literal A0 system row");
        a0_system["DM"] = json!(["discovery metadata is not an object"]);
        let a0_planet = rows
            .iter_mut()
            .find(|row| {
                row.pointer("/DD/DT").and_then(Value::as_str) == Some("Planet")
                    && row.pointer("/DD/UA").and_then(Value::as_str) == Some("0x10070000000064")
            })
            .expect("literal A0 planet row");
        a0_planet["DM"] = json!(17);
    }
    let mut save = base_source();
    save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!(rows);
    if phase == SourcePhase::CounterOnly {
        save["BaseContext"]["PlayerStateData"]["Units"] = json!(70);
    }
    serde_json::to_vec(&save).expect("serialize literal source fixture")
}

/// Produce a source with one otherwise-valid modeled address outside 56 bits.
pub fn invalid_address_source() -> Vec<u8> {
    let mut source: Value = serde_json::from_slice(&source_bytes(SourcePhase::Recover))
        .expect("parse fixed recovery fixture");
    let records = source
        .pointer_mut("/DiscoveryManagerData/DiscoveryData-v1/Store/Record")
        .and_then(Value::as_array_mut)
        .expect("literal source record list");
    let a0 = records
        .iter_mut()
        .find(|record| {
            record.pointer("/DD/DT").and_then(Value::as_str) == Some("SolarSystem")
                && record.pointer("/DD/UA").and_then(Value::as_str) == Some("0x00070000000064")
        })
        .expect("literal A0 system row");
    a0["DD"]["UA"] = json!("0x100000000000000");
    serde_json::to_vec(&source).expect("serialize invalid-address fixture")
}

/// Produce distinct rejected candidates while preserving the same invalid
/// 57-bit discovery address and an attempted name change.
pub fn invalid_address_attempt(attempt: u8) -> Vec<u8> {
    let mut source: Value =
        serde_json::from_slice(&invalid_address_source()).expect("parse invalid-address fixture");
    let records = source
        .pointer_mut("/DiscoveryManagerData/DiscoveryData-v1/Store/Record")
        .and_then(Value::as_array_mut)
        .expect("literal source record list");
    let row = records
        .iter_mut()
        .find(|record| {
            record.pointer("/DD/DT").and_then(Value::as_str) == Some("SolarSystem")
                && record.pointer("/DD/UA").and_then(Value::as_str) == Some("0x100000000000000")
        })
        .expect("invalid A0 system row");
    row["DM"]["CN"] = json!(format!("Rejected name attempt {attempt}"));
    serde_json::to_vec(&source).expect("serialize distinct rejected candidate")
}

pub fn base_source() -> Value {
    json!({
        "Version": 4720,
        "Platform": "Mac|Final",
        "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "NameObservation", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {
                    "VoxelX": 100, "VoxelY": 0, "VoxelZ": 0,
                    "SolarSystemIndex": 7, "PlanetIndex": 1
                }},
                "PreviousUniverseAddress": {"RealityIndex": 9, "GalacticAddress": {
                    "VoxelX": 99, "VoxelY": 3, "VoxelZ": 4,
                    "SolarSystemIndex": 6, "PlanetIndex": 2
                }},
                "Units": 7, "Nanites": 8, "Specials": 9,
                "PersistentPlayerBases": [{
                    "BaseVersion": 8, "GalacticAddress": "0x10070000000064",
                    "Position": [1.0, 2.0, 3.0], "Forward": [1.0, 0.0, 0.0],
                    "LastUpdateTimestamp": 1700000000, "Objects": [], "RID": "",
                    "Owner": {"LID": "", "UID": "AnchorUID", "USN": "Anchor", "PTK": "ST", "TS": 0},
                    "Name": "Anchor", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
                    "LastEditedById": "", "LastEditedByUsername": ""
                }]
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {
            "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {
                "VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0
            }},
            "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []
        }},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
    })
}

fn system(address: GalacticAddress, name: Option<&str>, planets: Vec<Planet>) -> System {
    let timestamp = Some(
        "2023-11-14T22:13:20Z"
            .parse()
            .expect("fixed RFC3339 owner timestamp"),
    );
    System::new(
        address,
        name.map(str::to_owned),
        Some("Explorer".to_owned()),
        timestamp,
        planets,
    )
}

fn planet(index: u8, biome: Biome, infested: bool, name: Option<&str>, seed: u64) -> Planet {
    Planet::new(
        index,
        Some(biome),
        None,
        infested,
        name.map(str::to_owned),
        Some(seed),
    )
}

pub fn player_state(units: u64) -> PlayerState {
    PlayerState::new(
        a0_p1(),
        0,
        Some(GalacticAddress::new(99, 3, 4, 6, 2, 9)),
        None,
        units,
        8,
        9,
    )
}

pub fn expected_model(stage: ModelStage) -> GalaxyModel {
    expected_model_with_units(stage, 7)
}

pub fn expected_model_with_units(stage: ModelStage, units: u64) -> GalaxyModel {
    let (b0_name, b0_planet_name) = if stage == ModelStage::Initial {
        ("Community Rest", "Community Moon")
    } else {
        ("Kepler, \"α\"\nNorth", " Lush, \"β\"\nMoon ")
    };
    let (a0_name, a0_planet_name) = match stage {
        ModelStage::Initial | ModelStage::Overwrite => (None, None),
        ModelStage::Promote => (Some("Current"), Some("Current Moon")),
        ModelStage::Rename => (Some("Current Renamed"), Some("Renamed Moon")),
        ModelStage::Clear | ModelStage::Ignore => (None, None),
        ModelStage::Recover => (Some("Current Recovered"), Some("Recovered Moon")),
    };
    let mut model = GalaxyModel::new();
    if stage == ModelStage::Initial || stage == ModelStage::Overwrite {
        model.insert_player_placeholder(a0());
    } else {
        model.insert_system(system(
            a0(),
            a0_name,
            vec![planet(1, Biome::Toxic, true, a0_planet_name, 0xAB)],
        ));
    }
    model.insert_system(system(
        b0(),
        Some(b0_name),
        vec![planet(2, Biome::Lush, false, Some(b0_planet_name), 0xAC)],
    ));
    model.insert_system(system(
        a9(),
        Some("Community Nine"),
        vec![planet(1, Biome::Frozen, false, Some("Nine Moon"), 0xAD)],
    ));
    model.insert_system(system(
        c0(),
        Some("Absent Import"),
        vec![planet(3, Biome::Barren, false, Some("Absent Moon"), 0xAE)],
    ));
    model.insert_base(PlayerBase::new(
        "Anchor".into(),
        BaseType::HomePlanetBase,
        a0_p1(),
        [1.0, 2.0, 3.0],
        Some("AnchorUID".into()),
    ));
    model.player_state = Some(player_state(units));
    model.set_active_galaxy(9);
    model.build_edges(nms_graph::EdgeStrategy::default());
    model
}

/// Build a complete expected cache value from independent literal rows.
pub fn expected_cache(stage: ModelStage, units: u64) -> CacheData {
    let timestamp_secs = Some(1_700_000_000);
    let (a0_name, a0_planet_name, a0_placeholder) = match stage {
        ModelStage::Initial | ModelStage::Overwrite => (None, None, true),
        ModelStage::Promote => (Some("Current"), Some("Current Moon"), false),
        ModelStage::Rename => (Some("Current Renamed"), Some("Renamed Moon"), false),
        ModelStage::Clear | ModelStage::Ignore => (None, None, false),
        ModelStage::Recover => (Some("Current Recovered"), Some("Recovered Moon"), false),
    };
    let b0_names = stage != ModelStage::Initial;
    let systems = vec![
        CachedSystem {
            address: a0(),
            name: a0_name.map(str::to_owned),
            discoverer: if a0_placeholder {
                None
            } else {
                Some("Explorer".into())
            },
            timestamp_secs: if a0_placeholder { None } else { timestamp_secs },
            planets: if a0_placeholder {
                Vec::new()
            } else {
                vec![planet(1, Biome::Toxic, true, a0_planet_name, 0xAB)]
            },
            is_player_placeholder: a0_placeholder,
        },
        CachedSystem {
            address: b0(),
            name: Some(
                if b0_names {
                    "Kepler, \"α\"\nNorth"
                } else {
                    "Community Rest"
                }
                .into(),
            ),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(
                2,
                Biome::Lush,
                false,
                Some(if b0_names {
                    " Lush, \"β\"\nMoon "
                } else {
                    "Community Moon"
                }),
                0xAC,
            )],
            is_player_placeholder: false,
        },
        CachedSystem {
            address: a9(),
            name: Some("Community Nine".into()),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(1, Biome::Frozen, false, Some("Nine Moon"), 0xAD)],
            is_player_placeholder: false,
        },
        CachedSystem {
            address: c0(),
            name: Some("Absent Import".into()),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(3, Biome::Barren, false, Some("Absent Moon"), 0xAE)],
            is_player_placeholder: false,
        },
    ];
    CacheData {
        systems,
        bases: vec![PlayerBase::new(
            "Anchor".into(),
            BaseType::HomePlanetBase,
            a0_p1(),
            [1.0, 2.0, 3.0],
            Some("AnchorUID".into()),
        )],
        player_state: Some(player_state(units)),
        active_galaxy: 9,
        save_version: 4720,
        cached_at: 0,
    }
}

/// Literal cold-source cache/model expected after the authentic 3/4 rebuild.
pub fn expected_predecessor_cache() -> CacheData {
    let mut cache = expected_cache(ModelStage::Recover, 7);
    cache.systems.retain(|system| system.address != c0());
    let nine = cache
        .systems
        .iter_mut()
        .find(|system| system.address == a9())
        .expect("literal A9 cached system");
    nine.name = None;
    nine.planets[0].name = None;
    cache.active_galaxy = 0;
    cache
}

pub fn expected_predecessor_model() -> GalaxyModel {
    let mut model = GalaxyModel::new();
    model.insert_system(system(
        a0(),
        Some("Current Recovered"),
        vec![planet(1, Biome::Toxic, true, Some("Recovered Moon"), 0xAB)],
    ));
    model.insert_system(system(
        b0(),
        Some("Kepler, \"α\"\nNorth"),
        vec![planet(
            2,
            Biome::Lush,
            false,
            Some(" Lush, \"β\"\nMoon "),
            0xAC,
        )],
    ));
    model.insert_system(system(
        a9(),
        None,
        vec![planet(1, Biome::Frozen, false, None, 0xAD)],
    ));
    model.insert_base(PlayerBase::new(
        "Anchor".into(),
        BaseType::HomePlanetBase,
        a0_p1(),
        [1.0, 2.0, 3.0],
        Some("AnchorUID".into()),
    ));
    model.player_state = Some(player_state(7));
    model.set_active_galaxy(0);
    model.build_edges(nms_graph::EdgeStrategy::default());
    model
}

pub fn warm_seed_cache() -> CacheData {
    let timestamp_secs = Some(1_700_000_000);
    let systems = vec![
        CachedSystem {
            address: a0(),
            name: None,
            discoverer: None,
            timestamp_secs: None,
            planets: Vec::new(),
            is_player_placeholder: true,
        },
        CachedSystem {
            address: b0(),
            name: Some("Community Rest".into()),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(2, Biome::Lush, false, Some("Community Moon"), 0xAC)],
            is_player_placeholder: false,
        },
        CachedSystem {
            address: a9(),
            name: Some("Community Nine".into()),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(1, Biome::Frozen, false, Some("Nine Moon"), 0xAD)],
            is_player_placeholder: false,
        },
        CachedSystem {
            address: c0(),
            name: Some("Absent Import".into()),
            discoverer: Some("Explorer".into()),
            timestamp_secs,
            planets: vec![planet(3, Biome::Barren, false, Some("Absent Moon"), 0xAE)],
            is_player_placeholder: false,
        },
    ];
    CacheData {
        systems,
        bases: vec![PlayerBase::new(
            "Anchor".into(),
            BaseType::HomePlanetBase,
            a0_p1(),
            [1.0, 2.0, 3.0],
            Some("AnchorUID".into()),
        )],
        player_state: Some(player_state(7)),
        active_galaxy: 9,
        save_version: 4720,
        cached_at: 0,
    }
}

/// `(systems, planets, named systems, named planets)`.
pub fn expected_counts(stage: ModelStage) -> (usize, usize, usize, usize) {
    match stage {
        ModelStage::Initial => (4, 3, 3, 3),
        ModelStage::Overwrite => (4, 3, 3, 3),
        ModelStage::Promote | ModelStage::Rename | ModelStage::Recover => (4, 4, 4, 4),
        ModelStage::Clear | ModelStage::Ignore => (4, 4, 3, 3),
    }
}

pub fn expected_location(stage: ModelStage, predecessor: bool) -> Value {
    let system = if predecessor {
        "Current Recovered"
    } else {
        match stage {
            ModelStage::Initial | ModelStage::Overwrite => "(unknown)",
            ModelStage::Promote => "Current",
            ModelStage::Rename => "Current Renamed",
            ModelStage::Clear | ModelStage::Ignore => "-",
            ModelStage::Recover => "Current Recovered",
        }
    };
    let planets = if predecessor {
        1
    } else {
        match stage {
            ModelStage::Initial | ModelStage::Overwrite => 0,
            ModelStage::Promote
            | ModelStage::Rename
            | ModelStage::Clear
            | ModelStage::Ignore
            | ModelStage::Recover => 1,
        }
    };
    let portal_hex = "100700000064";
    json!({
        "system": system,
        "planets_in_system": planets,
        "galaxy": "Euclid",
        "voxel_x": 100,
        "voxel_y": 0,
        "voxel_z": 0,
        "solar_system_index": 7,
        "portal_glyphs_hex": portal_hex,
        "portal_glyphs_emoji": nms_query::display::hex_to_emoji(portal_hex),
        "signal_booster": "0863:007F:07FF:0007",
    })
}

pub fn expected_stats(stage: ModelStage, predecessor: bool) -> Value {
    let (systems, planets, named_systems, named_planets, infested, biomes) = if predecessor {
        (3, 3, 2, 2, 1, vec!["Frozen", "Lush", "Toxic"])
    } else {
        let counts = expected_counts(stage);
        let infested = usize::from(matches!(
            stage,
            ModelStage::Promote
                | ModelStage::Rename
                | ModelStage::Clear
                | ModelStage::Ignore
                | ModelStage::Recover
        ));
        let mut biomes = vec!["Barren", "Frozen", "Lush"];
        if infested == 1 {
            biomes.push("Toxic");
        }
        (counts.0, counts.1, counts.2, counts.3, infested, biomes)
    };
    json!({
        "systems": systems,
        "planets": planets,
        "bases": 1,
        "named_systems": named_systems,
        "named_planets": named_planets,
        "infested_planets": infested,
        "biome_distribution": biomes.into_iter().map(|biome| json!({"biome": biome, "count": 1})).collect::<Vec<_>>(),
        "unknown_biome_count": 0,
    })
}

/// Normalize only the unordered biome-distribution row order.
pub fn normalize_stats_biomes(mut stats: Value) -> Value {
    if let Some(rows) = stats
        .get_mut("biome_distribution")
        .and_then(Value::as_array_mut)
    {
        rows.sort_by(|left, right| {
            left.get("biome")
                .and_then(Value::as_str)
                .cmp(&right.get("biome").and_then(Value::as_str))
        });
    }
    stats
}

pub fn address_fields(address: GalacticAddress) -> Value {
    json!({
        "reality_index": address.reality_index,
        "voxel_x": address.voxel_x(),
        "voxel_y": address.voxel_y(),
        "voxel_z": address.voxel_z(),
        "solar_system_index": address.solar_system_index(),
        "planet_index": address.planet_index(),
    })
}
