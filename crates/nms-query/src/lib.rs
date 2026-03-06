//! Shared query engine for NMS Copilot.
//!
//! Pure, stateless query layer consumed by all three interfaces (CLI, REPL, MCP).
//! Takes an immutable reference to the `GalaxyModel` and returns typed results:
//!
//! - `FindQuery` — search planets/systems by biome, distance, name, discoverer
//! - `RouteQuery` — plan traversals with warp-range constraints
//! - `ShowQuery` — detail views of systems, planets, bases
//! - `ConvertQuery` — coordinate/glyph conversion (pure math, no model needed)
//! - `StatsQuery` — aggregate statistics and distributions
