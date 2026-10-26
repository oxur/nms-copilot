//! Rejected glyphs return bounded UTF-8 diagnostics through the real CLI.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_convert_glyphs_three_byte_rejection_is_bounded_error() {
    cargo_bin_cmd!("nms")
        .args(["convert", "--glyphs", "xxxxxxxxxxxxxxxxxxx界suffix"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "Invalid portal glyphs: invalid glyph: unrecognized glyph at: xxxxxxxxxxxxxxxxxxx",
        ))
        .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn test_convert_glyphs_four_byte_rejection_is_bounded_error() {
    cargo_bin_cmd!("nms")
        .args(["convert", "--glyphs", "xxxxxxxxxxxxxxxxxx🧪suffix"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "Invalid portal glyphs: invalid glyph: unrecognized glyph at: xxxxxxxxxxxxxxxxxx",
        ))
        .stderr(predicate::str::contains("panicked").not());
}
