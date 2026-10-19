//! The GalaxyModel -- central in-memory representation of the player's galaxy.

use std::collections::HashMap;

use petgraph::Undirected;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use rstar::RTree;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::delta::{SaveDelta, SaveUpdate};
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

    /// Per-galaxy 3D spatial indexes of system positions.
    pub spatial: HashMap<u8, RTree<SystemPoint>>,

    /// Currently active galaxy for spatial queries.
    pub active_galaxy: u8,

    /// System data by ID.
    pub systems: HashMap<SystemId, System>,

    /// Planet data by (SystemId, planet_index).
    pub planets: HashMap<PlanetKey, Planet>,

    /// Base lookup by name (lowercase).
    pub bases: HashMap<String, PlayerBase>,

    /// Biome -> list of planets with that biome.
    pub biome_index: HashMap<Biome, Vec<PlanetKey>>,

    /// Galaxy-qualified system name -> SystemId, keyed by `(reality_index, lowercase_name)`.
    ///
    /// The public key shape includes the galaxy so equal system names in
    /// different galaxies remain independently addressable. This is a derived
    /// index: cache archives store system names in the raw system records and
    /// rebuild this map when constructing a model.
    pub name_index: HashMap<(u8, String), SystemId>,

    /// Galaxy-qualified system key -> the matching `SystemId`.
    pub address_to_id: HashMap<SystemId, SystemId>,

    /// SystemId -> petgraph NodeIndex.
    pub node_map: HashMap<SystemId, NodeIndex>,

    /// Current player state (position, currencies).
    pub player_state: Option<PlayerState>,
}

impl Default for GalaxyModel {
    fn default() -> Self {
        Self::new()
    }
}

impl GalaxyModel {
    /// Create an empty `GalaxyModel`.
    pub fn new() -> Self {
        Self {
            graph: StableGraph::default(),
            spatial: HashMap::new(),
            active_galaxy: 0,
            systems: HashMap::new(),
            planets: HashMap::new(),
            bases: HashMap::new(),
            biome_index: HashMap::new(),
            name_index: HashMap::new(),
            address_to_id: HashMap::new(),
            node_map: HashMap::new(),
            player_state: None,
        }
    }

