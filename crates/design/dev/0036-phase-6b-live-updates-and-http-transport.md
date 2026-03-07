# Phase 6B -- Live Updates & HTTP Transport

Milestones 6.8-6.9: Watcher integration for real-time model updates, and optional streaming HTTP transport.

**Depends on:** Phase 6A (MCP server + tools), Phase 5 (nms-watch watcher + delta types).

---

## Milestone 6.8: Live Update Integration

### Goal

The MCP server's galaxy model stays current as the player plays. When the game auto-saves, the watcher detects the change, computes a delta, and the server applies it. The AI always sees the latest data without restarting.

### Architecture

Phase 6A uses `Arc<GalaxyModel>` (immutable). For live updates, we need mutability. Two options:

**Option A: `Arc<RwLock<GalaxyModel>>`** -- Tools take a read lock. The watcher thread takes a write lock to apply deltas. Simple, but write locks block all reads.

**Option B: `Arc` swap** -- The watcher builds a new `Arc<GalaxyModel>` after each delta and atomically swaps it via `ArcSwap` or `tokio::sync::watch`. Tools always read from the latest snapshot without locking.

**Recommendation: Option A (`Arc<RwLock>`)** -- Simpler, and write locks are rare (only on auto-save, every few minutes). Read lock contention is negligible for a single-user tool. `tokio::sync::RwLock` is used since tool handlers are async.

### Modified File: `crates/nms-mcp/src/tools.rs`

```rust
use tokio::sync::RwLock;

pub struct NmsTools {
    model: Arc<RwLock<GalaxyModel>>,
}

impl NmsTools {
    pub fn new(model: Arc<RwLock<GalaxyModel>>) -> Self {
        Self { model }
    }
}

// Each handler acquires a read lock:
async fn handle_where_am_i(
    model: Arc<RwLock<GalaxyModel>>,
    _args: Value,
) -> Result<CallToolResult, ErrorData> {
    let model = model.read().await;
    let addr = model.player_position()
        .ok_or_else(|| tool_error("Player position not available"))?;
    // ... rest unchanged
}
```

### Modified File: `crates/nms-mcp/src/main.rs`

```rust
use std::sync::mpsc;
use std::thread;
use tokio::sync::RwLock;

use nms_watch::{WatchConfig, start_watching, SaveDelta};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = resolve_save_path()?;
    let model = load_model(&save_path)?;
    let model = Arc::new(RwLock::new(model));

    // Start file watcher
    let watch_handle = start_watching(WatchConfig {
        save_path: save_path.clone(),
        ..Default::default()
    }).ok();

    // Spawn delta application task
    if let Some(handle) = watch_handle {
        let model_for_watcher = Arc::clone(&model);
        tokio::spawn(async move {
            apply_deltas_loop(handle.receiver, model_for_watcher).await;
        });
    }

    // Build tools and server (same as 6A but with RwLock model)
    let nms_tools = NmsTools::new(Arc::clone(&model));
    // ... rest of server setup ...

    server.serve_stdio().await?;
    Ok(())
}

/// Background loop: receive deltas from watcher, apply to model.
async fn apply_deltas_loop(
    receiver: mpsc::Receiver<SaveDelta>,
    model: Arc<RwLock<GalaxyModel>>,
) {
    // mpsc::Receiver is not Send, so we bridge to tokio
    // by polling in a blocking task
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    thread::spawn(move || {
        while let Ok(delta) = receiver.recv() {
            if tx.send(delta).is_err() {
                break;
            }
        }
    });

    while let Some(delta) = rx.recv().await {
        let mut model = model.write().await;
        model.apply_delta(&delta);

        // Log changes for debugging
        if !delta.new_systems.is_empty() {
            log::info!(
                "Live update: {} new system(s)",
                delta.new_systems.len()
            );
        }
        if delta.player_moved.is_some() {
            log::info!("Live update: player moved");
        }
    }
}
```

### Service Handle for Watcher

Use fabryk-mcp's service lifecycle to track watcher readiness:

```rust
use fabryk_mcp::model::ServiceHandle;

let watcher_service = ServiceHandle::new("file-watcher");

match start_watching(watch_config) {
    Ok(handle) => {
        watcher_service.set_state(ServiceState::Ready);
        // spawn delta loop ...
    }
    Err(e) => {
        watcher_service.set_state(ServiceState::Failed(e.to_string()));
        // Server still starts -- tools work with static model
    }
}

let server = FabrykMcpServer::new(discoverable)
    .with_service(watcher_service)
    // ...
```

The health tool and `nms_directory` will reflect watcher status. If the watcher fails, tools still work with the initial model snapshot.

### Tests (Milestone 6.8)

```rust
#[cfg(test)]
mod live_update_tests {
    use super::*;

    #[tokio::test]
    async fn test_model_updates_after_delta() {
        let model = Arc::new(RwLock::new(test_model()));
        let count_before = model.read().await.system_count();

        let delta = SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("New System".into()),
                None, None, vec![],
            )],
            ..SaveDelta::empty()
        };

        {
            let mut m = model.write().await;
            m.apply_delta(&delta);
        }

        assert_eq!(model.read().await.system_count(), count_before + 1);
    }

    #[tokio::test]
    async fn test_tools_see_updated_model() {
        let model = Arc::new(RwLock::new(test_model()));
        let tools = NmsTools::new(Arc::clone(&model));

        // Get initial stats
        let result = tools.call("galaxy_stats", json!({})).unwrap().await.unwrap();
        let initial_count = /* parse system count from result */;

        // Apply delta
        {
            let mut m = model.write().await;
            m.apply_delta(&delta_with_new_system());
        }

        // Stats should reflect new system
        let result = tools.call("galaxy_stats", json!({})).unwrap().await.unwrap();
        let updated_count = /* parse system count from result */;
        assert!(updated_count > initial_count);
    }

    #[tokio::test]
    async fn test_read_lock_during_tool_call() {
        // Ensure tool calls can proceed concurrently with read locks
        let model = Arc::new(RwLock::new(test_model()));
        let tools1 = NmsTools::new(Arc::clone(&model));
        let tools2 = NmsTools::new(Arc::clone(&model));

        // Two concurrent tool calls should not deadlock
        let (r1, r2) = tokio::join!(
            tools1.call("where_am_i", json!({})).unwrap(),
            tools2.call("galaxy_stats", json!({})).unwrap(),
        );
        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }
}
```

