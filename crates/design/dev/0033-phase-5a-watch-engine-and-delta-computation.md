# Phase 5A -- Watch Engine & Delta Computation

Milestones 5.1-5.4: File watcher, `SaveDelta` types, diffing algorithm, incremental model update, and event channel in nms-watch.

**Depends on:** Phases 1-4 (save parsing, galaxy model with insert methods).

---

## Architecture Overview

The watcher runs on a background thread, monitoring the NMS save directory. When a save file changes, it:

1. Debounces (500ms) to let the write complete
2. Re-parses the save file
3. Diffs against a snapshot of the previous parse
4. Produces a `SaveDelta` describing what changed
5. Sends the delta over an `mpsc` channel to consumers

No async runtime needed -- the watcher thread uses `notify`'s blocking API, and consumers drain the channel synchronously (the REPL checks between prompts).

```
                   +-----------+
  save.hg write -> |  notify   | (background thread)
                   +-----------+
                        |
                   debounce 500ms
                        |
                   re-parse save
                        |
                   diff vs snapshot
                        |
                   SaveDelta
                        |
                   mpsc::Sender
                        |
          +-------------+-------------+
          |             |             |
     REPL loop     MCP server    cache writer
  (try_recv)      (try_recv)    (try_recv)
```

---

## Milestone 5.1: File Watcher

### New Dependencies

Add to workspace `Cargo.toml`:

```toml
notify = "7"
notify-debouncer-mini = "0.5"
```

Add to `crates/nms-watch/Cargo.toml`:

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-graph = { workspace = true }
notify = { workspace = true }
notify-debouncer-mini = { workspace = true }
```

### New File: `crates/nms-watch/src/watcher.rs`

```rust
//! File system watcher for NMS save files.
//!
//! Uses `notify` with debouncing to detect save file modifications.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use crate::delta::{SaveDelta, compute_delta};
use crate::error::WatchError;
use crate::snapshot::SaveSnapshot;

/// Handle to a running file watcher.
///
/// Dropping this stops the watcher thread.
pub struct WatchHandle {
    /// Receive deltas from the watcher thread.
    pub receiver: mpsc::Receiver<SaveDelta>,
    /// Keep the debouncer alive (dropping it stops watching).
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

/// Configuration for the file watcher.
pub struct WatchConfig {
    /// Path to the save file to watch.
    pub save_path: PathBuf,
    /// Debounce duration (default: 500ms).
    pub debounce: Duration,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            save_path: PathBuf::new(),
            debounce: Duration::from_millis(500),
        }
    }
}

