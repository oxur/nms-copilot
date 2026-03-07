# Phase 5B -- Consumer Integration & Robustness

Milestones 5.5-5.7: REPL live notifications, cache invalidation on delta, and graceful error handling.

**Depends on:** Phase 5A (watcher, delta types, `apply_delta()`).

---

## Milestone 5.5: REPL Integration

### Strategy: Drain Between Prompts

The REPL uses reedline's blocking `read_line()`. Rather than introducing async, we drain pending deltas at natural break points:

1. **Before each prompt** -- check for watch events, apply deltas, print notifications
2. **After each command** -- same check

This means notifications appear when the user presses Enter or between commands. There's no interrupt-style notification mid-typing, which is acceptable and avoids terminal corruption.

### Modified File: `crates/nms-copilot/src/main.rs`

```rust
use std::sync::mpsc;
use nms_watch::{SaveDelta, WatchConfig, WatchHandle, start_watching};

fn main() {
    // ... existing config and model loading ...

    // Start file watcher (optional -- don't fail startup if watcher can't start)
    let watch_handle = if config.watch_enabled() {
        match start_watcher(&config) {
            Ok(handle) => {
                println!("Watching save file for live updates.");
                Some(handle)
            }
            Err(e) => {
                eprintln!("Warning: could not start file watcher: {e}");
                None
            }
        }
    } else {
        None
    };

    // Model must be mutable now for apply_delta
    let mut model = model;

    loop {
        // Drain any pending watch events
        if let Some(ref handle) = watch_handle {
            drain_watch_events(&handle.receiver, &mut model, &mut session);
        }

        prompt.update(PromptState::from_session(&session));
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                // ... existing command dispatch ...

                // Also drain after command execution
                if let Some(ref handle) = watch_handle {
                    drain_watch_events(&handle.receiver, &mut model, &mut session);
                }
            }
            // ... existing signal handling ...
        }
    }
}

/// Drain all pending deltas from the watcher and apply them.
fn drain_watch_events(
    receiver: &mpsc::Receiver<SaveDelta>,
    model: &mut GalaxyModel,
    session: &mut SessionState,
) {
    while let Ok(delta) = receiver.try_recv() {
        let notifications = apply_and_notify(model, session, &delta);
        for note in &notifications {
            println!("{note}");
        }
    }
}

/// Apply a delta to the model and generate human-readable notifications.
fn apply_and_notify(
    model: &mut GalaxyModel,
    session: &mut SessionState,
    delta: &SaveDelta,
) -> Vec<String> {
    let mut notes = Vec::new();

    // Player moved
    if let Some(ref moved) = delta.player_moved {
        let from_sys = model
            .nearest_systems(&moved.from, 1)
            .first()
            .and_then(|(id, _)| model.system(id))
            .and_then(|s| s.name.as_deref())
            .unwrap_or("unknown");
        let to_sys = model
            .nearest_systems(&moved.to, 1)
            .first()
            .and_then(|(id, _)| model.system(id))
            .and_then(|s| s.name.as_deref())
            .unwrap_or("unknown");
        notes.push(format!("  Warped: {from_sys} -> {to_sys}"));
    }

    // New systems
    for system in &delta.new_systems {
        let name = system.name.as_deref().unwrap_or("(unnamed)");
        let planets = system.planets.len();
        notes.push(format!(
            "  New system: {name} ({planets} planet{})",
            if planets == 1 { "" } else { "s" }
        ));
    }

    // New planets
    for (sys_id, planet) in &delta.new_planets {
        let sys_name = model
            .system(sys_id)
            .and_then(|s| s.name.as_deref())
            .unwrap_or("(unnamed)");
        let biome = planet
            .biome
            .map(|b| b.to_string())
            .unwrap_or_else(|| "?".into());
        let planet_name = planet.name.as_deref().unwrap_or("(unnamed)");
        notes.push(format!("  New scan: \"{planet_name}\" ({biome}) in {sys_name}"));
    }

    // New bases
    for base in &delta.new_bases {
        notes.push(format!("  New base: {}", base.name));
    }

    // Apply delta to model
    model.apply_delta(delta);

    // Update session counts
    session.system_count = model.system_count();
    session.planet_count = model.planet_count();

    // Update position if player moved
    if let Some(ref moved) = delta.player_moved {
        session.position = Some(
            crate::session::PositionContext::PlayerPosition(moved.to),
        );
    }

    notes
}

fn start_watcher(config: &Config) -> Result<WatchHandle, Box<dyn std::error::Error>> {
    let save_path = match config.save_path() {
        Some(p) => PathBuf::from(p),
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };

    let watch_config = WatchConfig {
        save_path,
        ..Default::default()
    };

    Ok(start_watching(watch_config)?)
}
```

