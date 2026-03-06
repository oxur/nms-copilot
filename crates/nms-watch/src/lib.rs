//! File watcher and delta computation for NMS Copilot.
//!
//! Monitors the NMS save directory for changes, re-parses on auto-save,
//! computes typed deltas (new discoveries, player movement, new bases),
//! and distributes updates via broadcast channel to all consumers.
