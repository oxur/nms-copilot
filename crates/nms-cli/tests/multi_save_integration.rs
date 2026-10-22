#![cfg(any(target_os = "macos", target_os = "linux"))]

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, FileTimes};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const SAVE56_FIXTURE: &str = include_str!("../../../data/test/save56_minimal.json");
const IMPORT_CSV: &[u8] = b"System Name,Portal Glyphs,Galaxy\nImported A,050003AB8C07,0\n";

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntrySnapshot {
    kind: &'static str,
    modified: SystemTime,
    bytes: Option<Vec<u8>>,
    symlink_target: Option<PathBuf>,
}

struct Fixture {
    temp: TempDir,
    home: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(home.join(".config")).unwrap();
        fs::create_dir_all(home.join(".local/share")).unwrap();
        fs::create_dir_all(home.join(".cache")).unwrap();
        Self { temp, home }
    }

    fn save_root(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        let root = self.home.join("Library/Application Support/HelloGames/NMS");
        #[cfg(target_os = "linux")]
        let root = self.home.join(
            ".local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/AppData/Roaming/HelloGames/NMS",
        );
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let root = self.home.join("Library/Application Support/HelloGames/NMS");
        root
    }

    fn create_save_root(&self) -> PathBuf {
        let root = self.save_root();
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn run(&self, args: &[OsString]) -> Output {
        let before = snapshot_tree(self.temp.path()).unwrap();
        let mut command = cargo_bin_cmd!("nms");
        command
            .args(args)
            .current_dir(self.temp.path())
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("XDG_DATA_HOME", self.home.join(".local/share"))
            .env("XDG_CACHE_HOME", self.home.join(".cache"))
            .timeout(Duration::from_secs(10));
        let output = command.output().unwrap();
        let after = snapshot_tree(self.temp.path()).unwrap();
        assert_eq!(
            before, after,
            "child changed its owned fixture tree: {args:?}"
        );
        output
    }
}

fn snapshot_tree(root: &Path) -> io::Result<BTreeMap<PathBuf, EntrySnapshot>> {
    fn visit(
        root: &Path,
        dir: &Path,
        entries: &mut BTreeMap<PathBuf, EntrySnapshot>,
    ) -> io::Result<()> {
        let mut children = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(|entry| entry.file_name());
        for entry in children {
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let metadata = fs::symlink_metadata(&path)?;
            let file_type = metadata.file_type();
            let (kind, bytes, symlink_target) = if file_type.is_dir() {
                ("dir", None, None)
            } else if file_type.is_file() {
                ("file", Some(fs::read(&path)?), None)
            } else if file_type.is_symlink() {
                ("symlink", None, Some(fs::read_link(&path)?))
            } else {
                ("other", None, None)
            };
            entries.insert(
                relative,
                EntrySnapshot {
                    kind,
                    modified: metadata.modified()?,
                    bytes,
                    symlink_target,
                },
            );
            if file_type.is_dir() {
                visit(root, &path, entries)?;
            }
        }
        Ok(())
    }

    let mut entries = BTreeMap::new();
    let metadata = fs::symlink_metadata(root)?;
    entries.insert(
        PathBuf::new(),
        EntrySnapshot {
            kind: "dir",
            modified: metadata.modified()?,
            bytes: None,
            symlink_target: None,
        },
    );
    visit(root, root, &mut entries)?;
    Ok(entries)
}

fn set_modified(path: &Path, time: SystemTime) {
    File::open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(time))
        .unwrap();
}

