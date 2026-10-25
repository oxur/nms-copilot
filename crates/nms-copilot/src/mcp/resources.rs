//! MCP resources for NMS Copilot.
//!
//! [`NmsResources`] lists three current-model JSON resources in a stable order.
//! Reads acquire the shared model lock when the returned future is polled;
//! constructing or listing this registry does not start a server, watcher, or
//! notification producer. Registering it alone does not send notifications.
//! See the [resource examples](crate::mcp::resources) for dispatch, typed
//! errors, wrappers, and read-time model access.
//!
//! # Examples
//!
//! This example checks registry order, typed absence, wrapper metadata, and
//! that a future reads the current model when polled rather than when created.
//!
//! ```
//! use std::sync::Arc;
//! use fabryk_mcp::ResourceRegistry;
//! use fabryk_mcp::model::{ErrorData, ResourceContents};
//! use nms_core::{
//!     address::GalacticAddress, biome::Biome, player::PlayerState,
//!     system::{Planet, System},
//! };
//! use nms_copilot::mcp::resources::{
//!     NmsResources, BASES_URI, GALAXY_STATS_URI, PLAYER_LOCATION_URI,
//! };
//! use nms_graph::GalaxyModel;
//! use serde_json::{json, Value};
//! use tokio::runtime::Builder;
//! use tokio::sync::RwLock;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! fn json_from(contents: Vec<ResourceContents>, uri: &str) -> Result<Value, Box<dyn std::error::Error>> {
//!     match contents.as_slice() {
//!         [ResourceContents::TextResourceContents { uri: actual_uri, mime_type, text, meta }] => {
//!             assert_eq!(actual_uri, uri);
//!             assert_eq!(mime_type.as_deref(), Some("application/json"));
//!             assert!(meta.is_none());
//!             Ok(serde_json::from_str(text)?)
//!         }
//!         _ => Err("expected exactly one text resource".into()),
//!     }
//! }
//! let model = Arc::new(RwLock::new(GalaxyModel::new()));
//! let registry = NmsResources::new(Arc::clone(&model));
//! let listed_resources = registry.resources();
//! let listed: Vec<_> = listed_resources.iter()
//!     .map(|resource| (resource.uri.as_str(), resource.name.as_str(), resource.mime_type.as_deref()))
//!     .collect();
//! assert_eq!(listed, [
//!     ("nms://player/location", "Player Location", Some("application/json")),
//!     ("nms://galaxy/stats", "Galaxy Stats", Some("application/json")),
//!     ("nms://bases", "Bases", Some("application/json")),
//! ]);
//! assert!(registry.read("nms://unknown").is_none());
//! let missing = registry.read(PLAYER_LOCATION_URI).ok_or("missing location")?;
//! let result = Builder::new_current_thread().build()?.block_on(missing);
//! assert!(matches!(result, Err(error) if error == ErrorData::invalid_params("Player position not available", None)));
//!
//! let pending = registry.read(PLAYER_LOCATION_URI).ok_or("missing location")?;
//! let address = GalacticAddress::new(3, 0, 0, 1, 2, 9);
//! let runtime = Builder::new_current_thread().build()?;
//! runtime.block_on(async {
//!     let mut guard = model.write().await;
//!     guard.insert_system(System::new(address, Some("Physical-9".into()), None, None,
//!         vec![Planet::new(2, Some(Biome::Lush), None, false, Some("Garden".into()), Some(7))]));
//!     guard.player_state = Some(PlayerState::new(address, 9, None, None, 0, 0, 0));
//!     guard.set_active_galaxy(7);
//! });
//! let location = json_from(runtime.block_on(pending)?, PLAYER_LOCATION_URI)?;
//! assert_eq!(location, json!({
//!     "system":"Physical-9", "planets_in_system":1, "galaxy":"Eissentam",
//!     "voxel_x":3, "voxel_y":0, "voxel_z":0, "solar_system_index":1,
//!     "portal_glyphs_hex":"200100000003",
//!     "portal_glyphs_emoji":"😑🌅🌅🕊🌅🌅🌅🌅🌅🌅🌅🦕",
//!     "signal_booster":"0802:007F:07FF:0001"
//! }));
//! let stats = runtime.block_on(registry.read(GALAXY_STATS_URI).ok_or("missing stats")?)?;
//! assert_eq!(json_from(stats, GALAXY_STATS_URI)?, json!({
//!     "systems":1, "planets":1, "bases":0, "named_systems":1,
//!     "named_planets":1, "infested_planets":0,
//!     "biome_distribution":[{"biome":"Lush","count":1}],
//!     "unknown_biome_count":0
//! }));
//! let bases = runtime.block_on(registry.read(BASES_URI).ok_or("missing bases")?)?;
//! assert_eq!(json_from(bases, BASES_URI)?, json!({"count":0,"bases":[]}));
//! # Ok(())
//! # }
//! ```
//!

