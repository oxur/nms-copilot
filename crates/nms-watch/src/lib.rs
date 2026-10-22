//! File watcher and delta computation for NMS Copilot.
//!
//! Monitors the NMS save directory for changes, re-parses on auto-save,
//! computes typed deltas (new discoveries, player movement, new bases),
//! and distributes updates via channel to all consumers.

pub mod delta;
pub mod error;
pub mod snapshot;
pub mod watcher;

pub use delta::compute_delta;
pub use error::WatchError;
pub use nms_core::delta::{DiscoveryNames, PlayerMoved, SaveDelta, SaveUpdate};
pub use snapshot::SaveSnapshot;
pub use watcher::{
    SaveObservation, WatchConfig, WatchHandle, WatchObservationHandle, WatchUpdateHandle,
    start_watching, start_watching_observations, start_watching_updates,
};