fn literal_save(save_name: &str, units: i64, is_a: bool) -> Value {
    let mut value: Value = serde_json::from_str(SAVE56_FIXTURE).unwrap();
    value["CommonStateData"]["SaveName"] = json!(save_name);
    value["BaseContext"]["PlayerStateData"]["Units"] = json!(units);

    let player = &mut value["BaseContext"]["PlayerStateData"];
    player["UniverseAddress"]["RealityIndex"] = json!(0);
    let current = &mut player["UniverseAddress"]["GalacticAddress"];
    current["VoxelX"] = json!(-1017);
    current["VoxelY"] = json!(3);
    current["VoxelZ"] = json!(-1352);
    current["SolarSystemIndex"] = if is_a { json!(1280) } else { json!(1281) };
    current["PlanetIndex"] = json!(0);

    let base = &mut player["PersistentPlayerBases"][0];
    base["Name"] = json!(if is_a { "Base A" } else { "Base B" });
    base["GalacticAddress"] = json!(if is_a {
        "0x05000003AB8C07"
    } else {
        "0x05010003AB8C07"
    });

    let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .unwrap();
    let mut system = records[0].clone();
    let mut planet = records[1].clone();
    if is_a {
        system["DD"]["UA"] = json!("0x05000003AB8C07");
        system["DM"]["CN"] = json!("System A");
        planet["DD"]["UA"] = json!("0x15000003AB8C07");
        planet["DM"]["CN"] = json!("Planet A");
    } else {
        system["DD"]["UA"] = json!("0x05010003AB8C07");
        system["DM"]["CN"] = json!("System B");
        planet["DD"]["UA"] = json!("0x15010003AB8C07");
        planet["DM"]["CN"] = json!("Planet B");
    }

    let mut destination = system.clone();
    if is_a {
        destination["DD"]["UA"] = json!("0x05020003AB8C11");
        destination["DM"]["CN"] = json!("Destination A");
    } else {
        destination["DD"]["UA"] = json!("0x05030003AB8C1B");
        destination["DM"]["CN"] = json!("Destination B");
    }

    let mut output_records = vec![system, planet, destination];
    if !is_a {
        let mut second_planet = output_records[1].clone();
        second_planet["DD"]["UA"] = json!("0x25010003AB8C07");
        second_planet["DM"]["CN"] = json!("Planet B2");
        output_records.insert(2, second_planet);
    }
    *records = output_records;
    assert_save_literals(&value, is_a);
    value
}