### Modified File: `crates/nms-copilot/src/config.rs`

Add watch configuration:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    /// Enable file watching (default: true).
    pub enabled: bool,
    /// Debounce duration in milliseconds (default: 500).
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 500,
        }
    }
}

// Add to Config struct:
pub struct Config {
    // ... existing fields ...
    #[serde(default)]
    pub watch: WatchConfig,
}

impl Config {
    pub fn watch_enabled(&self) -> bool {
        self.watch.enabled
    }

    pub fn watch_debounce(&self) -> Duration {
        Duration::from_millis(self.watch.debounce_ms)
    }
}
```

### Modified File: `crates/nms-copilot/Cargo.toml`

```toml
[dependencies]
nms-watch = { workspace = true }
```

### Tests (Milestone 5.5)

The `apply_and_notify` function is the testable core:

```rust
#[cfg(test)]
mod watch_tests {
    use super::*;

    #[test]
    fn test_apply_and_notify_empty_delta() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let delta = SaveDelta::empty();

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_apply_and_notify_new_system() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("New System".into()),
                None, None, vec![],
            )],
            ..SaveDelta::empty()
        };

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.iter().any(|n| n.contains("New system")));
        assert!(notes.iter().any(|n| n.contains("New System")));
    }

    #[test]
    fn test_apply_and_notify_player_moved() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let from = *model.player_position().unwrap();
        let to = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = SaveDelta {
            player_moved: Some(PlayerMoved { from, to }),
            ..SaveDelta::empty()
        };

        let notes = apply_and_notify(&mut model, &mut session, &delta);
        assert!(notes.iter().any(|n| n.contains("Warped")));
        assert_eq!(model.player_position().unwrap().voxel_x(), 999);
    }

    #[test]
    fn test_apply_and_notify_updates_session_counts() {
        let mut model = test_model();
        let mut session = SessionState::from_model(&model);
        let sys_before = session.system_count;

        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                None, None, None, vec![],
            )],
            ..SaveDelta::empty()
        };

        apply_and_notify(&mut model, &mut session, &delta);
        assert_eq!(session.system_count, sys_before + 1);
    }
}
```

---

## Milestone 5.6: Cache Invalidation

When the watcher applies a delta, the on-disk cache becomes stale. The cache should be updated to reflect the new model state.

### Strategy: Write-Through on Delta

After applying a delta, serialize the updated model to the cache file. This is a simple write-through: every delta application triggers a cache write. Since deltas are relatively infrequent (NMS auto-saves every few minutes), this is acceptable.

### Modified File: `crates/nms-copilot/src/main.rs`

Extend `drain_watch_events` to write cache after applying deltas:

```rust
fn drain_watch_events(
    receiver: &mpsc::Receiver<SaveDelta>,
    model: &mut GalaxyModel,
    session: &mut SessionState,
    cache_path: &Path,
) {
    let mut any_delta = false;

    while let Ok(delta) = receiver.try_recv() {
        let notifications = apply_and_notify(model, session, &delta);
        for note in &notifications {
            println!("{note}");
        }
        any_delta = true;
    }

    // Write updated cache if any deltas were applied
    if any_delta {
        if let Err(e) = nms_cache::write_cache(
            cache_path,
            &nms_cache::extract_cache_data(model),
        ) {
            eprintln!("Warning: could not update cache: {e}");
        }
    }
}
```

### Tests (Milestone 5.6)

```rust
#[cfg(test)]
mod cache_invalidation_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_updated_after_delta() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("test.cache");
        let mut model = test_model();

        // Write initial cache
        let data = nms_cache::extract_cache_data(&model);
        nms_cache::write_cache(&cache_path, &data).unwrap();
        let mtime_before = std::fs::metadata(&cache_path).unwrap().modified().unwrap();

        // Apply delta
        let delta = SaveDelta {
            new_systems: vec![/* new system */],
            ..SaveDelta::empty()
        };
        model.apply_delta(&delta);

        // Re-write cache
        let data = nms_cache::extract_cache_data(&model);
        nms_cache::write_cache(&cache_path, &data).unwrap();
        let mtime_after = std::fs::metadata(&cache_path).unwrap().modified().unwrap();

        assert!(mtime_after >= mtime_before);
    }
}
```

---

## Milestone 5.7: Graceful Error Handling

The watcher must be robust against real-world conditions:

1. **Partial writes** -- The game writes blocks sequentially. Reading mid-write produces corrupt JSON.
2. **Permission errors** -- File locked by another process.
3. **Save directory disappears** -- External drive unplugged, Steam cloud sync.
4. **Save file format changes** -- Game update modifies save structure.

### Strategy: Retry with Backoff

The watcher thread already skips parse failures (the `Err(_) => continue` in the event loop). This milestone adds:

1. **Consecutive failure counting** -- After N consecutive failures, log a warning.
2. **Parse validation** -- After debounce, verify the file size is stable before parsing (two reads 100ms apart should match).
3. **Watcher reconnection** -- If notify reports an error, attempt to re-watch.

### Modified File: `crates/nms-watch/src/watcher.rs`

```rust
/// Configuration constants for robustness.
const MAX_CONSECUTIVE_FAILURES: usize = 5;
const FILE_STABILITY_CHECK_MS: u64 = 100;