---

## Milestone 6.9: Streaming HTTP Transport

### Goal

Enable the MCP server to serve over HTTP in addition to stdio. This allows web-based AI clients to connect.

fabryk-mcp provides this out of the box via the `http` feature flag. No new code is needed -- just a CLI flag to select transport.

### Modified File: `crates/nms-mcp/src/main.rs`

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let transport = parse_transport(&args);

    // ... model loading, watcher setup, tool registration ...

    match transport {
        Transport::Stdio => {
            server.serve_stdio().await?;
        }
        #[cfg(feature = "http")]
        Transport::Http(addr) => {
            eprintln!("NMS Copilot MCP server listening on http://{addr}");
            server.serve_http(addr).await?;
        }
    }

    Ok(())
}

enum Transport {
    Stdio,
    #[cfg(feature = "http")]
    Http(std::net::SocketAddr),
}

fn parse_transport(args: &[String]) -> Transport {
    if let Some(pos) = args.iter().position(|a| a == "--http") {
        let addr = args.get(pos + 1)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
        #[cfg(feature = "http")]
        return Transport::Http(addr);
        #[cfg(not(feature = "http"))]
        {
            eprintln!("HTTP transport requires the 'http' feature. Falling back to stdio.");
            Transport::Stdio
        }
    } else {
        Transport::Stdio
    }
}
```

### Modified File: `crates/nms-mcp/Cargo.toml`

```toml
[features]
default = []
http = ["fabryk-mcp/http"]
```

### Usage

```bash
# Stdio transport (default, for Claude Desktop)
nms-mcp --save /path/to/save.json

# HTTP transport (for web clients)
nms-mcp --save /path/to/save.json --http 127.0.0.1:3000

# HTTP with auto-detected save
cargo run -p nms-mcp --features http -- --http 127.0.0.1:3000
```

When serving HTTP, fabryk-mcp automatically provides:
- `GET /health` -- JSON health status with service states
- `POST /mcp` -- Streaming HTTP MCP endpoint (JSON-RPC over HTTP)

### Tests (Milestone 6.9)

```rust
#[cfg(test)]
mod transport_tests {
    use super::*;

    #[test]
    fn test_parse_transport_default_stdio() {
        let args = vec!["nms-mcp".into()];
        assert!(matches!(parse_transport(&args), Transport::Stdio));
    }

    #[test]
    fn test_parse_transport_http_default_addr() {
        let args = vec!["nms-mcp".into(), "--http".into()];
        let transport = parse_transport(&args);
        #[cfg(feature = "http")]
        assert!(matches!(transport, Transport::Http(_)));
        #[cfg(not(feature = "http"))]
        assert!(matches!(transport, Transport::Stdio));
    }

    #[test]
    fn test_parse_transport_http_custom_addr() {
        let args = vec![
            "nms-mcp".into(),
            "--http".into(),
            "0.0.0.0:8080".into(),
        ];
        let transport = parse_transport(&args);
        #[cfg(feature = "http")]
        match transport {
            Transport::Http(addr) => assert_eq!(addr.port(), 8080),
            _ => panic!("Expected Http transport"),
        }
    }
}
```

---

## Claude Desktop Configuration

To use nms-mcp with Claude Desktop, add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "nms-copilot": {
      "command": "/path/to/nms-mcp",
      "args": ["--save", "/path/to/save.json"]
    }
  }
}
```

Or with auto-detected save path:

```json
{
  "mcpServers": {
    "nms-copilot": {
      "command": "/path/to/nms-mcp"
    }
  }
}
```

---

## Implementation Notes

1. **`std::sync::mpsc` to `tokio::sync::mpsc` bridge.** The nms-watch watcher uses `std::sync::mpsc` (Phase 5 is sync). The MCP server is async. We bridge with a std thread that receives from the sync channel and forwards to a tokio channel.

2. **Write lock duration is minimal.** `apply_delta()` only inserts new systems/planets and updates player position. It doesn't rebuild the R-tree or graph edges. Sub-millisecond for typical deltas.

3. **Graceful degradation.** If the watcher can't start (no save file, permission error), the server still runs with the initial model. The service handle reports `Failed` state, visible in health checks.

4. **HTTP is opt-in.** The `http` feature pulls in `axum` and related deps. Default builds (stdio only) stay lightweight.

5. **No cache integration in MCP server.** Unlike the REPL, the MCP server doesn't write cache. It loads from the save file directly (or from cache if available via nms-cache). Cache write-through remains a REPL concern.

6. **fabryk-mcp's `serve_http`** handles session management internally via `LocalSessionManager`. No manual session tracking needed.

7. **Logging.** Use `log` crate for delta application messages. The MCP server should initialize a logger (e.g., `env_logger`) at startup for debug visibility.
