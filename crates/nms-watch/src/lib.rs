//! File watcher and delta computation for NMS Copilot.
//!
//! The crate turns checked save-file parses into snapshot comparisons and
//! channel observations. [`snapshot`] documents the legacy and checked atlas
//! views; [`delta`] describes additive comparisons; [`watcher`] covers file
//! monitoring and its channel/lifetime contracts; and [`error`] documents
//! setup failures.
//!
//! [`start_watching`]: watcher::start_watching
//! [`start_watching_updates`]: watcher::start_watching_updates
//! [`start_watching_observations`]: watcher::start_watching_observations
//! [`SaveSnapshot`]: snapshot::SaveSnapshot
//! [`SaveDelta`]: nms_core::delta::SaveDelta
//! [`SaveUpdate`]: nms_core::delta::SaveUpdate
//! [`DiscoveryNames`]: nms_core::delta::DiscoveryNames

/// Additive comparison of two [`SaveSnapshot`] atlas views.
pub mod delta;
/// Setup errors returned by the watcher entry points.
pub mod error;
/// Mutable atlas maps and constructors from parsed saves.
pub mod snapshot;
/// Save-file monitoring, update streams, and watcher handles.
pub mod watcher;

/// Compute additive discovery, movement, and base observations.
pub use delta::compute_delta;
/// Error returned while preparing a watcher.
pub use error::WatchError;
/// Runtime observation types shared with `nms-core`.
pub use nms_core::delta::{DiscoveryNames, PlayerMoved, SaveDelta, SaveUpdate};
/// Mutable atlas snapshot built from a parsed save.
pub use snapshot::SaveSnapshot;
/// Watcher configurations, handles, observations, and start functions.
pub use watcher::{
    SaveObservation, WatchConfig, WatchHandle, WatchObservationHandle, WatchUpdateHandle,
    start_watching, start_watching_observations, start_watching_updates,
};
