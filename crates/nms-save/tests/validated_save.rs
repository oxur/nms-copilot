use std::error::Error;

use nms_core::GalacticAddress;
use nms_save::address::AddressDecodeError;
use nms_save::{SaveError, parse_save, parse_validated_save, parse_validated_save_file};
use serde_json::{Value, json};

const MINIMAL: &[u8] = include_bytes!("../../../data/test/save56_minimal.json");

fn fixture_value() -> Value {
    serde_json::from_slice(MINIMAL).expect("valid fixture JSON")
}

fn active_value(context: &str) -> Value {
    let mut value = fixture_value();
    value["ActiveContext"] = json!(context);
    value
}

fn remove_member(value: &mut Value, path: &str) {
    let (parent, member) = path.rsplit_once('/').unwrap();
    value
        .pointer_mut(parent)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove(member)
        .unwrap();
}

fn assert_missing_at(value: &Value, expected_path: &str) {
    let bytes = serde_json::to_vec(value).unwrap();
    assert!(matches!(
        parse_validated_save(&bytes),
        Err(SaveError::Address(AddressDecodeError::Missing { path }))
            if path == expected_path
    ));
}

fn assert_invalid_type_at(value: &Value, expected_path: &str) {
    let bytes = serde_json::to_vec(value).unwrap();
    assert!(matches!(
        parse_validated_save(&bytes),
        Err(SaveError::Address(AddressDecodeError::InvalidType { path, .. }))
            if path == expected_path
    ));
}

fn packed_error(error: &SaveError, expected_path: &str) {
    assert!(matches!(
        error,
        SaveError::Address(AddressDecodeError::PackedWidth { path }) if path == expected_path
    ));
    assert_eq!(
        Error::source(error)
            .and_then(|source| source.downcast_ref::<AddressDecodeError>())
            .map(AddressDecodeError::path),
        Some(expected_path)
    );
}

#[test]
fn test_validated_save_pairs_original_discoveries_and_builds_complete_active_state() {
    let save = parse_validated_save(MINIMAL).unwrap();
    let rows = save
        .discoveries()
        .map(|(record, address)| (record.dd.dt.as_str(), address))
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        [
            (
                "SolarSystem",
                GalacticAddress::new(-1017, 3, -1352, 1280, 0, 0)
            ),
            ("Planet", GalacticAddress::new(-1017, 3, -1352, 1280, 1, 0)),
        ]
    );
    assert_eq!(
        save.player_state(),
        &nms_core::PlayerState::new(
            GalacticAddress::new(100, 50, -200, 42, 0, 0),
            0,
            Some(GalacticAddress::new(80, 30, -150, 38, 0, 0)),
            None,
            5_000_000,
            10_000,
            500,
        )
    );
    assert_eq!(
        save.bases(),
        &[nms_core::PlayerBase::new(
            "Homestead Alpha".into(),
            nms_core::BaseType::HomePlanetBase,
            GalacticAddress::new(-1017, 3, -1352, 1280, 0, 0),
            [100.0, 200.0, 300.0],
            Some("76561198025707979".into()),
        )]
    );
    assert_eq!(save.root().version, 4720);
}

#[test]
fn test_checked_parser_rejects_late_bad_discovery_before_later_context_addresses() {
    let mut value = fixture_value();
    value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][1]["DD"]["UA"] =
        json!("0x100000000000000");
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(2048);
    let error = parse_validated_save(value.to_string().as_bytes()).unwrap_err();
    packed_error(
        &error,
        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/1/DD/UA",
    );
}

#[test]
fn test_checked_parser_validates_base_context_before_expedition_context() {
    let mut value = fixture_value();
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(2048);
    value["ExpeditionContext"] = json!(null);

    assert!(matches!(
        parse_validated_save(value.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::OutOfRange {
            path,
            value: 2048,
            minimum: -2048,
            maximum: 2047,
        })) if path == "/BaseContext/PlayerStateData/UniverseAddress/GalacticAddress/VoxelX"
    ));

    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(100);
    assert!(matches!(
        parse_validated_save(value.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::InvalidType { path, .. }))
            if path == "/ExpeditionContext"
    ));
}

