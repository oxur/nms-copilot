use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use tempfile::TempDir;

fn root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn checked_cli_source() -> Value {
    let mut source: Value =
        serde_json::from_slice(include_bytes!("../../../data/test/save56_minimal.json")).unwrap();
    source["CommonStateData"]["SaveName"] = json!("Checked CLI Save");
    source["BaseContext"]["PlayerStateData"]["UniverseAddress"] = json!({
        "RealityIndex": 9,
        "GalacticAddress": {
            "VoxelX": 1130, "VoxelY": 4, "VoxelZ": -1244,
            "SolarSystemIndex": 299, "PlanetIndex": 2
        }
    });
    source["BaseContext"]["PlayerStateData"]
        .as_object_mut()
        .unwrap()
        .remove("PreviousUniverseAddress");
    source["BaseContext"]["PlayerStateData"]["Units"] = json!(-7);
    source["BaseContext"]["PlayerStateData"]["Nanites"] = json!(8);
    source["BaseContext"]["PlayerStateData"]["Specials"] = json!(9);
    let bases = source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"]
        .as_array_mut()
        .unwrap();
    bases[0]["GalacticAddress"] = json!("0x212B0904B2446A");
    bases[0]["Name"] = json!("Checked Base");
    bases[0]["Owner"]["UID"] = json!("fixture-owner");
    bases[0]["Position"] = json!([1.0, 2.0, 3.0]);

    let row = |address: &str, kind: &str, vp: Value| {
        json!({
            "DD": {"UA": address, "DT": kind, "VP": vp},
            "DM": {},
            "OWS": {"LID":"", "UID":"fixture", "USN":"Checked", "PTK":"ST", "TS":1700000000},
            "FL": {"U":1}
        })
    };
    source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([
        row("0x012B0904B2446A", "SolarSystem", json!([])),
        row("0x212B0904B2446A", "Planet", json!(["0xAB", 1])),
        row("0x012B0004B2446A", "SolarSystem", json!([])),
        row("0x212B0004B2446A", "Planet", json!(["0xAC", 0])),
        row("0x012C0904B2446B", "SolarSystem", json!([])),
        row("0x112C0904B2446B", "Planet", json!(["0xAD", 0])),
    ]);
    source
}

fn write_source(dir: &TempDir, name: &str, value: &Value) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn run(args: &[String]) -> std::process::Output {
    cargo_bin_cmd!("nms").args(args).output().unwrap()
}

fn assert_failure_without_success_payload(args: &[String], expected_path: &str) {
    let output = run(args);
    assert!(!output.status.success(), "unexpected success for {args:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stdout.is_empty(),
        "partial success payload for {args:?}: {stdout}"
    );
    assert!(
        stderr.contains(expected_path),
        "missing exact diagnostic path for {args:?}: {stderr}"
    );
    assert!(
        !stderr.contains("Imported "),
        "import reported success for {args:?}: {stderr}"
    );
}

fn args_with_save(command: &str, save: &Path) -> Vec<String> {
    vec![
        command.to_owned(),
        "--save".to_owned(),
        save.to_string_lossy().into_owned(),
    ]
}

fn normalized_tokens(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut visible = String::with_capacity(line.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
            index += 2;
            while index < bytes.len() && bytes[index] != b'm' {
                index += 1;
            }
            index = (index + 1).min(bytes.len());
        } else {
            let ch = line[index..].chars().next().unwrap();
            visible.push(if ch.is_alphanumeric() || ch == '-' {
                ch
            } else {
                ' '
            });
            index += ch.len_utf8();
        }
    }
    visible.split_whitespace().map(str::to_owned).collect()
}

fn portal_projection(value: &mut Value) {
    fn expand(value: &mut Value) {
        let Some(text) = value.as_str() else {
            panic!("curated packed address must remain a string: {value}");
        };
        let digits = text
            .strip_prefix("0x")
            .expect("literal lowercase 0x prefix");
        assert_eq!(digits.len(), 12, "curated 48-bit source address: {text}");
        assert!(digits.bytes().all(|byte| byte.is_ascii_hexdigit()));
        *value = json!(format!("0x{}00{}", &digits[..4], &digits[4..]));
    }

    let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .expect("curated discovery rows");
    for record in records {
        let kind = record["DD"]["DT"].as_str().expect("discovery type");
        match kind {
            "SolarSystem" | "Planet" => expand(&mut record["DD"]["UA"]),
            "Flora" | "Sector" => {
                let address = record["DD"]["UA"].as_str().expect("packed address");
                let digits = address
                    .strip_prefix("0x")
                    .expect("literal lowercase 0x prefix");
                assert_eq!(
                    digits.len(),
                    14,
                    "existing 56-bit address remains unchanged"
                );
                assert!(digits.bytes().all(|byte| byte.is_ascii_hexdigit()));
            }
            _ => {}
        }
    }

    for context in ["BaseContext", "ExpeditionContext"] {
        if let Some(bases) =
            value[context]["PlayerStateData"]["PersistentPlayerBases"].as_array_mut()
        {
            for base in bases {
                expand(&mut base["GalacticAddress"]);
            }
        }
    }
}

