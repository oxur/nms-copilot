//! Delta types describing changes between two save file snapshots.
//!
//! These types live in nms-core so that both nms-graph (apply) and nms-watch
//! (produce) can use them without creating a dependency cycle.
//!
//! [SaveDelta] reports additive observations; it does not report that records
//! disappeared from a later snapshot. [SaveUpdate] pairs that delta with the
//! complete supported player state and version from the same parsed update.

use std::collections::HashMap;

use crate::address::GalacticAddress;
use crate::player::{PlayerBase, PlayerState};
use crate::system::{Planet, System, SystemId};

/// Runtime discovery names observed from one validated save.
///
/// The keys use galaxy-qualified system identities. A present empty string
/// explicitly clears a name; a missing key leaves the corresponding model
/// value unchanged. Values are retained literally, without trimming. This is
/// an in-memory observation type, not a save or cache representation.
///
/// # Examples
///
/// ```
/// use nms_core::address::GalacticAddress;
/// use nms_core::delta::DiscoveryNames;
/// use nms_core::system::SystemId;
///
/// let id = SystemId::from_address(&GalacticAddress::new(0, 0, 0, 1, 0, 0));
/// let mut observations = DiscoveryNames::default();
/// assert!(observations.is_empty());
///
/// // A present empty value means clear; an absent entry means preserve.
/// observations.system_names.insert(id, String::new());
/// assert!(!observations.is_empty());
/// assert_eq!(observations.system_names.get(&id).map(String::as_str), Some(""));
/// let absent = DiscoveryNames::default();
/// assert!(!absent.system_names.contains_key(&id));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscoveryNames {
    /// Literal system names keyed by galaxy-qualified [`SystemId`].
    ///
    /// An empty string clears a system name; absence preserves it.
    pub system_names: HashMap<SystemId, String>,
    /// Literal planet names keyed by `(system, planet index)`.
    ///
    /// An empty string clears a planet name; absence preserves it.
    pub planet_names: HashMap<(SystemId, u8), String>,
}

impl DiscoveryNames {
    /// Return whether this observation contains no system or planet entries.
    pub fn is_empty(&self) -> bool {
        self.system_names.is_empty() && self.planet_names.is_empty()
    }
}

/// Additive changes observed between two save snapshots.
///
/// The fields list newly observed records, movement, and base changes. The
/// delta does not promise to represent records removed from the source.
///
/// # Examples
///
/// ```
/// use nms_core::{GalacticAddress, PlayerMoved, SaveDelta};
///
/// let mut delta = SaveDelta::empty();
/// assert!(delta.is_empty());
/// assert_eq!(delta.change_count(), 0);
/// let from = GalacticAddress::new(3, 0, 0, 1, 2, 9);
/// let to = GalacticAddress::new(4, 0, 0, 1, 2, 9);
/// delta.player_moved = Some(PlayerMoved { from, to });
/// assert!(!delta.is_empty());
/// assert_eq!(delta.change_count(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct SaveDelta {
    /// Newly discovered systems (not in previous snapshot).
    pub new_systems: Vec<System>,
    /// Newly discovered planets (not in previous snapshot).
    pub new_planets: Vec<(SystemId, Planet)>,
    /// Player moved to a new position.
    pub player_moved: Option<PlayerMoved>,
    /// Newly placed bases.
    pub new_bases: Vec<PlayerBase>,
    /// Modified bases (name exists but content differs).
    pub modified_bases: Vec<PlayerBase>,
}

/// A complete supported player save update paired with its atlas delta.
///
/// This runtime event carries the discovery/base/movement changes in
/// [`SaveDelta`] together with the complete [`PlayerState`] conversion and the
/// source save version observed from the same parsed save. The state and
/// version are authoritative for the player snapshot even if a synthetic
/// delta contains a contradictory movement. This type is not a persisted or
/// archive format, and the delta does not promise that removed source records
/// will be reported.
#[derive(Debug, Clone)]
pub struct SaveUpdate {
    /// Additive atlas changes detected between the previous and current save.
    pub delta: SaveDelta,
    /// Complete supported player state from the same parse as `delta`.
    pub player_state: PlayerState,
    /// Version field from the same parsed source save.
    pub save_version: u32,
}

/// A change in the player's galactic address.
#[derive(Debug, Clone)]
pub struct PlayerMoved {
    /// Address before the movement.
    pub from: GalacticAddress,
    /// Address after the movement.
    pub to: GalacticAddress,
}

impl SaveDelta {
    /// Create an empty delta (no changes).
    pub fn empty() -> Self {
        Self {
            new_systems: Vec::new(),
            new_planets: Vec::new(),
            player_moved: None,
            new_bases: Vec::new(),
            modified_bases: Vec::new(),
        }
    }

    /// Returns true if no changes were detected.
    pub fn is_empty(&self) -> bool {
        self.new_systems.is_empty()
            && self.new_planets.is_empty()
            && self.player_moved.is_none()
            && self.new_bases.is_empty()
            && self.modified_bases.is_empty()
    }

    /// Total number of individual changes.
    pub fn change_count(&self) -> usize {
        self.new_systems.len()
            + self.new_planets.len()
            + self.player_moved.as_ref().map_or(0, |_| 1)
            + self.new_bases.len()
            + self.modified_bases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_delta_is_empty() {
        let delta = SaveDelta::empty();
        assert!(delta.is_empty());
        assert_eq!(delta.change_count(), 0);
    }

    #[test]
    fn test_delta_with_system_is_not_empty() {
        let addr = GalacticAddress::new(100, 50, -200, 0x42, 0, 0);
        let system = System::new(addr, Some("Test".into()), None, None, vec![]);
        let delta = SaveDelta {
            new_systems: vec![system],
            ..SaveDelta::empty()
        };
        assert!(!delta.is_empty());
        assert_eq!(delta.change_count(), 1);
    }

    #[test]
    fn test_delta_change_count_sums_all() {
        let addr = GalacticAddress::new(100, 50, -200, 0x42, 0, 0);
        let system = System::new(addr, None, None, None, vec![]);
        let sys_id = SystemId::from_address(&addr);
        let planet = Planet::new(0, None, None, false, None, None);
        let base = PlayerBase::new(
            "Base".into(),
            crate::player::BaseType::HomePlanetBase,
            addr,
            [0.0, 0.0, 0.0],
            None,
        );

        let delta = SaveDelta {
            new_systems: vec![system],
            new_planets: vec![(sys_id, planet)],
            player_moved: Some(PlayerMoved {
                from: addr,
                to: addr,
            }),
            new_bases: vec![base.clone()],
            modified_bases: vec![base],
        };
        assert_eq!(delta.change_count(), 5);
    }
}