use std::sync::Arc;

use fabryk_mcp::model::{Annotated, ErrorData, RawResource, Resource, ResourceContents};
use fabryk_mcp::{ResourceFuture, ResourceRegistry};
use tokio::sync::RwLock;

use nms_graph::GalaxyModel;

use super::tools::{build_bases_json, build_galaxy_stats_json, build_where_am_i_json};

/// URI for the player's current physical location.
///
/// The value is `nms://player/location`; the resource name is `Player
/// Location` and its MIME type is `application/json`. It follows physical
/// player position rather than the selected galaxy.
pub const PLAYER_LOCATION_URI: &str = "nms://player/location";

/// URI for aggregate statistics across the model's galaxies.
///
/// The value is `nms://galaxy/stats`; the resource name is `Galaxy Stats` and
/// its MIME type is `application/json`.
pub const GALAXY_STATS_URI: &str = "nms://galaxy/stats";

/// URI for all player bases in the model.
///
/// The value is `nms://bases`; the resource name is `Bases` and its MIME type
/// is `application/json`.
pub const BASES_URI: &str = "nms://bases";

/// MCP resources backed by the supplied shared [`GalaxyModel`].
///
/// Construction stores the `Arc<RwLock<GalaxyModel>>` without taking a
/// snapshot, starting a server or watcher, mutating model state, or writing
/// cache. Read futures acquire the lock when polled and query the current model.
/// A missing player position returns typed invalid-params data with message
/// `Player position not available`; statistics and bases work with an empty
/// model. Registry reads do not themselves subscribe or send notifications.
///
/// Resources are listed in this order: `nms://player/location` (`Player
/// Location`), `nms://galaxy/stats` (`Galaxy Stats`), then `nms://bases`
/// (`Bases`). All use `application/json`. Each successful read contains one
/// pretty-printed JSON text value with the requested URI, JSON MIME type, and
/// no annotations metadata.
///
/// See the [module examples](crate::mcp::resources) for the ordered resource
/// population, typed missing-position result, and current-model read behavior.
///
/// # Errors
///
/// Reading player location returns `INVALID_PARAMS` when player position is
/// unavailable. An unrecognized URI returns `None` before a future is made.
///
pub struct NmsResources {
    model: Arc<RwLock<GalaxyModel>>,
}

impl NmsResources {
    /// Store the shared model without taking a snapshot or starting a service.
    pub fn new(model: Arc<RwLock<GalaxyModel>>) -> Self {
        Self { model }
    }
}

impl ResourceRegistry for NmsResources {
    /// List location, statistics, then bases resources, all as JSON.
    fn resources(&self) -> Vec<Resource> {
        vec![
            make_resource(
                PLAYER_LOCATION_URI,
                "Player Location",
                "Current system, coordinates, portal glyphs, and galaxy. \
                 Updated when the player warps to a new system.",
            ),
            make_resource(
                GALAXY_STATS_URI,
                "Galaxy Stats",
                "Aggregate statistics: system/planet/base counts and biome \
                 distribution. Updated when new discoveries are made.",
            ),
            make_resource(
                BASES_URI,
                "Bases",
                "All player bases with locations and portal glyphs. \
                 Updated when bases are built or modified.",
            ),
        ]
    }

