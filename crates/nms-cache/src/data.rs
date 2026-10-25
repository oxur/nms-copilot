//! Data records for the portion of a galaxy model stored in the cache.
//!
//! [`CacheData`] is a public, mutable serialized subset, not a snapshot of every
//! `GalaxyModel` field or index. Systems carry their planets inline; bases and
//! the supported player-state values are retained, while graph topology and
//! derived lookup indexes are rebuilt when the model is loaded. Extracted
//! vectors come from hash maps and therefore have no promised order. System
//! timestamps are stored at whole-second precision.

use rkyv::{Archive, Deserialize, Serialize};

use nms_core::address::GalacticAddress;
use nms_core::player::{PlayerBase, PlayerState};
use nms_core::system::{DiscoverySource, Planet};

/// The supported, serialized subset of a galaxy model.
///
/// This is not a full model snapshot: it does not store custom graph edges,
/// derived indexes, orphan flat-planet entries, raw save JSON, session state,
/// or event history. Systems contain their planets, and their extracted vector
/// order is unspecified. Timestamps are stored as whole Unix seconds.
///
/// The selected galaxy is stored independently of the physical player's
/// address. The caller supplies `save_version`; this type does not validate it
/// against a save. Public struct literals must provide all six fields.
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

    /// Caller-supplied save version associated with this cache.
    ///
    /// The cache codec stores this value independently of the envelope
    /// revisions and does not verify it against a source save.
    pub save_version: u32,

    /// Cache timestamp in Unix seconds.
    ///
    /// Extraction records the current whole-second time, using zero if the
    /// system clock is before the Unix epoch. Decoding does not compare this
    /// value with the current clock.
    pub cached_at: u64,
}

/// A system record with cache-compatible timestamp storage.
///
/// We flatten `System` rather than embedding it because `System` contains
/// `Option<DateTime<Utc>>` which doesn't support rkyv. Timestamps are
/// stored as Unix seconds and converted on load; subsecond precision is lost.
/// The address is the stored system address, which may retain the supplied
/// planet index. System identity used for indexes is canonicalized separately.
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct CachedSystem {
    /// Galactic address of this system.
    pub address: GalacticAddress,
    /// Origin of this system's atlas contribution.
    pub source: DiscoverySource,
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
    /// # Examples
    ///
    /// ```
    /// use nms_cache::data::CachedSystem;
    /// use nms_core::address::GalacticAddress;
    ///
    /// let known_empty_discovery = CachedSystem {
    ///     address: GalacticAddress::new(0, 0, 0, 1, 0, 0),
    ///     source: nms_core::system::DiscoverySource::Personal,
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
