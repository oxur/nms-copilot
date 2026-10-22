use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const SOURCE: &str = " NMSCE α ";
const CSV: &str = "System Name,Galaxy,Portal Glyphs,Biome,Platform\nPersonal replacement,0,000100000000,Toxic,PC\n\"Catalog, \"\"Nine\"\"\",9,000100000000,Toxic,PC\nCatalog-B,0,000200000001,Frozen\nCatalog-C,0,000300000002,Barren,PC,ignored-extra\nLater,9,000100000000,Lush,PC\n";

fn fixture_save() -> Value {
    let mut save: Value =
        serde_json::from_slice(include_bytes!("../../../data/test/save56_minimal.json"))
            .expect("parse tracked save fixture");
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
    bases[0]["GalacticAddress"] = json!("0x00010000000000");
    bases[0]["Position"] = json!([1.0, 2.0, 3.0]);
    bases[0]["Name"] = json!("Anchor");
    bases[0]["BaseType"] = json!({"PersistentBaseTypes":"HomePlanetBase"});
    bases[0]["Owner"]["UID"] = json!("SourceOwner");

    let record = |address: &str, kind: &str, vp: Value, name: &str| {
        json!({
            "DD":{"UA":address,"DT":kind,"VP":vp},
            "DM":{"CN":name},
            "OWS":{"LID":"","UID":"SourceOwnerUID","USN":"SourceOwner","PTK":"ST","TS":1700000000},
            "FL":{"U":1}
        })
    };
    save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"] = json!([
        record("0x00010000000000", "SolarSystem", json!([]), "Personal"),
        record(
            "0x10010000000000",
            "Planet",
            json!(["0xAA", 0]),
            "Personal Moon"
        ),
    ]);
    save
}

