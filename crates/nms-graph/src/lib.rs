//! In-memory galactic model for NMS Copilot.
//!
//! Builds and maintains a spatial graph of all known star systems using
//! three parallel data structures:
//!
//! - **petgraph** -- topology layer for pathfinding and TSP routing
//! - **R-tree** -- geometric layer for nearest-neighbor and radius queries
//! - **HashMaps** -- associative layer for fast lookup by name, biome, etc.

pub mod error;
pub mod extract;
pub mod model;
pub mod spatial;

pub use error::GraphError;
pub use model::GalaxyModel;
pub use spatial::SystemPoint;
