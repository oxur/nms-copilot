use nms_core::GalacticAddress;
use nms_save::address::{
    AddressDecodeError, decode_base_address, decode_current_address, decode_discovery_address,
    decode_previous_address,
};
use serde::Deserialize;
use serde_json::{Value, json};

type PackedDecoder = fn(Option<&Value>, &str) -> Result<GalacticAddress, AddressDecodeError>;

const ORACLES: &str = include_str!("fixtures/address-oracles.json");

#[derive(Debug, Deserialize)]
struct OracleFile {
    discovery: Vec<Oracle>,
    bases: Vec<Oracle>,
}

#[derive(Debug, Deserialize)]
struct Oracle {
    id: String,
    json_kind: String,
    input: Value,
    expected: Expected,
}

#[derive(Debug, Clone, Deserialize)]
struct Expected {
    planet_index: u8,
    solar_system_index: u16,
    reality_index: u8,
    voxel_x: i16,
    voxel_y: i8,
    voxel_z: i16,
    portal: String,
}

fn unchanged<T>(value: Option<&Value>, run: impl FnOnce() -> T) -> T {
    let before = value.cloned();
    let result = run();
    assert_eq!(value, before.as_ref(), "decoder mutated its JSON input");
    result
}

fn assert_address(actual: GalacticAddress, expected: &Expected) {
    assert_eq!(actual.planet_index(), expected.planet_index);
    assert_eq!(actual.solar_system_index(), expected.solar_system_index);
    assert_eq!(actual.reality_index, expected.reality_index);
    assert_eq!(actual.voxel_x(), expected.voxel_x);
    assert_eq!(actual.voxel_y(), expected.voxel_y);
    assert_eq!(actual.voxel_z(), expected.voxel_z);
    assert_eq!(actual.to_portal_address().to_hex_string(), expected.portal);
}

