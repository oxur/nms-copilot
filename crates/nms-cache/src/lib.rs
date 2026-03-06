//! Zero-copy serialization cache for NMS Copilot.
//!
//! Serializes the in-memory `GalaxyModel` to an rkyv archive for near-instant
//! startup on subsequent runs. Handles freshness checks against save file
//! modification time and cache invalidation from the file watcher.
