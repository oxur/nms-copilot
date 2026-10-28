use nms_core::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{BaseType, PlayerBase, PlayerState};
use nms_core::system::{Planet, System, SystemId};
use nms_save::address::AddressDecodeError;
use nms_watch::SaveSnapshot;
use std::io::Write;
use std::time::Duration;

const MINIMAL: &[u8] = include_bytes!("../../../data/test/save56_minimal.json");

fn replace_once(source: &[u8], from: &str, to: &str) -> Vec<u8> {
    let source = std::str::from_utf8(source).unwrap();
    assert_eq!(
        source.matches(from).count(),
        1,
        "fixture marker must be unique"
    );
    source.replacen(from, to, 1).into_bytes()
}

fn with_valid_ssi6_records(source: &[u8]) -> Vec<u8> {
    const RECORDS: &str = r#"{
            "DD": { "UA": "0x00060000000000", "DT": "SolarSystem", "VP": ["0xABCD1234"] },
            "DM": {},
            "OWS": { "LID": "", "UID": "76561198025707979", "USN": "Explorer", "PTK": "ST", "TS": 1700000000 },
            "FL": { "C": 1, "U": 1 }
          },
          {
            "DD": { "UA": "0x10060000000000", "DT": "Planet", "VP": ["0xDEAD1234", 0] },
            "DM": {},
            "OWS": { "LID": "", "UID": "76561198025707979", "USN": "Explorer", "PTK": "ST", "TS": 1700000000 },
            "FL": { "C": 1, "U": 1 }
          },
          "#;
    replace_once(
        source,
        "\"Record\": [\n",
        &format!("\"Record\": [\n{RECORDS}"),
    )
}

fn invalid_base_address(source: &[u8]) -> Vec<u8> {
    replace_once(
        source,
        "\"BaseVersion\": 8,\n          \"GalacticAddress\": \"0x05000003AB8C07\"",
        "\"BaseVersion\": 8,\n          \"GalacticAddress\": \"0x100000000000000\"",
    )
}

fn with_synthetic_galaxy_nine(source: &[u8]) -> Vec<u8> {
    const RECORDS: &str = r#"{
            "DD": { "UA": "0x05000903AB8C07", "DT": "SolarSystem", "VP": ["0xABCD1234"] },
            "DM": {},
            "OWS": { "LID": "", "UID": "76561198025707979", "USN": "Nine Explorer", "PTK": "ST", "TS": 1700000000 },
            "FL": { "C": 1, "U": 1 }
          },
          {
            "DD": { "UA": "0x15000903AB8C07", "DT": "Planet", "VP": ["0xFACE1234", 1] },
            "DM": {},
            "OWS": { "LID": "", "UID": "76561198025707979", "USN": "Nine Explorer", "PTK": "ST", "TS": 1700000000 },
            "FL": { "C": 1, "U": 1 }
          },
          "#;
    const BASE: &str = r#"{
          "BaseVersion": 8,
          "GalacticAddress": "0x05000903AB8C07",
          "Position": [9.0, 8.0, 7.0],
          "Forward": [1.0, 0.0, 0.0],
          "LastUpdateTimestamp": 1700000000,
          "Objects": [],
          "RID": "",
          "Owner": {"LID": "", "UID": "nine-owner", "USN": "Nine Explorer", "PTK": "ST", "TS": 1700000000},
          "Name": "Nine Base",
          "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
          "LastEditedById": "",
          "LastEditedByUsername": ""
        }"#;
    let with_records = replace_once(
        source,
        "\"Record\": [\n",
        &format!("\"Record\": [\n{RECORDS}"),
    );
    replace_once(
        &with_records,
        "\"PersistentPlayerBases\": [\n",
        &format!("\"PersistentPlayerBases\": [\n{BASE},\n"),
    )
}

