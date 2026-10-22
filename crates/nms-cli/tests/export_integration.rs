use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use tempfile::TempDir;

const HEADER: &str = "planet_name,biome,system_name,distance_ly,portal_glyphs,portal_emoji,coords_x,coords_y,coords_z,galaxy,discoverer,infested\n";
const A_PORTAL: &str = "000100000000";
const A_EMOJI: &str = "🌅🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🌅";
const B_PORTAL: &str = "000204000003";
const B_EMOJI: &str = "🌅🌅🌅😑🌅🌜🌅🌅🌅🌅🌅🦕";
const C_PORTAL: &str = "000304000FFD";
const C_EMOJI: &str = "🌅🌅🌅🦕🌅🌜🌅🌅🌅🔺🔺🚀";

#[derive(Clone, Copy)]
struct Row {
    planet: &'static str,
    biome: &'static str,
    system: &'static str,
    distance: f64,
    portal: &'static str,
    emoji: &'static str,
    x: i64,
    y: i64,
    z: i64,
    galaxy: &'static str,
    discoverer: &'static str,
    infested: bool,
}

const A1: Row = Row {
    planet: "Ocean, \"α\"\nMoon",
    biome: "Lush",
    system: "Alpha, \"δ\"\nHome",
    distance: 0.0,
    portal: A_PORTAL,
    emoji: A_EMOJI,
    x: 0,
    y: 0,
    z: 0,
    galaxy: "Euclid",
    discoverer: "Explorer, \"δ\"\nOne",
    infested: false,
};
const A2: Row = Row {
    planet: "",
    biome: "Toxic",
    system: "Alpha, \"δ\"\nHome",
    distance: 0.0,
    portal: A_PORTAL,
    emoji: A_EMOJI,
    x: 0,
    y: 0,
    z: 0,
    galaxy: "Euclid",
    discoverer: "Explorer, \"δ\"\nOne",
    infested: true,
};
const A3: Row = A1;
const B1: Row = Row {
    planet: "Meadow",
    biome: "Lush",
    system: "Beta",
    distance: 2000.0,
    portal: B_PORTAL,
    emoji: B_EMOJI,
    x: 3,
    y: 4,
    z: 0,
    galaxy: "Euclid",
    discoverer: "Bob Builder",
    infested: false,
};
const C1: Row = Row {
    planet: "",
    biome: "",
    system: "",
    distance: 2000.0,
    portal: C_PORTAL,
    emoji: C_EMOJI,
    x: -3,
    y: 4,
    z: 0,
    galaxy: "Euclid",
    discoverer: "",
    infested: false,
};
const N1: Row = Row {
    planet: "Nine Moon",
    biome: "Frozen",
    system: "Nine Decoy",
    distance: 0.0,
    portal: B_PORTAL,
    emoji: B_EMOJI,
    x: 3,
    y: 4,
    z: 0,
    galaxy: "Eissentam",
    discoverer: "NineOwner",
    infested: false,
};

const B_ANCHOR: Row = Row {
    distance: 0.0,
    ..B1
};
const A1_ANCHOR: Row = Row {
    distance: 2000.0,
    ..A1
};
const A2_ANCHOR: Row = Row {
    distance: 2000.0,
    ..A2
};
const A3_ANCHOR: Row = Row {
    distance: 2000.0,
    ..A3
};
const C_ANCHOR: Row = Row {
    distance: 2400.0,
    ..C1
};

fn row_json(row: Row) -> Value {
    json!({
        "planet_name": row.planet, "biome": row.biome,
        "system_name": row.system, "distance_ly": row.distance,
        "portal_glyphs": row.portal, "portal_emoji": row.emoji,
        "coords_x": row.x, "coords_y": row.y, "coords_z": row.z,
        "galaxy": row.galaxy, "discoverer": row.discoverer,
        "infested": row.infested
    })
}

