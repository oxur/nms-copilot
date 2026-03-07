# Milestone 2.1 -- Galaxy Model (nms-graph)

The core in-memory galactic model: a struct holding a petgraph, R-tree spatial index, and HashMap lookup tables, constructed from a parsed save file.

## Crate: `nms-graph`

Path: `crates/nms-graph/`

### Dependencies to add to `crates/nms-graph/Cargo.toml`

```toml
[dependencies]
nms-core = { workspace = true }
nms-save = { workspace = true }
petgraph = "0.7"
rstar = "0.12"
thiserror = "2"
serde_json = "1"

[dev-dependencies]
serde_json = "1"
```

Also add to workspace root `Cargo.toml` `[workspace.dependencies]`:

```toml
petgraph = "0.7"
rstar = "0.12"
```

---

## Types

### File: `crates/nms-graph/src/lib.rs`

Replace the doc-only stub with the module structure:

```rust
//! In-memory galactic model for NMS Copilot.
//!
//! Builds and maintains a spatial graph of all known star systems using
//! three parallel data structures:
//!
//! - **petgraph** -- topology layer for pathfinding and TSP routing
//! - **R-tree** -- geometric layer for nearest-neighbor and radius queries
//! - **HashMaps** -- associative layer for fast lookup by name, biome, etc.

pub mod error;
pub mod model;
pub mod spatial;
pub mod extract;

pub use error::GraphError;
pub use model::GalaxyModel;
pub use spatial::SystemPoint;
```

### File: `crates/nms-graph/src/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GraphError {
    #[error("system not found: {0}")]
    SystemNotFound(String),

    #[error("base not found: {0}")]
    BaseNotFound(String),

    #[error("no player position available")]
    NoPlayerPosition,
}
```

### File: `crates/nms-graph/src/spatial.rs`

```rust
use rstar::{PointDistance, RTreeObject, AABB};

/// Unique identifier for a star system.
///
/// The value is the packed 48-bit galactic address with planet index zeroed out
/// (i.e., `packed & 0xFFF_FFFF_FFFF` with planet nibble cleared). Two systems
/// at the same voxel coordinates but different SSI values get different IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub u64);

impl SystemId {
    /// Create from a GalacticAddress by zeroing the planet index bits.
    pub fn from_address(addr: &nms_core::address::GalacticAddress) -> Self {
        // Clear the top 4 bits (planet index) of the 48-bit packed value
        let packed = addr.packed() & 0x0FFF_FFFF_FFFF;
        SystemId(packed)
    }
}

/// A system's position in 3D voxel space, stored in the R-tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemPoint {
    pub id: SystemId,
    pub point: [f64; 3],
}

impl SystemPoint {
    pub fn new(id: SystemId, x: f64, y: f64, z: f64) -> Self {
        Self {
            id,
            point: [x, y, z],
        }
    }

    /// Create from a GalacticAddress.
    pub fn from_address(addr: &nms_core::address::GalacticAddress) -> Self {
        let id = SystemId::from_address(addr);
        Self::new(
            id,
            addr.voxel_x() as f64,
            addr.voxel_y() as f64,
            addr.voxel_z() as f64,
        )
    }
}

