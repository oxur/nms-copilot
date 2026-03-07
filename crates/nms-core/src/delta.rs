//! Delta types describing changes between two save file snapshots.
//!
//! These types live in nms-core so that both nms-graph (apply) and nms-watch
//! (produce) can use them without creating a dependency cycle.

use crate::address::GalacticAddress;
use crate::player::PlayerBase;
use crate::system::{Planet, System, SystemId};

/// A typed description of what changed between two save file snapshots.
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

/// Player position change.
#[derive(Debug, Clone)]
pub struct PlayerMoved {
    pub from: GalacticAddress,
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
