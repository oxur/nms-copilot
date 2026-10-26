//! Startup integration tests for the `nms-copilot` binary.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_nms_copilot_help_exits_before_startup() {
    let assertion = cargo_bin_cmd!("nms-copilot")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: nms-copilot"))
        .stdout(predicate::str::contains("mcp-smoke"))
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stdout(predicate::str::contains("MCP server listening").not());
    #[cfg(feature = "http")]
    assertion.stdout(predicate::str::contains("--http"));
    #[cfg(not(feature = "http"))]
    assertion.stdout(predicate::str::contains("--http").not());
}

#[test]
fn test_nms_copilot_mcp_smoke_help() {
    cargo_bin_cmd!("nms-copilot")
        .args(["mcp-smoke", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: nms-copilot"))
        .stdout(predicate::str::contains("mcp-smoke [OPTIONS]"))
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stdout(predicate::str::contains("MCP server listening").not());
}

#[test]
#[cfg(unix)]
fn test_missing_save_non_tty_shows_guidance_without_starting_setup() {
    let home = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("nms-copilot")
        .env("HOME", home.path())
        .env_remove("NMS_SAVE_DIR")
        .env_remove("NMS_SAVE_FILE")
        .env_remove("NMS_SAVE_FORMAT")
        .arg("--no-cache")
        .write_stdin(Vec::<u8>::new())
        .assert()
        .failure()
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stderr(predicate::str::contains("No save file was found"))
        .stderr(predicate::str::contains("--save /path/to/save.hg"));
}

#[test]
#[cfg(unix)]
fn test_headless_explicit_setup_shows_guidance_without_starting_setup() {
    let home = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("nms-copilot")
        .env("HOME", home.path())
        .env_remove("NMS_SAVE_DIR")
        .env_remove("NMS_SAVE_FILE")
        .env_remove("NMS_SAVE_FORMAT")
        .args(["--headless", "--setup"])
        .write_stdin(Vec::<u8>::new())
        .assert()
        .failure()
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stderr(predicate::str::contains(
            "Interactive setup requires a non-headless terminal",
        ))
        .stderr(predicate::str::contains("--save /path/to/save.hg"));
}
