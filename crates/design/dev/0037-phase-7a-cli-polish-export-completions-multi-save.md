# Phase 7A -- CLI Polish: Export, Completions, Multi-Save

Milestones 7.1-7.3: Export command, shell completions, and multi-save slot support.

**Depends on:** Phases 1-2 (save parsing, galaxy model, query engine, CLI).

---

## Architecture Overview

Phase 7A adds three CLI features that build directly on existing infrastructure:

1. **`nms export`** -- Serializes query results to JSON or CSV for scripting/pipelines
2. **Shell completions** -- `clap_complete` generation for bash, zsh, fish
3. **Multi-save** -- `--slot` flag and `nms saves` listing command

All three are additive -- no changes to existing tool behavior.

---

## New Dependencies

### Workspace `Cargo.toml`

```toml
csv = "1"
clap_complete = "4"
```

### `crates/nms-cli/Cargo.toml`

```toml
[dependencies]
csv = { workspace = true }
clap_complete = { workspace = true }
```

---

## Milestone 7.1: `nms export` Command

### Goal

Export filtered planet/system data as JSON or CSV for use in scripts, spreadsheets, or external tools.

### CLI Interface

```
nms export --biome Lush --within 500 --format json > lush_nearby.json
nms export --format csv > all_planets.csv
nms export --discoverer "oubiwann" --format json
```

Reuses `FindQuery` from nms-query -- same filter flags as `nms find`.

### New File: `crates/nms-cli/src/export.rs`

```rust
use std::io::{self, Write};
use std::path::PathBuf;

use nms_core::biome::Biome;
use nms_graph::GalaxyModel;
use nms_query::find::{execute_find, FindQuery, FindResult, ReferencePoint};

#[derive(Debug, Clone)]
pub struct ExportArgs {
    pub save: Option<PathBuf>,
    pub biome: Option<String>,
    pub infested: bool,
    pub within: Option<f64>,
    pub nearest: Option<usize>,
    pub named: bool,
    pub discoverer: Option<String>,
    pub from: Option<String>,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            other => Err(format!("unknown format: {other} (expected json or csv)").into()),
        }
    }
}

pub fn run(args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let model = crate::load_model(args.save)?;
    let query = build_query(&args)?;
    let results = execute_find(&model, &query)?;

    let stdout = io::stdout();
    let mut out = stdout.lock();

    match args.format {
        ExportFormat::Json => write_json(&mut out, &results)?,
        ExportFormat::Csv => write_csv(&mut out, &results)?,
    }

    Ok(())
}

fn write_json(out: &mut impl Write, results: &[FindResult]) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize as array of flat objects
    let records: Vec<ExportRecord> = results.iter().map(ExportRecord::from).collect();
    serde_json::to_writer_pretty(out, &records)?;
    writeln!(out)?;
    Ok(())
}

fn write_csv(out: &mut impl Write, results: &[FindResult]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(out);
    for result in results {
        wtr.serialize(ExportRecord::from(result))?;
    }
    wtr.flush()?;
    Ok(())
}
```

### Export Record

A flat struct for serialization (no nested objects):

```rust
#[derive(Debug, serde::Serialize)]
pub struct ExportRecord {
    pub planet_name: String,
    pub biome: String,
    pub system_name: String,
    pub distance_ly: f64,
    pub portal_glyphs: String,
    pub portal_emoji: String,
    pub coords_x: i32,
    pub coords_y: i32,
    pub coords_z: i32,
    pub galaxy: String,
    pub discoverer: String,
    pub infested: bool,
}

impl From<&FindResult> for ExportRecord {
    fn from(r: &FindResult) -> Self {
        Self {
            planet_name: r.planet.name.clone().unwrap_or_default(),
            biome: r.planet.biome.to_string(),
            system_name: r.system.name.clone().unwrap_or_default(),
            distance_ly: r.distance_ly,
            portal_glyphs: r.portal_hex.clone(),
            portal_emoji: nms_query::display::hex_to_emoji(&r.portal_hex),
            coords_x: r.system.address.voxel_x(),
            coords_y: r.system.address.voxel_y(),
            coords_z: r.system.address.voxel_z(),
            galaxy: nms_core::galaxy::Galaxy::by_index(
                r.system.address.reality_index()
            ).name.to_string(),
            discoverer: r.system.discoverer.clone().unwrap_or_default(),
            infested: r.planet.infested,
        }
    }
}
```