/// Start watching a save file for changes.
///
/// Returns a `WatchHandle` whose `receiver` yields `SaveDelta` values
/// whenever the save file is modified.
///
/// # Errors
///
/// Returns `WatchError::NotifyError` if the file watcher cannot be created,
/// or `WatchError::SaveNotFound` if the save file does not exist.
pub fn start_watching(config: WatchConfig) -> Result<WatchHandle, WatchError> {
    if !config.save_path.exists() {
        return Err(WatchError::SaveNotFound(config.save_path));
    }

    let (delta_tx, delta_rx) = mpsc::channel();
    let save_path = config.save_path.clone();

    // Take an initial snapshot for diffing
    let initial_snapshot = SaveSnapshot::from_file(&save_path)
        .map_err(|e| WatchError::ParseError(e.to_string()))?;

    // Channel for notify events (internal)
    let (notify_tx, notify_rx) = mpsc::channel();

    let mut debouncer = new_debouncer(config.debounce, notify_tx)
        .map_err(|e| WatchError::NotifyError(e.to_string()))?;

    // Watch the parent directory (more reliable than watching the file directly,
    // since games often write to a temp file and rename)
    let watch_dir = save_path
        .parent()
        .unwrap_or(&save_path);

    debouncer
        .watcher()
        .watch(watch_dir, notify::RecursiveMode::NonRecursive)
        .map_err(|e| WatchError::NotifyError(e.to_string()))?;

    // Background thread: receive notify events, re-parse, diff, send deltas
    let save_path_bg = save_path.clone();
    thread::spawn(move || {
        let mut snapshot = initial_snapshot;

        for events in notify_rx {
            let events = match events {
                Ok(evts) => evts,
                Err(_) => continue,
            };

            // Only react if our save file was modified
            let dominated_event = events.iter().any(|e| {
                e.kind == DebouncedEventKind::Any && e.path == save_path_bg
            });
            if !dominated_event {
                continue;
            }

            // Re-parse save file
            let new_snapshot = match SaveSnapshot::from_file(&save_path_bg) {
                Ok(s) => s,
                Err(_) => continue, // Partial write or permission error -- skip
            };

            // Compute delta
            let delta = compute_delta(&snapshot, &new_snapshot);

            if !delta.is_empty() {
                // Send delta; if receiver is dropped, stop
                if delta_tx.send(delta).is_err() {
                    break;
                }
            }

            snapshot = new_snapshot;
        }
    });

    Ok(WatchHandle {
        receiver: delta_rx,
        _debouncer: debouncer,
    })
}
```

### Tests (Milestone 5.1)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_watch_config_default_debounce() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce, Duration::from_millis(500));
    }

    #[test]
    fn test_start_watching_nonexistent_file_errors() {
        let config = WatchConfig {
            save_path: PathBuf::from("/tmp/nonexistent_nms_save.json"),
            ..Default::default()
        };
        assert!(matches!(
            start_watching(config),
            Err(WatchError::SaveNotFound(_))
        ));
    }

    #[test]
    fn test_start_watching_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        fs::write(&save_path, minimal_save_json()).unwrap();

        let config = WatchConfig {
            save_path,
            debounce: Duration::from_millis(100),
        };
        let handle = start_watching(config);
        assert!(handle.is_ok());
        // Dropping handle stops the watcher
    }

    #[test]
    fn test_watcher_detects_change() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        fs::write(&save_path, minimal_save_json()).unwrap();

        let config = WatchConfig {
            save_path: save_path.clone(),
            debounce: Duration::from_millis(100),
        };
        let handle = start_watching(config).unwrap();

        // Modify the file (add a new discovery)
        fs::write(&save_path, modified_save_json()).unwrap();

        // Wait for debounce + processing
        let delta = handle.receiver.recv_timeout(Duration::from_secs(2));
        assert!(delta.is_ok());
        assert!(!delta.unwrap().is_empty());
    }

    fn minimal_save_json() -> &'static str {
        // Same pattern as other test modules -- minimal valid save JSON
        // with 1 system, 1 planet
        /* ... */
    }

    fn modified_save_json() -> &'static str {
        // Same save JSON but with an additional system/planet discovery
        /* ... */
    }
}
```

---

## Milestone 5.2: Delta Types & Computation

### New File: `crates/nms-watch/src/delta.rs`