fn csv_row(row: Row) -> Vec<String> {
    vec![
        row.planet.into(),
        row.biome.into(),
        row.system.into(),
        format!("{:.1}", row.distance),
        row.portal.into(),
        row.emoji.into(),
        row.x.to_string(),
        row.y.to_string(),
        row.z.to_string(),
        row.galaxy.into(),
        row.discoverer.into(),
        row.infested.to_string(),
    ]
}

fn source_row(
    address: &str,
    kind: &str,
    vp: Value,
    name: Option<&str>,
    owner: Option<&str>,
) -> Value {
    let mut dm = serde_json::Map::new();
    if let Some(name) = name {
        dm.insert("CN".into(), json!(name));
    }
    let mut row = json!({
        "DD": {"UA": address, "DT": kind, "VP": vp},
        "DM": dm,
        "OWS": {"LID":"", "UID":"export-fixture", "USN": owner.unwrap_or(""), "PTK":"ST", "TS":1700000000},
        "FL": {"U":1}
    });
    if name.is_none() {
        row["DM"].as_object_mut().unwrap().remove("CN");
    }
    row
}

fn base_source(name: &str, address: &str, uid: &str, template: &Value) -> Value {
    let mut base = template.clone();
    base["Name"] = json!(name);
    base["GalacticAddress"] = json!(address);
    base["Owner"]["UID"] = json!(uid);
    base
}

fn fixture() -> Value {
    let mut source: Value =
        serde_json::from_slice(include_bytes!("../../../data/test/save56_minimal.json")).unwrap();
    source["Version"] = json!(4720);
    source["BaseContext"]["PlayerStateData"]["UniverseAddress"] = json!({"RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2}});
    let player = source["BaseContext"]["PlayerStateData"]
        .as_object_mut()
        .unwrap();
    player.remove("PreviousUniverseAddress");
    let template = player["PersistentPlayerBases"][0].clone();
    player["PersistentPlayerBases"] = json!([
        base_source("Anchor", "0x10020004000003", "anchor-owner", &template),
        base_source("Nine Base", "0x10020904000003", "nine-owner", &template)
    ]);
    let rows = vec![
        source_row(
            "0x00010000000000",
            "SolarSystem",
            json!([]),
            Some("Alpha, \"δ\"\nHome"),
            Some("Explorer, \"δ\"\nOne"),
        ),
        source_row(
            "0x10010000000000",
            "Planet",
            json!(["0xAB", 0]),
            Some("Ocean, \"α\"\nMoon"),
            Some("PlanetOwner"),
        ),
        source_row(
            "0x20010000000000",
            "Planet",
            json!(["0xAD", 65537]),
            None,
            Some("PlanetOwner"),
        ),
        source_row(
            "0x30010000000000",
            "Planet",
            json!(["0xAC", 0]),
            Some("Ocean, \"α\"\nMoon"),
            Some("PlanetOwner"),
        ),
        source_row(
            "0x00020004000003",
            "SolarSystem",
            json!([]),
            Some("Beta"),
            Some("Bob Builder"),
        ),
        source_row(
            "0x10020004000003",
            "Planet",
            json!(["0xAE", 0]),
            Some("Meadow"),
            Some("PlanetOwner"),
        ),
        source_row("0x00030004000FFD", "SolarSystem", json!([]), None, Some("")),
        source_row(
            "0x10030004000FFD",
            "Planet",
            json!([]),
            None,
            Some("PlanetOwner"),
        ),
        source_row(
            "0x00020904000003",
            "SolarSystem",
            json!([]),
            Some("Nine Decoy"),
            Some("NineOwner"),
        ),
        source_row(
            "0x10020904000003",
            "Planet",
            json!(["0xAF", 4]),
            Some("Nine Moon"),
            Some("PlanetOwner"),
        ),
        // A contradictory duplicate must not replace the first A/P1 discovery.
        source_row(
            "0x10010000000000",
            "Planet",
            json!(["0xFF", 65536]),
            Some("Wrong Duplicate"),
            Some("WrongOwner"),
        ),
    ];
    source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!(rows);
    let records = source["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array()
        .unwrap();
    let addresses = [
        "0x00010000000000",
        "0x10010000000000",
        "0x20010000000000",
        "0x30010000000000",
        "0x00020004000003",
        "0x10020004000003",
        "0x00030004000FFD",
        "0x10030004000FFD",
        "0x00020904000003",
        "0x10020904000003",
        "0x10010000000000",
    ];
    assert_eq!(records.len(), addresses.len());
    for (record, address) in records.iter().zip(addresses) {
        assert_eq!(record["DD"]["UA"], address);
    }
    assert_eq!(
        source["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"],
        json!({
            "VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":1,"PlanetIndex":2
        })
    );
    assert_eq!(
        source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][0]["GalacticAddress"],
        "0x10020004000003"
    );
    assert_eq!(
        source["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"][1]["GalacticAddress"],
        "0x10020904000003"
    );
    source
}

struct ChildFixture {
    _dir: TempDir,
    save: PathBuf,
    home: PathBuf,
    source_bytes: Vec<u8>,
    entries: Vec<String>,
    tree: BTreeMap<String, Option<Vec<u8>>>,
}

impl ChildFixture {
    fn new(value: &Value) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("save.json");
        let home = dir.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let bytes = serde_json::to_vec_pretty(value).unwrap();
        std::fs::write(&save, &bytes).unwrap();
        let entries = names(dir.path());
        let tree = tree_snapshot(dir.path());
        Self {
            _dir: dir,
            save,
            home,
            source_bytes: bytes,
            entries,
            tree,
        }
    }

    fn run(&self, format: &str, flags: &[&str]) -> Output {
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&self.save)
            .arg("--format")
            .arg(format)
            .args(flags)
            .env("HOME", &self.home)
            .timeout(std::time::Duration::from_secs(10));
        let output = cmd.output().unwrap();
        self.assert_unchanged();
        output
    }

    fn assert_unchanged(&self) {
        assert_eq!(std::fs::read(&self.save).unwrap(), self.source_bytes);
        assert_eq!(names(self._dir.path()), self.entries);
        assert_eq!(tree_snapshot(self._dir.path()), self.tree);
    }
}

