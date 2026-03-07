# Milestone 3.7 -- Cache Management (nms-cache)

Freshness checking (save file mtime vs cache mtime), auto-rebuild on stale cache, `--no-cache` CLI flag, and integration into the nms-copilot startup path.

## Crate: `nms-cache`

Path: `crates/nms-cache/`

### Dependencies

No new dependencies beyond what milestone 3.6 adds.

---

## New File: `crates/nms-cache/src/freshness.rs`

Cache freshness checking based on file modification times.

```rust
//! Cache freshness checking.
//!
//! Compares the modification time of the save file against the cache file
//! to determine if the cache is still valid.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::error::CacheError;

/// Check if the cache file is fresh relative to the save file.
///
/// Returns `true` if the cache exists and is newer than the save file.
/// Returns `false` if the cache is missing, older than the save, or if
/// timestamps can't be read.
pub fn is_cache_fresh(cache_path: &Path, save_path: &Path) -> bool {
    let cache_mtime = match file_mtime(cache_path) {
        Some(t) => t,
        None => return false,
    };
    let save_mtime = match file_mtime(save_path) {
        Some(t) => t,
        None => return false,
    };
    cache_mtime > save_mtime
}

/// Get the modification time of a file.
fn file_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

/// Load a model from cache if fresh, otherwise parse the save file.
///
/// This is the primary entry point for the startup path.
/// Returns `(GalaxyModel, was_cached)`.
pub fn load_or_rebuild(
    cache_path: &Path,
    save_path: &Path,
    no_cache: bool,
) -> Result<(nms_graph::GalaxyModel, bool), Box<dyn std::error::Error>> {
    // Try cache first (unless --no-cache)
    if !no_cache && is_cache_fresh(cache_path, save_path) {
        match crate::read_cache(cache_path) {
            Ok(data) => {
                let model = crate::rebuild_model(&data);
                return Ok((model, true));
            }
            Err(e) => {
                eprintln!("Warning: cache read failed ({e}), rebuilding from save");
            }
        }
    }

    // Parse save file
    let save = nms_save::parse_save_file(save_path)?;
    let model = nms_graph::GalaxyModel::from_save(&save);

    // Write cache for next time
    if !no_cache {
        let data = crate::extract_cache_data(&model, save.version);
        if let Err(e) = crate::write_cache(&data, cache_path) {
            eprintln!("Warning: could not write cache ({e})");
        }
    }

    Ok((model, false))
}
```

---

## Updated File: `crates/nms-cache/src/lib.rs`

```rust
//! Zero-copy serialization cache for NMS Copilot.
//!
//! Serializes the in-memory `GalaxyModel` discovery data to an rkyv archive
//! for near-instant startup on subsequent runs. Indices are rebuilt on load.

pub mod data;
pub mod error;
pub mod freshness;
pub mod serialize;

pub use data::CacheData;
pub use error::CacheError;
pub use freshness::{is_cache_fresh, load_or_rebuild};
pub use serialize::{extract_cache_data, read_cache, rebuild_model, write_cache};
```

---

## Updated Dependency: `nms-cache` needs `nms-save`

```toml
# crates/nms-cache/Cargo.toml
[dependencies]
nms-core = { workspace = true, features = ["archive"] }
nms-graph = { workspace = true }
nms-save = { workspace = true }
rkyv = { version = "0.8", features = ["validation"] }
```

---

## Integration into `nms-copilot`

### Updated File: `crates/nms-copilot/src/main.rs`

Replace the direct save parsing with cache-aware loading:

```rust
use nms_cache::load_or_rebuild;

fn load_model(
    save_path: Option<PathBuf>,
    no_cache: bool,
) -> Result<(GalaxyModel, bool), Box<dyn std::error::Error>> {
    let save_path = match save_path {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };

    let cache_path = paths::cache_path();

    // Ensure data directory exists for cache file
    paths::ensure_data_dir()?;

    load_or_rebuild(&cache_path, &save_path, no_cache)
}

// In main():
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let save_path = parse_save_arg(&args);
    let no_cache = args.iter().any(|a| a == "--no-cache");

    let (model, was_cached) = match load_model(save_path, no_cache) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading save: {e}");
            std::process::exit(1);
        }
    };

    let load_source = if was_cached { "cache" } else { "save file" };
    println!(
        "NMS Copilot v{}\n\
         Loaded {} systems, {} planets, {} bases (from {load_source})\n\
         Type 'help' for commands, 'exit' to quit.\n",
        env!("CARGO_PKG_VERSION"),
        model.systems.len(),
        model.planets.len(),
        model.bases.len(),
    );

    // ... REPL loop ...
}
```