```rust
//! Delta computation between save snapshots.
//!
//! Compares two `SaveSnapshot` values and produces a `SaveDelta` listing
//! all changes: new discoveries, player movement, new/modified bases.

use nms_core::address::GalacticAddress;
use nms_core::player::PlayerBase;
use nms_core::system::{Planet, System};
use nms_graph::SystemId;

/// A typed description of what changed between two save file snapshots.
#[derive(Debug, Clone)]
pub struct SaveDelta {
    /// Newly discovered systems (not in previous snapshot).
    pub new_systems: Vec<System>,
    /// Newly discovered planets (not in previous snapshot).
    pub new_planets: Vec<(SystemId, Planet)>,
    /// Player moved to a new position.
    pub player_moved: Option<PlayerMoved>,
    /// Newly placed bases.
    pub new_bases: Vec<PlayerBase>,
    /// Modified bases (name change, additions).
    pub modified_bases: Vec<PlayerBase>,
}

/// Player position change.
#[derive(Debug, Clone)]
pub struct PlayerMoved {
    pub from: GalacticAddress,
    pub to: GalacticAddress,
}

impl SaveDelta {
    /// Returns true if no changes were detected.
    pub fn is_empty(&self) -> bool {
        self.new_systems.is_empty()
            && self.new_planets.is_empty()
            && self.player_moved.is_none()
            && self.new_bases.is_empty()
            && self.modified_bases.is_empty()
    }

    /// Total number of individual changes.
    pub fn change_count(&self) -> usize {
        self.new_systems.len()
            + self.new_planets.len()
            + self.player_moved.as_ref().map_or(0, |_| 1)
            + self.new_bases.len()
            + self.modified_bases.len()
    }
}

/// Compute the delta between two snapshots.
///
/// Compares discovery records by universe address + discovery type,
/// player position, and base lists.
pub fn compute_delta(old: &SaveSnapshot, new: &SaveSnapshot) -> SaveDelta {
    // 1. Check player position change (cheapest check first)
    let player_moved = if old.player_address != new.player_address {
        Some(PlayerMoved {
            from: old.player_address,
            to: new.player_address,
        })
    } else {
        None
    };

    // 2. Find new systems (in new but not in old, keyed by SystemId)
    let new_systems: Vec<System> = new
        .systems
        .iter()
        .filter(|(id, _)| !old.systems.contains_key(id))
        .map(|(_, sys)| sys.clone())
        .collect();

    // 3. Find new planets (in new but not in old, keyed by (SystemId, planet_index))
    let new_planets: Vec<(SystemId, Planet)> = new
        .planets
        .iter()
        .filter(|(key, _)| !old.planets.contains_key(key))
        .map(|((sys_id, _), planet)| (*sys_id, planet.clone()))
        .collect();

    // 4. Find new bases (by name, case-insensitive)
    let new_bases: Vec<PlayerBase> = new
        .bases
        .iter()
        .filter(|(name, _)| !old.bases.contains_key(name.as_str()))
        .map(|(_, base)| base.clone())
        .collect();

    // 5. Find modified bases (same name, different content)
    let modified_bases: Vec<PlayerBase> = new
        .bases
        .iter()
        .filter(|(name, new_base)| {
            old.bases
                .get(name.as_str())
                .is_some_and(|old_base| old_base != new_base)
        })
        .map(|(_, base)| base.clone())
        .collect();

    SaveDelta {
        new_systems,
        new_planets,
        player_moved,
        new_bases,
        modified_bases,
    }
}
```

### New File: `crates/nms-watch/src/snapshot.rs`

A lightweight snapshot of the save file state used for diffing.

```rust
//! Save file snapshot for delta comparison.
//!
//! A `SaveSnapshot` captures the minimum state needed to detect changes
//! between two versions of a save file. Built by re-parsing the save
//! and extracting discovery records, player position, and bases.

use std::collections::HashMap;
use std::path::Path;

use nms_core::address::GalacticAddress;
use nms_core::player::PlayerBase;
use nms_core::system::{Planet, System};
use nms_graph::extract::extract_systems;
use nms_graph::SystemId;

/// A snapshot of save file state for diff comparison.
#[derive(Debug)]
pub struct SaveSnapshot {
    /// Systems keyed by SystemId.
    pub systems: HashMap<SystemId, System>,
    /// Planets keyed by (SystemId, planet_index).
    pub planets: HashMap<(SystemId, u8), Planet>,
    /// Bases keyed by lowercase name.
    pub bases: HashMap<String, PlayerBase>,
    /// Player's current galactic address.
    pub player_address: GalacticAddress,
}

impl SaveSnapshot {
    /// Build a snapshot by parsing a save file from disk.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let save = nms_save::parse_save_file(path)?;
        Ok(Self::from_save(&save))
    }

    /// Build a snapshot from an already-parsed save.
    pub fn from_save(save: &nms_save::model::SaveRoot) -> Self {
        let extracted = extract_systems(save);

        let mut planets = HashMap::new();
        for (sys_id, system) in &extracted {
            for planet in &system.planets {
                planets.insert((*sys_id, planet.index), planet.clone());
            }
        }

        let ps = save.active_player_state();
        let mut bases = HashMap::new();
        for base in &ps.persistent_player_bases {
            let core_base = base.to_core_base();
            if !core_base.name.is_empty() {
                bases.insert(core_base.name.to_lowercase(), core_base);
            }
        }

        let player_address = save.to_core_player_state().current_address;

        Self {
            systems: extracted,
            planets,
            bases,
            player_address,
        }
    }
}
```

