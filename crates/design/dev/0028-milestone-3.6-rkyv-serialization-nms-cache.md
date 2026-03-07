# Milestone 3.6 -- rkyv Serialization (nms-cache)

Serialize the `GalaxyModel`'s discovery data to an rkyv archive for near-instant startup. Since petgraph and rstar don't implement rkyv traits, we serialize the raw data (systems, planets, bases, player state) and rebuild indices on load.

## Crate: `nms-cache`

Path: `crates/nms-cache/`

### Dependencies to update in `crates/nms-cache/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-graph = { workspace = true }
rkyv = { version = "0.8", features = ["validation"] }

[dev-dependencies]
nms-save = { workspace = true }
serde_json = { workspace = true }
tempfile = "3"
```

Add rkyv to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# ... existing ...
rkyv = { version = "0.8", features = ["validation"] }
```

### Required: rkyv derives on nms-core types

The core types in `nms-core` need rkyv derives. This is the most invasive change -- it touches types across the `nms-core` crate.

Add `rkyv` as an **optional** dependency in `nms-core`:

```toml
# crates/nms-core/Cargo.toml
[dependencies]
# ... existing ...
rkyv = { workspace = true, optional = true }

[features]
default = []
archive = ["rkyv"]
```

Then add conditional derives to core types:

```rust
// Example for GalacticAddress:
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct GalacticAddress {
    pub voxel_x: i16,
    pub voxel_y: i16,
    pub voxel_z: i16,
    pub solar_system_index: u16,
    pub planet_index: u8,
    pub reality_index: u8,
}

// Similarly for: Biome, Planet, System, PlayerBase, PlayerState, etc.
```

In `nms-cache/Cargo.toml`, enable the feature:

```toml
[dependencies]
nms-core = { workspace = true, features = ["archive"] }
```

---

## Strategy: Serialize Data, Rebuild Indices