### CLI Registration in `main.rs`

```rust
/// Export filtered planets as JSON or CSV.
Export {
    #[arg(long)]
    save: Option<PathBuf>,

    #[arg(long)]
    biome: Option<String>,

    #[arg(long)]
    infested: bool,

    #[arg(long)]
    within: Option<f64>,

    #[arg(long)]
    nearest: Option<usize>,

    #[arg(long)]
    named: bool,

    #[arg(long)]
    discoverer: Option<String>,

    #[arg(long)]
    from: Option<String>,

    /// Output format: json, csv (default: json).
    #[arg(long, default_value = "json")]
    format: String,
},
```

### Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_str_json() {
        assert!(matches!(ExportFormat::from_str("json").unwrap(), ExportFormat::Json));
        assert!(matches!(ExportFormat::from_str("JSON").unwrap(), ExportFormat::Json));
    }

    #[test]
    fn test_export_format_from_str_csv() {
        assert!(matches!(ExportFormat::from_str("csv").unwrap(), ExportFormat::Csv));
    }

    #[test]
    fn test_export_format_from_str_unknown() {
        assert!(ExportFormat::from_str("xml").is_err());
    }

    #[test]
    fn test_write_json_empty() {
        let mut buf = Vec::new();
        write_json(&mut buf, &[]).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s.trim(), "[]");
    }

    #[test]
    fn test_write_csv_empty() {
        let mut buf = Vec::new();
        write_csv(&mut buf, &[]).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.is_empty() || s.contains("planet_name")); // header only or empty
    }

    #[test]
    fn test_export_record_from_find_result() {
        let result = test_find_result();
        let record = ExportRecord::from(&result);
        assert_eq!(record.biome, "Lush");
        assert!(!record.portal_glyphs.is_empty());
    }
}
```

---

## Milestone 7.2: Shell Completions

### Goal

Generate shell completions for bash, zsh, and fish via `nms completions <shell>`.

### New File: `crates/nms-cli/src/completions.rs`

```rust
use std::io;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::Cli;

pub fn run(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}
```

### CLI Registration

```rust
/// Generate shell completions.
Completions {
    /// Shell to generate completions for: bash, zsh, fish.
    #[arg(value_enum)]
    shell: clap_complete::Shell,
},
```

### Usage

```bash
# Install for bash
nms completions bash > ~/.bash_completion.d/nms

# Install for zsh
nms completions zsh > ~/.zfunc/_nms

# Install for fish
nms completions fish > ~/.config/fish/completions/nms.fish
```

### Tests

```rust
#[test]
fn test_completions_bash_generates_output() {
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(Shell::Bash, &mut cmd, "nms", &mut buf);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("nms"));
}