    /// Build a `GalaxyModel` from a parsed save file.
    pub fn from_save(save: &SaveRoot) -> Self {
        let extracted = extract_systems(save);

        let mut graph: StableGraph<SystemId, f64, Undirected> = StableGraph::default();
        let mut galaxy_points: HashMap<u8, Vec<SystemPoint>> = HashMap::new();
        let mut systems = HashMap::with_capacity(extracted.len());
        let mut planets = HashMap::new();
        let mut biome_index: HashMap<Biome, Vec<PlanetKey>> = HashMap::new();
        let mut name_index: HashMap<(u8, String), SystemId> = HashMap::new();
        let mut address_to_id = HashMap::new();
        let mut node_map = HashMap::new();

        for (sys_id, system) in extracted {
            // Add to petgraph
            let node_idx = graph.add_node(sys_id);
            node_map.insert(sys_id, node_idx);

            // Add to per-galaxy spatial point collector
            let point = SystemPoint::from_address(&system.address);
            let galaxy = system.address.reality_index;
            galaxy_points.entry(galaxy).or_default().push(point);

            // Add to address lookup
            address_to_id.insert(sys_id, sys_id);

            // Add to name index
            if let Some(ref name) = system.name {
                name_index.insert((system.address.reality_index, name.to_lowercase()), sys_id);
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

        // Build per-galaxy R-trees from collected points
        let spatial: HashMap<u8, RTree<SystemPoint>> = galaxy_points
            .into_iter()
            .map(|(galaxy, points)| (galaxy, RTree::bulk_load(points)))
            .collect();

        // Extract player state and determine active galaxy
        let player_state = Some(save.to_core_player_state());
        let active_galaxy = save.active_player_state().universe_address.reality_index;

        // Extract bases
        let ps = save.active_player_state();
        let mut bases = HashMap::new();
        for base in &ps.persistent_player_bases {
            let core_base = base.to_core_base();
            if !core_base.name.is_empty() {
                bases.insert(core_base.name.to_lowercase(), core_base);
            }
        }

        let mut model = Self {
            graph,
            spatial,
            active_galaxy,
            systems,
            planets,
            bases,
            biome_index,
            name_index,
            address_to_id,
            node_map,
            player_state,
        };

        model.build_edges(crate::edges::EdgeStrategy::default());
        model
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

    /// Look up a system by name in the currently active galaxy.
    ///
    /// Names are matched case-insensitively. Use
    /// [`system_by_name_in_galaxy`](Self::system_by_name_in_galaxy) to select
    /// a specific galaxy without changing the active selection.
    pub fn system_by_name(&self, name: &str) -> Option<(&SystemId, &System)> {
        self.system_by_name_in_galaxy(name, self.active_galaxy)
    }

    /// Look up a system by name in `galaxy` without changing active selection.
    ///
    /// Names are matched case-insensitively. This lookup never falls back to a
    /// system with the same name in another galaxy.
    pub fn system_by_name_in_galaxy(&self, name: &str, galaxy: u8) -> Option<(&SystemId, &System)> {
        self.name_index
            .get(&(galaxy, name.to_lowercase()))
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

    /// Get the spatial index for the active galaxy.
    pub fn active_spatial(&self) -> Option<&RTree<SystemPoint>> {
        self.spatial.get(&self.active_galaxy)
    }

    /// Get the spatial index for a specific galaxy.
    pub fn spatial_for(&self, galaxy: u8) -> Option<&RTree<SystemPoint>> {
        self.spatial.get(&galaxy)
    }

    /// Switch the active galaxy.
    pub fn set_active_galaxy(&mut self, galaxy: u8) {
        self.active_galaxy = galaxy;
    }

    /// List galaxies that have at least one discovered system.
    pub fn discovered_galaxies(&self) -> Vec<u8> {
        let mut galaxies: Vec<u8> = self.spatial.keys().copied().collect();
        galaxies.sort();
        galaxies
    }

    /// Rebuild all per-galaxy R-trees from the systems HashMap.
    pub fn rebuild_spatial(&mut self) {
        let mut galaxy_points: HashMap<u8, Vec<SystemPoint>> = HashMap::new();
        for system in self.systems.values() {
            let point = SystemPoint::from_address(&system.address);
            let galaxy = system.address.reality_index;
            galaxy_points.entry(galaxy).or_default().push(point);
        }
        self.spatial = galaxy_points
            .into_iter()
            .map(|(galaxy, points)| (galaxy, RTree::bulk_load(points)))
            .collect();
    }

    /// Total number of points across all per-galaxy spatial indexes.
    pub fn spatial_size(&self) -> usize {
        self.spatial.values().map(|t| t.size()).sum()
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
        let galaxy = system.address.reality_index;
        self.spatial.entry(galaxy).or_default().insert(point);

        self.address_to_id.insert(sys_id, sys_id);

        if let Some(ref name) = system.name {
            self.name_index
                .insert((system.address.reality_index, name.to_lowercase()), sys_id);
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

    /// Ensure the player's current system exists in the model.
    ///
    /// If the player's system has no discovery record, this inserts a
    /// placeholder system at the player's address so that distance queries
    /// correctly return 0.0 for in-system results.
    pub fn ensure_player_system(&mut self) {
        if let Some(ref ps) = self.player_state {
            let player_addr = ps.current_address;
            let player_sys_id = SystemId::from_address(&player_addr);
            if !self.systems.contains_key(&player_sys_id) {
                let sentinel = System::new(player_addr, None, None, None, vec![]);
                self.insert_system(sentinel);
            }
        }
    }

    /// Insert a base into the model.
    pub fn insert_base(&mut self, base: PlayerBase) {
        if !base.name.is_empty() {
            self.bases.insert(base.name.to_lowercase(), base);
        }
    }

    /// Apply a delta from the file watcher to update the model incrementally.
    ///
    /// Inserts new systems and planets, updates player position, and
    /// adds/updates bases. Does NOT rebuild graph edges -- call
    /// `build_edges()` afterward if needed for routing.
    pub fn apply_delta(&mut self, delta: &SaveDelta) {
        // 1. Insert new systems (with their planets)
        for system in &delta.new_systems {
            self.insert_system(system.clone());
        }

        // 2. Insert new planets into existing systems
        for (sys_id, planet) in &delta.new_planets {
            let key = (*sys_id, planet.index);
            if !self.planets.contains_key(&key) {
                if let Some(biome) = planet.biome {
                    self.biome_index.entry(biome).or_default().push(key);
                }
                self.planets.insert(key, planet.clone());

                // Also add to the system's planet list
                if let Some(system) = self.systems.get_mut(sys_id)
                    && !system.planets.iter().any(|p| p.index == planet.index)
                {
                    system.planets.push(planet.clone());
                }
            }
        }

        // 3. Update physical player position. End the player-state borrow
        // before updating the model's selected-galaxy view.
        if let Some(moved) = &delta.player_moved {
            let previous_reality = self.player_state.as_mut().map(|player| {
                let previous_reality = player.current_address.reality_index;
                player.current_address = moved.to;
                player.current_reality = moved.to.reality_index;
                previous_reality
            });

            if previous_reality.is_some_and(|reality| reality != moved.to.reality_index) {
                self.active_galaxy = moved.to.reality_index;
            }
        }

        // 4. Insert new bases
        for base in &delta.new_bases {
            self.insert_base(base.clone());
        }

        // 5. Update modified bases (insert_base overwrites by name)
        for base in &delta.modified_bases {
            self.insert_base(base.clone());
        }
    }

    /// Apply a complete parsed save update while preserving imported atlas data.
    ///
    /// The event's player state is authoritative for physical location and
    /// selection decisions; its delta remains the additive source for atlas
    /// changes. A prior physical galaxy change selects the incoming galaxy.
    /// When the prior player state is absent, or the physical galaxy is
    /// unchanged, the caller's selected view is retained.
    pub fn apply_save_update(&mut self, update: &SaveUpdate) {
        let previous_reality = self
            .player_state
            .as_ref()
            .map(|player| player.current_address.reality_index);
        let selected_galaxy = self.active_galaxy;

        self.apply_delta(&update.delta);
        self.player_state = Some(update.player_state.clone());
        self.active_galaxy = match previous_reality {
            Some(reality) if reality != update.player_state.current_address.reality_index => {
                update.player_state.current_address.reality_index
            }
            _ => selected_galaxy,
        };
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
                            {"DD": {"UA": "0x002A32F38064", "DT": "SolarSystem", "VP": ["0xAAAA"]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
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
        // 3 systems: player's system discovery + 2 others
        assert_eq!(model.system_count(), 3);
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
    fn test_from_save_name_index_uses_qualified_key_shape() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let name_index: &HashMap<(u8, String), SystemId> = &model.name_index;

        // Save discovery extraction intentionally supplies no system names.
        assert!(name_index.is_empty());
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
        // 2 from discovery + 1 sentinel
        assert_eq!(model.spatial_size(), 3);
    }

    #[test]
    fn test_from_save_graph_nodes() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        // 2 from discovery + 1 sentinel
        assert_eq!(model.graph.node_count(), 3);
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
        assert_eq!(model.spatial_size(), count_before + 1);
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
        let missing = SystemId::from_address(&GalacticAddress::from_packed(0xDEADBEEF, 0));
        assert!(model.system(&missing).is_none());
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
            assert_eq!(model.address_to_id.get(&sys_id), Some(&sys_id));
        }
    }

    #[test]
    fn test_insert_same_coordinates_in_two_galaxies_preserves_qualified_indexes() {
        let addr0 = GalacticAddress::from_packed(0x150003AB8C07, 0);
        let addr9 = GalacticAddress::from_packed(0x150003AB8C07, 9);
        let id0 = SystemId::from_address(&addr0);
        let id9 = SystemId::from_address(&addr9);

        for reverse in [false, true] {
            let mut model = GalaxyModel::new();
            let systems = [
                System::new(
                    addr0,
                    Some("Zero".into()),
                    None,
                    None,
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Lush marker".into()),
                        None,
                    )],
                ),
                System::new(
                    addr9,
                    Some("Nine".into()),
                    None,
                    None,
                    vec![Planet::new(
                        1,
                        Some(Biome::Toxic),
                        None,
                        false,
                        Some("Toxic marker".into()),
                        None,
                    )],
                ),
            ];
            let order = if reverse { [1, 0] } else { [0, 1] };
            for index in order {
                model.insert_system(systems[index].clone());
            }

            assert_eq!(model.system_count(), 2);
            assert_eq!(model.planet_count(), 2);
            assert_eq!(model.graph.node_count(), 2);
            assert_eq!(model.node_map.len(), 2);
            assert_eq!(model.address_to_id.len(), 2);
            assert_eq!(model.spatial_size(), 2);
            assert_eq!(model.spatial_for(0).unwrap().size(), 1);
            assert_eq!(model.spatial_for(9).unwrap().size(), 1);
            assert_eq!(model.address_to_id.get(&id0), Some(&id0));
            assert_eq!(model.address_to_id.get(&id9), Some(&id9));
            assert!(model.node_map.contains_key(&id0));
            assert!(model.node_map.contains_key(&id9));
            assert_eq!(model.system(&id0).unwrap().name.as_deref(), Some("Zero"));
            assert_eq!(model.system(&id9).unwrap().name.as_deref(), Some("Nine"));
            assert_eq!(
                model.planets.get(&(id0, 1)).unwrap().name.as_deref(),
                Some("Lush marker")
            );
            assert_eq!(
                model.planets.get(&(id9, 1)).unwrap().name.as_deref(),
                Some("Toxic marker")
            );
            assert_eq!(model.biome_index.get(&Biome::Lush), Some(&vec![(id0, 1)]));
            assert_eq!(model.biome_index.get(&Biome::Toxic), Some(&vec![(id9, 1)]));

            model.set_active_galaxy(0);
            assert_eq!(model.nearest_systems(&addr0, 1)[0].0, id0);
            assert_eq!(
                model.nearest_planets(&addr0, 1, &Default::default())[0].0,
                (id0, 1)
            );
            model.set_active_galaxy(9);
            assert_eq!(model.nearest_systems(&addr9, 1)[0].0, id9);
            assert_eq!(
                model.nearest_planets(&addr9, 1, &Default::default())[0].0,
                (id9, 1)
            );
            model.set_active_galaxy(7);
            assert!(model.nearest_systems(&addr0, 1).is_empty());
            assert!(
                model
                    .nearest_planets(&addr0, 1, &Default::default())
                    .is_empty()
            );

            let before = (
                model.system_count(),
                model.planet_count(),
                model.graph.node_count(),
                model.node_map.len(),
                model.address_to_id.len(),
                model.spatial_size(),
            );
            model.insert_system(System::new(
                addr9,
                Some("Should not overwrite".into()),
                None,
                None,
                vec![Planet::new(1, Some(Biome::Frozen), None, false, None, None)],
            ));
            assert_eq!(
                (
                    model.system_count(),
                    model.planet_count(),
                    model.graph.node_count(),
                    model.node_map.len(),
                    model.address_to_id.len(),
                    model.spatial_size(),
                ),
                before
            );
            assert_eq!(model.system(&id9).unwrap().name.as_deref(), Some("Nine"));
        }
    }

    #[test]
    fn test_system_by_name_is_galaxy_qualified_in_both_insertion_orders() {
        let address0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let address9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let id0 = SystemId::from_address(&address0);
        let id9 = SystemId::from_address(&address9);
        let systems = [
            System::new(
                address0,
                Some("Shared".into()),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Zero".into()),
                    None,
                )],
            ),
            System::new(
                address9,
                Some("Shared".into()),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Toxic),
                    None,
                    true,
                    Some("Nine".into()),
                    None,
                )],
            ),
        ];