    /// Create a future for a known URI; it reads the current model when polled.
    ///
    /// Success contains one pretty-printed text resource using the requested
    /// URI, `application/json`, and no metadata. Missing player position is a
    /// typed invalid-params error.
    fn read(&self, uri: &str) -> Option<ResourceFuture> {
        let model = Arc::clone(&self.model);
        let uri_owned = uri.to_string();

        match uri {
            PLAYER_LOCATION_URI => Some(Box::pin(async move {
                let model = model.read().await;
                let json = build_where_am_i_json(&model)
                    .map_err(|e| ErrorData::invalid_params(e, None))?;
                let text = serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string());
                Ok(vec![
                    ResourceContents::text(text, uri_owned).with_mime_type("application/json"),
                ])
            })),
            GALAXY_STATS_URI => Some(Box::pin(async move {
                let model = model.read().await;
                let json = build_galaxy_stats_json(&model);
                let text = serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string());
                Ok(vec![
                    ResourceContents::text(text, uri_owned).with_mime_type("application/json"),
                ])
            })),
            BASES_URI => Some(Box::pin(async move {
                let model = model.read().await;
                let json = build_bases_json(&model);
                let text = serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string());
                Ok(vec![
                    ResourceContents::text(text, uri_owned).with_mime_type("application/json"),
                ])
            })),
            _ => None,
        }
    }
}