### Tests (Milestone 5.2)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_empty_when_identical() {
        let snapshot = test_snapshot();
        let delta = compute_delta(&snapshot, &snapshot);
        assert!(delta.is_empty());
        assert_eq!(delta.change_count(), 0);
    }

    #[test]
    fn test_delta_detects_new_system() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let addr = GalacticAddress::new(500, 10, -300, 0x999, 0, 0);
        let sys_id = SystemId::from_address(&addr);
        new.systems.insert(sys_id, System::new(addr, Some("New".into()), None, None, vec![]));

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_systems.len(), 1);
        assert!(!delta.is_empty());
    }

    #[test]
    fn test_delta_detects_player_moved() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        new.player_address = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = compute_delta(&old, &new);
        assert!(delta.player_moved.is_some());
        let moved = delta.player_moved.unwrap();
        assert_eq!(moved.to.voxel_x(), 999);
    }

    #[test]
    fn test_delta_detects_new_base() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let base = PlayerBase { /* ... */ };
        new.bases.insert("new base".into(), base);

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_bases.len(), 1);
    }

    #[test]
    fn test_delta_detects_new_planet() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        let sys_id = *new.systems.keys().next().unwrap();
        new.planets.insert((sys_id, 5), Planet::new(5, Some(Biome::Lava), None, false, None, None));

        let delta = compute_delta(&old, &new);
        assert_eq!(delta.new_planets.len(), 1);
    }

    #[test]
    fn test_delta_no_false_positives_on_existing() {
        // Same data should produce no changes
        let snapshot = test_snapshot();
        let clone = /* deep clone of snapshot */;
        let delta = compute_delta(&snapshot, &clone);
        assert!(delta.is_empty());
    }

    #[test]
    fn test_change_count() {
        let old = test_snapshot();
        let mut new = test_snapshot();
        // Add 2 systems, move player, add 1 base
        new.player_address = GalacticAddress::new(1, 0, 0, 1, 0, 0);
        // ... add systems and bases ...
        let delta = compute_delta(&old, &new);
        assert_eq!(delta.change_count(), /* expected count */);
    }

    fn test_snapshot() -> SaveSnapshot {
        let json = /* minimal save JSON */;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        SaveSnapshot::from_save(&save)
    }
}
```

---

## Milestone 5.3: Incremental Model Update

### Modified File: `crates/nms-graph/src/model.rs`

Add `apply_delta()` method to `GalaxyModel`. The model already has `insert_system()` and `insert_base()` -- this method orchestrates applying a full delta.

```rust
use nms_watch::delta::SaveDelta;

