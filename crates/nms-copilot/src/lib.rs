//! Shared library modules used by the NMS Copilot binary's REPL and MCP server.
//!
//! # Module guide
//!
//! - **Startup and session:** [`banner`] handles startup presentation;
//!   [`config`] and [`paths`] cover configuration and filesystem locations;
//!   [`prompt`], [`session`], and [`completer`] support interactive use.
//! - **Commands and query dispatch:** [`commands`] defines parsed actions and
//!   command syntax; [`dispatch`] executes them.
//! - **Live coordination:** [`coordinator::LiveSaveCoordinator`] owns the
//!   normal live application boundary. Each instance owns one watcher and
//!   serializes update application, with optional best-effort cache writing
//!   after releasing the model lock; it does not enforce a process-wide
//!   singleton. [`watch`] contains legacy delta helpers and read-only
//!   applied-update notification drains. [`session::SessionState`] is a
//!   caller-owned refreshed snapshot that preserves base/manual-address
//!   anchors; it does not persist or write a cache.
//! - **Map and presentation:** [`map`] contains map presentation surfaces.
//! - **MCP:** [`mcp`] provides the server interfaces. Stdio is available across
//!   configurations; HTTP-specific transport and owned HTTP APIs require the
//!   existing `http` feature.
//! - **Setup:** [`setup`] exposes the interactive setup wizard and its error
//!   type.
//!
//! The binary performs startup and composes these reusable library modules;
//! these module links describe the public library surface, not root-level
//! re-exports.

pub mod banner;
pub mod commands;
pub mod completer;
pub mod config;
pub mod coordinator;
pub mod dispatch;
pub mod map;
pub mod mcp;
pub mod paths;
pub mod prompt;
pub mod session;
pub mod setup;
pub mod watch;
