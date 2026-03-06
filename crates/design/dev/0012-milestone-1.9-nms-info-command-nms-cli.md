# Milestone 1.9 -- `nms info` Command (nms-cli)

First working CLI command: point at a decompressed/deobfuscated save file, get a formatted summary.

## Crate: `nms-cli`

Path: `crates/nms-cli/`

### Dependencies to add to `crates/nms-cli/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-compat = { workspace = true }
clap = { version = "4", features = ["derive"] }
thiserror = "2"
```

---

## Command Syntax

```
nms info [--save <path>]
nms info                    # auto-detect save location
```

If `--save` is provided, read that file directly. If omitted, scan the default NMS save directory for the current platform and use the most recently modified `save*.hg` file.

---

## Output Format

```
NMS Copilot -- Save File Summary
================================

  Save Name:       main - Steam
  Platform:        Mac|Final
  Version:         4720
  Play Time:       684h 32m
  Game Mode:       Normal (1)

  Galaxy:          Euclid
  Voxel Position:  X=1699, Y=-2, Z=165
  System Index:    369
  Planet Index:    0

  Discoveries:     2,847
    Solar Systems: 412
    Planets:       893
    Sectors:       156
    Animals:       521
    Flora:         489
    Minerals:      376

  Bases:           5
    Gugestor Colony         HomePlanetBase  0x40050003AB8C07
    Dread Outpost           HomePlanetBase  0x20C200FE0A56A3
    ...

  Units:           -919,837,762
  Nanites:         272,127
  Quicksilver:     2,230
```

Notes on the output:

- Use ASCII box-drawing characters (equals signs) for the header rule, not Unicode
- Play time is computed from `CommonStateData.TotalPlayTime` (seconds)
- Game Mode is from `BaseContext.GameMode` (integer) plus a display name if known
- Galaxy name is looked up from `nms_core::galaxy_by_index(reality_index)`
- Discovery counts are tallied from `DiscoveryManagerData.DiscoveryData-v1.Store.Record[]` grouped by `DD.DT`
- Base listing shows Name, BaseType, and GalacticAddress (hex)
- Currency values are formatted with thousand separators (commas)

---

## CLI Setup with clap

### File: `crates/nms-cli/src/main.rs`

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod info;
mod resolve;

#[derive(Parser)]
#[command(
    name = "nms",
    about = "NMS Copilot CLI -- search planets, plan routes, convert glyphs",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display save file summary
    Info {
        /// Path to a decompressed/deobfuscated save file (JSON).
        /// If omitted, auto-detects the most recent save.
        #[arg(long)]
        save: Option<PathBuf>,
    },
    // Future commands: Convert, Find, Show, Route, Stats, Export
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { save } => info::run(save),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
```

---

## Info Command Implementation

### File: `crates/nms-cli/src/info.rs`

```rust
use std::path::PathBuf;
use std::collections::HashMap;
use nms_save::model::SaveFile;
use nms_core::galaxy::galaxy_by_index;

pub fn run(save_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve save file path
    let path = match save_path {
        Some(p) => p,
        None => crate::resolve::find_most_recent_save()?,
    };

    // 2. Read and parse
    let save = nms_save::parse_save_file(&path)?;

    // 3. Print summary
    print_summary(&save);

    Ok(())
}

fn print_summary(save: &SaveFile) {
    println!("NMS Copilot -- Save File Summary");
    println!("================================");
    println!();

    // Basic info
    println!("  Save Name:       {}", save.common_state_data.save_name);
    println!("  Platform:        {}", save.platform);
    println!("  Version:         {}", save.version);
    println!("  Play Time:       {}", format_play_time(save.common_state_data.total_play_time));
    println!("  Game Mode:       {}", format_game_mode(save.base_context.game_mode));
    println!();

    // Location from active context
    let ps = save.active_player_state();
    let ua = &ps.universe_address;
    let galaxy = galaxy_by_index(ua.reality_index);
    let ga = &ua.galactic_address;
    println!("  Galaxy:          {}", galaxy.name);
    println!("  Voxel Position:  X={}, Y={}, Z={}", ga.voxel_x, ga.voxel_y, ga.voxel_z);
    println!("  System Index:    {}", ga.solar_system_index);
    println!("  Planet Index:    {}", ga.planet_index);
    println!();

    // Discovery counts
    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for rec in records {
        *counts.entry(rec.dd.dt.as_str()).or_insert(0) += 1;
    }
    let total = records.len();
    println!("  Discoveries:     {}", format_number(total as i64));
    for (label, key) in [
        ("Solar Systems", "SolarSystem"),
        ("Planets", "Planet"),
        ("Sectors", "Sector"),
        ("Animals", "Animal"),
        ("Flora", "Flora"),
        ("Minerals", "Mineral"),
    ] {
        let count = counts.get(key).copied().unwrap_or(0);
        if count > 0 {
            println!("    {:<15}{}", label, format_number(count as i64));
        }
    }
    println!();

    // Bases
    let bases = &ps.persistent_player_bases;
    println!("  Bases:           {}", bases.len());
    for base in bases {
        let name = if base.name.is_empty() { "(unnamed)" } else { &base.name };
        let btype = &base.base_type.persistent_base_types;
        let addr = base.galactic_address.0;
        println!("    {:<24}{:<20}0x{:014X}", name, btype, addr);
    }
    println!();

    // Currencies
    println!("  Units:           {}", format_number(ps.units));
    println!("  Nanites:         {}", format_number(ps.nanites));
    println!("  Quicksilver:     {}", format_number(ps.specials));
}

/// Format seconds as "Xd Yh Zm" or "Xh Ym" or "Xm Ys"
fn format_play_time(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m {}s", minutes, seconds % 60)
    }
}

/// Format game mode integer to display string.
fn format_game_mode(mode: u32) -> String {
    let name = match mode {
        0 => "Unspecified",
        1 => "Normal",
        2 => "Creative",
        3 => "Survival",
        4 => "Ambient",
        5 => "Permadeath",
        6 => "Seasonal/Expedition",
        _ => "Unknown",
    };
    format!("{} ({})", name, mode)
}

/// Format an integer with thousands separators (commas).
/// Handles negative numbers.
fn format_number(n: i64) -> String {
    let negative = n < 0;
    let abs = if negative { (n as i128).unsigned_abs() } else { n as u128 };
    let s = abs.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    let formatted: String = result.chars().rev().collect();
    if negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}
```