### Integration into `nms-cli`

The CLI can also benefit from caching. Add `--no-cache` flag to CLI commands:

```rust
// In crates/nms-cli/src/main.rs, add to each command that loads a save:
/// Skip the cache and parse the save file directly.
#[arg(long)]
no_cache: bool,
```

The actual integration is optional for this milestone -- the CLI can continue to parse directly since it's one-shot. The cache is most valuable for the REPL's startup time.

---

## Tests

### File: `crates/nms-cache/src/freshness.rs` (inline tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_fresh_cache_returns_true() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        // Create save file first
        fs::write(&save_path, "save data").unwrap();

        // Wait a moment then create cache (so mtime is newer)
        thread::sleep(Duration::from_millis(50));
        fs::write(&cache_path, "cache data").unwrap();

        assert!(is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_stale_cache_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        // Create cache first
        fs::write(&cache_path, "cache data").unwrap();

        // Wait then modify save (so save is newer)
        thread::sleep(Duration::from_millis(50));
        fs::write(&save_path, "updated save").unwrap();

        assert!(!is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_missing_cache_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&save_path, "save data").unwrap();
        assert!(!is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_missing_save_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&cache_path, "cache data").unwrap();
        assert!(!is_cache_fresh(&cache_path, &save_path));
    }
}
```

### File: `crates/nms-cache/tests/load_integration.rs`

```rust
use std::fs;
use std::thread;
use std::time::Duration;

use nms_cache::{extract_cache_data, load_or_rebuild, write_cache};
use nms_graph::GalaxyModel;

fn test_save_json() -> &'static str {
    r#"{
        "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
        "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
        "BaseContext": {
            "GameMode": 1,
            "PlayerStateData": {
                "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                "Units": 0, "Nanites": 0, "Specials": 0,
                "PersistentPlayerBases": []
            }
        },
        "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
        "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "FL": {"U": 1}},
            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "FL": {"U": 1}}
        ]}}}
    }"#
}

#[test]
fn load_or_rebuild_from_save_when_no_cache() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    fs::write(&save_path, test_save_json()).unwrap();

    let (model, was_cached) = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(!was_cached);
    assert!(!model.systems.is_empty());

    // Cache file should now exist
    assert!(cache_path.exists());
}

#[test]
fn load_or_rebuild_uses_cache_when_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    // Write save and build cache
    fs::write(&save_path, test_save_json()).unwrap();
    let save = nms_save::parse_save(test_save_json().as_bytes()).unwrap();
    let model = GalaxyModel::from_save(&save);
    let data = extract_cache_data(&model, 4720);

    thread::sleep(Duration::from_millis(50));
    write_cache(&data, &cache_path).unwrap();

    let (_, was_cached) = load_or_rebuild(&cache_path, &save_path, false).unwrap();
    assert!(was_cached);
}

#[test]
fn load_or_rebuild_skips_cache_with_no_cache_flag() {
    let dir = tempfile::tempdir().unwrap();
    let save_path = dir.path().join("save.json");
    let cache_path = dir.path().join("galaxy.rkyv");

    fs::write(&save_path, test_save_json()).unwrap();

    // Even if cache exists, --no-cache skips it
    let (_, was_cached) = load_or_rebuild(&cache_path, &save_path, true).unwrap();
    assert!(!was_cached);
    // And no cache file should be written
    assert!(!cache_path.exists());
}
```

---

## Implementation Notes

1. **Freshness check**: Simple mtime comparison. If the save file is newer than the cache, the cache is stale. This handles the common case of the game auto-saving while the copilot is not running.

2. **Graceful degradation**: If the cache can't be read (corrupted, wrong version), the system falls back to parsing the save file and writes a new cache. Warnings are printed but execution continues.

3. **Atomic cache writes**: Uses write-to-temp-then-rename to prevent corruption from interrupted writes.

4. **`--no-cache` flag**: Useful for debugging or when the cache might be corrupted. Forces a full parse from the save file and skips writing a new cache.

5. **Cache location**: `~/.nms-copilot/galaxy.rkyv`, set up by the `paths` module from milestone 3.2.

6. **Startup flow**: `load_or_rebuild()` encapsulates the full decision: check cache freshness -> try cache -> fall back to save -> write new cache. Callers just get a `GalaxyModel`.

7. **Cache versioning**: The `save_version` field in `CacheData` can be used in the future to invalidate caches when the save format changes or when our data model evolves.