        for order in [[0, 1], [1, 0]] {
            let mut model = GalaxyModel::new();
            for index in order {
                model.insert_system(systems[index].clone());
            }

            for (galaxy, expected_id, expected_planet) in [(0, id0, "Zero"), (9, id9, "Nine")] {
                model.set_active_galaxy(galaxy);
                let (id, system) = model.system_by_name("sHaReD").unwrap();
                assert_eq!(*id, expected_id);
                assert_eq!(system.planets[0].name.as_deref(), Some(expected_planet));

                let (explicit_id, explicit_system) =
                    model.system_by_name_in_galaxy("SHARED", galaxy).unwrap();
                assert_eq!(*explicit_id, expected_id);
                assert_eq!(
                    explicit_system.planets[0].name.as_deref(),
                    Some(expected_planet)
                );
            }
            assert!(model.system_by_name_in_galaxy("Shared", 7).is_none());

            let before = model.system_count();
            model.insert_system(System::new(
                address9,
                Some("Changed".into()),
                None,
                None,
                vec![],
            ));
            assert_eq!(model.system_count(), before);
            assert_eq!(
                model
                    .system_by_name_in_galaxy("Shared", 9)
                    .unwrap()
                    .1
                    .planets[0]
                    .name
                    .as_deref(),
                Some("Nine")
            );
        }
    }

    #[test]
    fn test_same_galaxy_duplicate_system_name_keeps_last_inserted_system() {
        let mut model = GalaxyModel::new();
        let first = System::new(
            GalacticAddress::new(0, 0, 0, 1, 0, 0),
            Some("Shared".into()),
            None,
            None,
            vec![],
        );
        let last = System::new(
            GalacticAddress::new(1, 0, 0, 2, 0, 0),
            Some("Shared".into()),
            None,
            None,
            vec![],
        );
        let last_id = SystemId::from_address(&last.address);

        model.insert_system(first);
        model.insert_system(last);

        assert_eq!(*model.system_by_name("SHARED").unwrap().0, last_id);
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
    fn test_apply_delta_empty_is_noop() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let sys_count = model.system_count();
        let planet_count = model.planet_count();
        let base_count = model.base_count();

        model.apply_delta(&nms_core::delta::SaveDelta::empty());

        assert_eq!(model.system_count(), sys_count);
        assert_eq!(model.planet_count(), planet_count);
        assert_eq!(model.base_count(), base_count);
    }

    #[test]
    fn test_apply_delta_new_system() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let count_before = model.system_count();

        let delta = nms_core::delta::SaveDelta {
            new_systems: vec![System::new(
                GalacticAddress::new(500, 10, -300, 0x999, 0, 0),
                Some("Delta System".into()),
                None,
                None,
                vec![Planet::new(0, Some(Biome::Lava), None, false, None, None)],
            )],
            ..nms_core::delta::SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.system_count(), count_before + 1);
        assert!(model.system_by_name("Delta System").is_some());
    }

    #[test]
    fn test_apply_delta_player_moved() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let old_pos = *model.player_position().unwrap();
        let new_addr = GalacticAddress::new(999, 0, 0, 1, 0, 0);

        let delta = nms_core::delta::SaveDelta {
            player_moved: Some(nms_core::delta::PlayerMoved {
                from: old_pos,
                to: new_addr,
            }),
            ..nms_core::delta::SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.player_position().unwrap().voxel_x(), 999);
    }

    #[test]
    fn test_apply_delta_cross_reality_updates_physical_state_and_selection() {
        use nms_core::player::PlayerState;

        let from = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let to = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let previous = GalacticAddress::new(8, 1, 2, 3, 0, 4);
        let freighter = GalacticAddress::new(-7, 3, 5, 6, 0, 8);
        let mut model = GalaxyModel::new();
        model.player_state = Some(PlayerState::new(
            from,
            0,
            Some(previous),
            Some(freighter),
            7,
            8,
            9,
        ));
        model.set_active_galaxy(0);
        let delta = nms_core::delta::SaveDelta {
            player_moved: Some(nms_core::delta::PlayerMoved { from, to }),
            ..nms_core::delta::SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(
            model.player_state,
            Some(PlayerState::new(
                to,
                9,
                Some(previous),
                Some(freighter),
                7,
                8,
                9,
            ))
        );
        assert_eq!(model.active_galaxy, 9);

        model.set_active_galaxy(7);
        model.apply_delta(&delta);
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(
            model.player_state,
            Some(PlayerState::new(
                to,
                9,
                Some(previous),
                Some(freighter),
                7,
                8,
                9,
            ))
        );
        assert!(model.systems.is_empty());
        assert!(model.planets.is_empty());
        model.apply_delta(&nms_core::delta::SaveDelta::empty());
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(
            model.player_state,
            Some(PlayerState::new(
                to,
                9,
                Some(previous),
                Some(freighter),
                7,
                8,
                9,
            ))
        );

        let reverse_from = to;
        let reverse_to = from;
        let mut reverse = GalaxyModel::new();
        reverse.player_state = Some(PlayerState::new(
            reverse_from,
            9,
            Some(previous),
            Some(freighter),
            7,
            8,
            9,
        ));
        reverse.set_active_galaxy(9);
        reverse.apply_delta(&nms_core::delta::SaveDelta {
            player_moved: Some(nms_core::delta::PlayerMoved {
                from: reverse_from,
                to: reverse_to,
            }),
            ..nms_core::delta::SaveDelta::empty()
        });
        assert_eq!(reverse.active_galaxy, 0);
        assert_eq!(
            reverse.player_state,
            Some(PlayerState::new(
                reverse_to,
                0,
                Some(previous),
                Some(freighter),
                7,
                8,
                9,
            ))
        );
    }

    #[test]
    fn test_apply_delta_same_reality_keeps_selection_and_absent_player() {
        use nms_core::player::PlayerState;
        use nms_core::system::SystemId;

        let from = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let to = GalacticAddress::new(1, 0, 0, 2, 0, 9);
        let mut model = GalaxyModel::new();
        model.player_state = Some(PlayerState::new(from, 9, None, None, 7, 8, 9));
        model.set_active_galaxy(7);
        model.apply_delta(&nms_core::delta::SaveDelta {
            player_moved: Some(nms_core::delta::PlayerMoved { from, to }),
            ..nms_core::delta::SaveDelta::empty()
        });
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(model.player_state.as_ref().unwrap().current_address, to);
        assert_eq!(model.player_state.as_ref().unwrap().current_reality, 9);

        let destination = GalacticAddress::new(4, 0, 0, 3, 0, 9);
        let destination_id = SystemId::from_address(&destination);
        let extra_planet = Planet::new(
            1,
            Some(Biome::Lush),
            None,
            false,
            Some("Leaf-1".into()),
            None,
        );
        let mut absent = GalaxyModel::new();
        absent.set_active_galaxy(7);
        absent.apply_delta(&nms_core::delta::SaveDelta {
            new_systems: vec![System::new(
                destination,
                Some("Destination-9".into()),
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(Biome::Toxic),
                    None,
                    false,
                    Some("Leaf-9".into()),
                    None,
                )],
            )],
            new_planets: vec![(destination_id, extra_planet)],
            player_moved: Some(nms_core::delta::PlayerMoved {
                from,
                to: destination,
            }),
            ..nms_core::delta::SaveDelta::empty()
        });
        assert!(absent.player_state.is_none());
        assert_eq!(absent.active_galaxy, 7);
        assert_eq!(
            absent.system(&destination_id).unwrap().name.as_deref(),
            Some("Destination-9")
        );
        assert_eq!(
            absent.planets.get(&(destination_id, 0)).unwrap().biome,
            Some(Biome::Toxic)
        );
        assert_eq!(
            absent
                .planets
                .get(&(destination_id, 1))
                .unwrap()
                .name
                .as_deref(),
            Some("Leaf-1")
        );
    }

    #[test]
    fn test_apply_save_update_uses_full_state_and_preserves_imported_atlas() {
        let address_zero = GalacticAddress::new(10, 2, 30, 4, 0, 0);
        let address_nine = GalacticAddress::new(20, 3, 40, 5, 0, 9);
        let system_nine = System::new(
            address_nine,
            Some("Imported Nine".into()),
            None,
            None,
            vec![],
        );
        let system_nine_id = SystemId::from_address(&address_nine);
        let imported_planet = Planet::new(
            1,
            Some(Biome::Lush),
            None,
            false,
            Some("Imported leaf".into()),
            None,
        );
        let imported_base = PlayerBase::new(
            "Imported Base".into(),
            nms_core::player::BaseType::HomePlanetBase,
            address_nine,
            [1.0, 2.0, 3.0],
            Some("import-owner".into()),
        );
        let mut model = GalaxyModel::new();
        model.insert_system(system_nine);
        model
            .planets
            .insert((system_nine_id, 1), imported_planet.clone());
        model.insert_base(imported_base.clone());
        model.player_state = Some(PlayerState::new(address_zero, 0, None, None, 1, 2, 3));
        model.active_galaxy = 7;

        let incoming = PlayerState::new(
            address_zero,
            0,
            Some(GalacticAddress::new(90, 1, 2, 3, 0, 9)),
            Some(address_nine),
            70,
            80,
            90,
        );
        let update = SaveUpdate {
            delta: SaveDelta {
                player_moved: Some(nms_core::delta::PlayerMoved {
                    from: address_zero,
                    to: address_nine,
                }),
                ..SaveDelta::empty()
            },
            player_state: incoming.clone(),
            save_version: 4721,
        };
        model.apply_save_update(&update);

        assert_eq!(model.player_state, Some(incoming));
        assert_eq!(model.active_galaxy, 7);
        assert_eq!(
            model.system(&system_nine_id).unwrap().name.as_deref(),
            Some("Imported Nine")
        );
        assert_eq!(
            model.planets.get(&(system_nine_id, 1)),
            Some(&imported_planet)
        );
        assert_eq!(model.bases.get("imported base"), Some(&imported_base));
    }

    #[test]
    fn test_apply_save_update_selection_tracks_only_known_physical_transition() {
        let address_zero = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let address_nine = GalacticAddress::new(0, 0, 0, 2, 0, 9);
        let state_nine = PlayerState::new(address_nine, 9, None, None, 70, 80, 90);
        let contradictory = SaveDelta {
            player_moved: Some(nms_core::delta::PlayerMoved {
                from: address_zero,
                to: address_zero,
            }),
            ..SaveDelta::empty()
        };

        let mut model = GalaxyModel::new();
        model.active_galaxy = 7;
        model.player_state = Some(PlayerState::new(address_zero, 0, None, None, 1, 2, 3));
        model.apply_save_update(&SaveUpdate {
            delta: contradictory.clone(),
            player_state: state_nine.clone(),
            save_version: 4721,
        });
        assert_eq!(model.player_state, Some(state_nine.clone()));
        assert_eq!(model.active_galaxy, 9);

        model.active_galaxy = 7;
        let state_nine_same_reality = PlayerState::new(
            GalacticAddress::new(1, 0, 0, 2, 0, 9),
            9,
            Some(address_zero),
            None,
            71,
            81,
            91,
        );
        model.apply_save_update(&SaveUpdate {
            delta: SaveDelta::empty(),
            player_state: state_nine_same_reality.clone(),
            save_version: 4722,
        });
        assert_eq!(model.player_state, Some(state_nine_same_reality));
        assert_eq!(model.active_galaxy, 7);

        let mut absent = GalaxyModel::new();
        absent.active_galaxy = 7;
        absent.apply_save_update(&SaveUpdate {
            delta: SaveDelta::empty(),
            player_state: state_nine,
            save_version: 4723,
        });
        assert_eq!(
            absent.player_state,
            Some(PlayerState::new(address_nine, 9, None, None, 70, 80, 90))
        );
        assert_eq!(absent.active_galaxy, 7);
    }

    #[test]
    fn test_apply_delta_new_base() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let base_count_before = model.base_count();

        let addr = GalacticAddress::new(100, 50, -200, 42, 0, 0);
        let base = nms_core::player::PlayerBase::new(
            "Delta Base".into(),
            nms_core::player::BaseType::HomePlanetBase,
            addr,
            [0.0, 0.0, 0.0],
            None,
        );

        let delta = nms_core::delta::SaveDelta {
            new_bases: vec![base],
            ..nms_core::delta::SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.base_count(), base_count_before + 1);
        assert!(model.base("Delta Base").is_some());
    }

    #[test]
    fn test_apply_delta_new_planet_to_existing_system() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let planet_count_before = model.planet_count();

        let sys_id = *model.systems.keys().next().unwrap();
        let planet = Planet::new(7, Some(Biome::Frozen), None, false, None, None);

        let delta = nms_core::delta::SaveDelta {
            new_planets: vec![(sys_id, planet)],
            ..nms_core::delta::SaveDelta::empty()
        };

        model.apply_delta(&delta);
        assert_eq!(model.planet_count(), planet_count_before + 1);
        assert_eq!(model.planets_by_biome(Biome::Frozen).len(), 1);
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

    // -- Multi-galaxy spatial tests (milestone 7.4) --

    #[test]
    fn test_active_galaxy_defaults_to_zero() {
        let model = GalaxyModel::new();
        assert_eq!(model.active_galaxy, 0);
    }

    #[test]
    fn test_set_active_galaxy() {
        let mut model = GalaxyModel::new();
        model.set_active_galaxy(9);
        assert_eq!(model.active_galaxy, 9);
    }

    #[test]
    fn test_active_spatial_empty_model_returns_none() {
        let model = GalaxyModel::new();
        assert!(model.active_spatial().is_none());
    }

    #[test]
    fn test_spatial_for_unknown_galaxy_returns_none() {
        let model = GalaxyModel::new();
        assert!(model.spatial_for(42).is_none());
    }

    #[test]
    fn test_discovered_galaxies_empty() {
        let model = GalaxyModel::new();
        assert!(model.discovered_galaxies().is_empty());
    }

    #[test]
    fn test_insert_system_separate_galaxies() {
        let mut model = GalaxyModel::new();

        // Euclid (galaxy 0)
        let addr0 = GalacticAddress::new(10, 0, 0, 0x100, 0, 0);
        let sys0 = System::new(addr0, Some("Euclid Sys".into()), None, None, vec![]);
        model.insert_system(sys0);

        // Eissentam (galaxy 9)
        let addr9 = GalacticAddress::new(20, 0, 0, 0x200, 0, 9);
        let sys9 = System::new(addr9, Some("Eissentam Sys".into()), None, None, vec![]);
        model.insert_system(sys9);

        assert_eq!(model.system_count(), 2);
        assert_eq!(model.spatial_size(), 2);

        // Galaxy 0 has 1 point
        let tree0 = model.spatial_for(0).unwrap();
        assert_eq!(tree0.size(), 1);

        // Galaxy 9 has 1 point
        let tree9 = model.spatial_for(9).unwrap();
        assert_eq!(tree9.size(), 1);

        // No galaxy 5
        assert!(model.spatial_for(5).is_none());
    }

    #[test]
    fn test_discovered_galaxies_sorted() {
        let mut model = GalaxyModel::new();

        let addr9 = GalacticAddress::new(0, 0, 0, 0x100, 0, 9);
        model.insert_system(System::new(addr9, None, None, None, vec![]));

        let addr0 = GalacticAddress::new(0, 0, 0, 0x200, 0, 0);
        model.insert_system(System::new(addr0, None, None, None, vec![]));

        let addr2 = GalacticAddress::new(0, 0, 0, 0x300, 0, 2);
        model.insert_system(System::new(addr2, None, None, None, vec![]));

        let galaxies = model.discovered_galaxies();
        assert_eq!(galaxies, vec![0, 2, 9]);
    }

    #[test]
    fn test_rebuild_spatial() {
        let mut model = GalaxyModel::new();

        let addr0 = GalacticAddress::new(10, 0, 0, 0x100, 0, 0);
        model.insert_system(System::new(addr0, None, None, None, vec![]));

        let addr9 = GalacticAddress::new(20, 0, 0, 0x200, 0, 9);
        model.insert_system(System::new(addr9, None, None, None, vec![]));

        // Clear spatial and rebuild
        model.spatial.clear();
        assert_eq!(model.spatial_size(), 0);

        model.rebuild_spatial();
        assert_eq!(model.spatial_size(), 2);
        assert_eq!(model.spatial_for(0).unwrap().size(), 1);
        assert_eq!(model.spatial_for(9).unwrap().size(), 1);
    }

    #[test]
    fn test_active_spatial_returns_correct_tree() {
        let mut model = GalaxyModel::new();

        let addr0 = GalacticAddress::new(10, 0, 0, 0x100, 0, 0);
        model.insert_system(System::new(addr0, Some("E1".into()), None, None, vec![]));

        let addr9 = GalacticAddress::new(20, 0, 0, 0x200, 0, 9);
        model.insert_system(System::new(addr9, Some("E2".into()), None, None, vec![]));

        // Default active galaxy is 0
        assert_eq!(model.active_spatial().unwrap().size(), 1);

        model.set_active_galaxy(9);
        assert_eq!(model.active_spatial().unwrap().size(), 1);

        model.set_active_galaxy(42);
        assert!(model.active_spatial().is_none());
    }

    #[test]
    fn test_player_system_distance_zero() {
        // Player is at (100, 50, -200, SSI=42); multi_system_save has a
        // matching SolarSystem discovery record at UA 0x002A32F38064.
        let json = std::fs::read_to_string("../../data/test/multi_system_save.json").unwrap();
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let model = GalaxyModel::from_save(&save);

        let player_pos = model.player_position().unwrap();
        assert_eq!(player_pos.voxel_x(), 100);
        assert_eq!(player_pos.voxel_y(), 50);
        assert_eq!(player_pos.voxel_z(), -200);
        assert_eq!(player_pos.solar_system_index(), 42);

        // The nearest system should be the player's own system at distance 0.
        let nearest = model.nearest_systems(player_pos, 1);
        assert!(
            !nearest.is_empty(),
            "Expected at least one system near player"
        );
        let (_, distance) = &nearest[0];
        assert_eq!(
            *distance, 0.0,
            "Player's own system should be at distance 0.0, got {distance}"
        );
    }

    #[test]
    fn test_ensure_player_system_inserts_sentinel() {
        // Build a model where the player's system has no discovery record.
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {"GameMode": 1, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 999, "VoxelY": 10, "VoxelZ": -500, "SolarSystemIndex": 77, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        let mut model = GalaxyModel::from_save(&save);

        let player_pos = *model.player_position().unwrap();
        let player_sys_id = SystemId::from_address(&player_pos);
        assert!(model.system(&player_sys_id).is_none());

        model.ensure_player_system();
        assert!(model.system(&player_sys_id).is_some());

        let nearest = model.nearest_systems(&player_pos, 1);
        assert!(!nearest.is_empty());
        assert_eq!(nearest[0].1, 0.0);
    }
}
