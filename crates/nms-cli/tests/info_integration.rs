use nms_save::parse_save;

/// Build a minimal valid save JSON and verify it parses correctly.
#[test]
fn parse_minimal_save_and_verify_fields() {
    let json = r#"{
        "Version": 4720,
        "Platform": "Mac|Final",
        "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test Save", "TotalPlayTime": 3661},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {
                    "RealityIndex": 0,
                    "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 2}
                },
                "Units": 5000000,
                "Nanites": 10000,
                "Specials": 500,
                "PersistentPlayerBases": [
                    {
                        "BaseVersion": 8,
                        "GalacticAddress": "0x40050003AB8C07",
                        "Position": [100.0, 200.0, 300.0],
                        "Forward": [1.0, 0.0, 0.0],
                        "LastUpdateTimestamp": 1700000000,
                        "Objects": [],
                        "RID": "",
                        "Owner": {"LID": "", "UID": "12345", "USN": "TestUser", "PTK": "ST", "TS": 0},
                        "Name": "Test Base",
                        "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
                        "LastEditedById": "",
                        "LastEditedByUsername": ""
                    }
                ]
            }
        },
        "ExpeditionContext": {
            "GameMode": 6,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": []
            }
        },
        "DiscoveryManagerData": {
            "DiscoveryData-v1": {
                "ReserveStore": 100, "ReserveManaged": 100,
                "Store": {
                    "Record": [
                        {"DD": {"UA": "0x513300F79B1D82", "DT": "Flora", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                        {"DD": {"UA": "0x40050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                        {"DD": {"UA": 12345678, "DT": "Planet", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
                    ]
                }
            }
        }
    }"#;

    let save = parse_save(json.as_bytes()).unwrap();

    assert_eq!(save.version, 4720);
    assert_eq!(save.common_state_data.save_name, "Test Save");
    assert_eq!(save.common_state_data.total_play_time, 3661);

    let ps = save.active_player_state();
    assert_eq!(ps.units, 5000000);
    assert_eq!(ps.nanites, 10000);
    assert_eq!(ps.specials, 500);
    assert_eq!(ps.persistent_player_bases.len(), 1);
    assert_eq!(ps.persistent_player_bases[0].name, "Test Base");

    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    assert_eq!(records.len(), 3);

    let flora_count = records.iter().filter(|r| r.dd.dt == "Flora").count();
    let system_count = records
        .iter()
        .filter(|r| r.dd.dt == "SolarSystem")
        .count();
    let planet_count = records.iter().filter(|r| r.dd.dt == "Planet").count();
    assert_eq!(flora_count, 1);
    assert_eq!(system_count, 1);
    assert_eq!(planet_count, 1);
}