impl RTreeObject for SystemPoint {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for SystemPoint {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}
```

### File: `crates/nms-graph/src/extract.rs`

Extract structured data from raw discovery records:

```rust
//! Extract System/Planet data from raw save discovery records.

use std::collections::HashMap;

use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::system::{Planet, System};
use nms_save::model::{RawDiscoveryRecord, SaveRoot};

use crate::spatial::SystemId;

/// Temporary accumulator for building a System from multiple discovery records.
#[derive(Debug)]
struct SystemBuilder {
    address: GalacticAddress,
    name: Option<String>,
    discoverer: Option<String>,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    planets: Vec<Planet>,
}

/// Extract biome and infested flag from a discovery record's VP array.
///
/// VP array format (for Planet discovery type):
///   VP[0]: seed hash (hex string or integer)
///   VP[1]: biome/flags packed integer
///     - bits 0..15 (mask 0xFFFF): biome type index (GcBiomeType enum)
///     - bit 16 (mask 0x10000): infested flag
///
/// Encoding confirmed via workbench/old/nms-save-format.md.
///
/// Returns `(biome, infested)`. Returns `(None, false)` if VP is empty or
/// the format is unrecognized.
pub fn extract_biome_from_vp(vp: &[serde_json::Value]) -> (Option<Biome>, bool) {
    if vp.len() < 2 {
        return (None, false);
    }

    // VP[1] can be a hex string "0x..." or integer
    let flags = match &vp[1] {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => {
            let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
            u64::from_str_radix(hex, 16).ok()
        }
        _ => None,
    };

    let Some(flags) = flags else {
        return (None, false);
    };

    let infested = (flags >> 16) & 1 == 1;

    // Biome type is in the lower 16 bits (mask 0xFFFF).
    // Mapping matches GcBiomeType::BiomeEnum ordering from game data.
    // Confirmed via workbench/old/nms-save-format.md.
    let biome_index = (flags & 0xFFFF) as u16;
    let biome = match biome_index {
        0 => Some(Biome::Lush),
        1 => Some(Biome::Toxic),
        2 => Some(Biome::Scorched),
        3 => Some(Biome::Radioactive),
        4 => Some(Biome::Frozen),
        5 => Some(Biome::Barren),
        6 => Some(Biome::Dead),
        7 => Some(Biome::Weird),
        8 => Some(Biome::Red),
        9 => Some(Biome::Green),
        10 => Some(Biome::Blue),
        11 => None, // "Test" biome in game data -- skip
        12 => Some(Biome::Swamp),
        13 => Some(Biome::Lava),
        14 => Some(Biome::Waterworld),
        15 => Some(Biome::GasGiant),
        _ => None,
    };

    (biome, infested)
}

/// Extract seed hash from VP[0].
pub fn extract_seed_from_vp(vp: &[serde_json::Value]) -> Option<u64> {
    if vp.is_empty() {
        return None;
    }
    match &vp[0] {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => {
            let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
            u64::from_str_radix(hex, 16).ok()
        }
        _ => None,
    }
}

/// Build systems and planets from a parsed save file's discovery records.
///
/// Groups discovery records by system address, extracts planet biome data,
/// and returns a map of SystemId -> System.
pub fn extract_systems(save: &SaveRoot) -> HashMap<SystemId, System> {
    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    let mut builders: HashMap<SystemId, SystemBuilder> = HashMap::new();

    // First pass: collect SolarSystem discoveries (for system names/discoverers)
    for rec in records {
        if rec.dd.dt != "SolarSystem" {
            continue;
        }
        let addr = GalacticAddress::from_packed(rec.dd.ua.0, 0);
        let sys_id = SystemId::from_address(&addr);

        let timestamp = if rec.ows.ts > 0 {
            chrono::DateTime::from_timestamp(rec.ows.ts as i64, 0)
        } else {
            None
        };

        let discoverer = if rec.ows.usn.is_empty() {
            None
        } else {
            Some(rec.ows.usn.clone())
        };

        builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None, // System names aren't in discovery records
            discoverer,
            timestamp,
            planets: Vec::new(),
        });
    }

    // Second pass: collect Planet discoveries and attach to systems
    for rec in records {
        if rec.dd.dt != "Planet" {
            continue;
        }
        let addr = GalacticAddress::from_packed(rec.dd.ua.0, 0);
        let sys_id = SystemId::from_address(&addr);
        let planet_index = addr.planet_index();

        let (biome, infested) = extract_biome_from_vp(&rec.dd.vp);
        let seed_hash = extract_seed_from_vp(&rec.dd.vp);

        let planet = Planet::new(
            planet_index,
            biome,
            None, // BiomeSubType not extractable from VP
            infested,
            None, // Planet names aren't in discovery records
            seed_hash,
        );

        let builder = builders.entry(sys_id).or_insert_with(|| SystemBuilder {
            address: addr,
            name: None,
            discoverer: None,
            timestamp: None,
            planets: Vec::new(),
        });

        // Avoid duplicate planet indices
        if !builder.planets.iter().any(|p| p.index == planet_index) {
            builder.planets.push(planet);
        }
    }

    // Convert builders to Systems
    builders
        .into_iter()
        .map(|(id, b)| {
            let system = System::new(b.address, b.name, b.discoverer, b.timestamp, b.planets);
            (id, system)
        })
        .collect()
}
```

### File: `crates/nms-graph/src/model.rs`

```rust
//! The GalaxyModel -- central in-memory representation of the player's galaxy.

