//! Serialize and deserialize galaxy data to/from rkyv archives.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;
use rkyv::rancor::Error as RkyvError;
use rkyv::util::AlignedVec;

use nms_core::system::System;
use nms_graph::{EdgeStrategy, GalaxyModel};

use crate::data::{CacheData, CachedSystem};
use crate::error::{CacheCompatibilityError, CacheError};

// Bump the schema revision when the archive layout or codec changes. Bump the
// semantic revision when the meaning or identity rules of CacheData change.
const CACHE_MAGIC: &[u8; 8] = b"NMSCACHE";
const CACHE_SCHEMA_REVISION: u32 = 1;
const CACHE_SEMANTIC_REVISION: u32 = 1;
const CACHE_HEADER_LEN: usize = 16;

/// Extract cache data from a GalaxyModel.
pub fn extract_cache_data(model: &GalaxyModel, save_version: u32) -> CacheData {
    let systems: Vec<CachedSystem> = model
        .systems
        .values()
        .map(|system| CachedSystem {
            address: system.address,
            name: system.name.clone(),
            discoverer: system.discoverer.clone(),
            timestamp_secs: system.timestamp.map(|ts| ts.timestamp()),
            planets: system.planets.clone(),
        })
        .collect();

    let bases = model.bases.values().cloned().collect();

    let cached_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    CacheData {
        systems,
        bases,
        player_state: model.player_state.clone(),
        save_version,
        cached_at,
    }
}

/// Serialize cache data to bytes with a 16-byte compatibility envelope.
///
/// The envelope contains the archive schema and model semantic revisions;
/// these revisions are independent of [`CacheData::save_version`].
pub fn serialize(data: &CacheData) -> Result<Vec<u8>, CacheError> {
    let payload =
        rkyv::to_bytes::<RkyvError>(data).map_err(|e| CacheError::Serialize(e.to_string()))?;
    let mut bytes = Vec::with_capacity(CACHE_HEADER_LEN + payload.len());
    bytes.extend_from_slice(CACHE_MAGIC);
    bytes.extend_from_slice(&CACHE_SCHEMA_REVISION.to_le_bytes());
    bytes.extend_from_slice(&CACHE_SEMANTIC_REVISION.to_le_bytes());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

/// Validate the compatibility envelope, then checked-decode its rkyv payload.
///
/// Envelope and revision errors are returned before the payload is inspected.
pub fn deserialize(bytes: &[u8]) -> Result<CacheData, CacheError> {
    if bytes.len() < CACHE_HEADER_LEN {
        return Err(CacheError::Incompatible(
            CacheCompatibilityError::TruncatedHeader {
                actual: bytes.len(),
            },
        ));
    }

    if &bytes[..8] != CACHE_MAGIC {
        return Err(CacheError::Incompatible(
            CacheCompatibilityError::InvalidMagic,
        ));
    }

    let schema_revision = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if schema_revision != CACHE_SCHEMA_REVISION {
        return Err(CacheError::Incompatible(
            CacheCompatibilityError::SchemaMismatch {
                found: schema_revision,
                expected: CACHE_SCHEMA_REVISION,
            },
        ));
    }

    let semantic_revision = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
    if semantic_revision != CACHE_SEMANTIC_REVISION {
        return Err(CacheError::Incompatible(
            CacheCompatibilityError::SemanticMismatch {
                found: semantic_revision,
                expected: CACHE_SEMANTIC_REVISION,
            },
        ));
    }

    // The envelope is a Vec<u8>, so its payload offset need not satisfy rkyv's
    // alignment requirements. Copy into aligned storage before checked decode.
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len() - CACHE_HEADER_LEN);
    aligned.extend_from_slice(&bytes[CACHE_HEADER_LEN..]);
    rkyv::from_bytes::<CacheData, RkyvError>(&aligned)
        .map_err(|e| CacheError::Deserialize(e.to_string()))
}

/// Write cache data to a file.
pub fn write_cache(data: &CacheData, path: &Path) -> Result<(), CacheError> {
    let bytes = serialize(data)?;

    // Ensure parent directories exist (e.g. per-save cache paths)
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CacheError::Io)?;
    }

    // Write to a temp file first, then rename for atomicity
    let tmp_path = path.with_extension("rkyv.tmp");
    fs::write(&tmp_path, &bytes).map_err(CacheError::Io)?;
    fs::rename(&tmp_path, path).map_err(CacheError::Io)?;

    Ok(())
}

