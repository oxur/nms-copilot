//! MCP resources for NMS Copilot.
//!
//! Exposes three live resources that clients can subscribe to for
//! real-time updates as the player explores the galaxy.

use std::sync::Arc;

use fabryk_mcp::model::{Annotated, ErrorData, RawResource, Resource, ResourceContents};
use fabryk_mcp::{ResourceFuture, ResourceRegistry};
use tokio::sync::RwLock;

use nms_graph::GalaxyModel;

use super::tools::{build_bases_json, build_galaxy_stats_json, build_where_am_i_json};

/// MCP resource URI for the player's current location.
pub const PLAYER_LOCATION_URI: &str = "nms://player/location";

/// MCP resource URI for galaxy statistics.
pub const GALAXY_STATS_URI: &str = "nms://galaxy/stats";

/// MCP resource URI for player bases.
pub const BASES_URI: &str = "nms://bases";

/// MCP resources backed by a shared GalaxyModel.
pub struct NmsResources {
    model: Arc<RwLock<GalaxyModel>>,
}

impl NmsResources {
    pub fn new(model: Arc<RwLock<GalaxyModel>>) -> Self {
        Self { model }
    }
}

impl ResourceRegistry for NmsResources {
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