use std::collections::HashMap;

use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::Undirected;
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
    /// Build a GalaxyModel from a parsed save file.
    pub fn from_save(save: &SaveRoot) -> Self {
        let extracted = extract_systems(save);

        let mut graph = StableGraph::new_undirected();
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
    pub fn get_system(&self, id: &SystemId) -> Option<&System> {
        self.systems.get(id)
    }

    /// Look up a system by name (case-insensitive).
    pub fn get_system_by_name(&self, name: &str) -> Option<(&SystemId, &System)> {
        self.name_index
            .get(&name.to_lowercase())
            .and_then(|id| self.systems.get(id).map(|s| (id, s)))
    }

    /// Look up a base by name (case-insensitive).
    pub fn get_base(&self, name: &str) -> Option<&PlayerBase> {
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
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.planets.get(k))
                    .collect()
            })
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
```

---

## Tests

### File: `crates/nms-graph/src/extract.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_biome_from_vp_empty() {
        assert_eq!(extract_biome_from_vp(&[]), (None, false));
    }

    #[test]
    fn extract_biome_from_vp_single_element() {
        let vp = vec![serde_json::json!("0xABCD")];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn extract_biome_lush_not_infested() {
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(0)];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Lush), false));
    }

    #[test]
    fn extract_biome_toxic_infested() {
        // bit 16 set = infested, low byte = 1 = Toxic
        let flags = (1u64 << 16) | 1;
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(flags)];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Toxic), true));
    }

    #[test]
    fn extract_biome_from_hex_string() {
        // 0x00010005 = bit 16 set (infested) + 5 (Barren)
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!("0x10005")];
        assert_eq!(extract_biome_from_vp(&vp), (Some(Biome::Barren), true));
    }

    #[test]
    fn extract_biome_unknown_index() {
        let vp = vec![serde_json::json!("0xABCD"), serde_json::json!(255)];
        assert_eq!(extract_biome_from_vp(&vp), (None, false));
    }

    #[test]
    fn extract_seed_from_vp_hex() {
        let vp = vec![serde_json::json!("0xD6911E7B1D31085E")];
        assert_eq!(extract_seed_from_vp(&vp), Some(0xD6911E7B1D31085E));
    }

    #[test]
    fn extract_seed_from_vp_integer() {
        let vp = vec![serde_json::json!(12345)];
        assert_eq!(extract_seed_from_vp(&vp), Some(12345));
    }

    #[test]
    fn extract_seed_from_vp_empty() {
        assert_eq!(extract_seed_from_vp(&[]), None);
    }
}
```

### File: `crates/nms-graph/src/spatial.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;

    #[test]
    fn system_id_from_address_zeroes_planet() {
        let addr1 = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        let addr2 = GalacticAddress::new(100, 50, 200, 0x123, 5, 0);
        assert_eq!(SystemId::from_address(&addr1), SystemId::from_address(&addr2));
    }

    #[test]
    fn system_id_different_ssi() {
        let addr1 = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        let addr2 = GalacticAddress::new(100, 50, 200, 0x456, 0, 0);
        assert_ne!(SystemId::from_address(&addr1), SystemId::from_address(&addr2));
    }

    #[test]
    fn system_point_from_address() {
        let addr = GalacticAddress::new(100, -50, 200, 0x123, 3, 0);
        let point = SystemPoint::from_address(&addr);
        assert_eq!(point.point, [100.0, -50.0, 200.0]);
    }

    #[test]
    fn system_point_distance_squared() {
        use rstar::PointDistance;
        let p = SystemPoint::new(SystemId(0), 0.0, 0.0, 0.0);
        let target = [3.0, 4.0, 0.0];
        assert!((p.distance_2(&target) - 25.0).abs() < 1e-10);
    }

    #[test]
    fn rtree_nearest_neighbor() {
        use rstar::RTree;
        let points = vec![
            SystemPoint::new(SystemId(1), 0.0, 0.0, 0.0),
            SystemPoint::new(SystemId(2), 10.0, 0.0, 0.0),
            SystemPoint::new(SystemId(3), 100.0, 0.0, 0.0),
        ];
        let tree = RTree::bulk_load(points);
        let nearest = tree.nearest_neighbor(&[1.0, 0.0, 0.0]).unwrap();
        assert_eq!(nearest.id, SystemId(1));
    }
}
```

### File: `crates/nms-graph/src/model.rs` (inline tests at bottom)

```rust
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
    fn from_save_basic_counts() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.system_count(), 2);
        assert_eq!(model.planet_count(), 1);
        assert_eq!(model.base_count(), 1);
    }

    #[test]
    fn from_save_base_lookup() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let base = model.get_base("Home Base").unwrap();
        assert_eq!(base.name, "Home Base");
    }

    #[test]
    fn from_save_base_lookup_case_insensitive() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert!(model.get_base("home base").is_some());
        assert!(model.get_base("HOME BASE").is_some());
    }

    #[test]
    fn from_save_player_position() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        let pos = model.player_position().unwrap();
        assert_eq!(pos.voxel_x(), 100);
        assert_eq!(pos.voxel_y(), 50);
        assert_eq!(pos.voxel_z(), -200);
    }

    #[test]
    fn from_save_spatial_index_populated() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.spatial.size(), 2);
    }

    #[test]
    fn from_save_graph_nodes() {
        let save = minimal_save();
        let model = GalaxyModel::from_save(&save);
        assert_eq!(model.graph.node_count(), 2);
    }

    #[test]
    fn insert_system_adds_to_all_indexes() {
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
        assert!(model.get_system_by_name("New System").is_some());
        assert_eq!(model.spatial.size(), count_before + 1);
        assert_eq!(model.graph.node_count(), count_before + 1);
        assert_eq!(model.planets_by_biome(Biome::Lava).len(), 1);
    }

    #[test]
    fn insert_duplicate_system_is_noop() {
        let save = minimal_save();
        let mut model = GalaxyModel::from_save(&save);
        let count_before = model.system_count();

        // Insert a system with the same address as an existing one
        let existing_id = *model.systems.keys().next().unwrap();
        let existing = model.systems.get(&existing_id).unwrap().clone();
        model.insert_system(existing);

        assert_eq!(model.system_count(), count_before);
    }
}
```

---

## Implementation Notes

1. **SystemId zeroes planet bits** -- two addresses in the same system but different planet indices must map to the same SystemId. The top 4 bits of the 48-bit packed value are the planet index; mask them out with `& 0x0FFF_FFFF_FFFF`.

2. **rstar bulk_load** -- use `RTree::bulk_load()` during construction for O(n log n) performance. For incremental inserts (from the file watcher), use `RTree::insert()` which is O(log n).

3. **VP array biome encoding is confirmed** -- VP[1] lower 16 bits = `GcBiomeType` enum index, bit 16 = infested flag. Source: `workbench/old/nms-save-format.md`. Index 11 is "Test" (skip it). Indices 12-15 are Swamp, Lava, Waterworld, GasGiant. Verify against a real save with known biome planets if results look wrong.

4. **Discovery records don't carry names** -- system/planet names are not stored in the discovery store's Record array. They're in a separate mechanism. Initially, all names will be `None`. Base names come from `PersistentPlayerBases`.

5. **StableGraph vs Graph** -- use `petgraph::stable_graph::StableGraph` rather than `Graph` because node removal (if ever needed) doesn't invalidate existing NodeIndex values. This is important for the node_map HashMap.

6. **Base names are lowercased for lookup** but the original case is preserved in the `PlayerBase.name` field.

7. **Parallel data structures** -- every mutation (insert_system, etc.) must update ALL indexes. Missing an update creates inconsistency. The tests verify this by checking all counts after insertion.