/// Read and deserialize cache data from a file.
pub fn read_cache(path: &Path) -> Result<CacheData, CacheError> {
    let bytes = fs::read(path).map_err(CacheError::Io)?;
    deserialize(&bytes)
}

/// Rebuild a GalaxyModel from cache data.
///
/// Reconstructs the graph, R-tree, and all HashMap indices.
pub fn rebuild_model(data: &CacheData) -> GalaxyModel {
    let mut model = GalaxyModel::new();

    for cached in &data.systems {
        let timestamp = cached
            .timestamp_secs
            .and_then(|secs| chrono::Utc.timestamp_opt(secs, 0).single());

        let system = System::new(
            cached.address,
            cached.name.clone(),
            cached.discoverer.clone(),
            timestamp,
            cached.planets.clone(),
        );
        model.insert_system(system);
    }

    for base in &data.bases {
        model.insert_base(base.clone());
    }

    model.player_state = data.player_state.clone();
    model.build_edges(EdgeStrategy::default());

    model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CacheCompatibilityError;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 100, "VoxelY": 50, "VoxelZ": -200, "SolarSystemIndex": 42, "PlanetIndex": 0}},
                    "Units": 5000000, "Nanites": 10000, "Specials": 500,
                    "PersistentPlayerBases": [
                        {"BaseVersion": 8, "GalacticAddress": "0x050003AB8C07", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Test Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}
                    ]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x050003AB8C07", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x150003AB8C07", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}},
                {"DD": {"UA": "0x250003AB8C07", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID":"","UID":"1","USN":"Explorer","PTK":"ST","TS":0}, "FL": {"U": 1}}
            ]}}}
        }"#;
        let save = nms_save::parse_save(json.as_bytes()).unwrap();
        GalaxyModel::from_save(&save)
    }

    #[test]
    fn test_extract_cache_data() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        assert_eq!(data.systems.len(), model.systems.len());
        assert_eq!(data.bases.len(), model.bases.len());
        assert_eq!(data.save_version, 4720);
        assert!(data.cached_at > 0);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let bytes = serialize(&data).unwrap();
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(restored.systems.len(), data.systems.len());
        assert_eq!(restored.bases.len(), data.bases.len());
    }

    #[test]
    fn test_rebuild_model_preserves_counts() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let bytes = serialize(&data).unwrap();
        let restored_data = deserialize(&bytes).unwrap();
        let rebuilt = rebuild_model(&restored_data);

        assert_eq!(rebuilt.systems.len(), model.systems.len());
        assert_eq!(rebuilt.planets.len(), model.planets.len());
        assert_eq!(rebuilt.bases.len(), model.bases.len());
    }

    #[test]
    fn test_write_and_read_cache_file() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rkyv");

        write_cache(&data, &path).unwrap();
        assert!(path.exists());

        let restored = read_cache(&path).unwrap();
        assert_eq!(restored.systems.len(), data.systems.len());
    }

    #[test]
    fn test_read_nonexistent_cache_errors() {
        let result = read_cache(Path::new("/tmp/nonexistent_nms_cache.rkyv"));
        assert!(result.is_err());
    }

    #[test]
    fn test_rebuild_preserves_player_state() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let rebuilt = rebuild_model(&data);
        assert!(rebuilt.player_state.is_some());
        assert_eq!(
            rebuilt.player_state.as_ref().unwrap().current_address,
            model.player_state.as_ref().unwrap().current_address
        );
    }

    #[test]
    fn test_rebuild_preserves_base_lookup() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let rebuilt = rebuild_model(&data);
        assert!(rebuilt.base("Test Base").is_some());
    }

    #[test]
    fn test_envelope_has_literal_header_and_roundtrips() {
        let data = extract_cache_data(&test_model(), 4720);
        let bytes = serialize(&data).unwrap();

        assert_eq!(
            &bytes[..CACHE_HEADER_LEN],
            &[
                b'N', b'M', b'S', b'C', b'A', b'C', b'H', b'E', 1, 0, 0, 0, 1, 0, 0, 0
            ]
        );
        assert_eq!(data.save_version, 4720);
        assert_ne!(data.save_version, CACHE_SCHEMA_REVISION);
        assert_eq!(deserialize(&bytes).unwrap().save_version, 4720);
    }

    #[test]
    fn test_deserialize_handles_misaligned_borrowed_envelope() {
        let data = extract_cache_data(&test_model(), 4720);
        let bytes = serialize(&data).unwrap();
        let mut backing = vec![0_u8; bytes.len() + 16];
        let base = backing.as_ptr() as usize;
        let offset = (0..16)
            .find(|offset| !(base + offset).is_multiple_of(16))
            .unwrap();
        backing[offset..offset + bytes.len()].copy_from_slice(&bytes);
        let borrowed = &backing[offset..offset + bytes.len()];

        assert!(!(borrowed.as_ptr() as usize).is_multiple_of(16));
        assert_eq!(deserialize(borrowed).unwrap().save_version, 4720);
    }

    #[test]
    fn test_deserialize_compatibility_error_precedence() {
        for size in [7, 8, 15] {
            assert!(matches!(
                deserialize(&vec![0; size]),
                Err(CacheError::Incompatible(
                    CacheCompatibilityError::TruncatedHeader { actual }
                )) if actual == size
            ));
        }

        let mut bytes = vec![0; CACHE_HEADER_LEN];
        bytes[..8].copy_from_slice(b"BADMAGIC");
        assert!(matches!(
            deserialize(&bytes),
            Err(CacheError::Incompatible(
                CacheCompatibilityError::InvalidMagic
            ))
        ));

        for schema in [0_u32, 2] {
            bytes[..8].copy_from_slice(CACHE_MAGIC);
            bytes[8..12].copy_from_slice(&schema.to_le_bytes());
            bytes[12..16].copy_from_slice(&1_u32.to_le_bytes());
            assert!(matches!(
                deserialize(&bytes),
                Err(CacheError::Incompatible(CacheCompatibilityError::SchemaMismatch {
                    found,
                    expected: 1
                })) if found == schema
            ));
        }

        for semantic in [0_u32, 2] {
            bytes[8..12].copy_from_slice(&1_u32.to_le_bytes());
            bytes[12..16].copy_from_slice(&semantic.to_le_bytes());
            assert!(matches!(
                deserialize(&bytes),
                Err(CacheError::Incompatible(CacheCompatibilityError::SemanticMismatch {
                    found,
                    expected: 1
                })) if found == semantic
            ));
        }

        // Mismatches are returned before inspecting an invalid payload, with
        // schema precedence when both revision fields differ.
        bytes[8..12].copy_from_slice(&2_u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&2_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&bytes),
            Err(CacheError::Incompatible(
                CacheCompatibilityError::SchemaMismatch {
                    found: 2,
                    expected: 1
                }
            ))
        ));
    }

    #[test]
    fn test_current_header_with_bad_payload_is_deserialize_error() {
        let mut bytes = CACHE_MAGIC.to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&bytes),
            Err(CacheError::Deserialize(_))
        ));
    }

    #[test]
    fn test_headerless_archive_is_not_a_compatible_cache() {
        let data = extract_cache_data(&test_model(), 9999);
        let legacy = rkyv::to_bytes::<RkyvError>(&data).unwrap();
        assert!(matches!(
            deserialize(&legacy),
            Err(CacheError::Incompatible(
                CacheCompatibilityError::InvalidMagic
            ))
        ));
    }

    #[test]
    fn test_compatibility_error_display_and_source_are_observable() {
        let error = CacheError::Incompatible(CacheCompatibilityError::SemanticMismatch {
            found: 9,
            expected: 1,
        });
        assert!(error.to_string().contains("semantic revision 9"));
        assert!(error.to_string().contains("expected 1"));
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "cache model semantic revision 9 is incompatible (expected 1)"
        );
    }

    #[test]
    fn test_read_missing_cache_preserves_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let error = read_cache(&dir.path().join("missing.rkyv")).unwrap_err();
        assert!(matches!(error, CacheError::Io(_)));
    }
}