fn assert_save_literals(value: &Value, is_a: bool) {
    let records = value["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array()
        .unwrap();
    let (system, planet, destination, count) = if is_a {
        (
            "0x05000003AB8C07",
            "0x15000003AB8C07",
            "0x05020003AB8C11",
            3,
        )
    } else {
        (
            "0x05010003AB8C07",
            "0x15010003AB8C07",
            "0x05030003AB8C1B",
            4,
        )
    };
    assert_eq!(
        value["CommonStateData"]["SaveName"],
        if is_a { "Save A" } else { "Save B" }
    );
    assert_eq!(
        value["BaseContext"]["PlayerStateData"]["Units"],
        if is_a { 101 } else { 202 }
    );
    assert_eq!(records.len(), count);
    let expected_records = if is_a {
        vec![
            ("SolarSystem", system, "System A"),
            ("Planet", planet, "Planet A"),
            ("SolarSystem", destination, "Destination A"),
        ]
    } else {
        vec![
            ("SolarSystem", system, "System B"),
            ("Planet", planet, "Planet B"),
            ("Planet", "0x25010003AB8C07", "Planet B2"),
            ("SolarSystem", destination, "Destination B"),
        ]
    };
    let actual_records: Vec<_> = records
        .iter()
        .map(|record| {
            (
                record["DD"]["DT"].as_str().unwrap(),
                record["DD"]["UA"].as_str().unwrap(),
                record["DM"]["CN"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(actual_records, expected_records);
    let address = &value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["GalacticAddress"];
    assert_eq!(
        value["BaseContext"]["PlayerStateData"]["UniverseAddress"]["RealityIndex"],
        0
    );
    assert_eq!(address["VoxelX"], -1017);
    assert_eq!(address["VoxelY"], 3);
    assert_eq!(address["VoxelZ"], -1352);
    assert_eq!(address["SolarSystemIndex"], if is_a { 1280 } else { 1281 });
    assert_eq!(address["PlanetIndex"], 0);

    let player = &value["BaseContext"]["PlayerStateData"];
    let previous = &player["PreviousUniverseAddress"];
    assert_eq!(previous["RealityIndex"], 0);
    assert_eq!(previous["GalacticAddress"]["VoxelX"], 80);
    assert_eq!(previous["GalacticAddress"]["VoxelY"], 30);
    assert_eq!(previous["GalacticAddress"]["VoxelZ"], -150);
    assert_eq!(previous["GalacticAddress"]["SolarSystemIndex"], 38);
    assert_eq!(previous["GalacticAddress"]["PlanetIndex"], 0);

    let base = &player["PersistentPlayerBases"][0];
    assert_eq!(base["Name"], if is_a { "Base A" } else { "Base B" });
    assert_eq!(
        base["GalacticAddress"],
        if is_a {
            "0x05000003AB8C07"
        } else {
            "0x05010003AB8C07"
        }
    );
    assert_eq!(base["Position"], json!([100.0, 200.0, 300.0]));
    assert_eq!(base["Owner"]["LID"], "");
    assert_eq!(base["Owner"]["UID"], "76561198025707979");
    assert_eq!(base["Owner"]["USN"], "TestUser");
    assert_eq!(base["Owner"]["PTK"], "ST");
    assert_eq!(base["Owner"]["TS"], 1_700_000_000);
}

fn write_json(path: &Path, value: &Value, modified: SystemTime) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    set_modified(path, modified);
}

fn write_text(path: &Path, bytes: &[u8], modified: SystemTime) {
    fs::write(path, bytes).unwrap();
    set_modified(path, modified);
}

fn install_o4_saves(fixture: &Fixture) -> PathBuf {
    let root = fixture.create_save_root();
    let first = root.join("DefaultUser");
    let second = root.join("st_123");
    let empty = root.join("zz_empty");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::create_dir_all(&empty).unwrap();
    let epoch = UNIX_EPOCH + Duration::from_secs(4_000_000_000);
    let a = literal_save("Save A", 101, true);
    let b = literal_save("Save B", 202, false);

    for (name, value, offset) in [
        ("save.hg", &b, 100),
        ("save2.hg", &a, 200),
        ("save3.hg", &b, 400),
        ("save4.hg", &a, 300),
        ("save29.hg", &a, 500),
        ("save30.hg", &b, 500),
    ] {
        write_json(
            &first.join(name),
            value,
            epoch + Duration::from_secs(offset),
        );
    }
    write_json(
        &second.join("save.hg"),
        &b,
        epoch + Duration::from_secs(900),
    );
    write_text(
        &first.join("save31.hg"),
        b"poisoned out-of-range file",
        epoch + Duration::from_secs(5_000),
    );
    fs::write(root.join("mf_save.hg"), b"metadata").unwrap();
    fs::write(root.join("save2.hg.bak"), b"backup").unwrap();
    fs::write(root.join("README.txt"), b"non-account file").unwrap();

    let explicit = fixture.home.join("explicit-A.json");
    write_json(&explicit, &a, epoch + Duration::from_secs(1));
    write_text(
        &fixture.home.join("community.csv"),
        IMPORT_CSV,
        epoch + Duration::from_secs(2),
    );
    explicit
}

fn os_args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn clean_text(bytes: &[u8]) -> String {
    let raw = String::from_utf8_lossy(bytes);
    let mut output = String::with_capacity(raw.len());
    let mut remaining = raw.as_ref();
    while let Some(start) = remaining.find("\u{1b}[") {
        output.push_str(&remaining[..start]);
        let candidate = &remaining[start + 2..];
        let body_len = candidate
            .bytes()
            .take_while(|byte| byte.is_ascii_digit() || *byte == b';')
            .count();
        if candidate.as_bytes().get(body_len) == Some(&b'm') {
            remaining = &candidate[body_len + 1..];
        } else {
            output.push_str("\u{1b}[");
            remaining = candidate;
        }
    }
    output.push_str(remaining);
    output
}

#[test]
fn test_clean_text_strips_only_complete_sgr_and_preserves_other_escape_text() {
    let input = "é\u{1b}[31mred\u{1b}[0m / \u{1b}[2K / \u{1b}[31";
    assert_eq!(clean_text(input.as_bytes()), "éred / \u{1b}[2K / \u{1b}[31");
}

fn table_cells(line: &str) -> Option<Vec<String>> {
    let clean = line.trim();
    let cells: Vec<String> = if clean.contains('│') {
        clean
            .split('│')
            .map(|cell| cell.trim().to_owned())
            .collect()
    } else {
        clean
            .split("  ")
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .map(str::to_owned)
            .collect()
    };
    if cells.len() < 2 { None } else { Some(cells) }
}

fn table_rows(text: &str) -> Vec<Vec<String>> {
    text.lines().filter_map(table_cells).collect()
}

fn key_value(text: &str, key: &str) -> Option<String> {
    table_rows(text)
        .into_iter()
        .find(|row| row.len() >= 2 && row[0] == key)
        .map(|row| row[1].clone())
}

fn key_values(text: &str, key: &str) -> Vec<String> {
    table_rows(text)
        .into_iter()
        .filter(|row| row.len() >= 2 && row[0] == key)
        .map(|row| row[1].clone())
        .collect()
}

fn selection_matches_info(text: &str, save_name: &str, units: &str) -> bool {
    let names = key_values(text, "Save Name");
    let unit_values = key_values(text, "Units");
    names.len() == 1 && names[0] == save_name && unit_values.len() == 1 && unit_values[0] == units
}

fn replace_info_value(text: &str, key: &str, value: &str) -> Option<String> {
    let mut replaced = false;
    let lines = text
        .split('\n')
        .map(|line| {
            if replaced {
                return line.to_owned();
            }
            let mut cells: Vec<_> = line.split('│').map(str::to_owned).collect();
            if cells.len() >= 2 && cells[0].trim() == key {
                cells[1] = format!(" {value} ");
                replaced = true;
                cells.join("│")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    replaced.then_some(lines)
}

fn assert_only_table_cell_changed(
    before: &str,
    after: &str,
    row_key: &str,
    key_column: usize,
    changed_column: usize,
    expected_after: &str,
) {
    let before_lines: Vec<_> = before.lines().collect();
    let after_lines: Vec<_> = after.lines().collect();
    assert_eq!(before_lines.len(), after_lines.len());
    let target_indices: Vec<_> = before_lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            table_cells(line)
                .filter(|cells| cells.get(key_column).is_some_and(|cell| cell == row_key))
                .map(|_| index)
        })
        .collect();
    assert_eq!(target_indices.len(), 1, "target row {row_key:?}");
    let target_index = target_indices[0];
    let changed_indices: Vec<_> = before_lines
        .iter()
        .zip(&after_lines)
        .enumerate()
        .filter_map(|(index, (before, after))| (before != after).then_some(index))
        .collect();
    assert_eq!(changed_indices, vec![target_index]);

    let before_cells = table_cells(before_lines[target_index]).unwrap();
    let after_cells = table_cells(after_lines[target_index]).unwrap();
    let mut expected_cells = before_cells;
    assert_eq!(expected_cells[key_column], row_key);
    expected_cells[changed_column] = expected_after.to_owned();
    assert_eq!(after_cells, expected_cells);
}

fn named_find_matches(text: &str, expected: &[(&str, &str)]) -> bool {
    let rows = table_rows(text);
    let mut names: Vec<_> = rows
        .iter()
        .filter_map(|row| {
            if row.len() == 7 && row[0].parse::<usize>().is_ok() {
                Some((row[1].as_str(), row[3].as_str()))
            } else {
                None
            }
        })
        .collect();
    let mut expected = expected.to_vec();
    names.sort();
    expected.sort();
    names == expected
}

fn assert_no_stderr(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}; stderr={}",
        output.status,
        clean_text(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected stderr: {}",
        clean_text(&output.stderr)
    );
}

fn append_slot_mode(args: &mut Vec<OsString>, mode: &str, explicit: &Path) {
    match mode {
        "default" => {}
        "slot1" => {
            args.splice(0..0, ["--slot".into(), "1".into()]);
        }
        "explicit" => {
            args.splice(1..1, ["--save".into(), explicit.as_os_str().to_owned()]);
            args.splice(0..0, ["--slot".into(), "2".into()]);
        }
        _ => unreachable!(),
    }
}

fn command_for(route: &str, is_a: bool, fixture: &Fixture) -> Vec<OsString> {
    let (system, base, destination) = if is_a {
        ("System A", "Base A", "Destination A")
    } else {
        ("System B", "Base B", "Destination B")
    };
    match route {
        "info" => os_args(&["info"]),
        "find" => os_args(&["find", "--named"]),
        "show-system" => os_args(&["show", "system", system]),
        "show-base" => os_args(&["show", "base", base]),
        "stats" => os_args(&["stats"]),
        "route" => os_args(&["route", "--target", destination, "--from", base]),
        "list-systems" => os_args(&["list", "systems", "--all"]),
        "list-bases" => os_args(&["list", "bases", "--all"]),
        "export" => os_args(&["export", "--format", "json"]),
        "import" => os_args(&[
            "import",
            fixture.home.join("community.csv").to_str().unwrap(),
        ]),
        _ => unreachable!(),
    }
}

fn assert_export_projection(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let records = json.as_array().unwrap();
    let expected = if is_a {
        vec![("Planet A", "System A", "Euclid")]
    } else {
        vec![
            ("Planet B", "System B", "Euclid"),
            ("Planet B2", "System B", "Euclid"),
        ]
    };
    assert_eq!(records.len(), expected.len());
    for (record, (planet, system, galaxy)) in records.iter().zip(expected) {
        let object = record.as_object().unwrap();
        assert_eq!(object.len(), 12);
        assert_eq!(record["planet_name"], planet);
        assert_eq!(record["system_name"], system);
        assert_eq!(record["galaxy"], galaxy);
    }
}

fn assert_route_population(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let text = clean_text(&output.stdout);
    let (destination, opposite) = if is_a {
        ("Destination A", "Destination B")
    } else {
        ("Destination B", "Destination A")
    };
    assert!(text.contains(destination), "missing {destination}: {text}");
    assert!(!text.contains(opposite), "unexpected {opposite}: {text}");
}

fn assert_named_find(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let text = clean_text(&output.stdout);
    let expected = if is_a {
        vec![("Planet A", "System A")]
    } else {
        vec![("Planet B", "System B"), ("Planet B2", "System B")]
    };
    assert!(named_find_matches(&text, &expected), "{text}");
}

fn assert_system_list(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let rows = table_rows(&clean_text(&output.stdout));
    let title = vec!["SYSTEMS", "", "", ""];
    let header = vec!["Name", "Discovered Planets", "Address", "Portal Glyphs"];
    let footer = vec!["Total:", "2 Systems", "", ""];
    assert!(rows.len() >= 3);
    assert_eq!(rows.first().unwrap(), &title);
    assert_eq!(rows.get(1).unwrap(), &header);
    assert_eq!(rows.last().unwrap(), &footer);
    let mut names: Vec<_> = rows[2..rows.len() - 1]
        .iter()
        .map(|row| {
            assert_eq!(row.len(), 4);
            row[0].clone()
        })
        .collect();
    names.sort();
    let mut expected = if is_a {
        vec!["Destination A".to_owned(), "System A".to_owned()]
    } else {
        vec!["Destination B".to_owned(), "System B".to_owned()]
    };
    expected.sort();
    assert_eq!(names, expected);
}

fn assert_base_list(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let rows = table_rows(&clean_text(&output.stdout));
    let title = vec!["BASES", "", "", "", ""];
    let header = vec!["Name", "Type", "Galaxy", "Address", "Portal Glyphs"];
    let footer = vec!["Total:", "1 Bases", "", "", ""];
    assert!(rows.len() >= 3);
    assert_eq!(rows.first().unwrap(), &title);
    assert_eq!(rows.get(1).unwrap(), &header);
    assert_eq!(rows.last().unwrap(), &footer);
    let names: Vec<_> = rows[2..rows.len() - 1]
        .iter()
        .map(|row| {
            assert_eq!(row.len(), 5);
            row[0].clone()
        })
        .collect();
    assert_eq!(names, vec![if is_a { "Base A" } else { "Base B" }]);
}

fn assert_system_show(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let text = clean_text(&output.stdout);
    let (system, planet, opposite) = if is_a {
        ("System A", "Planet A", "System B")
    } else {
        ("System B", "Planet B", "System A")
    };
    assert_eq!(key_value(&text, "Name").as_deref(), Some(system));
    assert!(!text.contains(opposite), "unexpected {opposite}: {text}");
    let mut planet_names: Vec<_> = table_rows(&text)
        .iter()
        .filter(|row| (row.len() == 3 || row.len() == 4) && row[0].parse::<u8>().is_ok())
        .map(|row| row[1].clone())
        .collect();
    planet_names.sort();
    let mut expected = if is_a {
        vec![planet.to_owned()]
    } else {
        vec!["Planet B".to_owned(), "Planet B2".to_owned()]
    };
    expected.sort();
    assert_eq!(planet_names, expected, "{text}");
}

fn assert_base_show(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let expected = if is_a { "Base A" } else { "Base B" };
    let opposite = if is_a { "Base B" } else { "Base A" };
    let text = clean_text(&output.stdout);
    assert_eq!(key_value(&text, "Name").as_deref(), Some(expected));
    assert!(!text.contains(opposite));
}

fn assert_stats(output: &Output, is_a: bool) {
    assert_no_stderr(output);
    let text = clean_text(&output.stdout);
    assert_eq!(key_value(&text, "Systems").as_deref(), Some("2"));
    assert_eq!(
        key_value(&text, "Planets").as_deref(),
        Some(if is_a { "1" } else { "2" })
    );
    assert_eq!(key_value(&text, "Bases").as_deref(), Some("1"));
}

fn save_sections(text: &str) -> BTreeMap<String, Vec<Vec<String>>> {
    let mut sections = BTreeMap::new();
    let mut current = String::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("Account: ") {
            current = name.to_owned();
            sections.entry(current.clone()).or_insert_with(Vec::new);
        } else if let Some(row) = table_cells(line)
            && row.iter().any(|cell| !cell.is_empty())
            && !current.is_empty()
        {
            sections
                .entry(current.clone())
                .or_insert_with(Vec::new)
                .push(row);
        }
    }
    sections
}

fn o4_listing_matches(text: &str) -> bool {
    let headings = [
        "Account: DefaultUser (GOG)",
        "Account: st_123 (Steam (123))",
        "Account: zz_empty (Unknown (zz_empty))",
    ];
    let clean = clean_text(text.as_bytes());
    let actual_headings: Vec<_> = clean
        .lines()
        .filter_map(|line| line.strip_prefix("Account: "))
        .map(|name| format!("Account: {name}"))
        .collect();
    if actual_headings != headings {
        return false;
    }
    let sections = save_sections(&clean);
    let expected = BTreeMap::from([
        (
            "DefaultUser (GOG)".to_owned(),
            vec![
                vec!["SAVE".into(), "SLOTS".into(), "".into(), "".into()],
                vec![
                    "Number".into(),
                    "Manual".into(),
                    "Auto".into(),
                    "Most Recent".into(),
                ],
                vec![
                    "1".into(),
                    "yes".into(),
                    "yes".into(),
                    "Auto (just now)".into(),
                ],
                vec![
                    "2".into(),
                    "yes".into(),
                    "yes".into(),
                    "Manual (just now)".into(),
                ],
                vec![
                    "15".into(),
                    "yes".into(),
                    "yes".into(),
                    "Manual (just now)".into(),
                ],
                vec!["Total:".into(), "3 Slots".into(), "".into(), "".into()],
            ],
        ),
        (
            "st_123 (Steam (123))".to_owned(),
            vec![
                vec!["SAVE".into(), "SLOTS".into(), "".into(), "".into()],
                vec![
                    "Number".into(),
                    "Manual".into(),
                    "Auto".into(),
                    "Most Recent".into(),
                ],
                vec![
                    "1".into(),
                    "yes".into(),
                    "-".into(),
                    "Manual (just now)".into(),
                ],
                vec!["Total:".into(), "1 Slots".into(), "".into(), "".into()],
            ],
        ),
        ("zz_empty (Unknown (zz_empty))".to_owned(), Vec::new()),
    ]);
    sections == expected && clean.contains("No save files found.")
}

fn drop_default_slot15(text: &str) -> String {
    let mut current = String::new();
    clean_text(text.as_bytes())
        .lines()
        .filter_map(|line| {
            if let Some(name) = line.strip_prefix("Account: ") {
                current = name.to_owned();
            }
            let remove = current == "DefaultUser (GOG)"
                && table_cells(line)
                    .is_some_and(|row| row.first().is_some_and(|cell| cell == "15"));
            (!remove).then_some(line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn swap_slot1_recent(text: &str) -> String {
    let mut current = String::new();
    clean_text(text.as_bytes())
        .lines()
        .map(|line| {
            if let Some(name) = line.strip_prefix("Account: ") {
                current = name.to_owned();
            }
            if current == "DefaultUser (GOG)"
                && table_cells(line).is_some_and(|row| row.first().is_some_and(|cell| cell == "1"))
            {
                line.replacen("Auto (just now)", "Manual (just now)", 1)
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_actual_cli_routes_select_literal_save_rows_for_all_ten_save_backed_commands() {
    let cases = [
        "info",
        "find",
        "show-system",
        "show-base",
        "stats",
        "route",
        "list-systems",
        "list-bases",
        "export",
        "import",
    ];
    for case in cases {
        for mode in ["default", "slot1", "explicit"] {
            let fixture = Fixture::new();
            let explicit = install_o4_saves(&fixture);
            let is_a = mode != "default";
            let mut args = command_for(case, is_a, &fixture);
            append_slot_mode(&mut args, mode, &explicit);
            let output = fixture.run(&args);
            match case {
                "info" => {
                    assert_no_stderr(&output);
                    let text = clean_text(&output.stdout);
                    let expected_name = if is_a { "Save A" } else { "Save B" };
                    let expected_units = if is_a { "101" } else { "202" };
                    assert!(
                        selection_matches_info(&text, expected_name, expected_units),
                        "{text}"
                    );
                    if case == "info" && mode == "slot1" {
                        let wrong_name = text.replacen("Save A", "Save B", 1);
                        assert_ne!(wrong_name, text);
                        assert_only_table_cell_changed(
                            &text,
                            &wrong_name,
                            "Save Name",
                            0,
                            1,
                            "Save B",
                        );
                        assert!(!selection_matches_info(&wrong_name, "Save A", "101"));
                        let wrong_units = replace_info_value(&text, "Units", "202").unwrap();
                        assert_ne!(wrong_units, text);
                        assert_only_table_cell_changed(&text, &wrong_units, "Units", 0, 1, "202");
                        assert!(!selection_matches_info(&wrong_units, "Save A", "101"));
                        let missing_name = text.replacen("Save Name", "Other Name", 1);
                        assert_ne!(missing_name, text);
                        assert_only_table_cell_changed(
                            &text,
                            &missing_name,
                            "Save Name",
                            0,
                            0,
                            "Other Name",
                        );
                        assert!(!selection_matches_info(&missing_name, "Save A", "101"));
                        let name_line = text
                            .lines()
                            .find(|line| {
                                table_cells(line).is_some_and(|row| {
                                    row.first().is_some_and(|cell| cell == "Save Name")
                                })
                            })
                            .unwrap();
                        let duplicate_name = format!("{text}\n{name_line}");
                        assert!(!selection_matches_info(&duplicate_name, "Save A", "101"));
                    }
                }
                "find" => {
                    assert_named_find(&output, is_a);
                    if mode == "slot1" {
                        let text = clean_text(&output.stdout);
                        let wrong_planet = text.replacen("Planet A", "Planet B", 1);
                        assert_ne!(wrong_planet, text);
                        assert_only_table_cell_changed(
                            &text,
                            &wrong_planet,
                            "Planet A",
                            1,
                            1,
                            "Planet B",
                        );
                        assert!(!named_find_matches(
                            &wrong_planet,
                            &[("Planet A", "System A")]
                        ));
                    }
                }
                "show-system" => assert_system_show(&output, is_a),
                "show-base" => assert_base_show(&output, is_a),
                "stats" => assert_stats(&output, is_a),
                "route" => assert_route_population(&output, is_a),
                "list-systems" => assert_system_list(&output, is_a),
                "list-bases" => assert_base_list(&output, is_a),
                "export" => assert_export_projection(&output, is_a),
                "import" => {
                    assert!(output.status.success(), "{}", clean_text(&output.stderr));
                    assert!(output.stdout.is_empty());
                    let expected = if is_a {
                        "Imported 0 systems (1 duplicates, 0 skipped)\n"
                    } else {
                        "Imported 1 systems (0 duplicates, 0 skipped)\n"
                    };
                    assert_eq!(output.stderr, expected.as_bytes());
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_actual_cli_flag_positions_explicit_file_and_slot_errors_are_distinguishable() {
    // The after-command slot-2 child is retained, but default is also B; the
    // opposed slot-1 child makes ignoring a trailing global flag observable.
    for (args, expected_name, units) in [
        (vec!["info", "--slot", "2"], "Save B", "202"),
        (vec!["info", "--slot", "1"], "Save A", "101"),
        (
            vec!["show", "system", "System A", "--slot", "15"],
            "Save A",
            "101",
        ),
    ] {
        let fixture = Fixture::new();
        install_o4_saves(&fixture);
        let output = fixture.run(&os_args(&args));
        assert_no_stderr(&output);
        let text = clean_text(&output.stdout);
        if args[0] == "show" {
            assert_eq!(key_value(&text, "Name").as_deref(), Some("System A"));
            assert!(text.contains("Planet A"), "{text}");
            assert!(!text.contains("System B"), "{text}");
        } else {
            assert!(selection_matches_info(&text, expected_name, units));
        }
    }

    let fixture = Fixture::new();
    let explicit = fixture.home.join("explicit-only.json");
    write_json(
        &explicit,
        &literal_save("Save A", 101, true),
        UNIX_EPOCH + Duration::from_secs(4_000_000_001),
    );
    let args = vec![
        "info".into(),
        "--save".into(),
        explicit.as_os_str().to_owned(),
        "--slot".into(),
        "2".into(),
    ];
    let output = fixture.run(&args);
    assert_no_stderr(&output);
    assert!(selection_matches_info(
        &clean_text(&output.stdout),
        "Save A",
        "101"
    ));

    let fixture = Fixture::new();
    install_o4_saves(&fixture);
    let output = fixture.run(&os_args(&["info", "--slot", "3"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).contains("save slot 3 not found"));

    let fixture = Fixture::new();
    let root = fixture.create_save_root();
    fs::create_dir_all(root.join("DefaultUser")).unwrap();
    fs::create_dir_all(root.join("st_123")).unwrap();
    write_json(
        &root.join("st_123/save.hg"),
        &literal_save("Save B", 202, false),
        UNIX_EPOCH + Duration::from_secs(4_000_000_001),
    );
    let output = fixture.run(&os_args(&["info", "--slot", "1"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).contains("no save files found"));

    let fixture = Fixture::new();
    install_o4_saves(&fixture);
    write_text(
        &fixture.save_root().join("DefaultUser/save2.hg"),
        b"malformed selected save",
        UNIX_EPOCH + Duration::from_secs(4_000_000_200),
    );
    let output = fixture.run(&os_args(&["info", "--slot", "1"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).starts_with("Error:"));
    assert!(!clean_text(&output.stdout).contains("Save B"));

    let fixture = Fixture::new();
    install_o4_saves(&fixture);
    let missing = fixture.home.join("absent.json");
    let args = vec![
        "info".into(),
        "--save".into(),
        missing.into_os_string(),
        "--slot".into(),
        "1".into(),
    ];
    let output = fixture.run(&args);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).starts_with("Error:"));
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_saves_command_reports_exact_accounts_slots_and_recent_types() {
    let fixture = Fixture::new();
    install_o4_saves(&fixture);
    let output = fixture.run(&os_args(&["saves"]));
    assert!(output.status.success(), "{}", clean_text(&output.stderr));
    assert!(output.stderr.is_empty());
    let text = clean_text(&output.stdout);
    assert!(o4_listing_matches(&text), "{text}");

    let removed = drop_default_slot15(&text);
    assert_ne!(removed, text);
    assert!(!o4_listing_matches(&removed));
    let original_sections = save_sections(&text);
    let removed_sections = save_sections(&removed);
    assert_eq!(
        removed_sections.get("st_123 (Steam (123))"),
        original_sections.get("st_123 (Steam (123))")
    );
    assert_eq!(
        removed_sections.get("zz_empty (Unknown (zz_empty))"),
        original_sections.get("zz_empty (Unknown (zz_empty))")
    );
    let swapped = swap_slot1_recent(&text);
    assert_ne!(swapped, text);
    assert!(!o4_listing_matches(&swapped));
    let swapped_sections = save_sections(&swapped);
    assert_eq!(
        swapped_sections.get("st_123 (Steam (123))"),
        original_sections.get("st_123 (Steam (123))")
    );
    assert_eq!(
        swapped_sections.get("zz_empty (Unknown (zz_empty))"),
        original_sections.get("zz_empty (Unknown (zz_empty))")
    );
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_saves_lists_all_thirty_canonical_fake_files_without_parsing_payloads() {
    let fixture = Fixture::new();
    let root = fixture.create_save_root();
    let account = root.join("DefaultUser");
    fs::create_dir_all(&account).unwrap();
    let epoch = UNIX_EPOCH + Duration::from_secs(4_000_000_000);
    for index in 1..=30u8 {
        let name = if index == 1 {
            "save.hg".to_owned()
        } else {
            format!("save{index}.hg")
        };
        write_text(
            &account.join(name),
            b"not a save document",
            epoch + Duration::from_secs(index.into()),
        );
    }
    for name in [
        "mf_save.hg",
        "save.hg.bak",
        "save31.hg",
        "save255.hg",
        "save256.hg",
    ] {
        write_text(
            &account.join(name),
            b"noise",
            epoch + Duration::from_secs(5000),
        );
    }
    let output = fixture.run(&os_args(&["saves"]));
    assert!(output.status.success(), "{}", clean_text(&output.stderr));
    assert!(output.stderr.is_empty());
    let text = clean_text(&output.stdout);
    let sections = save_sections(&text);
    let rows = sections.get("DefaultUser (GOG)").unwrap();
    assert_eq!(rows.len(), 18);
    assert_eq!(rows.first().unwrap(), &["SAVE", "SLOTS", "", ""]);
    assert_eq!(
        rows.get(1).unwrap(),
        &["Number", "Manual", "Auto", "Most Recent"]
    );
    for (slot, row) in rows.iter().enumerate().skip(2).take(15) {
        assert_eq!(row[0], (slot - 1).to_string());
        assert_eq!(&row[1..], ["yes", "yes", "Auto (just now)"]);
    }
    assert_eq!(rows.last().unwrap(), &["Total:", "15 Slots", "", ""]);
    for rejected_index in ["31", "255", "256"] {
        assert!(!text.contains(rejected_index));
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_save_listing_errors_distinguish_missing_empty_and_io_and_preserve_static_commands() {
    let fixture = Fixture::new();
    let output = fixture.run(&os_args(&["info"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).contains("NMS save directory not found"));

    let fixture = Fixture::new();
    fixture.create_save_root();
    let output = fixture.run(&os_args(&["info"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).contains("no account directories found"));

    let fixture = Fixture::new();
    let root = fixture.create_save_root();
    fs::write(root.join("root-file.txt"), b"not an account").unwrap();
    let output = fixture.run(&os_args(&["info"]));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(clean_text(&output.stderr).contains("no account directories found"));

    let fixture = Fixture::new();
    let root = fixture.save_root();
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::write(&root, b"not a directory").unwrap();
    let output = fixture.run(&os_args(&["info"]));
    assert_eq!(output.status.code(), Some(1));
    let stderr = clean_text(&output.stderr);
    assert!(stderr.starts_with("Error:"));
    assert!(
        stderr.to_lowercase().contains("not a directory"),
        "{stderr}"
    );
    assert!(!stderr.contains("No save files found."));
    assert!(output.stdout.is_empty());

    for slot in ["0", "16", "256", "not-a-number"] {
        let fixture = Fixture::new();
        let output = fixture.run(&os_args(&["--slot", slot, "info"]));
        assert_eq!(
            output.status.code(),
            Some(2),
            "slot={slot}: {}",
            clean_text(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(clean_text(&output.stderr).contains("--slot"));
    }

    let fixture = Fixture::new();
    let output = fixture.run(&os_args(&["--slot", "15", "list", "biomes"]));
    assert!(output.status.success(), "{}", clean_text(&output.stderr));
    assert!(output.stderr.is_empty());
    assert!(clean_text(&output.stdout).contains("BIOMES"));
}