#[test]
fn test_checked_parser_requires_each_active_address_member_in_both_contexts() {
    for (context, selector) in [("BaseContext", "Main"), ("ExpeditionContext", "Expedition")] {
        let player = format!("/{context}/PlayerStateData");
        let current = format!("{player}/UniverseAddress");
        let galactic = format!("{current}/GalacticAddress");
        let mut required = vec![player.clone(), current.clone(), galactic.clone()];
        required.push(format!("{current}/RealityIndex"));
        for component in [
            "VoxelX",
            "VoxelY",
            "VoxelZ",
            "SolarSystemIndex",
            "PlanetIndex",
        ] {
            required.push(format!("{galactic}/{component}"));
        }

        for path in &required {
            let mut value = active_value(selector);
            remove_member(&mut value, path);
            assert_missing_at(&value, path);
        }
        for path in [&player, &current, &galactic] {
            for invalid in [Value::Null, json!(false)] {
                let mut value = active_value(selector);
                *value.pointer_mut(path).unwrap() = invalid;
                assert_invalid_type_at(&value, path);
            }
        }
        for path in required.iter().skip(3) {
            for invalid in [Value::Null, json!(1.5)] {
                let mut value = active_value(selector);
                *value.pointer_mut(path).unwrap() = invalid;
                assert_invalid_type_at(&value, path);
            }
        }
    }
}

#[test]
fn test_checked_parser_reports_every_expanded_component_boundary_before_narrowing() {
    let cases = [
        ("VoxelX", -2049, -2048, 2047),
        ("VoxelX", 2048, -2048, 2047),
        ("VoxelY", -129, -128, 127),
        ("VoxelY", 128, -128, 127),
        ("VoxelZ", -2049, -2048, 2047),
        ("VoxelZ", 2048, -2048, 2047),
        ("SolarSystemIndex", -1, 0, 4095),
        ("SolarSystemIndex", 4096, 0, 4095),
        ("PlanetIndex", -1, 0, 15),
        ("PlanetIndex", 16, 0, 15),
    ];
    let mut all_cases = cases.to_vec();
    all_cases.extend([("RealityIndex", -1, 0, 255), ("RealityIndex", 256, 0, 255)]);

    for (component, value_to_set, minimum, maximum) in all_cases {
        for (context, selector) in [("BaseContext", "Main"), ("ExpeditionContext", "Expedition")] {
            let component_path = if component == "RealityIndex" {
                format!("/{context}/PlayerStateData/UniverseAddress/RealityIndex")
            } else {
                format!("/{context}/PlayerStateData/UniverseAddress/GalacticAddress/{component}")
            };
            let mut value = active_value(selector);
            *value.pointer_mut(&component_path).unwrap() = json!(value_to_set);
            let error = parse_validated_save(&serde_json::to_vec(&value).unwrap()).unwrap_err();
            assert!(matches!(
                error,
                SaveError::Address(AddressDecodeError::OutOfRange {
                    path,
                    value,
                    minimum: actual_min,
                    maximum: actual_max,
                }) if path == component_path
                    && value == i128::from(value_to_set)
                    && actual_min == i128::from(minimum)
                    && actual_max == i128::from(maximum)
            ));
        }
    }
}