#[test]
fn test_actual_cli_callers_use_checked_save_and_keep_portal_and_signed_currency_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let valid = checked_cli_source();
    let save = write_source(&dir, "checked.json", &valid);
    let save_text = save.to_string_lossy().into_owned();

    let info = run(&args_with_save("info", &save));
    assert!(info.status.success());
    let info = String::from_utf8(info.stdout).unwrap();
    assert!(info.contains("Checked CLI Save"));
    assert!(info.contains("Eissentam"));
    assert!(info.contains("212B04B2446A"));
    assert!(info.contains("HomePlanetBase"));
    assert!(info.contains("-7"));

    let mut find = args_with_save("find", &save);
    find.extend(["--biome".into(), "Toxic".into()]);
    let find = run(&find);
    assert!(find.status.success());
    let find = String::from_utf8(find.stdout).unwrap();
    assert_eq!(find.matches("012B04B2446A").count(), 1, "{find}");
    assert_eq!(find.matches("Toxic").count(), 1, "{find}");
    assert!(
        !find.contains("012C04B2446B"),
        "unexpected second result: {find}"
    );
    assert_eq!(
        find.lines()
            .map(normalized_tokens)
            .filter(|tokens| {
                tokens.len() == 3
                    && tokens[0] == "Total"
                    && tokens[1] == "1"
                    && tokens[2] == "Results"
            })
            .count(),
        1,
        "exact one-result footer in:\n{find}"
    );

    let mut show = args_with_save("show", &save);
    show.extend(["base".into(), "Checked Base".into()]);
    let show = run(&show);
    assert!(show.status.success());
    let show = String::from_utf8(show.stdout).unwrap();
    assert!(show.contains("Checked Base"), "{show}");
    assert!(show.contains("212B04B2446A"), "{show}");
    assert!(show.contains("Eissentam"), "{show}");

    let stats = run(&args_with_save("stats", &save));
    assert!(stats.status.success());
    let stats = String::from_utf8(stats.stdout).unwrap();
    let mut core_count_rows = stats
        .lines()
        .map(normalized_tokens)
        .filter(|tokens| {
            tokens.len() == 2 && ["Systems", "Planets", "Bases"].contains(&tokens[0].as_str())
        })
        .collect::<Vec<_>>();
    core_count_rows.sort();
    assert_eq!(
        core_count_rows,
        [
            vec!["Bases".to_owned(), "1".to_owned()],
            vec!["Planets".to_owned(), "3".to_owned()],
            vec!["Systems".to_owned(), "3".to_owned()],
        ],
        "exact global count rows in:\n{stats}"
    );

    let mut route_args = args_with_save("route", &save);
    route_args.extend([
        "--biome".into(),
        "Lush".into(),
        "--algo".into(),
        "nn".into(),
    ]);
    let route = run(&route_args);
    assert!(route.status.success());
    let route = String::from_utf8(route.stdout).unwrap();
    assert!(route.contains("ROUTE ITINERARY"), "{route}");
    assert!(route.contains("Algorithm: nearest-neighbor"), "{route}");
    assert!(
        route.contains("Route: 1 targets, ~400 ly (±163) total"),
        "{route}"
    );
    assert!(route.contains("012B04B2446A"), "{route}");
    assert!(route.contains("012C04B2446B"), "{route}");

    let mut export_args = args_with_save("export", &save);
    export_args.extend(["--format".into(), "json".into()]);
    let export = run(&export_args);
    assert!(export.status.success());
    let exported: Vec<Value> = serde_json::from_slice(&export.stdout).unwrap();
    let mut export_rows = exported;
    export_rows.sort_by(|left, right| {
        left["portal_glyphs"]
            .as_str()
            .cmp(&right["portal_glyphs"].as_str())
    });
    let expected = |biome: &str, distance: f64, portal: &str, x: i64| {
        json!({
            "planet_name":"",
            "biome":biome,
            "system_name":"",
            "distance_ly":distance,
            "portal_glyphs":portal,
            "portal_emoji":nms_query::display::hex_to_emoji(portal),
            "coords_x":x,
            "coords_y":4,
            "coords_z":-1244,
            "galaxy":"Eissentam",
            "discoverer":"Checked",
            "infested":false
        })
    };
    assert_eq!(
        export_rows,
        vec![
            expected("Toxic", 0.0, "012B04B2446A", 1130),
            expected("Lush", 400.0, "012C04B2446B", 1131),
        ]
    );

    let mut list_bases = vec![
        "list".into(),
        "--save".into(),
        save_text.clone(),
        "bases".into(),
    ];
    let listed = run(&list_bases);
    assert!(listed.status.success());
    let listed = String::from_utf8(listed.stdout).unwrap();
    assert!(listed.contains("Checked Base"), "{listed}");
    assert!(listed.contains("Eissentam"), "{listed}");
    assert!(listed.contains("212B04B2446A"), "{listed}");
    list_bases.pop();
    list_bases.push("systems".into());
    let systems = run(&list_bases);
    assert!(systems.status.success());
    let systems = String::from_utf8(systems.stdout).unwrap();
    assert!(systems.contains("SYSTEMS"), "{systems}");

    let csv = dir.path().join("duplicate.csv");
    std::fs::write(
        &csv,
        "system_name,galaxy,portal_glyphs\nChecked Duplicate,Eissentam,012B04B2446A\n",
    )
    .unwrap();
    let imported = cargo_bin_cmd!("nms")
        .args(["import", csv.to_str().unwrap(), "--save", &save_text])
        .output()
        .unwrap();
    assert!(imported.status.success());
    assert!(imported.stdout.is_empty());
    let imported_stderr = String::from_utf8(imported.stderr).unwrap();
    assert_eq!(
        imported_stderr.trim(),
        "Imported 0 systems (1 duplicates, 0 skipped)"
    );
}