// In the background thread:
thread::spawn(move || {
    let mut snapshot = initial_snapshot;
    let mut consecutive_failures: usize = 0;

    for events in notify_rx {
        let events = match events {
            Ok(evts) => evts,
            Err(errs) => {
                // Log notify errors but keep running
                for e in errs {
                    eprintln!("Watch error: {e}");
                }
                continue;
            }
        };

        let dominated_event = events.iter().any(|e| {
            e.kind == DebouncedEventKind::Any && e.path == save_path_bg
        });
        if !dominated_event {
            continue;
        }

        // File stability check: ensure file size is stable
        if !is_file_stable(&save_path_bg) {
            continue;
        }

        match SaveSnapshot::from_file(&save_path_bg) {
            Ok(new_snapshot) => {
                consecutive_failures = 0;
                let delta = compute_delta(&snapshot, &new_snapshot);

                if !delta.is_empty() {
                    if delta_tx.send(delta).is_err() {
                        break;
                    }
                }

                snapshot = new_snapshot;
            }
            Err(e) => {
                consecutive_failures += 1;
                if consecutive_failures <= MAX_CONSECUTIVE_FAILURES {
                    // Silently ignore occasional parse failures (partial writes)
                } else if consecutive_failures == MAX_CONSECUTIVE_FAILURES + 1 {
                    eprintln!(
                        "Warning: {MAX_CONSECUTIVE_FAILURES} consecutive parse failures. \
                         Save file may be corrupt or format changed: {e}"
                    );
                }
                // Keep watching -- the next save might succeed
            }
        }
    }
});

/// Check that a file's size is stable (not being actively written).
fn is_file_stable(path: &Path) -> bool {
    let size1 = std::fs::metadata(path).map(|m| m.len()).ok();
    std::thread::sleep(Duration::from_millis(FILE_STABILITY_CHECK_MS));
    let size2 = std::fs::metadata(path).map(|m| m.len()).ok();
    size1 == size2 && size1.is_some()
}
```

### Tests (Milestone 5.7)

```rust
#[cfg(test)]
mod robustness_tests {
    use super::*;

