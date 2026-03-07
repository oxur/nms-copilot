//! Cache data types -- the subset of GalaxyModel that gets serialized.

use rkyv::{Archive, Deserialize, Serialize};

use nms_core::address::GalacticAddress;
use nms_core::player::{PlayerBase, PlayerState};
use nms_core::system::Planet;

/// Flattened galaxy data for cache serialization.
///
/// Contains all the raw data needed to reconstruct a `GalaxyModel`.
/// Indices (graph, R-tree, HashMaps) are rebuilt on load.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CacheData {
    /// All discovered systems.
    pub systems: Vec<CachedSystem>,

    /// All player bases.
    pub bases: Vec<PlayerBase>,

    /// Player state at time of caching.
    pub player_state: Option<PlayerState>,

    /// Save file version that produced this cache.
    pub save_version: u32,

    /// Timestamp when the cache was created (Unix seconds).
    pub cached_at: u64,
}

/// A system with flattened fields for cache storage.
///
/// We flatten `System` rather than embedding it because `System` contains
/// `Option<DateTime<Utc>>` which doesn't support rkyv. Timestamps are
/// stored as Unix seconds and converted on load.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CachedSystem {
    pub address: GalacticAddress,
    pub name: Option<String>,
    pub discoverer: Option<String>,
    /// Timestamp as Unix seconds (converted from `DateTime<Utc>`).
    pub timestamp_secs: Option<i64>,
    pub planets: Vec<Planet>,
}