fn make_resource(uri: &str, name: &str, description: &str) -> Resource {
    let mut raw = RawResource::new(uri, name);
    raw.description = Some(description.into());
    raw.mime_type = Some("application/json".into());
    Annotated::new(raw, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn location_json(resources: &NmsResources) -> serde_json::Value {
        let contents = resources
            .read(PLAYER_LOCATION_URI)
            .expect("location resource is registered")
            .await
            .expect("player location should be available");
        let Some(ResourceContents::TextResourceContents { text, .. }) = contents.into_iter().next()
        else {
            panic!("location resource must return text JSON");
        };
        serde_json::from_str(&text).expect("resource text is valid JSON")
    }

    fn test_model() -> Arc<RwLock<GalaxyModel>> {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x003000001900", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x103000001900", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        Arc::new(RwLock::new(
            nms_save::parse_save(json.as_bytes())
                .map(|save| GalaxyModel::from_save(&save))
                .expect("test model JSON is valid"),
        ))
    }

    #[test]
    fn test_resources_lists_three() {
        let resources = NmsResources::new(test_model());
        let list = resources.resources();
        assert_eq!(list.len(), 3);

        let uris: Vec<&str> = list.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&PLAYER_LOCATION_URI));
        assert!(uris.contains(&GALAXY_STATS_URI));
        assert!(uris.contains(&BASES_URI));
    }

    #[test]
    fn test_resources_have_descriptions() {
        let resources = NmsResources::new(test_model());
        for r in resources.resources() {
            assert!(
                r.description.is_some(),
                "Resource {} missing description",
                r.name
            );
            assert_eq!(r.mime_type.as_deref(), Some("application/json"));
        }
    }

    #[tokio::test]
    async fn test_read_player_location() {
        let resources = NmsResources::new(test_model());
        let future = resources
            .read(PLAYER_LOCATION_URI)
            .expect("should match URI");
        let contents = future.await.expect("should succeed");
        assert_eq!(contents.len(), 1);
    }

    #[tokio::test]
    async fn test_player_location_resource_uses_physical_galaxy() {
        use nms_core::biome::Biome;
        use nms_core::player::PlayerState;
        use nms_core::system::{Planet, System};

        let model = test_model();
        let a0 = nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let a9 = nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, 9);
        {
            let mut guard = model.write().await;
            guard.insert_system(System::new(
                a0,
                Some("Origin-0".into()),
                None,
                None,
                vec![Planet::new(
                    0,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Leaf-0".into()),
                    None,
                )],
            ));
            guard.insert_system(System::new(
                a9,
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
            ));
            guard.player_state = Some(PlayerState::new(a0, 0, None, None, 7, 8, 9));
            guard.set_active_galaxy(9);
        }
        let resources = NmsResources::new(Arc::clone(&model));
        let origin = location_json(&resources).await;
        assert_eq!(origin["system"], "Origin-0");
        assert_eq!(origin["galaxy"], "Euclid");
        assert_eq!(origin["planets_in_system"], 1);

        {
            let mut guard = model.write().await;
            guard.apply_delta(&nms_core::SaveDelta {
                player_moved: Some(nms_core::delta::PlayerMoved { from: a0, to: a9 }),
                ..nms_core::SaveDelta::empty()
            });
            guard.set_active_galaxy(0);
        }
        let physical = model.read().await.player_state.clone();
        let selected = model.read().await.active_galaxy;
        let destination = location_json(&resources).await;
        assert_eq!(destination["system"], "Destination-9");
        assert_eq!(destination["galaxy"], "Eissentam");
        assert_eq!(destination["planets_in_system"], 1);
        {
            let guard = model.read().await;
            assert_eq!(guard.player_state, physical);
            assert_eq!(guard.active_galaxy, selected);
        }
        {
            let mut guard = model.write().await;
            guard.apply_delta(&nms_core::SaveDelta {
                player_moved: Some(nms_core::delta::PlayerMoved { from: a9, to: a0 }),
                ..nms_core::SaveDelta::empty()
            });
            guard.set_active_galaxy(9);
        }
        let reverse_state = model.read().await.player_state.clone();
        let reverse_selected = model.read().await.active_galaxy;
        let origin_again = location_json(&resources).await;
        assert_eq!(origin_again["system"], "Origin-0");
        assert_eq!(origin_again["galaxy"], "Euclid");
        let guard = model.read().await;
        assert_eq!(guard.player_state, reverse_state);
        assert_eq!(guard.active_galaxy, reverse_selected);
        drop(guard);

        {
            let mut guard = model.write().await;
            guard.player_state = None;
        }
        let error = resources.read(PLAYER_LOCATION_URI).unwrap().await;
        assert!(error.is_err());
    }

    #[tokio::test]
    async fn test_location_resource_distinguishes_marked_from_genuine_empty_system() {
        use nms_core::biome::Biome;
        use nms_core::player::PlayerState;
        use nms_core::system::{Planet, System};

        let model = test_model();
        let physical = nms_core::address::GalacticAddress::new(0, 0, 0, 2, 0, 0);
        let physical_id = nms_graph::SystemId::from_address(&physical);
        {
            let mut guard = model.write().await;
            guard.player_state = Some(PlayerState::new(physical, 0, None, None, 7, 8, 9));
            guard.set_active_galaxy(9);
            guard.insert_player_placeholder(physical);
            guard.insert_system(System::new(
                nms_core::address::GalacticAddress::new(0, 0, 0, 2, 0, 9),
                Some("Selected Galaxy Decoy".into()),
                None,
                None,
                vec![Planet::new(0, Some(Biome::Toxic), None, false, None, None)],
            ));
        }
        let snapshot = |guard: &nms_graph::GalaxyModel| {
            let id_key = |id: &nms_graph::SystemId| (id.reality_index(), id.system_address());
            let mut graph_nodes = guard.graph.node_weights().map(id_key).collect::<Vec<_>>();
            graph_nodes.sort();
            let mut graph_edges = guard
                .graph
                .edge_indices()
                .map(|edge| {
                    let (left, right) = guard.graph.edge_endpoints(edge).expect("edge endpoints");
                    let mut endpoints = [
                        id_key(guard.graph.node_weight(left).expect("left endpoint exists")),
                        id_key(
                            guard
                                .graph
                                .node_weight(right)
                                .expect("right endpoint exists"),
                        ),
                    ];
                    endpoints.sort();
                    (
                        endpoints,
                        guard
                            .graph
                            .edge_weight(edge)
                            .expect("edge weight exists")
                            .to_bits(),
                    )
                })
                .collect::<Vec<_>>();
            graph_edges.sort();
            let mut spatial_rows = guard
                .spatial
                .iter()
                .flat_map(|(galaxy, tree)| {
                    tree.iter().map(|point| {
                        (
                            *galaxy,
                            point.id.reality_index(),
                            point.id.system_address(),
                            point.point.map(f64::to_bits),
                        )
                    })
                })
                .collect::<Vec<_>>();
            spatial_rows.sort();
            (
                (
                    guard.systems.clone(),
                    guard.planets.clone(),
                    guard.bases.clone(),
                ),
                (
                    guard.node_map.clone(),
                    graph_nodes,
                    graph_edges,
                    spatial_rows,
                ),
                (
                    guard.name_index.clone(),
                    guard.biome_index.clone(),
                    guard.address_to_id.clone(),
                ),
                (
                    guard.player_state.clone(),
                    guard.active_galaxy,
                    guard
                        .systems
                        .keys()
                        .filter(|id| guard.is_player_placeholder(id))
                        .copied()
                        .collect::<std::collections::HashSet<_>>(),
                ),
            )
        };
        let state = || snapshot(&model.try_read().expect("location read released model lock"));
        let initial_state = state();
        let resources = NmsResources::new(Arc::clone(&model));
        let marked = location_json(&resources).await;
        let marked_expected = serde_json::json!({
            "system": "(unknown)",
            "planets_in_system": 0,
            "galaxy": "Euclid",
            "voxel_x": 0,
            "voxel_y": 0,
            "voxel_z": 0,
            "solar_system_index": 2,
            "portal_glyphs_hex": "000200000000",
            "portal_glyphs_emoji": nms_query::display::hex_to_emoji("000200000000"),
            "signal_booster": "07FF:007F:07FF:0002",
        });
        assert_eq!(marked, marked_expected);
        assert!(model.read().await.is_player_placeholder(&physical_id));
        assert_eq!(state(), initial_state);

        {
            let mut guard = model.write().await;
            guard.insert_system(System::new(physical, None, None, None, vec![]));
        }
        let empty_state = state();
        let genuine_empty = location_json(&resources).await;
        let empty_expected = serde_json::json!({
            "system": "-",
            "planets_in_system": 0,
            "galaxy": "Euclid",
            "voxel_x": 0,
            "voxel_y": 0,
            "voxel_z": 0,
            "solar_system_index": 2,
            "portal_glyphs_hex": "000200000000",
            "portal_glyphs_emoji": nms_query::display::hex_to_emoji("000200000000"),
            "signal_booster": "07FF:007F:07FF:0002",
        });
        assert_eq!(genuine_empty, empty_expected);
        assert!(!model.read().await.is_player_placeholder(&physical_id));
        assert_eq!(empty_state, state());

        for physical_galaxy in [0, 9] {
            for reverse_order in [false, true] {
                for case_state in ["absent", "marked", "empty", "unnamed", "named", "duplicate"] {
                    let opposite_galaxy = if physical_galaxy == 0 { 9 } else { 0 };
                    let physical =
                        nms_core::address::GalacticAddress::new(0, 0, 0, 2, 5, physical_galaxy);
                    let physical_id = nms_graph::SystemId::from_address(&physical);
                    let decoys = [
                        System::new(
                            nms_core::address::GalacticAddress::new(0, 0, 0, 1, 0, physical_galaxy),
                            Some("Same voxel other SSI".into()),
                            None,
                            None,
                            vec![
                                Planet::new(0, Some(Biome::Lush), None, false, None, None),
                                Planet::new(1, Some(Biome::Toxic), None, false, None, None),
                            ],
                        ),
                        System::new(
                            nms_core::address::GalacticAddress::new(0, 0, 0, 2, 0, opposite_galaxy),
                            Some("Same SSI other galaxy".into()),
                            None,
                            None,
                            vec![
                                Planet::new(0, Some(Biome::Lush), None, false, None, None),
                                Planet::new(1, Some(Biome::Toxic), None, false, None, None),
                                Planet::new(2, Some(Biome::Lush), None, false, None, None),
                            ],
                        ),
                    ];
                    let mut case = nms_graph::GalaxyModel::new();
                    case.player_state = Some(nms_core::player::PlayerState::new(
                        physical,
                        physical_galaxy,
                        None,
                        None,
                        7,
                        8,
                        9,
                    ));
                    case.set_active_galaxy(opposite_galaxy);
                    let player = case.player_state.as_ref().expect("matrix player state");
                    assert_eq!(player.current_reality, player.current_address.reality_index);
                    let target = match case_state {
                        "absent" | "marked" => None,
                        "empty" => Some(System::new(
                            nms_core::address::GalacticAddress::from_packed(
                                physical_id.system_address(),
                                physical_galaxy,
                            ),
                            None,
                            None,
                            None,
                            vec![],
                        )),
                        "unnamed" | "duplicate" => Some(System::new(
                            nms_core::address::GalacticAddress::from_packed(
                                physical_id.system_address(),
                                physical_galaxy,
                            ),
                            None,
                            None,
                            None,
                            vec![Planet::new(
                                5,
                                Some(Biome::Toxic),
                                None,
                                false,
                                Some("U2".into()),
                                None,
                            )],
                        )),
                        "named" => Some(System::new(
                            nms_core::address::GalacticAddress::from_packed(
                                physical_id.system_address(),
                                physical_galaxy,
                            ),
                            Some("Physical".into()),
                            None,
                            None,
                            vec![
                                Planet::new(
                                    5,
                                    Some(Biome::Toxic),
                                    None,
                                    false,
                                    Some("U2".into()),
                                    None,
                                ),
                                Planet::new(
                                    6,
                                    Some(Biome::Lush),
                                    None,
                                    false,
                                    Some("U3".into()),
                                    None,
                                ),
                            ],
                        )),
                        _ => unreachable!(),
                    };
                    if reverse_order {
                        if case_state == "marked" {
                            case.insert_player_placeholder(physical);
                        }
                        if let Some(target) = target.clone() {
                            case.insert_system(target);
                        }
                    }
                    for index in if reverse_order { [1, 0] } else { [0, 1] } {
                        case.insert_system(decoys[index].clone());
                    }
                    if !reverse_order {
                        if case_state == "marked" {
                            case.insert_player_placeholder(physical);
                        }
                        if let Some(target) = target {
                            case.insert_system(target);
                        }
                    }
                    if case_state == "duplicate" {
                        let before_duplicate = snapshot(&case);
                        case.insert_system(System::new(
                            nms_core::address::GalacticAddress::from_packed(
                                physical_id.system_address(),
                                physical_galaxy,
                            ),
                            Some("Ignored duplicate".into()),
                            None,
                            None,
                            vec![Planet::new(
                                5,
                                Some(Biome::Toxic),
                                None,
                                false,
                                Some("U2".into()),
                                None,
                            )],
                        ));
                        assert_eq!(snapshot(&case), before_duplicate);
                    }
                    let before_query = snapshot(&case);
                    let case = Arc::new(RwLock::new(case));
                    let resources = NmsResources::new(Arc::clone(&case));
                    let actual = location_json(&resources).await;
                    let (name, planets) = match case_state {
                        "absent" | "marked" => ("(unknown)", 0),
                        "empty" => ("-", 0),
                        "unnamed" | "duplicate" => ("-", 1),
                        "named" => ("Physical", 2),
                        _ => unreachable!(),
                    };
                    let portal = "500200000000";
                    assert_eq!(
                        actual,
                        serde_json::json!({
                            "system": name,
                            "planets_in_system": planets,
                            "galaxy": if physical_galaxy == 0 { "Euclid" } else { "Eissentam" },
                            "voxel_x": 0,
                            "voxel_y": 0,
                            "voxel_z": 0,
                            "solar_system_index": 2,
                            "portal_glyphs_hex": portal,
                            "portal_glyphs_emoji": nms_query::display::hex_to_emoji(portal),
                            "signal_booster": "07FF:007F:07FF:0002",
                        }),
                        "state={case_state} physical_galaxy={physical_galaxy} reverse_order={reverse_order}"
                    );
                    let guard = case.read().await;
                    assert_eq!(snapshot(&guard), before_query);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_read_galaxy_stats() {
        let resources = NmsResources::new(test_model());
        let future = resources.read(GALAXY_STATS_URI).expect("should match URI");
        let contents = future.await.expect("should succeed");
        assert_eq!(contents.len(), 1);
    }

    #[tokio::test]
    async fn test_read_bases() {
        let resources = NmsResources::new(test_model());
        let future = resources.read(BASES_URI).expect("should match URI");
        let contents = future.await.expect("should succeed");
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_read_unknown_uri_returns_none() {
        let resources = NmsResources::new(test_model());
        assert!(resources.read("nms://unknown").is_none());
    }
}