    #[test]
    fn test_is_file_stable_for_static_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stable.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(is_file_stable(&path));
    }

    #[test]
    fn test_is_file_stable_nonexistent_returns_false() {
        assert!(!is_file_stable(Path::new("/tmp/nonexistent_xyz")));
    }

    #[test]
    fn test_watcher_survives_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        std::fs::write(&save_path, minimal_save_json()).unwrap();

        let config = WatchConfig {
            save_path: save_path.clone(),
            debounce: Duration::from_millis(100),
        };
        let handle = start_watching(config).unwrap();

        // Write corrupt data
        std::fs::write(&save_path, "not valid json {{{").unwrap();
        std::thread::sleep(Duration::from_millis(300));

        // Write valid data again -- watcher should recover
        std::fs::write(&save_path, modified_save_json()).unwrap();

        let delta = handle.receiver.recv_timeout(Duration::from_secs(2));
        assert!(delta.is_ok());
    }

    #[test]
    fn test_watcher_continues_after_file_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        std::fs::write(&save_path, minimal_save_json()).unwrap();

        let config = WatchConfig {
            save_path: save_path.clone(),
            debounce: Duration::from_millis(100),
        };
        let handle = start_watching(config).unwrap();

        // Delete and recreate
        std::fs::remove_file(&save_path).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        std::fs::write(&save_path, modified_save_json()).unwrap();

        // Should eventually produce a delta (file recreated)
        let delta = handle.receiver.recv_timeout(Duration::from_secs(3));
        assert!(delta.is_ok());
    }
}
```

---

## Modified File: `crates/nms-copilot/src/dispatch.rs` (help text)

Update the help text to mention live watching:

```rust
fn help_text() -> String {
    "\
NMS Copilot -- Interactive Galaxy Explorer

Commands:
  find       Search planets by biome, distance, name
  route      Plan a route through target systems
  show       Show system or base details
  stats      Display aggregate galaxy statistics
  convert    Convert between coordinate formats
  set        Set session context (position, biome, warp-range)
  reset      Reset session state (position, biome, warp-range, all)
  status     Show current session state
  info       Show loaded model summary
  help       Show this help message
  exit/quit  Exit the REPL

Live updates are shown between commands when file watching is enabled.
"
    .into()
}
```

---

## Implementation Notes

1. **No async runtime.** The entire system works with `std::thread` and `std::sync::mpsc`. The REPL stays synchronous. The watcher thread is the only background thread.

2. **Notifications print before the prompt.** When the user presses Enter, pending events are drained and printed before the next prompt appears. This avoids interleaving with command output.

3. **Model becomes `mut`.** Currently `model` is immutable in the REPL loop. Adding watch support requires `mut model` so `apply_delta()` can modify it. This also means `dispatch()` takes `&model` (immutable borrow), which is fine as long as we don't drain events during dispatch.

4. **Session updates.** When a delta is applied, the session's system/planet counts must be updated. If the player moved, the session position should update too. This keeps the prompt accurate.

5. **Cache writes are best-effort.** If the cache write fails (disk full, permissions), we log a warning and continue. The next startup will rebuild from the save file.

6. **Config controls watching.** Users can disable file watching in `~/.nms-copilot/config.toml` with `[watch] enabled = false`. The debounce duration is also configurable.

7. **REPL tab completions.** After a delta adds new systems or bases, the tab completions become stale. For Phase 5, this is acceptable -- completions are built once at startup. Refreshing completions would require replacing the completer in reedline, which is possible but adds complexity. Consider for Phase 7 polish.

---

## Dependency Changes Summary

### Workspace `Cargo.toml`

```toml
notify = "7"
notify-debouncer-mini = "0.5"
tempfile = "3"
```

### `crates/nms-watch/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
nms-graph = { workspace = true }
notify = { workspace = true }
notify-debouncer-mini = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

### `crates/nms-copilot/Cargo.toml`

```toml
[dependencies]
nms-watch = { workspace = true }
```