fn tree_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>, SystemTime)> {
    fn visit(root: &Path, current: &Path, rows: &mut Vec<(PathBuf, Vec<u8>, SystemTime)>) {
        for entry in std::fs::read_dir(current).expect("read fixture directory") {
            let path = entry.expect("read fixture entry").path();
            let metadata = std::fs::symlink_metadata(&path).expect("fixture metadata");
            assert!(
                !metadata.file_type().is_symlink(),
                "fixture has no symlinks"
            );
            if metadata.is_dir() {
                visit(root, &path, rows);
            } else {
                rows.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(&path).expect("read fixture bytes"),
                    metadata.modified().expect("fixture mtime"),
                ));
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

fn run_import(csv: &Path, save: &Path, home: &Path, source: Option<&str>) -> std::process::Output {
    let mut command = cargo_bin_cmd!("nms");
    command
        .arg("import")
        .arg(csv)
        .arg("--save")
        .arg(save)
        .env("HOME", home)
        .timeout(std::time::Duration::from_secs(10));
    if let Some(source) = source {
        command.args(["--source", source]);
    }
    command.output().expect("run actual nms import child")
}

fn assert_success(output: &std::process::Output, summary: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "import emitted stdout payload");
    assert_eq!(output.stderr, format!("{summary}\n").as_bytes());
}

#[test]
fn test_import_csv_real_child_preserves_fixture_and_reports_exact_counts() {
    let dir = tempfile::tempdir().expect("isolated import fixture");
    let home = dir.path().join("home");
    std::fs::create_dir_all(home.join(".nms-copilot")).unwrap();
    let config = home.join(".nms-copilot/config.toml");
    std::fs::write(&config, "[display]\ncolor = false\n").unwrap();
    let save = dir.path().join("save.json");
    std::fs::write(&save, serde_json::to_vec(&fixture_save()).unwrap()).unwrap();
    let csv = dir.path().join("community.csv");
    std::fs::write(&csv, CSV).unwrap();
    let sentinel = dir.path().join("sentinel");
    std::fs::write(&sentinel, b"fixture sentinel\n").unwrap();
    let before = tree_snapshot(dir.path());

    let explicit = run_import(&csv, &save, &home, Some(SOURCE));
    assert_success(&explicit, "Imported 3 systems (2 duplicates, 0 skipped)");
    assert_eq!(
        tree_snapshot(dir.path()),
        before,
        "explicit import is session-only"
    );

    let default_save = dir.path().join("default-save.json");
    std::fs::write(&default_save, serde_json::to_vec(&fixture_save()).unwrap()).unwrap();
    let default_before = tree_snapshot(dir.path());
    let default = run_import(&csv, &default_save, &home, None);
    assert_success(&default, "Imported 3 systems (2 duplicates, 0 skipped)");
    assert_eq!(
        tree_snapshot(dir.path()),
        default_before,
        "default source is session-only"
    );
}

#[test]
fn test_import_csv_real_child_rejects_missing_or_invalid_inputs_without_success_payload() {
    let dir = tempfile::tempdir().expect("isolated invalid import fixture");
    let home = dir.path().join("home");
    std::fs::create_dir_all(home.join(".nms-copilot")).unwrap();
    let config = home.join(".nms-copilot/config.toml");
    std::fs::write(&config, "[display]\ncolor = false\n").unwrap();
    let save = dir.path().join("save.json");
    let mut valid = fixture_save();
    std::fs::write(&save, serde_json::to_vec(&valid).unwrap()).unwrap();
    let csv = dir.path().join("community.csv");
    std::fs::write(&csv, CSV).unwrap();
    let initial_tree = tree_snapshot(dir.path());

    let missing_csv = dir.path().join("missing.csv");
    let before_missing_csv = tree_snapshot(dir.path());
    let missing_csv_result = run_import(&missing_csv, &save, &home, Some(SOURCE));
    assert_eq!(
        tree_snapshot(dir.path()),
        before_missing_csv,
        "missing CSV child leaves fixture unchanged"
    );
    assert!(!missing_csv_result.status.success());
    assert!(missing_csv_result.stdout.is_empty());
    let missing_csv_stderr = String::from_utf8_lossy(&missing_csv_result.stderr);
    assert!(
        missing_csv_stderr.contains("CSV error:"),
        "{missing_csv_stderr}"
    );
    assert!(!missing_csv_stderr.contains("Imported "));

    let missing_save = dir.path().join("missing-save.json");
    let before_missing_save = tree_snapshot(dir.path());
    let missing_save_result = run_import(&csv, &missing_save, &home, Some(SOURCE));
    assert_eq!(
        tree_snapshot(dir.path()),
        before_missing_save,
        "missing save child leaves fixture unchanged"
    );
    assert!(!missing_save_result.status.success());
    assert!(missing_save_result.stdout.is_empty());
    let missing_save_stderr = String::from_utf8_lossy(&missing_save_result.stderr);
    assert!(
        missing_save_stderr.contains("I/O error:"),
        "{missing_save_stderr}"
    );
    assert!(!missing_save_stderr.contains("Imported "));

    let bases = valid["BaseContext"]["PlayerStateData"]["PersistentPlayerBases"]
        .as_array_mut()
        .unwrap();
    bases[0]["GalacticAddress"] = json!("0x100000000000000");
    assert_eq!(bases[0]["GalacticAddress"], "0x100000000000000");
    let invalid_save = dir.path().join("invalid-57-bit-save.json");
    std::fs::write(&invalid_save, serde_json::to_vec(&valid).unwrap()).unwrap();
    let before_invalid = tree_snapshot(dir.path());
    let invalid = run_import(&csv, &invalid_save, &home, Some(SOURCE));
    assert_eq!(
        tree_snapshot(dir.path()),
        before_invalid,
        "invalid address child leaves fixture unchanged"
    );
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    let invalid_stderr = String::from_utf8_lossy(&invalid.stderr);
    assert!(
        invalid_stderr.contains(
            "packed address exceeds 56 bits at /BaseContext/PlayerStateData/PersistentPlayerBases/0/GalacticAddress"
        ),
        "{invalid_stderr}"
    );
    assert!(!invalid_stderr.contains("Imported "));

    assert_eq!(
        std::fs::read(&save).unwrap(),
        serde_json::to_vec(&fixture_save()).unwrap()
    );
    assert_eq!(std::fs::read(&csv).unwrap(), CSV.as_bytes());
    assert_eq!(
        std::fs::read(&config).unwrap(),
        b"[display]\ncolor = false\n"
    );
    assert!(initial_tree.iter().all(|(path, bytes, _)| {
        let current = dir.path().join(path);
        current.is_file()
            && std::fs::read(current)
                .map(|actual| actual.as_slice() == bytes.as_slice())
                .unwrap_or(false)
    }));
}