---

## Save File Path Resolution

### File: `crates/nms-cli/src/resolve.rs`

```rust
use std::path::PathBuf;
use std::fs;

/// Auto-detect the most recently modified NMS save file.
///
/// Searches the default NMS save directory for the current platform,
/// finds all `save*.hg` files, and returns the one with the newest
/// modification time.
pub fn find_most_recent_save() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base_dir = nms_save_directory()?;
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    // Scan all account directories (st_* for Steam, DefaultUser for GOG)
    if base_dir.exists() {
        for entry in fs::read_dir(&base_dir)? {
            let entry = entry?;
            let account_dir = entry.path();
            if !account_dir.is_dir() {
                continue;
            }
            // Look for save*.hg files in this account directory
            scan_save_files(&account_dir, &mut candidates)?;
        }
    }

    if candidates.is_empty() {
        return Err(format!(
            "No NMS save files found in {}. Use --save <path> to specify manually.",
            base_dir.display()
        ).into());
    }

    // Sort by modification time, newest first
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(candidates[0].0.clone())
}

/// Scan a directory for save*.hg files and add to candidates with their mtime.
fn scan_save_files(
    dir: &PathBuf,
    candidates: &mut Vec<(PathBuf, std::time::SystemTime)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("save") && name.ends_with(".hg") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        candidates.push((path, mtime));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Return the platform-appropriate NMS save directory.
fn nms_save_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join("Library/Application Support/HelloGames/NMS"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")?;
        // Steam on Linux via Proton
        Ok(PathBuf::from(home).join(
            ".local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/Application Data/HelloGames/NMS"
        ))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")?;
        Ok(PathBuf::from(appdata).join("HelloGames/NMS"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported platform for auto-detection. Use --save <path>.".into())
    }
}
```

---

## Tests

### Play time formatting tests (`crates/nms-cli/src/info.rs`, inline)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_play_time_zero() {
        assert_eq!(format_play_time(0), "0m 0s");
    }

    #[test]
    fn format_play_time_seconds_only() {
        assert_eq!(format_play_time(45), "0m 45s");
    }

    #[test]
    fn format_play_time_minutes_and_seconds() {
        assert_eq!(format_play_time(125), "2m 5s");
    }

    #[test]
    fn format_play_time_hours_and_minutes() {
        assert_eq!(format_play_time(3661), "1h 1m");
    }

    #[test]
    fn format_play_time_days() {
        assert_eq!(format_play_time(90061), "1d 1h 1m");
    }

    #[test]
    fn format_play_time_actual_save_value() {
        // 2464349 seconds from actual save file
        // = 28d 12h 32m 29s
        assert_eq!(format_play_time(2464349), "28d 12h 32m");
    }

    #[test]
    fn format_number_positive() {
        assert_eq!(format_number(1234567890), "1,234,567,890");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(format_number(-919837762), "-919,837,762");
    }

    #[test]
    fn format_number_zero() {
        assert_eq!(format_number(0), "0");
    }

    #[test]
    fn format_number_small() {
        assert_eq!(format_number(42), "42");
    }

    #[test]
    fn format_number_thousands() {
        assert_eq!(format_number(1000), "1,000");
    }

    #[test]
    fn format_game_mode_normal() {
        assert_eq!(format_game_mode(1), "Normal (1)");
    }

    #[test]
    fn format_game_mode_creative() {
        assert_eq!(format_game_mode(2), "Creative (2)");
    }

    #[test]
    fn format_game_mode_expedition() {
        assert_eq!(format_game_mode(6), "Seasonal/Expedition (6)");
    }

    #[test]
    fn format_game_mode_unknown() {
        assert_eq!(format_game_mode(99), "Unknown (99)");
    }
}
```

### Integration test (`crates/nms-cli/tests/info_integration.rs`)

```rust
use nms_save::model::*;
use nms_save::parse_save;

