//! Startup integration tests for the `nms-copilot` binary.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_nms_copilot_help_exits_before_startup() {
    cargo_bin_cmd!("nms-copilot")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: nms-copilot"))
        .stdout(predicate::str::contains("--http"))
        .stdout(predicate::str::contains("mcp-smoke"))
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stdout(predicate::str::contains("MCP server listening").not());
}

#[test]
fn test_nms_copilot_mcp_smoke_help() {
    cargo_bin_cmd!("nms-copilot")
        .args(["mcp-smoke", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: nms-copilot mcp-smoke"))
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("NMS Copilot Setup").not())
        .stdout(predicate::str::contains("MCP server listening").not());
}
