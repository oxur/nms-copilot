//! # NMS Copilot
//!
//! A real-time galactic copilot for No Man's Sky, built in Rust.
//!
//! This is the umbrella crate for the NMS Copilot workspace. See the
//! individual crates for functionality:
//!
//! - [`nms-core`](https://crates.io/crates/nms-core) — types, enums, address math, portal glyphs
//! - [`nms-save`](https://crates.io/crates/nms-save) — raw binary save parser
//! - [`nms-graph`](https://crates.io/crates/nms-graph) — in-memory galactic model and routing
//! - [`nms-query`](https://crates.io/crates/nms-query) — shared query engine
//! - [`nms-cli`](https://crates.io/crates/nms-cli) — one-shot CLI (`nms` binary)
//! - [`nms-copilot`](https://crates.io/crates/nms-copilot) — interactive REPL
//! - [`nms-mcp`](https://crates.io/crates/nms-mcp) — MCP server for AI integration
