//! Shared query engine for NMS Copilot.
//!
//! Pure, stateless query layer consumed by all three interfaces (CLI, REPL, MCP).
//! Takes an immutable reference to the `GalaxyModel` and returns typed results.

pub mod display;
pub mod find;
pub mod route;
pub mod show;
pub mod stats;
pub mod theme;

pub use display::{
    format_distance, format_find_results, format_route, format_show_result, format_stats,
    hex_to_emoji,
};
pub use find::{FindQuery, FindResult, ReferencePoint};
pub use route::{RouteFrom, RouteQuery, RouteResult, TargetSelection};
pub use show::{ShowQuery, ShowResult};
pub use stats::{StatsQuery, StatsResult};
pub use theme::{Theme, should_use_colors};
