//! Cache data types -- the subset of GalaxyModel that gets serialized.

use rkyv::{Archive, Deserialize, Serialize};

use nms_core::address::GalacticAddress;
use nms_core::player::{PlayerBase, PlayerState};
use nms_core::system::Planet;

/// Flattened galaxy data for cache serialization.
///
/// Contains all the raw data needed to reconstruct a `GalaxyModel`.
///
/// The archive schema stores the selected galaxy explicitly. Downstream Rust
/// struct literals must provide [`active_galaxy`](Self::active_galaxy).
/// Indices (graph, R-tree, HashMaps) are rebuilt on load.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CacheData {
    /// All model systems, including discoveries and marked player-location placeholders.
    pub systems: Vec<CachedSystem>,

    /// All player bases.
    pub bases: Vec<PlayerBase>,

    /// Player state at time of caching.
    pub player_state: Option<PlayerState>,

    /// Galaxy selected for spatial queries, independent of player position.
    pub active_galaxy: u8,

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
    /// Galactic address of this system.
    pub address: GalacticAddress,
    /// Discovered system name, when known.
    pub name: Option<String>,
    /// Discoverer name, when known.
    pub discoverer: Option<String>,
    /// Timestamp as Unix seconds (converted from `DateTime<Utc>`).
    pub timestamp_secs: Option<i64>,
    /// Discovered planets belonging to this system.
    pub planets: Vec<Planet>,
    /// Whether this entry is the explicit synthetic player-location placeholder.
    ///
    /// Checked cache boundaries validate this flag; an ordinary empty discovery
    /// record must set it to `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use nms_cache::data::CachedSystem;
    /// use nms_core::address::GalacticAddress;
    ///
    /// let known_empty_discovery = CachedSystem {
    ///     address: GalacticAddress::new(0, 0, 0, 1, 0, 0),
    ///     name: None,
    ///     discoverer: None,
    ///     timestamp_secs: None,
    ///     planets: vec![],
    ///     is_player_placeholder: false,
    /// };
    /// assert!(!known_empty_discovery.is_player_placeholder);
    /// ```
    pub is_player_placeholder: bool,
}
