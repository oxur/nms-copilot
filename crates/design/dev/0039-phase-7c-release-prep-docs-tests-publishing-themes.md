# Phase 7C -- Release Prep: Documentation, Tests, Publishing, Themes

Milestones 7.6-7.8, 7.10: Documentation, integration tests, crates.io publishing, and color themes.

**Depends on:** All prior phases (this is the final polish pass).

---

## Architecture Overview

Phase 7C wraps up the project for public release:

1. **Documentation** (7.6) -- README with examples, rustdoc for public APIs, CONTRIBUTING guide
2. **Integration tests** (7.7) -- End-to-end tests with fixture save files
3. **crates.io publishing** (7.8) -- Publish order, metadata, CI
4. **Color themes** (7.10) -- Configurable ANSI colors for terminal output

---

## Milestone 7.6: Documentation

### Goal

Comprehensive documentation for users and contributors: README with usage examples and screenshots, rustdoc for all public APIs, and a CONTRIBUTING guide.

### README Structure

```markdown
# NMS Copilot

Real-time galactic copilot for No Man's Sky.

## Features
- Search planets by biome, distance, discoverer
- Plan optimal routes through star systems
- Convert between portal glyphs, signal booster, and galactic addresses
- Live model updates as the game auto-saves
- AI integration via MCP server

## Installation
cargo install nms-copilot

## Quick Start
nms info                          # Save file summary
nms find --biome Lush --within 500  # Find lush planets nearby
nms route --biome Lush --max-targets 5  # Plan a route
nms convert --glyphs 01717D8A4EA2   # Convert coordinates

## Portal Glyph Table
[emoji table from data/glyphs.toml]

## MCP Server
nms-mcp                    # stdio transport
nms-mcp --http 127.0.0.1:3000  # HTTP transport

## Configuration
~/.nms-copilot/config.toml

## Shell Completions
nms completions bash > ~/.bash_completion.d/nms
```

### Rustdoc

Every public type, function, and module gets a doc comment. Priority crates:

1. `nms-core` -- All types used across the workspace
2. `nms-save` -- The parsing pipeline
3. `nms-graph` -- The galaxy model
4. `nms-query` -- The query engine

Standard: `//!` for module-level, `///` for items, include `# Examples` sections for key functions.

### CONTRIBUTING.md

```markdown
# Contributing

## Development Setup
git clone https://github.com/oxur/nms-copilot
cd nms-copilot
make build && make test

## Test Data
Fixture save files in data/test/. Never commit real save data.

## Code Style
- make format && make lint before commits
- Test naming: test_<fn>_<scenario>_<expectation>
- Target 95%+ code coverage

## Architecture
See CLAUDE.md for crate dependency graph and design decisions.
```

### Tests

```rust
#[test]
fn test_readme_code_examples_compile() {
    // Use doc_comment crate or compile-test to verify README examples
}
```

---

## Milestone 7.7: Integration Tests

### Goal

End-to-end tests that exercise the full pipeline: save file -> parser -> model -> query -> formatted output. Use fixture save files committed to the repo.

### Test Fixtures

Create minimal JSON save fixtures in `data/test/`:

```
data/test/
  minimal_save.json       # 1 system, 1 planet, 1 base
  multi_system_save.json  # 5 systems, 10 planets, varied biomes
  multi_galaxy_save.json  # Systems across Euclid + Eissentam
  empty_save.json         # Valid structure, no discoveries
```

These are deobfuscated JSON (not binary `.hg`) to keep the test data human-readable and small.

### Integration Test Location

`tests/` directory at workspace root (or per-crate `tests/` directories):

```
tests/
  cli_integration.rs      # CLI command tests
  mcp_integration.rs      # MCP tool tests
  pipeline_integration.rs # Full pipeline tests
```

### CLI Integration Tests

Use `assert_cmd` and `predicates`:

