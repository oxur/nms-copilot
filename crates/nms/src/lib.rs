//! # NMS Copilot
//!
//! A real-time galactic copilot for No Man's Sky, built in Rust.
//!
//! This crate is the workspace documentation and navigation landing page; it
//! has no dependencies, re-exports, or API of its own. It is not an umbrella
//! re-export facade. Follow the individual workspace crates for functionality:
//!
//! - [`nms-core`](https://crates.io/crates/nms-core) — core types, enums,
//!   address math, and portal glyphs.
//! - [`nms-save`](https://crates.io/crates/nms-save) — raw binary save parsing.
//! - [`nms-compat`](https://crates.io/crates/nms-compat) — save-format adapters.
//! - [`nms-graph`](https://crates.io/crates/nms-graph) — galactic model and
//!   routing.
//! - [`nms-query`](https://crates.io/crates/nms-query) — shared query engine.
//! - [`nms-watch`](https://crates.io/crates/nms-watch) — save-file watching.
//! - [`nms-cache`](https://crates.io/crates/nms-cache) — serialized model cache.
//! - [`nms-cli`](https://crates.io/crates/nms-cli) — package for the one-shot
//!   `nms` binary.
//! - [`nms-copilot`](https://crates.io/crates/nms-copilot) — package for the
//!   interactive REPL and MCP server.
//!
//! These external crate links are navigation references, not claims about
//! publication or release status. This `nms` package is the documentation
//! landing page; the executable named `nms` is supplied by the `nms-cli`
//! package.