#[test]
fn test_new_cli_fixtures_are_exact_known_field_projections_of_original_jsons() {
    let root = root_path();
    for (original_name, checked_name) in [
        ("minimal_save.json", "save56_minimal.json"),
        ("multi_system_save.json", "save56_multi_system.json"),
    ] {
        let original_path = root.join("data/test").join(original_name);
        let checked_path = root.join("data/test").join(checked_name);
        let mut original: Value =
            serde_json::from_slice(&std::fs::read(&original_path).unwrap()).unwrap();
        let checked: Value =
            serde_json::from_slice(&std::fs::read(&checked_path).unwrap()).unwrap();
        portal_projection(&mut original);
        assert_eq!(
            original, checked,
            "faithful field-specific projection of {original_name}"
        );
    }
}

#[test]
fn test_every_actual_cli_save_consumer_rejects_late_legacy_accepted_57_bit_base() {
    let dir = tempfile::tempdir().unwrap();
    let mut invalid = checked_cli_source();
    invalid["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"]
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
    let save = write_source(&dir, "invalid-checked.json", &invalid);
    let invalid_bytes = std::fs::read(&save).unwrap();
    assert!(
        nms_save::parse_save(&invalid_bytes).is_ok(),
        "the exact CLI input remains accepted by the legacy u64 parser"
    );
    let csv = dir.path().join("duplicate.csv");
    std::fs::write(
        &csv,
        "system_name,galaxy,portal_glyphs\nChecked Duplicate,Eissentam,012B04B2446A\n",
    )
    .unwrap();
    let path = "/BaseContext/PlayerStateData/PersistentPlayerBases/1/GalacticAddress";
    assert!(matches!(
        nms_save::parse_validated_save(&invalid_bytes),
        Err(nms_save::SaveError::Address(
            nms_save::address::AddressDecodeError::PackedWidth { path: actual }
        )) if actual == path
    ));
    let mut commands = vec![
        args_with_save("info", &save),
        args_with_save("find", &save),
        args_with_save("stats", &save),
        args_with_save("export", &save),
        args_with_save("route", &save),
    ];
    let mut show = args_with_save("show", &save);
    show.extend(["base".into(), "Checked Base".into()]);
    commands.push(show);
    commands.push(vec![
        "list".into(),
        "--save".into(),
        save.to_string_lossy().into_owned(),
        "bases".into(),
    ]);
    commands.push(vec![
        "import".into(),
        csv.to_string_lossy().into_owned(),
        "--save".into(),
        save.to_string_lossy().into_owned(),
    ]);
    for command in commands {
        assert_failure_without_success_payload(&command, path);
    }
}