```rust
// tests/cli_integration.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_nms_info_with_fixture() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["info", "--save", "data/test/minimal_save.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Systems:"));
}

#[test]
fn test_nms_find_biome_filter() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["find", "--save", "data/test/multi_system_save.json", "--biome", "Lush"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Lush"));
}

#[test]
fn test_nms_convert_glyphs_roundtrip() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["convert", "--glyphs", "01717D8A4EA2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("01717D8A4EA2"));
}

#[test]
fn test_nms_export_json() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["export", "--save", "data/test/multi_system_save.json", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_nms_export_csv() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["export", "--save", "data/test/multi_system_save.json", "--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("planet_name"));
}

#[test]
fn test_nms_route_basic() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&[
            "route",
            "--save", "data/test/multi_system_save.json",
            "--biome", "Lush",
            "--max-targets", "3",
        ])
        .assert()
        .success();
}

#[test]
fn test_nms_stats() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["stats", "--save", "data/test/multi_system_save.json", "--biomes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Biome"));
}

#[test]
fn test_nms_show_system() {
    Command::cargo_bin("nms")
        .unwrap()
        .args(&["show", "--save", "data/test/minimal_save.json", "system", "Test System"])
        .assert()
        .success();
}

#[test]
fn test_nms_unknown_command() {
    Command::cargo_bin("nms")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### Pipeline Integration Tests

Test the full data flow without CLI:

```rust
// tests/pipeline_integration.rs
use nms_save::parse_save_file;
use nms_graph::GalaxyModel;
use nms_query::find::{execute_find, FindQuery};
use nms_query::stats::execute_stats;

