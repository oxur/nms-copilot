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
pub use nms_core::delta::{PlayerMoved, SaveDelta};
pub use snapshot::SaveSnapshot;
pub use watcher::{WatchConfig, WatchHandle, start_watching};
