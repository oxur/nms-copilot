//! In-memory galactic model for NMS Copilot.
//!
//! Builds and maintains a spatial graph of all known star systems using
//! three parallel data structures:
//!
//! - **petgraph** -- edge topology used by shortest-path routing
//! - **R-trees** -- per-galaxy neighbor searches when building graph edges
//! - **HashMaps** -- model data and indexes; public spatial queries and route
//!   ordering also measure positions from the system maps directly
//!
//! The [checked extraction example](crate::extract#checked-extraction-example)
//! demonstrates model construction. The [routing example](crate::route#example)
//! exercises spatial queries, edge building, and route APIs.

/// Build and rebuild per-galaxy graph edges.
pub mod edges;
/// Error types returned by graph and query operations.
pub mod error;
/// Extract systems, planets, and discovery names from save records.
pub mod extract;
/// Import community coordinate catalogs from CSV.
pub mod import;
/// The in-memory galaxy model and its mutation methods.
pub mod model;
/// Spatially scoped system and planet queries.
pub mod query;
/// Shortest-path, tour, hop-constrained, and reachability operations.
pub mod route;
/// Galaxy-qualified system identity and voxel-space point types.
pub mod spatial;

/// Strategy used to construct model graph edges.
pub use edges::EdgeStrategy;
/// Errors from graph lookups and spatial query resolution.
pub use error::GraphError;
/// In-memory model of systems, planets, bases, and graph indexes.
pub use model::GalaxyModel;
/// Planet predicates used by spatial query methods.
pub use query::BiomeFilter;
/// Route result, error, hop, and ordering-algorithm types.
pub use route::{Route, RouteError, RouteHop, RoutingAlgorithm};
/// Canonical galaxy-qualified system identity and associated voxel-space point.
pub use spatial::{SystemId, SystemPoint};
