//! Shared query engine for NMS Copilot.
//!
//! Pure, stateless query layer consumed by all three interfaces (CLI, REPL, MCP).
//! Takes an immutable reference to the `GalaxyModel` and returns typed results.

pub mod find;
pub mod show;
pub mod stats;

pub use find::{FindQuery, FindResult, ReferencePoint};
pub use show::{ShowQuery, ShowResult};
pub use stats::{StatsQuery, StatsResult};