#[test]
fn test_completions_zsh_generates_output() {
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(Shell::Zsh, &mut cmd, "nms", &mut buf);
    assert!(!buf.is_empty());
}
```

---

## Milestone 7.3: Multi-Save Support

### Goal

Let users work with any save slot, not just the most recent. Add `nms saves` to list all slots, and `--slot N` to target a specific one.

### Existing Infrastructure

`nms-save::locate` already provides everything needed:

- `list_accounts(save_dir)` -- finds all account directories
- `list_saves(account_dir)` -- lists all save files sorted by mtime
- `group_into_slots(saves)` -- pairs manual+auto saves into `SaveSlot` structs
- `SaveSlot::most_recent()` -- picks the newest file in a slot

### New File: `crates/nms-cli/src/saves.rs`

```rust
use nms_save::locate::{
    list_accounts, list_saves, group_into_slots, nms_save_dir_checked,
    SaveSlot, SaveType,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let save_dir = nms_save_dir_checked()?;
    let accounts = list_accounts(&save_dir)?;

    for account in &accounts {
        println!("Account: {} ({})", account.name(), account.kind());

        let saves = list_saves(account.path())?;
        let slots = group_into_slots(&saves);

        if slots.is_empty() {
            println!("  No save slots found.\n");
            continue;
        }

        println!("  {:<6} {:<10} {:<10} {}", "Slot", "Manual", "Auto", "Most Recent");
        for slot in &slots {
            let manual = if slot.manual().is_some() { "yes" } else { "-" };
            let auto = if slot.auto().is_some() { "yes" } else { "-" };
            let recent = slot.most_recent()
                .map(|s| format!("{} ({})", s.save_type(), format_mtime(s.modified())))
                .unwrap_or_default();
            println!("  {:<6} {:<10} {:<10} {}", slot.slot(), manual, auto, recent);
        }
        println!();
    }
    Ok(())
}
```

### `--slot` Flag

Add a global `--slot` argument. When present, resolve the save file from that slot instead of the most recent:

```rust
/// Resolve a save file path from --save, --slot, or auto-detect.
pub fn resolve_save(save: Option<PathBuf>, slot: Option<u8>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = save {
        return Ok(path);
    }
    if let Some(slot_num) = slot {
        let save_dir = nms_save::locate::nms_save_dir_checked()?;
        let accounts = nms_save::locate::list_accounts(&save_dir)?;
        // Use first account (most common case)
        let saves = nms_save::locate::list_saves(accounts[0].path())?;
        let slots = nms_save::locate::group_into_slots(&saves);
        let target = slots.iter()
            .find(|s| s.slot() == slot_num)
            .ok_or_else(|| format!("save slot {slot_num} not found"))?;
        let file = target.most_recent()
            .ok_or_else(|| format!("save slot {slot_num} is empty"))?;
        return Ok(file.path().to_path_buf());
    }
    // Auto-detect most recent
    Ok(nms_save::locate::find_most_recent_save()?.path().to_path_buf())
}
```

### CLI Changes

Add `--slot` to the top-level `Cli` struct as a global arg, and add the `Saves` subcommand:

```rust
#[derive(Parser)]
#[command(name = "nms", about = "...", version)]
struct Cli {
    /// Use a specific save slot (1-15) instead of most recent.
    #[arg(long, global = true)]
    slot: Option<u8>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// List all save slots.
    Saves,

    // ...
}
```

Then update every command's `run()` to use `resolve_save(save, slot)` instead of the direct path.

### Tests

```rust
#[test]
fn test_resolve_save_explicit_path() {
    let path = PathBuf::from("/tmp/save.hg");
    let result = resolve_save(Some(path.clone()), None).unwrap();
    assert_eq!(result, path);
}

#[test]
fn test_resolve_save_explicit_path_overrides_slot() {
    let path = PathBuf::from("/tmp/save.hg");
    let result = resolve_save(Some(path.clone()), Some(2)).unwrap();
    assert_eq!(result, path); // --save takes priority
}

#[test]
fn test_saves_list_with_tempdir() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::write(tmp.path().join("save.hg"), b"data").unwrap();
    std::fs::write(tmp.path().join("save2.hg"), b"data").unwrap();
    std::fs::write(tmp.path().join("save3.hg"), b"data").unwrap();

    let saves = nms_save::locate::list_saves(tmp.path()).unwrap();
    let slots = nms_save::locate::group_into_slots(&saves);

    assert_eq!(slots.len(), 2);
    assert!(slots[0].manual().is_some());
    assert!(slots[0].auto().is_some());
    assert!(slots[1].manual().is_some());
}
```

---

## Implementation Notes

### Shared `load_model` Refactor

All CLI commands currently duplicate save loading. With `--slot` support, extract a shared helper:

```rust
// crates/nms-cli/src/lib.rs or common.rs
pub fn load_model(save: Option<PathBuf>, slot: Option<u8>) -> Result<GalaxyModel, Box<dyn std::error::Error>> {
    let path = resolve_save(save, slot)?;
    let save = nms_save::parse_save_file(&path)?;
    Ok(nms_graph::GalaxyModel::from_save(&save))
}
```

### Output to stdout

`nms export` writes to stdout so it can be piped. Progress messages and errors go to stderr. This is the standard Unix convention and already matches the existing CLI pattern.

### CSV Column Order

CSV columns match `ExportRecord` field order: `planet_name, biome, system_name, distance_ly, portal_glyphs, portal_emoji, coords_x, coords_y, coords_z, galaxy, discoverer, infested`. The `csv` crate's `Serialize` derive handles headers automatically.
