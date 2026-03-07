//! The GalaxyModel -- central in-memory representation of the player's galaxy.

use std::collections::HashMap;

use petgraph::Undirected;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use rstar::RTree;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::player::{PlayerBase, PlayerState};
use nms_core::system::{Planet, System};
use nms_save::model::SaveRoot;

use crate::extract::extract_systems;
use crate::spatial::{SystemId, SystemPoint};

/// Key for looking up a specific planet: (system, planet index).
pub type PlanetKey = (SystemId, u8);

/// The in-memory galactic model.
///
/// Three parallel data structures kept in sync:
/// 1. petgraph -- topology (pathfinding, routing)
/// 2. R-tree -- spatial (nearest-neighbor, radius queries)
/// 3. HashMaps -- associative (name, biome, address lookups)
#[derive(Debug)]
pub struct GalaxyModel {
    /// Graph topology: nodes are systems, edge weights are distance in ly.
    pub graph: StableGraph<SystemId, f64, Undirected>,

    /// 3D spatial index of system positions.
    pub spatial: RTree<SystemPoint>,

    /// System data by ID.
    pub systems: HashMap<SystemId, System>,

    /// Planet data by (SystemId, planet_index).
    pub planets: HashMap<PlanetKey, Planet>,

    /// Base lookup by name (lowercase).
    pub bases: HashMap<String, PlayerBase>,

    /// Biome -> list of planets with that biome.
    pub biome_index: HashMap<Biome, Vec<PlanetKey>>,

    /// System name -> SystemId (lowercase, only for named systems).
    pub name_index: HashMap<String, SystemId>,

    /// Packed address (planet bits zeroed) -> SystemId.
    pub address_to_id: HashMap<u64, SystemId>,

    /// SystemId -> petgraph NodeIndex.
    pub node_map: HashMap<SystemId, NodeIndex>,

    /// Current player state (position, currencies).
    pub player_state: Option<PlayerState>,
}

impl GalaxyModel {
    /// Build a `GalaxyModel` from a parsed save file.
    pub fn from_save(save: &SaveRoot) -> Self {
        let extracted = extract_systems(save);

        let mut graph: StableGraph<SystemId, f64, Undirected> = StableGraph::default();
        let mut spatial_points = Vec::with_capacity(extracted.len());
        let mut systems = HashMap::with_capacity(extracted.len());
        let mut planets = HashMap::new();
        let mut biome_index: HashMap<Biome, Vec<PlanetKey>> = HashMap::new();
        let mut name_index = HashMap::new();
        let mut address_to_id = HashMap::new();
        let mut node_map = HashMap::new();

        for (sys_id, system) in extracted {
            // Add to petgraph
            let node_idx = graph.add_node(sys_id);
            node_map.insert(sys_id, node_idx);

            // Add to spatial index
            let point = SystemPoint::from_address(&system.address);
            spatial_points.push(point);

            // Add to address lookup
            address_to_id.insert(sys_id.0, sys_id);

            // Add to name index
            if let Some(ref name) = system.name {
                name_index.insert(name.to_lowercase(), sys_id);
            }

            // Extract planets into flat index
            for planet in &system.planets {
                let key = (sys_id, planet.index);
                if let Some(biome) = planet.biome {
                    biome_index.entry(biome).or_default().push(key);
                }
                planets.insert(key, planet.clone());
            }

            systems.insert(sys_id, system);
        }

        // Build R-tree from collected points
        let spatial = RTree::bulk_load(spatial_points);

        // Extract player state
        let player_state = Some(save.to_core_player_state());

        // Extract bases
        let ps = save.active_player_state();
        let mut bases = HashMap::new();
        for base in &ps.persistent_player_bases {
            let core_base = base.to_core_base();
            if !core_base.name.is_empty() {
                bases.insert(core_base.name.to_lowercase(), core_base);
            }
        }

        Self {
            graph,
            spatial,
            systems,
            planets,
            bases,
            biome_index,
            name_index,
            address_to_id,
            node_map,
            player_state,
        }
    }

    /// Number of systems in the model.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Number of planets in the model.
    pub fn planet_count(&self) -> usize {
        self.planets.len()
    }

    /// Number of bases in the model.
    pub fn base_count(&self) -> usize {
        self.bases.len()
    }

    /// Look up a system by its ID.
    pub fn system(&self, id: &SystemId) -> Option<&System> {
        self.systems.get(id)
    }

    /// Look up a system by name (case-insensitive).
    pub fn system_by_name(&self, name: &str) -> Option<(&SystemId, &System)> {
        self.name_index
            .get(&name.to_lowercase())
            .and_then(|id| self.systems.get(id).map(|s| (id, s)))
    }

    /// Look up a base by name (case-insensitive).
    pub fn base(&self, name: &str) -> Option<&PlayerBase> {
        self.bases.get(&name.to_lowercase())
    }

    /// Get the player's current position, if available.
    pub fn player_position(&self) -> Option<&GalacticAddress> {
        self.player_state.as_ref().map(|ps| &ps.current_address)
    }

    /// Get all planets with a given biome.
    pub fn planets_by_biome(&self, biome: Biome) -> Vec<&Planet> {
        self.biome_index
            .get(&biome)
            .map(|keys| keys.iter().filter_map(|k| self.planets.get(k)).collect())
            .unwrap_or_default()
    }