#[test]
fn test_checked_parser_keeps_active_selector_and_required_context_container_rules() {
    let mut missing_active = fixture_value();
    remove_member(&mut missing_active, "/ActiveContext");
    assert_missing_at(&missing_active, "/ActiveContext");
    for invalid in [Value::Null, json!(1)] {
        let mut value = fixture_value();
        value["ActiveContext"] = invalid;
        assert_invalid_type_at(&value, "/ActiveContext");
    }

    let mut other_selector = active_value("OtherValidContext");
    other_selector["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(101);
    let parsed = parse_validated_save(&serde_json::to_vec(&other_selector).unwrap()).unwrap();
    assert_eq!(parsed.player_state().current_address.voxel_x(), 101);

    for context in ["BaseContext", "ExpeditionContext"] {
        let path = format!("/{context}");
        let mut missing = fixture_value();
        remove_member(&mut missing, &path);
        assert_missing_at(&missing, &path);
        for invalid in [Value::Null, json!(false)] {
            let mut value = fixture_value();
            *value.pointer_mut(&path).unwrap() = invalid;
            assert_invalid_type_at(&value, &path);
        }
    }

    let mut missing_top_manager = fixture_value();
    remove_member(&mut missing_top_manager, "/DiscoveryManagerData");
    assert!(matches!(
        parse_validated_save(&serde_json::to_vec(&missing_top_manager).unwrap()),
        Err(SaveError::JsonParseError { .. })
    ));
}

#[test]
fn test_checked_parser_preserves_inactive_and_defaulted_container_presence_rules() {
    for (inactive, active_selector) in
        [("ExpeditionContext", "Main"), ("BaseContext", "Expedition")]
    {
        let player = format!("/{inactive}/PlayerStateData");
        let current = format!("{player}/UniverseAddress");
        let mut absent_player = active_value(active_selector);
        remove_member(&mut absent_player, &player);
        assert!(parse_validated_save(&serde_json::to_vec(&absent_player).unwrap()).is_ok());

        for path in [&player, &current] {
            for invalid_value in [Value::Null, json!(false)] {
                let mut invalid = active_value(active_selector);
                *invalid.pointer_mut(path).unwrap() = invalid_value;
                assert_invalid_type_at(&invalid, path);
            }
        }
        let mut absent_current = active_value(active_selector);
        remove_member(&mut absent_current, &current);
        assert!(parse_validated_save(&serde_json::to_vec(&absent_current).unwrap()).is_ok());
        let mut bad_inactive_current = active_value(active_selector);
        *bad_inactive_current.pointer_mut(&current).unwrap() = Value::Null;
        assert_invalid_type_at(&bad_inactive_current, &current);
    }

    for path in [
        "/DiscoveryManagerData/DiscoveryData-v1",
        "/DiscoveryManagerData/DiscoveryData-v1/Store",
        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record",
    ] {
        let mut value = fixture_value();
        remove_member(&mut value, path);
        assert!(
            parse_validated_save(&serde_json::to_vec(&value).unwrap())
                .unwrap()
                .discoveries()
                .len()
                == 0
        );
    }
    for (context, selector) in [("BaseContext", "Main"), ("ExpeditionContext", "Expedition")] {
        let path = format!("/{context}/PlayerStateData/PersistentPlayerBases");
        let mut no_active_bases = active_value(selector);
        remove_member(&mut no_active_bases, &path);
        let parsed = parse_validated_save(&serde_json::to_vec(&no_active_bases).unwrap()).unwrap();
        assert!(parsed.bases().is_empty());
    }

    for path in [
        "/DiscoveryManagerData",
        "/DiscoveryManagerData/DiscoveryData-v1",
        "/DiscoveryManagerData/DiscoveryData-v1/Store",
        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record",
        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/0",
        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/0/DD",
        "/BaseContext/PlayerStateData/PersistentPlayerBases",
        "/BaseContext/PlayerStateData/PersistentPlayerBases/0",
    ] {
        for invalid in [Value::Null, json!(false)] {
            let mut value = fixture_value();
            *value.pointer_mut(path).unwrap() = invalid;
            assert_invalid_type_at(&value, path);
        }
    }
}

#[test]
fn test_checked_parser_requires_top_level_discovery_manager_and_validates_late_rows_and_bases() {
    let mut unknown = fixture_value();
    let records = unknown["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    let mut late_unknown = records[0].clone();
    late_unknown["DD"]["DT"] = json!("UnknownDiscoveryType");
    late_unknown["DD"]["UA"] = json!("0x100000000000000");
    records.push(late_unknown);
    assert!(matches!(
        parse_validated_save(&serde_json::to_vec(&unknown).unwrap()),
        Err(SaveError::Address(AddressDecodeError::PackedWidth { path }))
            if path == "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/2/DD/UA"
    ));

    let mut base_context = fixture_value();
    base_context["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "BaseVersion":8,"GalacticAddress":"0x100000000000000","Position":[0.0,0.0,0.0],
            "Forward":[1.0,0.0,0.0],"LastUpdateTimestamp":1700000000,"Objects":[],"RID":"",
            "Owner":{"LID":"","UID":"late","USN":"","PTK":"ST","TS":0},
            "Name":"Late Base","BaseType":{"PersistentBaseTypes":"HomePlanetBase"},
            "LastEditedById":"","LastEditedByUsername":""
        }));
    assert!(matches!(
        parse_validated_save(&serde_json::to_vec(&base_context).unwrap()),
        Err(SaveError::Address(AddressDecodeError::PackedWidth { path }))
            if path == "/BaseContext/PlayerStateData/PersistentPlayerBases/1/GalacticAddress"
    ));

    let mut expedition_context = fixture_value();
    expedition_context["ExpeditionContext"]["PlayerStateData"]["PersistentPlayerBases"] = json!([
        expedition_context["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0].clone(),
        {
            "BaseVersion":8,"GalacticAddress":"0x100000000000000","Position":[0.0,0.0,0.0],
            "Forward":[1.0,0.0,0.0],"LastUpdateTimestamp":1700000000,"Objects":[],"RID":"",
            "Owner":{"LID":"","UID":"late","USN":"","PTK":"ST","TS":0},
            "Name":"Late Base","BaseType":{"PersistentBaseTypes":"HomePlanetBase"},
            "LastEditedById":"","LastEditedByUsername":""
        }
    ]);
    assert!(matches!(
        parse_validated_save(&serde_json::to_vec(&expedition_context).unwrap()),
        Err(SaveError::Address(AddressDecodeError::PackedWidth { path }))
            if path == "/ExpeditionContext/PlayerStateData/PersistentPlayerBases/1/GalacticAddress"
    ));
}

#[test]
fn test_checked_parser_validates_all_previous_components_and_six_component_sentinel() {
    for (context, selector) in [("BaseContext", "Main"), ("ExpeditionContext", "Expedition")] {
        let path = format!("/{context}/PlayerStateData/PreviousUniverseAddress");
        let mut absent = active_value(selector);
        let removed = absent
            .pointer_mut(&format!("/{context}/PlayerStateData"))
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("PreviousUniverseAddress");
        assert_eq!(removed.is_some(), context == "BaseContext");
        assert!(absent.pointer(&path).is_none());
        assert_eq!(
            parse_validated_save(&serde_json::to_vec(&absent).unwrap())
                .unwrap()
                .player_state()
                .previous_address,
            None
        );

        let zero = json!({
            "RealityIndex": 0,
            "GalacticAddress": {"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":0,"PlanetIndex":0}
        });
        let mut all_zero = active_value(selector);
        all_zero[context]["PlayerStateData"]["PreviousUniverseAddress"] = zero.clone();
        assert_eq!(
            parse_validated_save(&serde_json::to_vec(&all_zero).unwrap())
                .unwrap()
                .player_state()
                .previous_address,
            None
        );

        let fields = [
            ("RealityIndex", true, 0, 0, 0, 0, 0, 1),
            ("VoxelX", false, 1, 0, 0, 0, 0, 0),
            ("VoxelY", false, 0, 1, 0, 0, 0, 0),
            ("VoxelZ", false, 0, 0, 1, 0, 0, 0),
            ("SolarSystemIndex", false, 0, 0, 0, 1, 0, 0),
            ("PlanetIndex", false, 0, 0, 0, 0, 1, 0),
        ];
        for (component, reality, x, y, z, ssi, planet, galaxy) in fields {
            let member = if reality {
                format!("{path}/RealityIndex")
            } else {
                format!("{path}/GalacticAddress/{component}")
            };
            let mut value = active_value(selector);
            value[context]["PlayerStateData"]["PreviousUniverseAddress"] = zero.clone();
            *value.pointer_mut(&member).unwrap() = json!(1);
            let parsed = parse_validated_save(&serde_json::to_vec(&value).unwrap()).unwrap();
            assert_eq!(
                parsed.player_state().previous_address,
                Some(GalacticAddress::new(x, y, z, ssi, planet, galaxy))
            );
        }

        let mut null_previous = active_value(selector);
        null_previous[context]["PlayerStateData"]["PreviousUniverseAddress"] = Value::Null;
        assert_invalid_type_at(&null_previous, &path);
        let mut partial = active_value(selector);
        partial[context]["PlayerStateData"]["PreviousUniverseAddress"] = zero;
        remove_member(&mut partial, &format!("{path}/GalacticAddress/PlanetIndex"));
        assert_missing_at(&partial, &format!("{path}/GalacticAddress/PlanetIndex"));
    }
}

#[test]
fn test_checked_whole_parse_pairs_independent_packed_literals_and_preserves_raw_integer() {
    let mut value = fixture_value();
    let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    records[0]["DD"]["UA"] = json!("0x212B0004B2446A");
    records[1]["DD"]["UA"] = json!("0x212B0904B2446A");
    let mut below48 = records[0].clone();
    below48["DD"]["UA"] = json!("0x002B0004B2446A");
    let mut precise_integer = records[1].clone();
    precise_integer["DD"]["UA"] = json!(9_007_199_254_740_993_u64);
    let mut uppercase_prefix = records[0].clone();
    uppercase_prefix["DD"]["UA"] = json!("0X212B0004B2446A");
    records.extend([below48, precise_integer, uppercase_prefix]);

    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(
        parse_save(&bytes).is_ok(),
        "legacy typed parsing remains compatible"
    );
    let save = parse_validated_save(&bytes).unwrap();
    let expected = [
        (
            "SolarSystem",
            GalacticAddress::new(1130, 4, -1244, 299, 2, 0),
        ),
        ("Planet", GalacticAddress::new(1130, 4, -1244, 299, 2, 9)),
        (
            "SolarSystem",
            GalacticAddress::new(1130, 4, -1244, 43, 0, 0),
        ),
        ("Planet", GalacticAddress::new(1, 0, 0, 0, 2, 0)),
        (
            "SolarSystem",
            GalacticAddress::new(1130, 4, -1244, 299, 2, 0),
        ),
    ];
    assert_eq!(
        save.discoveries()
            .map(|(row, address)| (row.dd.dt.as_str(), address))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        save.root()
            .discovery_manager_data
            .discovery_data_v1
            .store
            .record[3]
            .dd
            .ua
            .0,
        9_007_199_254_740_993
    );
    assert_eq!(
        save.player_state().current_address,
        GalacticAddress::new(100, 50, -200, 42, 0, 0)
    );
}

#[test]
fn test_checked_parser_keeps_unnamed_unknown_type_base_and_validates_unknown_discovery_address() {
    let mut value = fixture_value();
    value["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["Name"] = json!("");
    value["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["BaseType"]["PersistentBaseTypes"] =
        json!("UnrecognizedBaseType");
    let save = parse_validated_save(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(save.bases().len(), 1);
    assert_eq!(save.bases()[0].name, "");
    assert_eq!(
        save.bases()[0].base_type,
        nms_core::BaseType::ExternalPlanetBase
    );

    for (context, selector) in [("BaseContext", "Main"), ("ExpeditionContext", "Expedition")] {
        for missing in [true, false] {
            let mut invalid = active_value(selector);
            let context_path = format!("/{context}/PlayerStateData/PersistentPlayerBases");
            let mut base =
                invalid["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0].clone();
            base["Name"] = json!("");
            base["BaseType"]["PersistentBaseTypes"] = json!("UnrecognizedBaseType");
            invalid[context]["PlayerStateData"]["PersistentPlayerBases"] = json!([base]);
            let path = format!("{context_path}/0/GalacticAddress");
            if missing {
                let removed = invalid
                    .pointer_mut(&context_path)
                    .unwrap()
                    .as_array_mut()
                    .unwrap()[0]
                    .as_object_mut()
                    .unwrap()
                    .remove("GalacticAddress");
                assert!(removed.is_some());
                assert!(invalid.pointer(&path).is_none());
                assert_missing_at(&invalid, &path);
            } else {
                *invalid.pointer_mut(&path).unwrap() = Value::Null;
                assert_invalid_type_at(&invalid, &path);
            }
        }
    }

    let mut invalid = fixture_value();
    invalid["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][1]["DD"]["DT"] =
        json!("UnknownDiscoveryType");
    invalid["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][1]["DD"]["UA"] =
        json!("0x100000000000000");
    assert!(matches!(
        parse_validated_save(&serde_json::to_vec(&invalid).unwrap()),
        Err(SaveError::Address(AddressDecodeError::PackedWidth { path }))
            if path == "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/1/DD/UA"
    ));
}

#[test]
fn test_legacy_parser_accepts_the_same_57_bit_u64_but_checked_parser_rejects_atomically() {
    let mut value = fixture_value();
    value["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["GalacticAddress"] =
        json!("0x100000000000000");
    let bytes = value.to_string().into_bytes();
    assert!(
        parse_save(&bytes).is_ok(),
        "legacy u64 parse is the counterfactual"
    );
    let error = parse_validated_save(&bytes).unwrap_err();
    packed_error(
        &error,
        "/BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress",
    );
}

#[test]
fn test_checked_parser_rejects_late_legacy_accepted_base_after_valid_base_and_discoveries() {
    let mut value = fixture_value();
    let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    records.push(json!({
        "DD": {"UA":"0x00060000000000", "DT":"SolarSystem", "VP":[]},
        "DM": {}, "OWS":{"LID":"","UID":"1","USN":"Checked","PTK":"ST","TS":1700000000}, "FL":{"U":1}
    }));
    records.push(json!({
        "DD": {"UA":"0x10060000000000", "DT":"Planet", "VP":["0xAC",0]},
        "DM": {}, "OWS":{"LID":"","UID":"1","USN":"Checked","PTK":"ST","TS":1700000000}, "FL":{"U":1}
    }));
    value["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "BaseVersion":8,
            "GalacticAddress":"0x100000000000000",
            "Position":[9.0,8.0,7.0],
            "Forward":[1.0,0.0,0.0],
            "LastUpdateTimestamp":1700000000,
            "Objects":[],
            "RID":"",
            "Owner":{"LID":"","UID":"rejected-owner","USN":"","PTK":"ST","TS":0},
            "Name":"Rejected Base",
            "BaseType":{"PersistentBaseTypes":"HomePlanetBase"},
            "LastEditedById":"",
            "LastEditedByUsername":""
        }));

    let bytes = value.to_string().into_bytes();
    assert!(
        parse_save(&bytes).is_ok(),
        "legacy parser accepts the 57-bit u64"
    );
    packed_error(
        &parse_validated_save(&bytes).unwrap_err(),
        "/BaseContext/PlayerStateData/PersistentPlayerBases/1/GalacticAddress",
    );
}

#[test]
fn test_checked_parser_distinguishes_missing_null_and_wrong_type_before_serde_defaults() {
    let path = "/BaseContext/PlayerStateData/UniverseAddress/GalacticAddress/VoxelX";
    for invalid in [json!(null), json!(1.5), json!("100")] {
        let mut value = fixture_value();
        value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
            invalid;
        assert!(matches!(
            parse_validated_save(value.to_string().as_bytes()),
            Err(SaveError::Address(AddressDecodeError::InvalidType { path: actual, .. }))
                if actual == path
        ));
    }

    let mut value = fixture_value();
    value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]
        .as_object_mut()
        .unwrap()
        .remove("VoxelX");
    assert!(matches!(
        parse_validated_save(value.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::Missing { path: actual })) if actual == path
    ));

    let discovery_path = "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/0/DD";
    let mut missing_discovery = fixture_value();
    missing_discovery["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][0]
        .as_object_mut()
        .unwrap()
        .remove("DD");
    assert!(matches!(
        parse_validated_save(missing_discovery.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::Missing { path: actual }))
            if actual == discovery_path
    ));

    let mut null_discovery = fixture_value();
    null_discovery["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][0]["DD"] =
        json!(null);
    assert!(matches!(
        parse_validated_save(null_discovery.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::InvalidType { path: actual, .. }))
            if actual == discovery_path
    ));

    for (unknown_type, row_index) in [(false, 1), (true, 2)] {
        for invalid_kind in ["missing", "null", "invalid-hex"] {
            let mut value = fixture_value();
            if unknown_type {
                let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
                    .as_array_mut()
                    .unwrap();
                let mut unknown = records[0].clone();
                unknown["DD"]["DT"] = json!("UnknownDiscoveryType");
                records.push(unknown);
            }
            let path =
                format!("/DiscoveryManagerData/DiscoveryData-v1/Store/Record/{row_index}/DD/UA");
            match invalid_kind {
                "missing" => {
                    let parent = format!(
                        "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/{row_index}/DD"
                    );
                    let removed = value
                        .pointer_mut(&parent)
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .remove("UA");
                    assert!(removed.is_some());
                    assert!(value.pointer(&path).is_none());
                    assert_missing_at(&value, &path);
                }
                "null" => {
                    *value.pointer_mut(&path).unwrap() = Value::Null;
                    assert_invalid_type_at(&value, &path);
                }
                "invalid-hex" => {
                    *value.pointer_mut(&path).unwrap() = json!("0xGG");
                    assert!(matches!(
                        parse_validated_save(&serde_json::to_vec(&value).unwrap()),
                        Err(SaveError::Address(AddressDecodeError::InvalidHex { path: actual }))
                            if actual == path
                    ));
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn test_checked_parser_handles_inactive_and_previous_address_presence_without_defaulting_bad_values()
 {
    let mut value = fixture_value();
    value["ActiveContext"] = json!("Expedition");
    value["ExpeditionContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"] =
        json!({"VoxelX": 1, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0});
    value["ExpeditionContext"]["PlayerStateData"]["UniverseAddress"]["RealityIndex"] = json!(0);
    value["ExpeditionContext"]["PlayerStateData"]["PreviousUniverseAddress"] = json!({
        "RealityIndex": 0,
        "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 3}
    });
    let save = parse_validated_save(value.to_string().as_bytes()).unwrap();
    assert_eq!(
        save.player_state().current_address,
        GalacticAddress::new(1, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        save.player_state().previous_address,
        Some(GalacticAddress::new(0, 0, 0, 0, 3, 0))
    );

    let mut invalid_inactive = fixture_value();
    invalid_inactive["ExpeditionContext"]["PlayerStateData"]["UniverseAddress"] = json!(null);
    assert!(matches!(
        parse_validated_save(invalid_inactive.to_string().as_bytes()),
        Err(SaveError::Address(AddressDecodeError::InvalidType { path, .. }))
            if path == "/ExpeditionContext/PlayerStateData/UniverseAddress"
    ));

    let zero_previous = json!({
        "RealityIndex": 0,
        "GalacticAddress": {
            "VoxelX": 0, "VoxelY": 0, "VoxelZ": 0,
            "SolarSystemIndex": 0, "PlanetIndex": 0
        }
    });
    for (inactive, active_selector) in
        [("BaseContext", "Expedition"), ("ExpeditionContext", "Main")]
    {
        let path = format!("/{inactive}/PlayerStateData/PreviousUniverseAddress");
        let parent = format!("/{inactive}/PlayerStateData");

        let mut null_previous = active_value(active_selector);
        null_previous
            .pointer_mut(&parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("PreviousUniverseAddress".into(), Value::Null);
        assert_invalid_type_at(&null_previous, &path);

        let mut partial_previous = active_value(active_selector);
        partial_previous
            .pointer_mut(&parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("PreviousUniverseAddress".into(), zero_previous.clone());
        let planet_path = format!("{path}/GalacticAddress/PlanetIndex");
        let removed = partial_previous
            .pointer_mut(&path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .get_mut("GalacticAddress")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("PlanetIndex");
        assert!(removed.is_some());
        assert_missing_at(&partial_previous, &planet_path);

        let mut out_of_range = active_value(active_selector);
        let mut previous = zero_previous.clone();
        previous["RealityIndex"] = json!(256);
        out_of_range
            .pointer_mut(&parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("PreviousUniverseAddress".into(), previous);
        let reality_path = format!("{path}/RealityIndex");
        assert!(matches!(
            parse_validated_save(&serde_json::to_vec(&out_of_range).unwrap()),
            Err(SaveError::Address(AddressDecodeError::OutOfRange {
                path: actual,
                value: 256,
                minimum: 0,
                maximum: 255,
            })) if actual == reality_path
        ));
    }
}

#[test]
fn test_checked_file_parser_uses_plaintext_pipeline_and_preserves_errors() {
    let path = std::env::temp_dir().join(format!("checked-save-{}.json", std::process::id()));
    std::fs::write(&path, MINIMAL).unwrap();
    let save = parse_validated_save_file(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(save.discoveries().len(), 2);

    let missing =
        std::env::temp_dir().join(format!("missing-checked-save-{}.json", std::process::id()));
    assert!(matches!(
        parse_validated_save_file(&missing),
        Err(SaveError::Io(_))
    ));
}

#[test]
fn test_checked_file_parser_validates_after_key_mapping_and_lz4_decompression() {
    let mut source: Value = serde_json::from_slice(MINIMAL).unwrap();
    let mapping: Value =
        serde_json::from_str(include_str!("../data/mapping_mbincompiler.json")).unwrap();
    let readable_to_obfuscated = mapping["Mapping"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["Value"].as_str().unwrap().to_owned(),
                entry["Key"].as_str().unwrap().to_owned(),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    let _ = obfuscate_keys(&mut source, &readable_to_obfuscated);
    let obfuscated = root_f2p_first(&source);
    assert!(obfuscated.starts_with(r#"{"F2P""#));
    let compressed = compressed_save(obfuscated.as_bytes());
    let mut invalid_source: Value = serde_json::from_slice(MINIMAL).unwrap();
    invalid_source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["GalacticAddress"] =
        json!("0x100000000000000");
    let invalid_plain = serde_json::to_vec(&invalid_source).unwrap();
    let mut invalid_obfuscated_value = invalid_source.clone();
    let _ = obfuscate_keys(&mut invalid_obfuscated_value, &readable_to_obfuscated);
    let invalid_obfuscated = root_f2p_first(&invalid_obfuscated_value).into_bytes();
    let invalid_compressed = compressed_save(&invalid_obfuscated);

    for (name, bytes, invalid_bytes) in [
        ("plain.json", MINIMAL.to_vec(), invalid_plain),
        ("obfuscated.hg", obfuscated.into_bytes(), invalid_obfuscated),
        ("compressed.hg", compressed, invalid_compressed),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        let save = parse_validated_save_file(&path).unwrap();
        assert_eq!(save.discoveries().len(), 2);
        assert_eq!(save.root().version, 4720);
        assert_eq!(
            save.discoveries()
                .map(|(record, address)| (record.dd.dt.as_str(), address))
                .collect::<Vec<_>>(),
            [
                (
                    "SolarSystem",
                    GalacticAddress::new(-1017, 3, -1352, 1280, 0, 0)
                ),
                ("Planet", GalacticAddress::new(-1017, 3, -1352, 1280, 1, 0)),
            ]
        );
        assert_eq!(
            save.player_state().current_address,
            GalacticAddress::new(100, 50, -200, 42, 0, 0)
        );

        std::fs::write(&path, invalid_bytes).unwrap();
        assert!(matches!(
            parse_validated_save_file(&path),
            Err(SaveError::Address(AddressDecodeError::PackedWidth { path }))
                if path == "/BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress"
        ));
    }
}

fn root_f2p_first(value: &Value) -> String {
    let object = value.as_object().expect("mapped save root object");
    let version = serde_json::to_string(object.get("F2P").expect("mapped Version/F2P"))
        .expect("serialize mapped version");
    let mut remainder = object.clone();
    remainder.remove("F2P");
    let remainder = serde_json::to_string(&remainder).expect("serialize mapped root members");
    format!("{{\"F2P\":{version},{}", &remainder[1..])
}

fn obfuscate_keys(
    value: &mut Value,
    mapping: &std::collections::HashMap<String, String>,
) -> String {
    match value {
        Value::Object(object) => {
            let old = std::mem::take(object);
            for (key, mut child) in old {
                let key = mapping.get(&key).cloned().unwrap_or(key);
                let _ = obfuscate_keys(&mut child, mapping);
                object.insert(key, child);
            }
            serde_json::to_string(value).unwrap()
        }
        Value::Array(items) => {
            for item in items {
                let _ = obfuscate_keys(item, mapping);
            }
            String::new()
        }
        _ => String::new(),
    }
}

fn compressed_save(json: &[u8]) -> Vec<u8> {
    let compressed = lz4_flex::block::compress(json);
    let mut bytes = Vec::with_capacity(16 + compressed.len());
    bytes.extend_from_slice(&0xFEED_A1E5_u32.to_le_bytes());
    bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(json.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&compressed);
    bytes
}