#[test]
fn test_snapshot_uses_checked_discoveries_player_and_base_addresses() {
    let save = nms_save::parse_validated_save(MINIMAL).unwrap();
    let snapshot = SaveSnapshot::from_validated_save(&save);
    let system_address = GalacticAddress::new(-1017, 3, -1352, 1280, 0, 0);
    let id = SystemId::from_address(&system_address);
    assert_eq!(
        snapshot.systems[&id]
            .timestamp
            .as_ref()
            .map(|value| value.timestamp()),
        Some(1_700_000_000)
    );
    let expected_systems = std::collections::HashMap::from([(
        id,
        System::new(
            system_address,
            None,
            Some("Explorer".into()),
            Some("2023-11-14T22:13:20Z".parse().unwrap()),
            vec![Planet::new(
                1,
                Some(Biome::Lush),
                None,
                false,
                None,
                Some(0xDEAD1234),
            )],
        ),
    )]);
    let expected_planets = std::collections::HashMap::from([(
        (id, 1),
        Planet::new(1, Some(Biome::Lush), None, false, None, Some(0xDEAD1234)),
    )]);
    assert_eq!(
        snapshot.player_address,
        GalacticAddress::new(100, 50, -200, 42, 0, 0)
    );
    assert_eq!(snapshot.systems, expected_systems);
    assert_eq!(snapshot.planets, expected_planets);
    assert_eq!(
        snapshot.bases,
        std::collections::HashMap::from([(
            "homestead alpha".to_owned(),
            PlayerBase::new(
                "Homestead Alpha".into(),
                BaseType::HomePlanetBase,
                system_address,
                [100.0, 200.0, 300.0],
                Some("76561198025707979".into()),
            ),
        )])
    );
}

#[test]
fn test_snapshot_keeps_synthetic_galaxy_nine_discovery_and_base_separate_from_player_galaxy_zero() {
    let source = with_synthetic_galaxy_nine(MINIMAL);
    let save = nms_save::parse_validated_save(&source).unwrap();
    let snapshot = SaveSnapshot::from_validated_save(&save);
    let galaxy_zero = GalacticAddress::new(-1017, 3, -1352, 1280, 0, 0);
    let galaxy_nine = GalacticAddress::new(-1017, 3, -1352, 1280, 0, 9);
    let id_zero = SystemId::from_address(&galaxy_zero);
    let id_nine = SystemId::from_address(&galaxy_nine);
    assert_eq!(
        snapshot.systems[&id_zero]
            .timestamp
            .as_ref()
            .map(|value| value.timestamp()),
        Some(1_700_000_000)
    );
    assert_eq!(
        snapshot.systems[&id_nine]
            .timestamp
            .as_ref()
            .map(|value| value.timestamp()),
        Some(1_700_000_000)
    );
    assert_eq!(
        snapshot.player_address,
        GalacticAddress::new(100, 50, -200, 42, 0, 0)
    );
    assert_eq!(
        snapshot.systems,
        std::collections::HashMap::from([
            (
                id_zero,
                System::new(
                    galaxy_zero,
                    None,
                    Some("Explorer".into()),
                    Some("2023-11-14T22:13:20Z".parse().unwrap()),
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        None,
                        Some(0xDEAD1234)
                    ),]
                )
            ),
            (
                id_nine,
                System::new(
                    galaxy_nine,
                    None,
                    Some("Nine Explorer".into()),
                    Some("2023-11-14T22:13:20Z".parse().unwrap()),
                    vec![Planet::new(
                        1,
                        Some(Biome::Toxic),
                        None,
                        false,
                        None,
                        Some(0xFACE1234)
                    ),]
                )
            ),
        ])
    );
    assert_eq!(
        snapshot.planets,
        std::collections::HashMap::from([
            (
                (id_zero, 1),
                Planet::new(1, Some(Biome::Lush), None, false, None, Some(0xDEAD1234))
            ),
            (
                (id_nine, 1),
                Planet::new(1, Some(Biome::Toxic), None, false, None, Some(0xFACE1234))
            ),
        ])
    );
    assert_eq!(
        snapshot.bases,
        std::collections::HashMap::from([
            (
                "homestead alpha".to_owned(),
                PlayerBase::new(
                    "Homestead Alpha".into(),
                    BaseType::HomePlanetBase,
                    galaxy_zero,
                    [100.0, 200.0, 300.0],
                    Some("76561198025707979".into())
                )
            ),
            (
                "nine base".to_owned(),
                PlayerBase::new(
                    "Nine Base".into(),
                    BaseType::HomePlanetBase,
                    galaxy_nine,
                    [9.0, 8.0, 7.0],
                    Some("nine-owner".into())
                )
            ),
        ])
    );
}

