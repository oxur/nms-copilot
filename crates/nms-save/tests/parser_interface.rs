use std::error::Error;

use nms_core::GalacticAddress;
use nms_save::address::AddressDecodeError;
use nms_save::{NmsSaveParser, SaveError, SaveParser, parse_validated_save_file};
use serde_json::{Value, json};

const MINIMAL: &[u8] = include_bytes!("../../../data/test/save56_minimal.json");

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

fn both_representations(json: &[u8]) -> [Vec<u8>; 2] {
    [json.to_vec(), compressed_save(json)]
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
fn test_dyn_save_parser_accepts_plaintext_and_lz4_with_literal_checked_output() {
    let parser: &dyn SaveParser = &NmsSaveParser;
    for bytes in both_representations(MINIMAL) {
        let save = parser.parse(&bytes).unwrap();
        assert_eq!(save.root().version, 4720);
        assert_eq!(
            save.player_state().current_address,
            GalacticAddress::new(100, 50, -200, 42, 0, 0)
        );
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
        assert_eq!(save.bases().len(), 1);
        assert_eq!(save.bases()[0].name, "Homestead Alpha");
    }
}

#[test]
fn test_dyn_save_parser_preserves_source_validation_for_plaintext_and_lz4() {
    let parser: &dyn SaveParser = &NmsSaveParser;

    let mut wide = serde_json::from_slice::<Value>(MINIMAL).unwrap();
    wide["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"][1]["DD"]["UA"] =
        json!("0x100000000000000");
    for bytes in both_representations(&serde_json::to_vec(&wide).unwrap()) {
        let error = parser.parse(&bytes).unwrap_err();
        packed_error(
            &error,
            "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/1/DD/UA",
        );
    }

    let mut missing = serde_json::from_slice::<Value>(MINIMAL).unwrap();
    missing["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]
        .as_object_mut()
        .unwrap()
        .remove("PlanetIndex");
    for bytes in both_representations(&serde_json::to_vec(&missing).unwrap()) {
        assert!(matches!(
            parser.parse(&bytes),
            Err(SaveError::Address(AddressDecodeError::Missing { path }))
                if path == "/BaseContext/PlayerStateData/UniverseAddress/GalacticAddress/PlanetIndex"
        ));
    }

    let mut out_of_range = serde_json::from_slice::<Value>(MINIMAL).unwrap();
    out_of_range["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"]["VoxelX"] =
        json!(2048);
    for bytes in both_representations(&serde_json::to_vec(&out_of_range).unwrap()) {
        assert!(matches!(
            parser.parse(&bytes),
            Err(SaveError::Address(AddressDecodeError::OutOfRange {
                path,
                value: 2048,
                minimum: -2048,
                maximum: 2047,
            })) if path == "/BaseContext/PlayerStateData/UniverseAddress/GalacticAddress/VoxelX"
        ));
    }
}

#[test]
fn test_dyn_save_parser_returns_typed_malformed_json_and_truncated_lz4_errors() {
    let parser: &dyn SaveParser = &NmsSaveParser;
    assert!(matches!(
        parser.parse(b"{malformed json"),
        Err(SaveError::JsonParseError { .. })
    ));
    assert!(matches!(
        parser.parse(&[0xE5, 0xA1, 0xED, 0xFE]),
        Err(SaveError::UnexpectedEof {
            offset: 0,
            expected: 16
        })
    ));
}

#[test]
fn test_checked_file_parser_delegates_and_preserves_rejection() {
    let mut invalid = serde_json::from_slice::<Value>(MINIMAL).unwrap();
    invalid["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["GalacticAddress"] =
        json!("0x100000000000000");

    for (index, (valid_bytes, invalid_bytes)) in both_representations(MINIMAL)
        .into_iter()
        .zip(both_representations(&serde_json::to_vec(&invalid).unwrap()))
        .enumerate()
    {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("save-{index}.hg"));
        std::fs::write(&path, valid_bytes).unwrap();
        let save = parse_validated_save_file(&path).unwrap();
        assert_eq!(save.root().version, 4720);
        assert_eq!(save.discoveries().len(), 2);

        std::fs::write(&path, invalid_bytes).unwrap();
        let error = parse_validated_save_file(&path).unwrap_err();
        packed_error(
            &error,
            "/BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress",
        );
    }
}