impl GalaxyModel {
    /// Apply a delta from the file watcher to update the model incrementally.
    ///
    /// Inserts new systems and planets, updates player position, and
    /// adds/updates bases. Does NOT rebuild graph edges -- call
    /// `build_edges()` afterward if needed for routing.
    pub fn apply_delta(&mut self, delta: &SaveDelta) {
        // 1. Insert new systems (with their planets)
        for system in &delta.new_systems {
            self.insert_system(system.clone());
        }

        // 2. Insert new planets into existing systems
        for (sys_id, planet) in &delta.new_planets {
            let key = (*sys_id, planet.index);
            if !self.planets.contains_key(&key) {
                if let Some(biome) = planet.biome {
                    self.biome_index.entry(biome).or_default().push(key);
                }
                self.planets.insert(key, planet.clone());

                // Also add to the system's planet list
                if let Some(system) = self.systems.get_mut(sys_id) {
                    if !system.planets.iter().any(|p| p.index == planet.index) {
                        system.planets.push(planet.clone());
                    }
                }
            }
        }

        // 3. Update player position
        if let Some(ref moved) = delta.player_moved {
            if let Some(ref mut ps) = self.player_state {
                ps.current_address = moved.to;
            }
        }

        // 4. Insert new bases
        for base in &delta.new_bases {
            self.insert_base(base.clone());
        }

        // 5. Update modified bases
        for base in &delta.modified_bases {
            self.insert_base(base.clone()); // insert_base overwrites by name
        }
    }
}
```

### Circular Dependency Note

`nms-graph` cannot depend on `nms-watch` (that would create a cycle). Instead, `apply_delta` takes the delta types directly. Two approaches:

**Option A (recommended):** Define `SaveDelta` and related types in `nms-core` (or a new small `nms-delta` module within nms-core), so both `nms-graph` and `nms-watch` can use them without a cycle.

**Option B:** Define `apply_delta` in `nms-watch` instead, taking `&mut GalaxyModel` as a parameter. This avoids the cycle but puts model mutation logic outside the model crate.

**Recommendation:** Use Option A. Move `SaveDelta`, `PlayerMoved` to `nms-core::delta` (they only depend on core types like `GalacticAddress`, `System`, `Planet`, `PlayerBase`). The `SystemId` type would need to be in nms-core too, or `SaveDelta` can use `GalacticAddress` as the key and let consumers convert. Since `SystemId` is just a newtype over `u64` derived from `GalacticAddress`, the simplest approach is to move `SystemId` to nms-core or just use `u64` packed addresses as keys in `SaveDelta`.

### Tests (Milestone 5.3)

```rust
#[cfg(test)]
mod delta_tests {
    use super::*;

    #[test]
    fn test_apply_delta_new_system() {
        let mut model = test_model();
        let count_before = model.system_count();

        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("Delta System".into()),
                None, None,
                vec![Planet::new(0, Some(Biome::Lush), None, false, None, None)],
            )],
            ..SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.system_count(), count_before + 1);
        assert!(model.system_by_name("Delta System").is_some());
    }

    #[test]
    fn test_apply_delta_player_moved() {
        let mut model = test_model();
        let new_addr = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = SaveDelta {
            player_moved: Some(PlayerMoved {
                from: *model.player_position().unwrap(),
                to: new_addr,
            }),
            ..SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.player_position().unwrap().voxel_x(), 999);
    }

    #[test]
    fn test_apply_delta_new_base() {
        let mut model = test_model();
        let base_count_before = model.base_count();

        let delta = SaveDelta {
            new_bases: vec![/* PlayerBase */],
            ..SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.base_count(), base_count_before + 1);
    }

    #[test]
    fn test_apply_empty_delta_is_noop() {
        let mut model = test_model();
        let sys_count = model.system_count();
        let planet_count = model.planet_count();

        model.apply_delta(&SaveDelta::empty());

        assert_eq!(model.system_count(), sys_count);
        assert_eq!(model.planet_count(), planet_count);
    }
}
```

---

## Milestone 5.4: Event Channel

The channel is already implicit in milestone 5.1 (`mpsc::channel` in `start_watching`). This milestone formalizes the API and adds support for multiple consumers.

### Design: Single vs Multiple Consumers

The REPL is the primary consumer in Phase 5. The MCP server (Phase 6) will be a second consumer. Options:

1. **`mpsc` with clone** -- `mpsc::Sender` can be cloned, but `mpsc::Receiver` cannot. Only one consumer.
2. **Multiple `mpsc` channels** -- The watcher sends to multiple senders. Simple, explicit.
3. **`broadcast` channel** -- Multiple receivers via `std::sync::mpsc` doesn't support this natively. Would need `tokio::sync::broadcast` or a custom fanout.

**Recommendation:** Use approach #2. The watcher holds a `Vec<mpsc::Sender<SaveDelta>>`. Consumers register by calling `subscribe()` which returns a new `mpsc::Receiver<SaveDelta>`. This keeps the implementation simple with no async runtime.

### Modified File: `crates/nms-watch/src/watcher.rs`

```rust
/// Handle to a running file watcher.
pub struct WatchHandle {
    /// Initial receiver for the first consumer.
    pub receiver: mpsc::Receiver<SaveDelta>,
    /// Sender side for creating additional consumers.
    additional_senders: Vec<mpsc::Sender<SaveDelta>>,
    /// Keep the debouncer alive.
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

// Actually, since we can't easily add senders to a running thread,
// the simpler approach is: the watcher thread sends to a single channel,
// and the consumer (main thread) is responsible for forwarding if needed.
// For Phase 5, only one consumer (the REPL) exists.
// Phase 6 (MCP server) will refactor to multi-consumer if needed.
```

For Phase 5, the single `mpsc` channel from milestone 5.1 is sufficient. The REPL is the only consumer. We'll add multi-consumer support in Phase 6 when the MCP server needs it.

### New File: `crates/nms-watch/src/error.rs`

```rust
//! Error types for the file watcher.

use std::path::PathBuf;

/// Errors from the file watch system.
#[derive(Debug)]
pub enum WatchError {
    /// Save file not found at the expected path.
    SaveNotFound(PathBuf),
    /// File system notification error.
    NotifyError(String),
    /// Error parsing the save file during watch.
    ParseError(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaveNotFound(p) => write!(f, "save file not found: {}", p.display()),
            Self::NotifyError(e) => write!(f, "file watcher error: {e}"),
            Self::ParseError(e) => write!(f, "save parse error during watch: {e}"),
        }
    }
}