fn names(path: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn tree_snapshot(root: &Path) -> BTreeMap<String, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<String, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let child = entry.path();
            let relative = child
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if child.is_dir() {
                entries.insert(relative, None);
                visit(root, &child, entries);
            } else {
                entries.insert(relative, Some(std::fs::read(child).unwrap()));
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn assert_child_ok(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.contains(&0x1b), "ANSI escape in payload");
    output.stdout
}

fn assert_json(output: Output, expected: &[Row]) {
    let stdout = assert_child_ok(output);
    assert!(stdout.ends_with(b"\n"));
    let actual: Value = serde_json::from_slice(&stdout).unwrap();
    let expected: Vec<_> = expected.iter().copied().map(row_json).collect();
    assert_eq!(actual, json!(expected));
}

fn assert_csv(output: Output, expected: &[Row]) {
    let stdout = assert_child_ok(output);
    if expected.is_empty() {
        assert!(stdout.is_empty(), "empty CSV should be zero bytes");
        return;
    }
    let mut reader = csv::Reader::from_reader(stdout.as_slice());
    assert_eq!(
        reader.headers().unwrap().iter().collect::<Vec<_>>(),
        HEADER.trim_end().split(',').collect::<Vec<_>>()
    );
    let actual: Vec<Vec<String>> = reader
        .records()
        .map(|row| row.unwrap().iter().map(str::to_owned).collect())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(csv_row).collect();
    assert_eq!(actual, expected);
}

fn run_pair(flags: &[&str], expected: &[Row]) {
    let value = fixture();
    for format in ["json", "csv"] {
        let child = ChildFixture::new(&value);
        let output = child.run(format, flags);
        if format == "json" {
            assert_json(output, expected);
        } else {
            assert_csv(output, expected);
        }
    }
}

fn json_validates(actual: &Value, expected: &[Row]) -> bool {
    let rows: Vec<_> = expected.iter().copied().map(row_json).collect();
    actual == &json!(rows)
}

fn csv_validates(actual: &[Vec<String>], expected: &[Row]) -> bool {
    let rows: Vec<_> = expected.iter().copied().map(csv_row).collect();
    actual == rows
}

fn csv_payload_validates(bytes: &[u8], expected: &[Row]) -> bool {
    let mut reader = csv::Reader::from_reader(bytes);
    if reader.headers().map(|h| h.iter().collect::<Vec<_>>()).ok()
        != Some(HEADER.trim_end().split(',').collect())
    {
        return false;
    }
    let actual: Result<Vec<Vec<String>>, _> = reader
        .records()
        .map(|record| record.map(|r| r.iter().map(str::to_owned).collect()))
        .collect();
    actual.is_ok_and(|rows| csv_validates(&rows, expected))
}

fn csv_records(bytes: &[u8]) -> Vec<Vec<String>> {
    let mut reader = csv::Reader::from_reader(bytes);
    assert_eq!(reader.headers().unwrap().len(), 12);
    reader
        .records()
        .map(|row| row.unwrap().iter().map(str::to_owned).collect())
        .collect()
}

#[test]
fn test_export_literal_csv_bytes_and_json_oracle_reject_mutations() {
    let child = ChildFixture::new(&fixture());
    let bytes = assert_child_ok(child.run("csv", &[]));
    let golden = concat!(
        "planet_name,biome,system_name,distance_ly,portal_glyphs,portal_emoji,coords_x,coords_y,coords_z,galaxy,discoverer,infested\n",
        "\"Ocean, \"\"α\"\"\nMoon\",Lush,\"Alpha, \"\"δ\"\"\nHome\",0.0,000100000000,🌅🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🌅,0,0,0,Euclid,\"Explorer, \"\"δ\"\"\nOne\",false\n",
        ",Toxic,\"Alpha, \"\"δ\"\"\nHome\",0.0,000100000000,🌅🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🌅,0,0,0,Euclid,\"Explorer, \"\"δ\"\"\nOne\",true\n",
        "\"Ocean, \"\"α\"\"\nMoon\",Lush,\"Alpha, \"\"δ\"\"\nHome\",0.0,000100000000,🌅🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🌅,0,0,0,Euclid,\"Explorer, \"\"δ\"\"\nOne\",false\n",
        "Meadow,Lush,Beta,2000.0,000204000003,🌅🌅🌅😑🌅🌜🌅🌅🌅🌅🌅🦕,3,4,0,Euclid,Bob Builder,false\n",
        ",,,2000.0,000304000FFD,🌅🌅🌅🦕🌅🌜🌅🌅🌅🔺🔺🚀,-3,4,0,Euclid,,false\n"
    );
    assert_eq!(bytes, golden.as_bytes());
    let decoded = csv_records(&bytes);
    assert!(csv_validates(&decoded, &[A1, A2, A3, B1, C1]));
    let mut changed_csv = decoded.clone();
    changed_csv[1][1] = "Frozen".into();
    assert!(!csv_validates(&changed_csv, &[A1, A2, A3, B1, C1]));
    let mut missing_duplicate = decoded.clone();
    missing_duplicate.remove(2);
    assert!(!csv_validates(&missing_duplicate, &[A1, A2, A3, B1, C1]));
    let mut extra_duplicate = decoded.clone();
    extra_duplicate.insert(3, decoded[0].clone());
    assert!(!csv_validates(&extra_duplicate, &[A1, A2, A3, B1, C1]));
    let mut reordered_csv = decoded.clone();
    reordered_csv.swap(0, 3);
    assert!(!csv_validates(&reordered_csv, &[A1, A2, A3, B1, C1]));
    let mut wrong_header = bytes.clone();
    wrong_header.splice(0.."planet_name".len(), b"planet_nameX".iter().copied());
    assert!(!csv_payload_validates(&wrong_header, &[A1, A2, A3, B1, C1]));
    let mut reordered_header = bytes.clone();
    reordered_header[0.."planet_name".len()].copy_from_slice(b"system_name");
    let system_name_start = "planet_name".len() + 1 + "biome".len() + 1;
    reordered_header[system_name_start..system_name_start + "system_name".len()]
        .copy_from_slice(b"planet_name");
    assert!(!csv_payload_validates(
        &reordered_header,
        &[A1, A2, A3, B1, C1]
    ));
    let mut malformed_escape = bytes.clone();
    let malformed_quote = malformed_escape
        .windows(b"Moon\",Lush".len())
        .position(|window| window == b"Moon\",Lush")
        .unwrap()
        + 4;
    malformed_escape.remove(malformed_quote);
    assert_ne!(malformed_escape, golden.as_bytes());
    assert!(!csv_payload_validates(
        &malformed_escape,
        &[A1, A2, A3, B1, C1]
    ));
    assert!(csv_payload_validates(&bytes, &[A1, A2, A3, B1, C1]));

    let child = ChildFixture::new(&fixture());
    let output = assert_child_ok(child.run("json", &[]));
    let observed: Value = serde_json::from_slice(&output).unwrap();
    let expected = [A1, A2, A3, B1, C1];
    assert!(json_validates(&observed, &expected));
    let fields = [
        ("planet_name", json!("different")),
        ("biome", json!("Frozen")),
        ("system_name", json!("different")),
        ("distance_ly", json!(1.0)),
        ("portal_glyphs", json!("FFFFFFFFFFFF")),
        ("portal_emoji", json!("different")),
        ("coords_x", json!(42)),
        ("coords_y", json!(42)),
        ("coords_z", json!(42)),
        ("galaxy", json!("Hilbert Dimension")),
        ("discoverer", json!("different")),
        ("infested", json!(true)),
    ];
    for (field, replacement) in fields {
        let mut mutant = observed.clone();
        mutant[0][field] = replacement;
        assert!(
            !json_validates(&mutant, &expected),
            "oracle accepted changed {field}"
        );
    }
    let mut wrong_type = observed.clone();
    wrong_type[1]["infested"] = json!("true");
    assert!(!json_validates(&wrong_type, &expected));
    let mut missing_key = observed.clone();
    missing_key[0].as_object_mut().unwrap().remove("galaxy");
    assert!(!json_validates(&missing_key, &expected));
    let mut extra_key = observed.clone();
    extra_key[0]["unexpected"] = json!(true);
    assert!(!json_validates(&extra_key, &expected));
    let mut reordered = observed.clone();
    reordered.as_array_mut().unwrap().swap(0, 3);
    assert!(!json_validates(&reordered, &expected));
    let mut dropped = observed.clone();
    dropped.as_array_mut().unwrap().remove(2);
    assert!(!json_validates(&dropped, &expected));
    let mut extra_duplicate = observed.clone();
    extra_duplicate
        .as_array_mut()
        .unwrap()
        .push(observed[0].clone());
    assert!(!json_validates(&extra_duplicate, &expected));
    let mut wrong_projection = observed.clone();
    wrong_projection[0]["portal_glyphs"] = json!("100100000000");
    assert!(!json_validates(&wrong_projection, &expected));
    let mut wrong_system_alias = observed.clone();
    wrong_system_alias[0]["system_name"] = json!("Nine Decoy");
    assert!(!json_validates(&wrong_system_alias, &expected));
    let mut wrong_galaxy_alias = observed.clone();
    wrong_galaxy_alias[0]["galaxy"] = json!("Eissentam");
    assert!(!json_validates(&wrong_galaxy_alias, &expected));
    let mut leaked = observed.clone();
    leaked.as_array_mut().unwrap().push(row_json(N1));
    assert!(!json_validates(&leaked, &expected));
}

#[test]
fn test_export_defaults_uppercase_and_independent_empty_source() {
    let source = fixture();
    let expected_rows = [A1, A2, A3, B1, C1];
    for format in ["JSON", "CSV"] {
        let child = ChildFixture::new(&source);
        let actual = child.run(format, &[]);
        let actual_bytes = actual.stdout.clone();
        if format == "JSON" {
            assert_json(actual, &expected_rows);
        } else {
            assert_csv(actual, &expected_rows);
        }
        let expected_child = ChildFixture::new(&source);
        let explicit = expected_child.run(&format.to_ascii_lowercase(), &[]);
        assert_eq!(actual_bytes, explicit.stdout);
        if format == "JSON" {
            assert_json(explicit, &expected_rows);
        } else {
            assert_csv(explicit, &expected_rows);
        }
    }
    let child = ChildFixture::new(&source);
    let mut command = cargo_bin_cmd!("nms");
    command
        .arg("export")
        .arg("--save")
        .arg(&child.save)
        .env("HOME", &child.home)
        .timeout(std::time::Duration::from_secs(10));
    let default_output = command.output().unwrap();
    child.assert_unchanged();
    let default_bytes = default_output.stdout.clone();
    assert_json(default_output, &expected_rows);
    let explicit = ChildFixture::new(&source);
    let explicit_output = explicit.run("json", &[]);
    assert_eq!(default_bytes, explicit_output.stdout);
    assert_json(explicit_output, &expected_rows);
    assert!(String::from_utf8(default_bytes).unwrap().starts_with("[\n"));

    let mut empty = fixture();
    empty["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([]);
    empty["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"] = json!([]);
    let json_child = ChildFixture::new(&empty);
    assert_eq!(assert_child_ok(json_child.run("json", &[])), b"[]\n");
    let csv_child = ChildFixture::new(&empty);
    assert!(assert_child_ok(csv_child.run("csv", &[])).is_empty());
}

fn assert_child_error(output: Output, code: i32, needle: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "unexpected status; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "error path emitted stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(needle), "missing {needle:?} in {stderr:?}");
    assert!(
        !stderr.to_ascii_lowercase().contains("panicked"),
        "child panic: {stderr}"
    );
}

fn error_args(format: &str, save: &Path) -> Vec<String> {
    vec![
        "export".into(),
        "--save".into(),
        save.to_string_lossy().into_owned(),
        "--format".into(),
        format.into(),
    ]
}

#[test]
fn test_export_errors_are_stderr_only_and_leave_source_unchanged() {
    for format in ["json", "csv"] {
        let cases: Vec<(Value, Vec<&str>, &str)> = vec![
            (fixture(), vec!["--biome", "NotABiome"], "Invalid biome"),
            (fixture(), vec!["--from", "Missing Base"], "Missing Base"),
            (fixture(), vec!["--nearest", "nope"], "invalid value"),
        ];
        for (source, flags, message) in cases {
            let child = ChildFixture::new(&source);
            let mut args = error_args(format, &child.save);
            args.extend(flags.iter().map(|s| (*s).to_owned()));
            let mut command = cargo_bin_cmd!("nms");
            command
                .args(&args)
                .env("HOME", &child.home)
                .timeout(std::time::Duration::from_secs(10));
            let output = command.output().unwrap();
            let expected = if flags.contains(&"--nearest") { 2 } else { 1 };
            assert_child_error(output, expected, message);
            assert_eq!(std::fs::read(&child.save).unwrap(), child.source_bytes);
            assert_eq!(names(child._dir.path()), child.entries);
        }

        let child = ChildFixture::new(&fixture());
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&child.save)
            .arg("--format")
            .arg("xml")
            .env("HOME", &child.home)
            .timeout(std::time::Duration::from_secs(10));
        assert_child_error(cmd.output().unwrap(), 1, "unknown format");
        assert_eq!(std::fs::read(&child.save).unwrap(), child.source_bytes);
        assert_eq!(names(child._dir.path()), child.entries);

        let child = ChildFixture::new(&fixture());
        let missing = child._dir.path().join("does-not-exist.json");
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&missing)
            .arg("--format")
            .arg(format)
            .env("HOME", &child.home)
            .timeout(std::time::Duration::from_secs(10));
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("Error: I/O error:"), "{stderr}");
        assert!(!stderr["Error: I/O error:".len()..].trim().is_empty());
        assert_eq!(std::fs::read(&child.save).unwrap(), child.source_bytes);
        assert_eq!(names(child._dir.path()), child.entries);

        let mut malformed = fixture();
        malformed["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([]);
        let child = ChildFixture::new(&malformed);
        let malformed_bytes = b"{ definitely not json";
        std::fs::write(&child.save, malformed_bytes).unwrap();
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&child.save)
            .arg("--format")
            .arg(format)
            .env("HOME", &child.home)
            .timeout(std::time::Duration::from_secs(10));
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr).unwrap().contains("JSON"));
        assert_eq!(std::fs::read(&child.save).unwrap(), malformed_bytes);
        assert_eq!(names(child._dir.path()), child.entries);

        let mut wide = fixture();
        wide["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([source_row(
            "0x100000000000000",
            "SolarSystem",
            json!([]),
            Some("Too Wide"),
            Some("Owner")
        )]);
        let child = ChildFixture::new(&wide);
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&child.save)
            .arg("--format")
            .arg(format)
            .env("HOME", &child.home)
            .timeout(std::time::Duration::from_secs(10));
        assert_child_error(
            cmd.output().unwrap(),
            1,
            "packed address exceeds 56 bits at /DiscoveryManagerData/DiscoveryData-v1/Store/Record/0/DD/UA",
        );
        assert_eq!(std::fs::read(&child.save).unwrap(), child.source_bytes);
        assert_eq!(names(child._dir.path()), child.entries);

        let mut missing_current = fixture();
        missing_current["BaseContext"]["PlayerStateData"]
            .as_object_mut()
            .unwrap()
            .remove("UniverseAddress");
        let child = ChildFixture::new(&missing_current);
        let mut cmd = cargo_bin_cmd!("nms");
        cmd.arg("export")
            .arg("--save")
            .arg(&child.save)
            .arg("--format")
            .arg(format)
            .env("HOME", &child.home)
            .timeout(std::time::Duration::from_secs(10));
        assert_child_error(
            cmd.output().unwrap(),
            1,
            "/BaseContext/PlayerStateData/UniverseAddress",
        );
        assert_eq!(std::fs::read(&child.save).unwrap(), child.source_bytes);
        assert_eq!(names(child._dir.path()), child.entries);
    }
}