    /// Insert a new system into all indexes.
    pub fn insert_system(&mut self, system: System) {
        let sys_id = SystemId::from_address(&system.address);

        if self.systems.contains_key(&sys_id) {
            return; // Already exists
        }

        let node_idx = self.graph.add_node(sys_id);
        self.node_map.insert(sys_id, node_idx);

        let point = SystemPoint::from_address(&system.address);
        self.spatial.insert(point);

        self.address_to_id.insert(sys_id.0, sys_id);

        if let Some(ref name) = system.name {
            self.name_index.insert(name.to_lowercase(), sys_id);
        }

        for planet in &system.planets {
            let key = (sys_id, planet.index);
            if let Some(biome) = planet.biome {
                self.biome_index.entry(biome).or_default().push(key);
            }
            self.planets.insert(key, planet.clone());
        }

        self.systems.insert(sys_id, system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal SaveRoot JSON and parse it.
    fn minimal_save() -> SaveRoot {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                    "Units": 1000000, "Nanites": 5000, "Specials": 200,
                    "PersistentPlayerBases": [
                        {
                            "BaseVersion": 8, "GalacticAddress": "0x050003AB8C07",
                            "Position": [0.0, 0.0, 0.0], "Forward": [1.0, 0.0, 0.0],
                            "LastUpdateTimestamp": 1700000000, "Objects": [], "RID": "",
                            "Owner": {"LID": "", "UID": "123", "USN": "Test", "PTK": "ST", "TS": 0},
                            "Name": "Home Base",
                            "BaseType": {"PersistentBaseTypes": "HomePlanetBase"},
                            "LastEditedById": "", "LastEditedByUsername": ""
                        }
                    ]
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 100, "ReserveManaged": 100,
                    "Store": {
                        "Record": [
                            {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": ["0xABCD"]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                            {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xDEAD", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                            {"DD": {"UA": "0x0A0002001234", "DT": "SolarSystem", "VP": ["0x1234"]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
                        ]
                    }
                }
            }
        }"#;
        nms_save::parse_save(json.as_bytes()).unwrap()
    }

    #[test]
    fn test_from_save_basic_counts() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.system_count(), 2);
        assert_eq!(model.planet_count(), 1);
        assert_eq!(model.base_count(), 1);
    }

    #[test]
    fn test_from_save_base_lookup() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let base = model.base("Home Base").unwrap();
        assert_eq!(base.name, "Home Base");
    }

    #[test]
    fn test_from_save_base_lookup_case_insensitive() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert!(model.base("home base").is_some());
        assert!(model.base("HOME BASE").is_some());
    }

    #[test]
    fn test_from_save_player_position() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let pos = model.player_position().unwrap();
        assert_eq!(pos.voxel_x(), 100);
        assert_eq!(pos.voxel_y(), 50);
        assert_eq!(pos.voxel_z(), -200);
    }

    #[test]
    fn test_from_save_spatial_index_populated() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.spatial.size(), 2);
    }

    #[test]
    fn test_from_save_graph_nodes() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.graph.node_count(), 2);
    }

    #[test]
    fn test_from_save_biome_index() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let lush = model.planets_by_biome(Biome::Lush);
        assert_eq!(lush.len(), 1);
    }

    #[test]
    fn test_from_save_biome_index_empty() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let toxic = model.planets_by_biome(Biome::Toxic);
        assert!(toxic.is_empty());
    }

    #[test]
    fn test_insert_system_adds_to_all_indexes() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let count_before = model.system_count();

        let addr = GalacticAddress::new(500, 10, -300, 0x999, 0, 0);
        let system = System::new(
            addr,
            Some("New System".to_string()),
            None,
            None,
            vec![Planet::new(0, Some(Biome::Lava), None, false, None, None)],
        );
        model.insert_system(system);

        assert_eq!(model.system_count(), count_before + 1);
        assert!(model.system_by_name("New System").is_some());
        assert_eq!(model.spatial.size(), count_before + 1);
        assert_eq!(model.graph.node_count(), count_before + 1);
        assert_eq!(model.planets_by_biome(Biome::Lava).len(), 1);
    }

    #[test]
    fn test_insert_duplicate_system_is_noop() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let count_before = model.system_count();

        // Insert a system with the same address as an existing one
        let existing_id = *model.systems.keys().next().unwrap();
        let existing = model.systems.get(&existing_id).unwrap().clone();
        model.insert_system(existing);

        assert_eq!(model.system_count(), count_before);
    }

    #[test]
    fn test_system_not_found_returns_none() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert!(model.system(&SystemId(0xDEADBEEF)).is_none());
    }

    #[test]
    fn test_system_by_name_not_found() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert!(model.system_by_name("No Such System").is_none());
    }

    #[test]
    fn test_base_not_found() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert!(model.base("No Such Base").is_none());
    }

    #[test]
    fn test_from_save_address_to_id() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        // Every system should have an address_to_id entry
        for &sys_id in model.systems.keys() {
            assert!(model.address_to_id.contains_key(&sys_id.0));
        }
    }

    #[test]
    fn test_from_save_node_map() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        // Every system should have a node_map entry
        for &sys_id in model.systems.keys() {
            assert!(model.node_map.contains_key(&sys_id));
        }
    }

    #[test]
    fn test_insert_system_unnamed() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let name_count_before = model.name_index.len();

        let addr = GalacticAddress::new(600, 20, -400, 0xAAA, 0, 0);
        let system = System::new(addr, None, None, None, vec![]);
        model.insert_system(system);

        // No new name index entry for unnamed system
        assert_eq!(model.name_index.len(), name_count_before);
    }
}