/// Build a minimal valid save JSON and verify it parses correctly.
#[test]
fn parse_minimal_save_and_verify_fields() {
    let json = r#"{
        "Version": 4720,
        "Platform": "Mac|Final",
        "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test Save", "TotalPlayTime": 3661},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {
                    "RealityIndex": 0,
                    "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 2}
                },
                "Units": 5000000,
                "Nanites": 10000,
                "Specials": 500,
                "PersistentPlayerBases": [
                    {
                        "BaseVersion": 8,
                        "GalacticAddress": "0x40050003AB8C07",
                        "Position": [100.0, 200.0, 300.0],
                        "Forward": [1.0, 0.0, 0.0],
                        "LastUpdateTimestamp": 1700000000,
                        "Objects": [],
                        "RID": "",
                        "Owner": {"LID": "", "UID": "12345", "USN": "TestUser", "PTK": "ST", "TS": 0},
                        "Name": "Test Base",
                        "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
                        "LastEditedById": "",
                        "LastEditedByUsername": ""
                    }
                ]
            }
        },
        "ExpeditionContext": {
            "GameMode": 6,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": []
            }
        },
        "DiscoveryManagerData": {
            "DiscoveryData-v1": {
                "ReserveStore": 100, "ReserveManaged": 100,
                "Store": {
                    "Record": [
                        {"DD": {"UA": "0x513300F79B1D82", "DT": "Flora", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                        {"DD": {"UA": "0x40050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                        {"DD": {"UA": 12345678, "DT": "Planet", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "A", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
                    ]
                }
            }
        }
    }"#;

    let save = parse_save(json.as_bytes()).unwrap();

    assert_eq!(save.version, 4720);
    assert_eq!(save.common_state_data.save_name, "Test Save");
    assert_eq!(save.common_state_data.total_play_time, 3661);

    let ps = save.active_player_state();
    assert_eq!(ps.units, 5000000);
    assert_eq!(ps.nanites, 10000);
    assert_eq!(ps.specials, 500);
    assert_eq!(ps.persistent_player_bases.len(), 1);
    assert_eq!(ps.persistent_player_bases[0].name, "Test Base");

    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    assert_eq!(records.len(), 3);

    // Count by type
    let flora_count = records.iter().filter(|r| r.dd.dt == "Flora").count();
    let system_count = records.iter().filter(|r| r.dd.dt == "SolarSystem").count();
    let planet_count = records.iter().filter(|r| r.dd.dt == "Planet").count();
    assert_eq!(flora_count, 1);
    assert_eq!(system_count, 1);
    assert_eq!(planet_count, 1);
}
```

---

## Pipeline Summary

The `nms info` command pipeline for this milestone:

1. Resolve save file path (explicit `--save` or auto-detect)
2. Read file bytes from disk
3. Parse JSON into `SaveFile` struct (nms-save)
4. Extract and format summary information
5. Print formatted output to stdout

**Note:** This milestone assumes the save file is **already decompressed and deobfuscated** (plain JSON). The full pipeline (LZ4 decompression from milestone 1.5, XXTEA from 1.6, key deobfuscation from 1.7) is integrated later. For now, the `--save` flag should point to a `.json` file that has already been processed.

---

## Implementation Notes

1. **The `--save` path should accept both `.hg` (compressed) and `.json` (decompressed) files.** For this milestone, only `.json` files work. When LZ4/deobfuscation integration is complete, `.hg` files will also work. Emit a clear error message if the file appears to be binary (starts with LZ4 magic bytes) but the decompression pipeline is not yet wired up.

2. **Auto-detection scans for `.hg` files** but the parse may fail until the full pipeline is integrated. This is acceptable -- the auto-detect code is correct; it just needs the decompression layer.

3. **Units can be negative.** The save file stores Units as a signed 32-bit integer that overflows. Display it as-is with a negative sign.

4. **Quicksilver is stored as "Specials"** in the JSON. Display it as "Quicksilver" in the output.

5. **The active context** is determined by `save.active_context` -- either `"Main"` (BaseContext) or `"Expedition"` (ExpeditionContext).

6. **Game mode integer mapping** may not be exhaustive. The known values are from observation. Unknown values display as "Unknown (N)".