#[test]
fn test_export_real_child_filter_order_reference_and_empty_matrix() {
    run_pair(&[], &[A1, A2, A3, B1, C1]);
    run_pair(&["--biome", "Lush"], &[A1, A3, B1]);
    run_pair(&["--infested"], &[A2]);
    run_pair(&["--named"], &[A1, A3, B1]);
    run_pair(&["--discoverer", "eXpLoReR"], &[A1, A2, A3]);
    run_pair(&["--discoverer", "bUiLd"], &[B1]);
    run_pair(&["--within", "1999"], &[A1, A2, A3]);
    run_pair(&["--within", "2000"], &[A1, A2, A3, B1, C1]);
    run_pair(&["--nearest", "4"], &[A1, A2, A3, B1]);
    run_pair(&["--nearest", "0"], &[]);
    run_pair(
        &[
            "--biome",
            "Lush",
            "--named",
            "--discoverer",
            "bOb",
            "--within",
            "2000",
            "--nearest",
            "1",
        ],
        &[B1],
    );
    run_pair(
        &["--from", "Anchor"],
        &[B_ANCHOR, A1_ANCHOR, A2_ANCHOR, A3_ANCHOR, C_ANCHOR],
    );
    run_pair(
        &["--from", "Anchor", "--within", "0", "--nearest", "1"],
        &[B_ANCHOR],
    );
    run_pair(&["--from", "Nine Base"], &[N1]);
    run_pair(&["--from", "Nine Base", "--biome", "Lush"], &[]);
    run_pair(&["--biome", "Dead"], &[]);
    run_pair(&["--discoverer", "Absent"], &[]);
    run_pair(&["--within=-1"], &[]);
    run_pair(&["--within", "NaN"], &[]);
    run_pair(&["--within", "NaN", "--nearest", "2"], &[A1, A2]);
    run_pair(&["--within", "inf"], &[A1, A2, A3, B1, C1]);
}