Rather than trying to serialize petgraph's `StableGraph` and rstar's `RTree` directly (which don't support rkyv), we serialize a flattened data snapshot and rebuild the indices on load.

```
Save → GalaxyModel (full, with graph + R-tree + HashMaps)
         ↓ serialize
    CacheData (flat: Vec<System>, Vec<Planet>, Vec<Base>, PlayerState)
         ↓ write
    ~/.nms-copilot/galaxy.rkyv
         ↓ read
    CacheData (deserialized)
         ↓ rebuild
    GalaxyModel (full, indices reconstructed)
```

Rebuilding indices for ~300 systems takes <10ms, so this is acceptable.

---

## New File: `crates/nms-cache/src/data.rs`

The serializable cache data structure.

```rust
//! Cache data types -- the subset of GalaxyModel that gets serialized.

use rkyv::{Archive, Serialize, Deserialize};

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{PlayerBase, PlayerState};
use nms_core::system::{Planet, System};

/// Flattened galaxy data for cache serialization.
///
/// Contains all the raw data needed to reconstruct a `GalaxyModel`.
/// Indices (graph, R-tree, HashMaps) are rebuilt on load.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CacheData {
    /// All discovered systems.
    pub systems: Vec<CachedSystem>,

    /// All discovered planets, with their parent system address.
    pub planets: Vec<CachedPlanet>,

    /// All player bases.
    pub bases: Vec<PlayerBase>,

    /// Player state at time of caching.
    pub player_state: Option<PlayerState>,

    /// Save file version that produced this cache.
    pub save_version: u32,

    /// Timestamp when the cache was created (Unix seconds).
    pub cached_at: u64,
}

/// A system with its address for cache storage.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CachedSystem {
    pub address: GalacticAddress,
    pub system: System,
}

/// A planet with its parent system address for cache storage.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CachedPlanet {
    pub system_address: GalacticAddress,
    pub planet_index: u8,
    pub planet: Planet,
}
```

---

## New File: `crates/nms-cache/src/serialize.rs`

Serialization and deserialization functions.

```rust
//! Serialize and deserialize galaxy data to/from rkyv archives.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rkyv::rancor::Error as RkyvError;

use nms_graph::GalaxyModel;

use crate::data::{CacheData, CachedPlanet, CachedSystem};
use crate::error::CacheError;

/// Extract cache data from a GalaxyModel.
pub fn extract_cache_data(model: &GalaxyModel, save_version: u32) -> CacheData {
    let systems: Vec<CachedSystem> = model
        .systems
        .iter()
        .map(|(id, system)| CachedSystem {
            address: id.to_galactic_address(),
            system: system.clone(),
        })
        .collect();

    let planets: Vec<CachedPlanet> = model
        .planets
        .iter()
        .map(|((sys_id, planet_index), planet)| CachedPlanet {
            system_address: sys_id.to_galactic_address(),
            planet_index: *planet_index,
            planet: planet.clone(),
        })
        .collect();

    let bases: Vec<_> = model.bases.values().cloned().collect();

    let cached_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    CacheData {
        systems,
        planets,
        bases,
        player_state: model.player_state.clone(),
        save_version,
        cached_at,
    }
}

/// Serialize cache data to bytes.
pub fn serialize(data: &CacheData) -> Result<Vec<u8>, CacheError> {
    rkyv::to_bytes::<RkyvError>(data)
        .map(|v| v.to_vec())
        .map_err(|e| CacheError::Serialize(e.to_string()))
}

/// Write cache data to a file.
pub fn write_cache(data: &CacheData, path: &Path) -> Result<(), CacheError> {
    let bytes = serialize(data)?;

    // Write to a temp file first, then rename for atomicity
    let tmp_path = path.with_extension("rkyv.tmp");
    fs::write(&tmp_path, &bytes).map_err(CacheError::Io)?;
    fs::rename(&tmp_path, path).map_err(CacheError::Io)?;

    Ok(())
}

/// Read and deserialize cache data from a file.
pub fn read_cache(path: &Path) -> Result<CacheData, CacheError> {
    let bytes = fs::read(path).map_err(CacheError::Io)?;
    deserialize(&bytes)
}

/// Deserialize cache data from bytes.
pub fn deserialize(bytes: &[u8]) -> Result<CacheData, CacheError> {
    rkyv::from_bytes::<CacheData, RkyvError>(bytes)
        .map_err(|e| CacheError::Deserialize(e.to_string()))
}

/// Rebuild a GalaxyModel from cache data.
///
/// Reconstructs the graph, R-tree, and HashMap indices.
pub fn rebuild_model(data: &CacheData) -> GalaxyModel {
    // Use GalaxyModel's builder methods to reconstruct from raw data.
    // This requires a method on GalaxyModel that accepts pre-extracted data.
    // For now, we'll use a from_cache_data constructor.
    GalaxyModel::from_cache_data(data)
}
```

**Note:** This requires adding a `from_cache_data` constructor to `GalaxyModel` in `nms-graph`. See the implementation notes below.

---

## New File: `crates/nms-cache/src/error.rs`

```rust
//! Cache error types.

use std::io;

#[derive(Debug)]
pub enum CacheError {
    /// rkyv serialization failed.
    Serialize(String),
    /// rkyv deserialization failed.
    Deserialize(String),
    /// File I/O error.
    Io(io::Error),
    /// Cache is stale (save file is newer).
    Stale,
    /// Cache file not found.
    NotFound,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "cache serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "cache deserialization error: {e}"),
            Self::Io(e) => write!(f, "cache I/O error: {e}"),
            Self::Stale => write!(f, "cache is stale"),
            Self::NotFound => write!(f, "cache file not found"),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
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
pub mod serialize;

pub use data::CacheData;
pub use error::CacheError;
pub use serialize::{extract_cache_data, read_cache, rebuild_model, write_cache};
```

---

## Required Addition to `nms-graph`: `GalaxyModel::from_cache_data`

Add a constructor to `GalaxyModel` that rebuilds from cache data:

```rust
// In crates/nms-graph/src/model.rs:

impl GalaxyModel {
    /// Reconstruct a GalaxyModel from cached data.
    ///
    /// Rebuilds the graph, R-tree, and all HashMap indices.
    pub fn from_cache_data(data: &nms_cache::data::CacheData) -> Self {
        let mut model = Self::new();

        // Insert systems
        for cached_sys in &data.systems {
            let sys_id = SystemId::from_galactic_address(&cached_sys.address);
            model.insert_system(sys_id, cached_sys.system.clone());
        }

        // Insert planets
        for cached_planet in &data.planets {
            let sys_id = SystemId::from_galactic_address(&cached_planet.system_address);
            model.insert_planet(sys_id, cached_planet.planet_index, cached_planet.planet.clone());
        }

        // Insert bases
        for base in &data.bases {
            model.insert_base(base.clone());
        }

        // Set player state
        model.player_state = data.player_state.clone();

        // Build edges
        model.build_edges(crate::edges::EdgeStrategy::default());

        model
    }
}
```

**Note:** This depends on `GalaxyModel` having `insert_system`, `insert_planet`, and `insert_base` methods. These should be factored out of the existing `from_save()` constructor during this milestone. The `new()` method creates an empty model.

---

## Avoiding Circular Dependencies

The design above has `nms-cache` depending on `nms-graph` (for `GalaxyModel`), and `nms-graph` depending on `nms-cache` (for `CacheData`). This is a **circular dependency** and won't compile.

**Solution:** `GalaxyModel::from_cache_data` should NOT live in `nms-graph`. Instead:

1. `nms-cache` depends on `nms-graph` (reads `GalaxyModel` fields for extraction)
2. The `rebuild_model` function lives in `nms-cache` and uses `GalaxyModel`'s public API
3. `GalaxyModel` needs `new()` and `insert_*` methods (added as part of this milestone), but these don't depend on `nms-cache`

```rust
// In crates/nms-cache/src/serialize.rs:
pub fn rebuild_model(data: &CacheData) -> GalaxyModel {
    let mut model = GalaxyModel::new();

    for cached_sys in &data.systems {
        let sys_id = SystemId::from_galactic_address(&cached_sys.address);
        model.insert_system(sys_id, cached_sys.system.clone());
    }

    for cached_planet in &data.planets {
        let sys_id = SystemId::from_galactic_address(&cached_planet.system_address);
        model.insert_planet(sys_id, cached_planet.planet_index, cached_planet.planet.clone());
    }

    for base in &data.bases {
        model.insert_base(base.clone());
    }

    model.player_state = data.player_state.clone();
    model.build_edges(nms_graph::EdgeStrategy::default());

    model
}
```

This requires adding to `GalaxyModel`:
- `pub fn new() -> Self` -- empty model
- `pub fn insert_system(&mut self, id: SystemId, system: System)`
- `pub fn insert_planet(&mut self, sys_id: SystemId, index: u8, planet: Planet)`
- `pub fn insert_base(&mut self, base: PlayerBase)`

These are useful methods to have regardless, and they should update the graph, R-tree, and all HashMaps. Factoring them out of `from_save()` improves that code too.

---

## Tests

### File: `crates/nms-cache/src/serialize.rs` (inline tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                    "Units": 5000000, "Nanites": 10000, "Specials": 500,
                    "PersistentPlayerBases": [
                        {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                    ]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x250003AB8C07", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        GalaxyModel::from_save(&save)
    }

    #[test]
    fn test_extract_cache_data() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        assert_eq!(data.systems.len(), model.systems.len());
        assert_eq!(data.planets.len(), model.planets.len());
        assert_eq!(data.bases.len(), model.bases.len());
        assert_eq!(data.save_version, 4720);
        assert!(data.cached_at > 0);
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let bytes = serialize(&data).unwrap();
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(restored.systems.len(), data.systems.len());
        assert_eq!(restored.planets.len(), data.planets.len());
        assert_eq!(restored.bases.len(), data.bases.len());
    }

    #[test]
    fn test_rebuild_model_preserves_counts() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let bytes = serialize(&data).unwrap();
        let restored_data = deserialize(&bytes).unwrap();
        let rebuilt = rebuild_model(&restored_data);

        assert_eq!(rebuilt.systems.len(), model.systems.len());
        assert_eq!(rebuilt.planets.len(), model.planets.len());
        assert_eq!(rebuilt.bases.len(), model.bases.len());
    }

    #[test]
    fn test_write_and_read_cache_file() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rkyv");

        write_cache(&data, &path).unwrap();
        assert!(path.exists());

        let restored = read_cache(&path).unwrap();
        assert_eq!(restored.systems.len(), data.systems.len());
    }

    #[test]
    fn test_read_nonexistent_cache_errors() {
        let result = read_cache(Path::new("/tmp/nonexistent_nms_cache.rkyv"));
        assert!(result.is_err());
    }
}
```

---

## Implementation Notes

1. **rkyv version**: Using rkyv 0.8 which has a different API from 0.7. The `rkyv::to_bytes` and `rkyv::from_bytes` functions are the primary interface. The `validation` feature enables safe deserialization with bounds checking.

2. **Feature-gated derives on nms-core**: The `archive` feature is opt-in so crates that don't need caching don't pay the compile-time cost of rkyv derives. Only `nms-cache` enables it.

3. **Atomic writes**: `write_cache` writes to a `.tmp` file then renames, ensuring the cache file is never in a half-written state if the process is interrupted.

4. **Rebuild vs zero-copy**: True zero-copy (mmap + access archived data directly) would be faster but requires all query code to work with rkyv's `Archived<T>` types. The simpler approach -- deserialize to owned types and rebuild indices -- is fast enough (<100ms for ~300 systems) and doesn't infect the rest of the codebase with rkyv types.

5. **`SystemId::from_galactic_address` and `to_galactic_address`**: These methods may need to be added to `SystemId` if they don't exist. `from_galactic_address` zeros the planet bits and packs; `to_galactic_address` unpacks.

6. **`GalaxyModel` mutation methods**: Adding `new()`, `insert_system()`, `insert_planet()`, `insert_base()` to `GalaxyModel` is the main cross-crate change. These should be factored out of the existing `from_save()` to avoid code duplication.