impl std::error::Error for WatchError {}
```

### Modified File: `crates/nms-watch/src/lib.rs`

```rust
//! File watcher and delta computation for NMS Copilot.
//!
//! Monitors the NMS save directory for changes, re-parses on auto-save,
//! computes typed deltas (new discoveries, player movement, new bases),
//! and distributes updates via channel to all consumers.

pub mod delta;
pub mod error;
pub mod snapshot;
pub mod watcher;

pub use delta::{SaveDelta, PlayerMoved, compute_delta};
pub use error::WatchError;
pub use snapshot::SaveSnapshot;
pub use watcher::{WatchConfig, WatchHandle, start_watching};
```

---

## Implementation Notes

1. **Watch the directory, not the file.** NMS (and many games) write to a temp file and rename. Watching the directory catches renames that `notify` might miss when watching the file directly.

2. **Debounce is critical.** NMS writes multiple blocks during a save. Without debouncing, we'd get partial reads. 500ms is conservative; 200ms might work but isn't worth the risk.

3. **Delta computation is a full re-parse.** The save file is ~22MB compressed, ~50MB JSON. Parsing takes <1s on modern hardware. This is simpler and more reliable than trying to do incremental parsing.

4. **Player position is the most common delta.** During normal gameplay (warping, flying), the player position changes on every auto-save but discoveries don't. The diff should check position first (O(1)) and skip the more expensive discovery diff if nothing else changed.

5. **`SaveSnapshot` is separate from `GalaxyModel`.** The snapshot stores raw extracted data for diffing. The model has spatial indexes. We don't want to build a full model just to diff -- snapshots are cheaper.

6. **`insert_system` already handles duplicates.** The existing method returns early if the system already exists, so applying a delta with already-known systems is safe.

7. **Thread safety.** The `GalaxyModel` lives on the main thread. The watcher thread only sends `SaveDelta` values over the channel. The main thread calls `apply_delta()` synchronously when it drains the channel. No `Arc<Mutex<>>` needed.

8. **`extract_systems` must be pub.** Currently `pub` in nms-graph -- confirm this is accessible from nms-watch. If not, the snapshot can call `nms_save::parse_save_file` and then re-extract. Since nms-watch depends on nms-graph, this should work.

---

## Dev Dependencies

Add to `crates/nms-watch/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

Add `tempfile` to workspace dependencies if not already present.