fn packed_decoders() -> [(PackedDecoder, &'static str); 2] {
    [
        (decode_discovery_address, "/Discovery/UA"),
        (decode_base_address, "/Bases/0/GalacticAddress"),
    ]
}

fn assert_packed_both(value: &Value, expected: &Expected) {
    for (decoder, path) in packed_decoders() {
        let actual = unchanged(Some(value), || decoder(Some(value), path))
            .unwrap_or_else(|error| panic!("{path}: {error}"));
        assert_address(actual, expected);
    }
}

fn assert_packed_error_both(value: Option<&Value>, expected: impl Fn(&str) -> AddressDecodeError) {
    for (decoder, path) in packed_decoders() {
        let actual = unchanged(value, || decoder(value, path)).unwrap_err();
        assert_eq!(actual, expected(path));
    }
}

fn assert_current(value: &Value, path: &str, expected: &Expected) {
    let actual = unchanged(Some(value), || decode_current_address(Some(value), path))
        .unwrap_or_else(|error| panic!("{path}: {error}"));
    assert_address(actual, expected);
}

fn assert_previous(value: &Value, path: &str, expected: &Expected) {
    let actual = unchanged(Some(value), || decode_previous_address(Some(value), path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
        .unwrap_or_else(|| panic!("{path}: expected a present previous address"));
    assert_address(actual, expected);
}

fn assert_expanded_error_both(value: &Value, path: &str, expected: AddressDecodeError) {
    let current = unchanged(Some(value), || decode_current_address(Some(value), path)).unwrap_err();
    assert_eq!(current, expected);
    let previous =
        unchanged(Some(value), || decode_previous_address(Some(value), path)).unwrap_err();
    assert_eq!(previous, expected);
}

fn expanded(reality: Value, x: Value, y: Value, z: Value, ssi: Value, planet: Value) -> Value {
    json!({
        "RealityIndex": reality,
        "GalacticAddress": {
            "VoxelX": x,
            "VoxelY": y,
            "VoxelZ": z,
            "SolarSystemIndex": ssi,
            "PlanetIndex": planet,
        }
    })
}

fn valid_expanded() -> Value {
    expanded(
        json!(9),
        json!(1130),
        json!(4),
        json!(-1244),
        json!(299),
        json!(2),
    )
}

#[test]
fn test_decode_independent_chili_and_all_base_oracles() {
    let oracles: OracleFile = serde_json::from_str(ORACLES).expect("oracle JSON is valid");
    assert_eq!(oracles.discovery.len(), 1);
    assert_eq!(oracles.bases.len(), 44);
    assert_eq!(
        oracles
            .bases
            .iter()
            .filter(|oracle| oracle.json_kind == "string")
            .count(),
        31
    );
    assert_eq!(
        oracles
            .bases
            .iter()
            .filter(|oracle| oracle.json_kind == "integer")
            .count(),
        13
    );

    for oracle in &oracles.discovery {
        assert_eq!(oracle.id, "W-CHILI");
        assert_eq!(oracle.json_kind, "string");
        assert_eq!(
            oracle.input,
            json!("0x212B0004B2446A"),
            "{} input",
            oracle.id
        );
        let decoded = unchanged(Some(&oracle.input), || {
            decode_discovery_address(Some(&oracle.input), "/DD/UA")
        })
        .unwrap_or_else(|error| panic!("{}: {error}", oracle.id));
        assert_address(decoded, &oracle.expected);
    }

    for oracle in &oracles.bases {
        assert!(matches!(oracle.json_kind.as_str(), "string" | "integer"));
        let decoded = unchanged(Some(&oracle.input), || {
            decode_base_address(Some(&oracle.input), "/Bases/GalacticAddress")
        })
        .unwrap_or_else(|error| panic!("{}: {error}", oracle.id));
        assert_address(decoded, &oracle.expected);
    }
}

#[test]
fn test_decode_field_families_preserve_high_galaxy_and_p0_width() {
    let galaxy_nine = json!("0x212B0904B2446A");
    let galaxy_nine_expected = Expected {
        planet_index: 2,
        solar_system_index: 299,
        reality_index: 9,
        voxel_x: 1130,
        voxel_y: 4,
        voxel_z: -1244,
        portal: "212B04B2446A".to_owned(),
    };
    assert_packed_both(&galaxy_nine, &galaxy_nine_expected);
    assert_packed_both(&json!(0x212B0904B2446A_u64), &galaxy_nine_expected);

    let p0_ssi_299_text = "0x012B0004B2446A";
    let p0_ssi_299_integer = 328_754_055_496_810_u64;
    let two_to_48 = 281_474_976_710_656_u64;
    assert!(u64::from_str_radix(&p0_ssi_299_text[2..], 16).unwrap() >= two_to_48);
    assert!(p0_ssi_299_integer >= two_to_48);
    let p0_ssi_299_expected = Expected {
        planet_index: 0,
        solar_system_index: 299,
        reality_index: 0,
        voxel_x: 1130,
        voxel_y: 4,
        voxel_z: -1244,
        portal: "012B04B2446A".to_owned(),
    };
    assert_packed_both(&json!(p0_ssi_299_text), &p0_ssi_299_expected);
    assert_packed_both(&json!(p0_ssi_299_integer), &p0_ssi_299_expected);

    let narrow_p0_text = "0x002B0004B2446A";
    let narrow_p0_integer = 47_279_078_786_154_u64;
    assert!(u64::from_str_radix(&narrow_p0_text[2..], 16).unwrap() < two_to_48);
    assert!(narrow_p0_integer < two_to_48);
    let narrow_p0_expected = Expected {
        planet_index: 0,
        solar_system_index: 43,
        reality_index: 0,
        voxel_x: 1130,
        voxel_y: 4,
        voxel_z: -1244,
        portal: "002B04B2446A".to_owned(),
    };
    assert_packed_both(&json!(narrow_p0_text), &narrow_p0_expected);
    assert_packed_both(&json!(narrow_p0_integer), &narrow_p0_expected);

    let galaxy_zero = json!("0x212B0004B2446A");
    let galaxy_zero_expected = Expected {
        reality_index: 0,
        ..galaxy_nine_expected.clone()
    };
    assert_packed_both(&galaxy_zero, &galaxy_zero_expected);
    let galaxy_nine_same_voxel = json!("0x212B0904B2446A");
    assert_packed_both(&galaxy_nine_same_voxel, &galaxy_nine_expected);
    for (decoder, path) in packed_decoders() {
        let zero = unchanged(Some(&galaxy_zero), || decoder(Some(&galaxy_zero), path)).unwrap();
        let nine = unchanged(Some(&galaxy_nine_same_voxel), || {
            decoder(Some(&galaxy_nine_same_voxel), path)
        })
        .unwrap();
        assert_eq!(zero.packed(), nine.packed());
        assert_ne!(zero, nine);
        assert_ne!(zero.reality_index, nine.reality_index);
        for (address, expected) in [(zero, &galaxy_zero_expected), (nine, &galaxy_nine_expected)] {
            assert_address(address, expected);
        }
        for (address, expected) in [(nine, &galaxy_nine_expected), (zero, &galaxy_zero_expected)] {
            assert_address(address, expected);
        }
    }

    let old_core = GalacticAddress::from_packed(0x212B04B2446A, 9);
    assert_eq!(old_core.packed(), 0x212B04B2446A);
    assert_eq!(old_core.reality_index, 9);
    let old_mask = GalacticAddress::from_packed(0xFF21_2B09_04B2_446A, 0);
    assert_eq!(old_mask.packed(), 0x2B09_04B2_446A);
}

#[test]
fn test_decode_packed_integer_string_boundaries_and_precision() {
    let zero_expected = Expected {
        planet_index: 0,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: 0,
        voxel_y: 0,
        voxel_z: 0,
        portal: "000000000000".to_owned(),
    };
    assert_packed_both(&json!(0), &zero_expected);
    assert_packed_both(&json!("0"), &zero_expected);

    let max_expected = Expected {
        planet_index: 15,
        solar_system_index: 4095,
        reality_index: 255,
        voxel_x: -1,
        voxel_y: -1,
        voxel_z: -1,
        portal: "FFFFFFFFFFFF".to_owned(),
    };
    assert_packed_both(&json!("0xFFFFFFFFFFFFFF"), &max_expected);
    assert_packed_both(&json!(72_057_594_037_927_935_u64), &max_expected);

    let precise: Value = serde_json::from_str("9007199254740993").unwrap();
    let precise_expected = Expected {
        planet_index: 2,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: 1,
        voxel_y: 0,
        voxel_z: 0,
        portal: "200000000001".to_owned(),
    };
    assert_packed_both(&precise, &precise_expected);
    let rounded: Value = serde_json::from_str("9007199254740992").unwrap();
    let rounded_expected = Expected {
        voxel_x: 0,
        portal: "200000000000".to_owned(),
        ..precise_expected.clone()
    };
    assert_packed_both(&rounded, &rounded_expected);
    assert_ne!(precise, rounded);
    let precise_address = unchanged(Some(&precise), || {
        decode_discovery_address(Some(&precise), "/UA")
    })
    .unwrap();
    let rounded_address = unchanged(Some(&rounded), || {
        decode_discovery_address(Some(&rounded), "/UA")
    })
    .unwrap();
    assert_ne!(precise_address, rounded_address);

    let short_hex = json!("A");
    let decimal_string = json!("10");
    let integer = json!(10);
    let x_ten_expected = Expected {
        voxel_x: 10,
        portal: "00000000000A".to_owned(),
        ..zero_expected.clone()
    };
    let x_sixteen_expected = Expected {
        voxel_x: 16,
        portal: "000000000010".to_owned(),
        ..zero_expected.clone()
    };
    assert_packed_both(&short_hex, &x_ten_expected);
    assert_packed_both(&integer, &x_ten_expected);
    assert_packed_both(&decimal_string, &x_sixteen_expected);
    for text in ["00000000000000", "0x00000000000000", "0X00000000000000"] {
        assert_packed_both(&json!(text), &zero_expected);
    }
    let lowercase = json!("abcdef01234567");
    let lowercase_expected = Expected {
        planet_index: 10,
        solar_system_index: 3021,
        reality_index: 239,
        voxel_x: 1383,
        voxel_y: 1,
        voxel_z: 564,
        portal: "ABCD01234567".to_owned(),
    };
    assert_packed_both(&lowercase, &lowercase_expected);

    let signed_min = json!("0x00000080800800");
    let signed_min_expected = Expected {
        planet_index: 0,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: -2048,
        voxel_y: -128,
        voxel_z: -2048,
        portal: "000080800800".to_owned(),
    };
    assert_packed_both(&signed_min, &signed_min_expected);
    let signed_max = json!("0x0000007F7FF7FF");
    let signed_max_expected = Expected {
        planet_index: 0,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: 2047,
        voxel_y: 127,
        voxel_z: 2047,
        portal: "00007F7FF7FF".to_owned(),
    };
    assert_packed_both(&signed_max, &signed_max_expected);
}

#[test]
fn test_decode_both_packed_apis_reject_invalid_values_with_exact_paths() {
    let negative_one = json!(-1);
    let signed_min = json!(i64::MIN);
    let over_width = json!(72_057_594_037_927_936_u64);
    let u64_max = json!(u64::MAX);
    let too_large_json: Value = serde_json::from_str("18446744073709551616").unwrap();
    let invalid_values = [
        json!(null),
        json!(true),
        json!([]),
        json!({}),
        json!(1.5),
        serde_json::from_str::<Value>("1.0").unwrap(),
        serde_json::from_str::<Value>("1e0").unwrap(),
        too_large_json,
    ];

    assert_packed_error_both(None, |path| AddressDecodeError::Missing {
        path: path.to_owned(),
    });
    for value in &invalid_values {
        assert_packed_error_both(Some(value), |path| AddressDecodeError::InvalidType {
            path: path.to_owned(),
            expected: "an unsigned integer or hexadecimal string",
        });
    }
    for value in [&negative_one, &signed_min] {
        let numeric = value.as_i64().unwrap();
        assert_packed_error_both(Some(value), |path| AddressDecodeError::OutOfRange {
            path: path.to_owned(),
            value: i128::from(numeric),
            minimum: 0,
            maximum: 72_057_594_037_927_935,
        });
    }
    for value in [&over_width, &u64_max] {
        assert_packed_error_both(Some(value), |path| AddressDecodeError::PackedWidth {
            path: path.to_owned(),
        });
    }
    for text in ["", "0x", "0XGG", "+1", "-1", " 1", "1 ", "0x_1", "é"] {
        let value = json!(text);
        assert_packed_error_both(Some(&value), |path| AddressDecodeError::InvalidHex {
            path: path.to_owned(),
        });
    }
    for text in ["000000000000000", "0000000000000000"] {
        let value = json!(text);
        assert_packed_error_both(Some(&value), |path| AddressDecodeError::PackedWidth {
            path: path.to_owned(),
        });
    }
    let invalid_over_width = json!("0x000000000000000G");
    assert_packed_error_both(Some(&invalid_over_width), |path| {
        AddressDecodeError::InvalidHex {
            path: path.to_owned(),
        }
    });
}

#[test]
fn test_decode_expanded_full_object_and_component_edges() {
    let value = valid_expanded();
    let expected = Expected {
        planet_index: 2,
        solar_system_index: 299,
        reality_index: 9,
        voxel_x: 1130,
        voxel_y: 4,
        voxel_z: -1244,
        portal: "212B04B2446A".to_owned(),
    };
    assert_current(&value, "/Player/UniverseAddress", &expected);
    let zero = expanded(json!(0), json!(0), json!(0), json!(0), json!(0), json!(0));
    let zero_expected = Expected {
        planet_index: 0,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: 0,
        voxel_y: 0,
        voxel_z: 0,
        portal: "000000000000".to_owned(),
    };
    assert_current(&zero, "/Player/UniverseAddress", &zero_expected);

    let boundary_cases = [
        (
            "RealityIndex",
            0,
            Expected {
                reality_index: 0,
                ..expected.clone()
            },
        ),
        (
            "RealityIndex",
            255,
            Expected {
                reality_index: 255,
                ..expected.clone()
            },
        ),
        (
            "VoxelX",
            -2048,
            Expected {
                voxel_x: -2048,
                portal: "212B04B24800".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "VoxelX",
            2047,
            Expected {
                voxel_x: 2047,
                portal: "212B04B247FF".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "VoxelY",
            -128,
            Expected {
                voxel_y: -128,
                portal: "212B80B2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "VoxelY",
            127,
            Expected {
                voxel_y: 127,
                portal: "212B7FB2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "VoxelZ",
            -2048,
            Expected {
                voxel_z: -2048,
                portal: "212B0480046A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "VoxelZ",
            2047,
            Expected {
                voxel_z: 2047,
                portal: "212B047FF46A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "SolarSystemIndex",
            0,
            Expected {
                solar_system_index: 0,
                portal: "200004B2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "SolarSystemIndex",
            4094,
            Expected {
                solar_system_index: 4094,
                portal: "2FFE04B2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "SolarSystemIndex",
            4095,
            Expected {
                solar_system_index: 4095,
                portal: "2FFF04B2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "PlanetIndex",
            0,
            Expected {
                planet_index: 0,
                portal: "012B04B2446A".to_owned(),
                ..expected.clone()
            },
        ),
        (
            "PlanetIndex",
            15,
            Expected {
                planet_index: 15,
                portal: "F12B04B2446A".to_owned(),
                ..expected.clone()
            },
        ),
    ];
    for (field, replacement, expected_boundary) in &boundary_cases {
        let mut input = valid_expanded();
        set_component(&mut input, field, json!(*replacement));
        assert_current(&input, "/P", expected_boundary);
    }

    for (field, low, high, below, above) in [
        ("RealityIndex", 0_i128, 255_i128, -1_i128, 256_i128),
        ("VoxelX", -2048, 2047, -2049, 2048),
        ("VoxelY", -128, 127, -129, 128),
        ("VoxelZ", -2048, 2047, -2049, 2048),
        ("SolarSystemIndex", 0, 4095, -1, 4096),
        ("PlanetIndex", 0, 15, -1, 16),
    ] {
        for number in [low, high] {
            assert!(
                boundary_cases
                    .iter()
                    .any(|(candidate, value, _)| { *candidate == field && *value == number }),
                "missing full-field positive boundary {field}={number}"
            );
        }
        for number in [below, above] {
            let mut invalid = valid_expanded();
            set_component(&mut invalid, field, json!(number));
            let path = if field == "RealityIndex" {
                "/P/RealityIndex".to_owned()
            } else {
                format!("/P/GalacticAddress/{field}")
            };
            assert_expanded_error_both(
                &invalid,
                "/P",
                AddressDecodeError::OutOfRange {
                    path,
                    value: number,
                    minimum: low,
                    maximum: high,
                },
            );
        }
    }
    let max_unsigned = expanded(
        json!(u64::MAX),
        json!(0),
        json!(0),
        json!(0),
        json!(0),
        json!(0),
    );
    assert_expanded_error_both(
        &max_unsigned,
        "/P",
        AddressDecodeError::OutOfRange {
            path: "/P/RealityIndex".to_owned(),
            value: i128::from(u64::MAX),
            minimum: 0,
            maximum: 255,
        },
    );
}

fn set_component(value: &mut Value, field: &str, replacement: Value) {
    if field == "RealityIndex" {
        value
            .as_object_mut()
            .unwrap()
            .insert(field.to_owned(), replacement);
    } else {
        value["GalacticAddress"]
            .as_object_mut()
            .unwrap()
            .insert(field.to_owned(), replacement);
    }
}

#[test]
fn test_decode_expanded_required_presence_type_paths_and_priority() {
    assert_eq!(
        unchanged(None, || decode_current_address(None, "/P")).unwrap_err(),
        AddressDecodeError::Missing {
            path: "/P".to_owned()
        }
    );
    for value in [json!(null), json!(true), json!("object"), json!([])] {
        assert_expanded_error_both(
            &value,
            "/P",
            AddressDecodeError::InvalidType {
                path: "/P".to_owned(),
                expected: "an object",
            },
        );
    }

    let mut no_reality = valid_expanded();
    no_reality.as_object_mut().unwrap().remove("RealityIndex");
    assert_expanded_error_both(
        &no_reality,
        "/P",
        AddressDecodeError::Missing {
            path: "/P/RealityIndex".to_owned(),
        },
    );
    for invalid_reality in [json!(null), json!("1"), json!(true), json!(1.0)] {
        let mut invalid = valid_expanded();
        invalid
            .as_object_mut()
            .unwrap()
            .insert("RealityIndex".to_owned(), invalid_reality);
        assert_expanded_error_both(
            &invalid,
            "/P",
            AddressDecodeError::InvalidType {
                path: "/P/RealityIndex".to_owned(),
                expected: "an integer",
            },
        );
    }

    for galactic in [
        None,
        Some(json!(null)),
        Some(json!(1)),
        Some(json!("object")),
    ] {
        let mut value = valid_expanded();
        if let Some(child) = galactic {
            value
                .as_object_mut()
                .unwrap()
                .insert("GalacticAddress".to_owned(), child);
            assert_expanded_error_both(
                &value,
                "/P",
                AddressDecodeError::InvalidType {
                    path: "/P/GalacticAddress".to_owned(),
                    expected: "an object",
                },
            );
        } else {
            value.as_object_mut().unwrap().remove("GalacticAddress");
            assert_expanded_error_both(
                &value,
                "/P",
                AddressDecodeError::Missing {
                    path: "/P/GalacticAddress".to_owned(),
                },
            );
        }
    }

    for field in [
        "VoxelX",
        "VoxelY",
        "VoxelZ",
        "SolarSystemIndex",
        "PlanetIndex",
    ] {
        let child_path = format!("/P/GalacticAddress/{field}");
        let mut missing = valid_expanded();
        missing["GalacticAddress"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_expanded_error_both(
            &missing,
            "/P",
            AddressDecodeError::Missing {
                path: child_path.clone(),
            },
        );
        for invalid in [json!(null), json!("1"), json!(true), json!(1.5)] {
            let mut typed = valid_expanded();
            set_component(&mut typed, field, invalid);
            assert_expanded_error_both(
                &typed,
                "/P",
                AddressDecodeError::InvalidType {
                    path: child_path.clone(),
                    expected: "an integer",
                },
            );
        }
    }

    let mut invalid_both = valid_expanded();
    invalid_both
        .as_object_mut()
        .unwrap()
        .insert("RealityIndex".to_owned(), json!(256));
    set_component(&mut invalid_both, "VoxelX", json!(2048));
    assert_expanded_error_both(
        &invalid_both,
        "/P",
        AddressDecodeError::OutOfRange {
            path: "/P/RealityIndex".to_owned(),
            value: 256,
            minimum: 0,
            maximum: 255,
        },
    );
    let mut extra = valid_expanded();
    extra
        .as_object_mut()
        .unwrap()
        .insert("Unknown".to_owned(), json!({"a": 1}));
    let golden = Expected {
        planet_index: 2,
        solar_system_index: 299,
        reality_index: 9,
        voxel_x: 1130,
        voxel_y: 4,
        voxel_z: -1244,
        portal: "212B04B2446A".to_owned(),
    };
    assert_current(&extra, "/P", &golden);
}

#[test]
fn test_decode_previous_address_absence_and_all_six_zero_sentinel() {
    assert_eq!(
        unchanged(None, || decode_previous_address(None, "/P")).unwrap(),
        None
    );
    assert_eq!(
        unchanged(None, || decode_current_address(None, "/P")).unwrap_err(),
        AddressDecodeError::Missing {
            path: "/P".to_owned(),
        }
    );
    let null = json!(null);
    assert_expanded_error_both(
        &null,
        "/P",
        AddressDecodeError::InvalidType {
            path: "/P".to_owned(),
            expected: "an object",
        },
    );
    let mut partial = expanded(json!(0), json!(0), json!(0), json!(0), json!(0), json!(0));
    partial["GalacticAddress"]
        .as_object_mut()
        .unwrap()
        .remove("PlanetIndex");
    assert_expanded_error_both(
        &partial,
        "/P",
        AddressDecodeError::Missing {
            path: "/P/GalacticAddress/PlanetIndex".to_owned(),
        },
    );

    let all_zero = expanded(json!(0), json!(0), json!(0), json!(0), json!(0), json!(0));
    assert_eq!(
        unchanged(Some(&all_zero), || decode_previous_address(
            Some(&all_zero),
            "/P"
        ))
        .unwrap(),
        None
    );
    let zero_expected = Expected {
        planet_index: 0,
        solar_system_index: 0,
        reality_index: 0,
        voxel_x: 0,
        voxel_y: 0,
        voxel_z: 0,
        portal: "000000000000".to_owned(),
    };
    assert_current(&all_zero, "/P", &zero_expected);

    let nonzero_cases = [
        (
            expanded(json!(0), json!(1), json!(0), json!(0), json!(0), json!(0)),
            Expected {
                voxel_x: 1,
                portal: "000000000001".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(0), json!(0), json!(1), json!(0), json!(0), json!(0)),
            Expected {
                voxel_y: 1,
                portal: "000001000000".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(0), json!(0), json!(0), json!(1), json!(0), json!(0)),
            Expected {
                voxel_z: 1,
                portal: "000000001000".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(0), json!(0), json!(0), json!(0), json!(1), json!(0)),
            Expected {
                solar_system_index: 1,
                portal: "000100000000".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(0), json!(0), json!(0), json!(0), json!(0), json!(1)),
            Expected {
                planet_index: 1,
                portal: "100000000000".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(9), json!(0), json!(0), json!(0), json!(0), json!(0)),
            Expected {
                reality_index: 9,
                portal: "000000000000".to_owned(),
                ..zero_expected.clone()
            },
        ),
        (
            expanded(json!(9), json!(0), json!(0), json!(0), json!(0), json!(1)),
            Expected {
                planet_index: 1,
                reality_index: 9,
                portal: "100000000000".to_owned(),
                ..zero_expected.clone()
            },
        ),
    ];
    for (value, expected) in nonzero_cases {
        assert_previous(&value, "/P", &expected);
    }
}

#[test]
fn test_decode_error_traits_and_diagnostics_are_public_and_stable() {
    fn assert_error_bounds<T: std::error::Error + Send + Sync>() {}
    assert_error_bounds::<AddressDecodeError>();
    let invalid = json!(false);
    let error = unchanged(Some(&invalid), || {
        decode_base_address(Some(&invalid), "/Bases/12/UA")
    })
    .unwrap_err();
    assert_eq!(error.path(), "/Bases/12/UA");
    assert_eq!(
        error.to_string(),
        "expected an unsigned integer or hexadecimal string at /Bases/12/UA"
    );
}
