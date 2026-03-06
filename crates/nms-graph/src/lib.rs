//! In-memory galactic model for NMS Copilot.
//!
//! The heart of the system — builds and maintains a spatial graph of all known
//! star systems using three parallel data structures:
//!
//! - **petgraph** — topology layer for pathfinding and TSP routing
//! - **R-tree** — geometric layer for nearest-neighbor and radius queries
//! - **HashMaps** — associative layer for fast lookup by name, biome, etc.
//!
//! Supports incremental updates from the file watcher.