#[test]
fn test_snapshot_file_path_rejects_late_bad_address_before_publishing_any_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid-address.json");
    std::fs::write(&path, invalid_base_address(MINIMAL)).unwrap();
    let error = SaveSnapshot::from_file(&path).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<nms_save::SaveError>(),
        Some(nms_save::SaveError::Address(AddressDecodeError::PackedWidth { path }))
            if path == "/BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress"
    ));
}

#[test]
fn test_semantic_address_failures_preserve_last_good_state_until_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    std::fs::write(&path, MINIMAL).unwrap();
    let watcher = nms_watch::start_watching_updates(nms_watch::WatchConfig {
        save_path: path.clone(),
        debounce: Duration::from_millis(20),
    })
    .unwrap();

    for attempt in 0..5 {
        let version = 4730 + attempt;
        let units = 700 + attempt;
        let nanites = 800 + attempt;
        let specials = 900 + attempt;
        let mut bytes = with_valid_ssi6_records(MINIMAL);
        for (from, to) in [
            ("\"Version\": 4720", format!("\"Version\": {version}")),
            ("\"Units\": 5000000", format!("\"Units\": {units}")),
            ("\"Nanites\": 10000", format!("\"Nanites\": {nanites}")),
            ("\"Specials\": 500", format!("\"Specials\": {specials}")),
        ] {
            bytes = replace_once(&bytes, from, &to);
        }
        bytes = invalid_base_address(&bytes);
        assert!(nms_save::parse_save(&bytes).is_ok());
        assert!(matches!(
            nms_save::parse_validated_save(&bytes),
            Err(nms_save::SaveError::Address(
                AddressDecodeError::PackedWidth { path }
            )) if path == "/BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress"
        ));
        let replacement = dir.path().join(format!("replacement-{attempt}.json"));
        let mut file = std::fs::File::create(&replacement).unwrap();
        file.write_all(&bytes).unwrap();
        file.sync_all().unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let observed_invalid_mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        // This bounded window proves no update was published; it does not prove
        // that the watcher parsed this particular replacement.
        assert!(matches!(
            watcher.receiver.recv_timeout(Duration::from_millis(400)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            observed_invalid_mtime
        );
    }

    let mut bytes = MINIMAL.to_vec();
    for (from, to) in [
        ("\"Version\": 4720", "\"Version\": 4740"),
        ("\"VoxelX\": 100", "\"VoxelX\": 101"),
        ("\"Units\": 5000000", "\"Units\": 700"),
        ("\"Nanites\": 10000", "\"Nanites\": 800"),
        ("\"Specials\": 500", "\"Specials\": 900"),
    ] {
        bytes = replace_once(&bytes, from, to);
    }
    let replacement = dir.path().join("recovery.json");
    std::fs::write(&replacement, &bytes).unwrap();
    std::fs::rename(replacement, &path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let recovery_mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let update = watcher
        .receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("valid recovery must update from the original last-good source");
    assert_eq!(update.save_version, 4740);
    let moved = update
        .delta
        .player_moved
        .as_ref()
        .expect("valid recovery must report the exact player movement");
    assert_eq!(moved.from, GalacticAddress::new(100, 50, -200, 42, 0, 0));
    assert_eq!(moved.to, GalacticAddress::new(101, 50, -200, 42, 0, 0));
    assert_eq!(
        update.player_state,
        PlayerState::new(
            GalacticAddress::new(101, 50, -200, 42, 0, 0),
            0,
            Some(GalacticAddress::new(80, 30, -150, 38, 0, 0)),
            None,
            700,
            800,
            900,
        )
    );
    assert!(update.delta.new_systems.is_empty());
    assert!(update.delta.new_planets.is_empty());
    assert!(update.delta.new_bases.is_empty());
    assert!(update.delta.modified_bases.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        recovery_mtime
    );
}
