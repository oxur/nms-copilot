//! Real CLI regressions for Unicode discovery names in formatted query output.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use tempfile::TempDir;

fn fixture_with_names(system_name: &str, planet_name: &str) -> (TempDir, PathBuf, Vec<u8>) {
    let temp = TempDir::new().expect("create isolated test directory");
    let mut save: Value =
        serde_json::from_slice(include_bytes!("../../../data/test/save56_minimal.json"))
            .expect("parse committed checked-save fixture");
    let records = save["DiscoveryManagerData"]["DiscoveryData-v1"]["Store"]["Record"]
        .as_array_mut()
        .expect("fixture discovery records are an array");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["DD"]["DT"], "SolarSystem");
    assert_eq!(records[0]["DD"]["UA"], "0x05000003AB8C07");
    assert_eq!(records[1]["DD"]["DT"], "Planet");
    assert_eq!(records[1]["DD"]["UA"], "0x15000003AB8C07");
    assert_eq!(records[0]["DM"], json!({}));
    assert_eq!(records[1]["DM"], json!({}));
    records[0]["DM"]["CN"] = json!(system_name);
    records[1]["DM"]["CN"] = json!(planet_name);
    let bytes = serde_json::to_vec_pretty(&save).expect("serialize temporary fixture");
    let path = temp.path().join("unicode-save.json");
    fs::write(&path, &bytes).expect("write temporary fixture");
    for directory in ["home", "config", "cache"] {
        fs::create_dir_all(temp.path().join(directory)).expect("create isolated user directory");
    }
    (temp, path, bytes)
}

fn command_with_isolated_home(temp: &TempDir) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("nms");
    command
        .current_dir(temp.path())
        .env("HOME", temp.path().join("home"))
        .env("XDG_CONFIG_HOME", temp.path().join("config"))
        .env("XDG_CACHE_HOME", temp.path().join("cache"));
    command
}

fn assert_child_succeeded<'a>(output: &'a std::process::Output, label: &str) -> &'a str {
    assert!(
        output.status.success(),
        "{label} failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::str::from_utf8(&output.stdout).expect("CLI stdout is valid UTF-8")
}

fn assert_fixture_unchanged(path: &Path, written: &[u8]) {
    let after = fs::read(path).expect("read temporary fixture after child execution");
    assert_eq!(after, written, "CLI must not modify its input save");
}

#[test]
fn test_nms_find_unicode_checked_save_names_render_without_panicking() {
    let system_name = "123456789012345678ééééé";
    let planet_name = "1234567890123456ééééé";
    let (temp, fixture, written) = fixture_with_names(system_name, planet_name);
    let output = command_with_isolated_home(&temp)
        .args(["find", "--save"])
        .arg(&fixture)
        .output()
        .expect("run real nms find child");
    let stdout = assert_child_succeeded(&output, "nms find");
    assert!(stdout.contains("SEARCH RESULTS"), "{stdout}");
    assert!(stdout.contains("123456789012345678é..."), "{stdout}");
    assert!(stdout.contains("1234567890123456é..."), "{stdout}");
    assert!(!stdout.contains(system_name), "{stdout}");
    assert!(!stdout.contains(planet_name), "{stdout}");
    assert_fixture_unchanged(&fixture, &written);
}

#[test]
fn test_nms_route_unicode_checked_save_target_renders_without_panicking() {
    let system_name = "12345678901234567ééééé";
    let (temp, fixture, written) = fixture_with_names(system_name, "Planet");
    let output = command_with_isolated_home(&temp)
        .args(["route", "--save"])
        .arg(&fixture)
        .args(["--target", system_name, "--algo", "nn"])
        .output()
        .expect("run real nms route child");
    let stdout = assert_child_succeeded(&output, "nms route");
    assert!(stdout.contains("ROUTE ITINERARY"), "{stdout}");
    assert!(stdout.contains("12345678901234567é..."), "{stdout}");
    assert!(!stdout.contains(system_name), "{stdout}");
    assert!(stdout.contains("  Route: 1 targets,"), "{stdout}");
    assert_fixture_unchanged(&fixture, &written);
}