#[test]
fn test_full_pipeline_parse_to_query() {
    let save = parse_save_file("data/test/multi_system_save.json").unwrap();
    let model = GalaxyModel::from_save(&save);

    let query = FindQuery {
        biome: Some(nms_core::biome::Biome::Lush),
        ..Default::default()
    };
    let results = execute_find(&model, &query).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_full_pipeline_empty_save() {
    let save = parse_save_file("data/test/empty_save.json").unwrap();
    let model = GalaxyModel::from_save(&save);
    assert_eq!(model.systems.len(), 0);

    let stats = execute_stats(&model);
    assert_eq!(stats.total_systems, 0);
}
```

### New Dev Dependencies

```toml
# Workspace Cargo.toml
assert_cmd = "2"
predicates = "3"

# crates/nms-cli/Cargo.toml
[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
```

---

## Milestone 7.8: crates.io Publishing

### Goal

Publish workspace crates to crates.io so users can `cargo install nms-copilot` and `cargo install nms`.

### Publishing Order

Crates must be published in dependency order:

```
1. nms-core        (no internal deps)
2. nms-save        (depends on nms-core)
3. nms-compat      (depends on nms-save)
4. nms-graph       (depends on nms-core, nms-save)
5. nms-query       (depends on nms-core, nms-graph)
6. nms-watch       (depends on nms-core, nms-save, nms-graph)
7. nms-cache       (depends on nms-core, nms-graph)
8. nms-cli         (depends on all above)  -> binary: `nms`
9. nms-copilot     (depends on all above)  -> binary: `nms-copilot`
10. nms-mcp        (depends on all above)  -> binary: `nms-mcp`
11. nms             (meta-crate, re-exports)
```

### Cargo.toml Metadata Check

Every crate needs:

```toml
[package]
name = "nms-core"
version = "0.2.0"
edition = "2024"
rust-version = "1.85"
license = "MIT OR Apache-2.0"
repository = "https://github.com/oxur/nms-copilot"
homepage = "https://github.com/oxur/nms-copilot"
description = "Core types for NMS Copilot — portal glyphs, coordinates, biomes"
keywords = ["nms", "no-mans-sky", "space", "exploration"]
categories = ["games", "command-line-utilities"]
```

### Pre-publish Checklist

```bash
# For each crate:
cargo publish --dry-run -p nms-core
cargo publish --dry-run -p nms-save
# ... etc

# Verify all workspace path deps have version specs
# (already done: `nms-core = { path = "crates/nms-core", version = "0.2.0" }`)
```

### fabryk-mcp Dependency

`nms-mcp` depends on `fabryk-mcp` which is a local path dep (`../ecl/crates/fabryk-mcp`). Before publishing `nms-mcp`, `fabryk-mcp` must be published to crates.io first, and the dependency changed from path to version.

If `fabryk-mcp` isn't ready for crates.io, `nms-mcp` can be excluded from the initial publish:

```toml
[workspace]
exclude = ["crates/nms-mcp"]
```

### CI Publishing Workflow

```yaml
# .github/workflows/publish.yml
name: Publish
on:
  push:
    tags: ['v*']
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish -p nms-core
      - run: cargo publish -p nms-save
      # ... in order, with sleep between for index updates
```

---

## Milestone 7.10: Color Themes

### Goal

Add configurable ANSI colors to terminal output. The comment in `display.rs:4` says "No ANSI color codes yet (added in Phase 7 polish)."

### Config

Add a `[display]` section to `~/.nms-copilot/config.toml`:

```toml
[display]
# Use ANSI colors in output (default: true if terminal supports it)
colors = true
# Color theme: "default", "light", "dark", "none"
theme = "default"
# Use emoji for portal glyphs (default: true)
emoji_glyphs = true
```

### Theme Definition

```rust
// crates/nms-query/src/theme.rs

/// A color theme for terminal output.
#[derive(Debug, Clone)]
pub struct Theme {
    pub header: Style,
    pub system_name: Style,
    pub planet_name: Style,
    pub biome_lush: Style,
    pub biome_toxic: Style,
    pub biome_scorched: Style,
    pub biome_frozen: Style,
    pub biome_barren: Style,
    pub biome_dead: Style,
    pub biome_exotic: Style,
    pub biome_other: Style,
    pub distance: Style,
    pub glyphs: Style,
    pub muted: Style,
}

/// A simple ANSI style: foreground color + bold flag.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Gray,
}

impl Style {
    /// Wrap a string with ANSI escape codes.
    pub fn paint(&self, s: &str) -> String {
        if self.fg.is_none() && !self.bold {
            return s.to_string();
        }
        let mut codes = Vec::new();
        if self.bold {
            codes.push("1".to_string());
        }
        if let Some(color) = self.fg {
            codes.push(color.ansi_code().to_string());
        }
        format!("\x1b[{}m{}\x1b[0m", codes.join(";"), s)
    }
}
```

### Default Theme

```rust
impl Theme {
    pub fn default_dark() -> Self {
        Self {
            header: Style { fg: Some(Color::BrightWhite), bold: true },
            system_name: Style { fg: Some(Color::BrightCyan), bold: false },
            planet_name: Style { fg: Some(Color::White), bold: false },
            biome_lush: Style { fg: Some(Color::BrightGreen), bold: false },
            biome_toxic: Style { fg: Some(Color::Yellow), bold: false },
            biome_scorched: Style { fg: Some(Color::BrightRed), bold: false },
            biome_frozen: Style { fg: Some(Color::BrightBlue), bold: false },
            biome_barren: Style { fg: Some(Color::Gray), bold: false },
            biome_dead: Style { fg: Some(Color::Gray), bold: false },
            biome_exotic: Style { fg: Some(Color::BrightMagenta), bold: false },
            biome_other: Style { fg: None, bold: false },
            distance: Style { fg: Some(Color::Yellow), bold: false },
            glyphs: Style { fg: None, bold: false },
            muted: Style { fg: Some(Color::Gray), bold: false },
        }
    }

    pub fn none() -> Self {
        // All styles are no-op
        Self {
            header: Style { fg: None, bold: false },
            // ... all fields None/false
        }
    }

    /// Get the style for a specific biome.
    pub fn biome_style(&self, biome: &Biome) -> &Style {
        match biome {
            Biome::Lush => &self.biome_lush,
            Biome::Toxic => &self.biome_toxic,
            Biome::Scorched => &self.biome_scorched,
            Biome::Frozen => &self.biome_frozen,
            Biome::Barren => &self.biome_barren,
            Biome::Dead => &self.biome_dead,
            Biome::Exotic => &self.biome_exotic,
            _ => &self.biome_other,
        }
    }
}
```

### Integration with Display

Update `format_find_results` and other formatters in `display.rs` to accept an optional theme:

```rust
pub fn format_find_results(results: &[FindResult], theme: &Theme) -> String {
    if results.is_empty() {
        return theme.muted.paint("  No results found.\n");
    }

    let mut out = String::new();

    // Header
    out.push_str(&theme.header.paint(&format!(
        "  {:<3} {:<18} {:<11} {:<20} {:<11} {}\n",
        "#", "Planet", "Biome", "System", "Distance", "Portal Glyphs"
    )));

    for (i, r) in results.iter().enumerate() {
        let biome_str = theme.biome_style(&r.planet.biome)
            .paint(&r.planet.biome.to_string());
        let system_str = theme.system_name
            .paint(r.system.name.as_deref().unwrap_or("(unnamed)"));
        let dist_str = theme.distance
            .paint(&format_distance(r.distance_ly));
        // ...
    }

    out
}
```

### Terminal Detection

Only use colors when outputting to a terminal (not when piped):

```rust
pub fn should_use_colors(config_colors: bool) -> bool {
    config_colors && atty::is(atty::Stream::Stdout)
}
```

Or use `std::io::IsTerminal` (stabilized in Rust 1.70):

```rust
use std::io::IsTerminal;

pub fn should_use_colors(config_colors: bool) -> bool {
    config_colors && std::io::stdout().is_terminal()
}
```

### Tests

```rust
#[test]
fn test_style_paint_with_color() {
    let style = Style { fg: Some(Color::Green), bold: false };
    let result = style.paint("hello");
    assert!(result.contains("\x1b["));
    assert!(result.contains("hello"));
    assert!(result.ends_with("\x1b[0m"));
}

#[test]
fn test_style_paint_no_color() {
    let style = Style { fg: None, bold: false };
    let result = style.paint("hello");
    assert_eq!(result, "hello");
}

#[test]
fn test_style_paint_bold() {
    let style = Style { fg: None, bold: true };
    let result = style.paint("hello");
    assert!(result.contains("\x1b[1m"));
}

#[test]
fn test_theme_biome_style() {
    let theme = Theme::default_dark();
    let style = theme.biome_style(&Biome::Lush);
    assert!(matches!(style.fg, Some(Color::BrightGreen)));
}

#[test]
fn test_theme_none_no_escapes() {
    let theme = Theme::none();
    let result = theme.header.paint("Header");
    assert!(!result.contains("\x1b["));
}

#[test]
fn test_colored_find_results() {
    let theme = Theme::default_dark();
    let results = vec![test_find_result()];
    let output = format_find_results(&results, &theme);
    assert!(output.contains("\x1b[")); // Has ANSI codes
}

#[test]
fn test_uncolored_find_results() {
    let theme = Theme::none();
    let results = vec![test_find_result()];
    let output = format_find_results(&results, &theme);
    assert!(!output.contains("\x1b[")); // No ANSI codes
}
```

---

## Implementation Notes

### No External Color Crate

The theme system uses raw ANSI codes via a small `Style` type. This avoids adding `colored`, `owo-colors`, or `termcolor` as dependencies. The color palette is intentionally limited -- NMS Copilot only needs ~15 colors.

### Backward Compatibility of `display.rs`

Adding a `theme` parameter to format functions is a breaking change within the workspace. Update all callers (CLI, REPL, MCP) in the same commit. The MCP server can use `Theme::none()` since AI clients don't render ANSI.

### Coverage Target

Per CLAUDE.md, target 95%+ code coverage. The integration tests in 7.7 are critical for hitting this across the full pipeline. Use `make coverage` to verify before publishing.
