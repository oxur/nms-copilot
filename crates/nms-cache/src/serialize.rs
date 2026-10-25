//! Serialize and deserialize galaxy data using a versioned cache envelope.
//!
//! The header is exactly 16 bytes: ASCII `NMSCACHE` at bytes 0–7, the schema
//! revision as a little-endian `u32` at bytes 8–11, and the semantic revision
//! as a little-endian `u32` at bytes 12–15. The checked rkyv payload begins at
//! byte 16. Both revisions must exactly match the reader's current values;
//! headerless legacy caches and mismatched revisions are rejected. Startup
//! fallback and cache rewriting are handled by [`crate::load_or_rebuild`], not
//! by the direct decoding functions in this module.
//!
//! Checked encoding and decoding validate only that a player state's
//! `current_reality` equals its `current_address.reality_index`, and that a
//! marked placeholder is Personal, planet zero, and has no name, discoverer,
//! timestamp, or planets. Selected galaxy, previous address, and freighter
//! address remain independent. This is not a general domain, duplicate-key,
//! or derived-index consistency validator.
//!
//! The schema revision describes archive layout, codec, and configuration
//! compatibility. The semantic revision describes the meaning and identity of
//! decoded model data. Future address-decoder or galaxy-identity changes must
//! assess and bump the semantic revision when meaning changes, even if archive
//! shape stays the same. One change may require both revisions to advance.
//! The current schema revision is 4 and semantic revision is 6. Schema 2 adds the
//! selected galaxy; schema 3 adds explicit player-placeholder provenance; schema 4
//! stores discovery contribution source.
//! Semantic 4 preserves validated 56-bit address identity when rebuilding a model;
//! semantic 5 adds checked discovery names to cold and live application models;
//! semantic 3 predates the application-wide identity boundary. Both
//! revisions are separate from game save version, crate version, and
//! `cached_at`.
//!
//! # Examples
//!
//! This example uses constructed cache data to exercise the checked codec,
//! model rebuild, and extraction boundaries. It compares complete literal
//! projections; vector order from hash-map-backed extraction is not promised.
//!
//! ```
//! use chrono::TimeZone;
//! use nms_cache::data::{CacheData, CachedSystem};
//! use nms_cache::serialize::{deserialize, serialize};
//! use nms_cache::{extract_cache_data, rebuild_model};
//! use nms_core::address::GalacticAddress;
//! use nms_core::player::{BaseType, PlayerBase, PlayerState};
//! use nms_core::system::{DiscoverySource, Planet, System, SystemId};
//!
//! let community_address = GalacticAddress::new(3, 0, 0, 1, 2, 0);
//! let empty_address = GalacticAddress::new(3, 0, 0, 1, 0, 7);
//! let marked_address = GalacticAddress::new(3, 0, 0, 1, 0, 9);
//! let garden = Planet::new(2, Some(nms_core::Biome::Lush), None, true, Some("Garden".into()), Some(42));
//! let cache = CacheData {
//!     systems: vec![
//!         CachedSystem {
//!             address: community_address,
//!             source: DiscoverySource::Community { source: "NMSCE".into() },
//!             name: Some("Atlas".into()),
//!             discoverer: Some("Explorer".into()),
//!             timestamp_secs: Some(1_700_000_000),
//!             planets: vec![garden.clone()],
//!             is_player_placeholder: false,
//!         },
//!         CachedSystem {
//!             address: empty_address,
//!             source: DiscoverySource::Personal,
//!             name: None,
//!             discoverer: None,
//!             timestamp_secs: None,
//!             planets: vec![],
//!             is_player_placeholder: false,
//!         },
//!         CachedSystem {
//!             address: marked_address,
//!             source: DiscoverySource::Personal,
//!             name: None,
//!             discoverer: None,
//!             timestamp_secs: None,
//!             planets: vec![],
//!             is_player_placeholder: true,
//!         },
//!     ],
//!     bases: vec![PlayerBase::new(
//!         "Home".into(), BaseType::HomePlanetBase,
//!         GalacticAddress::new(3, 0, 0, 1, 2, 0), [1.0, 2.0, 3.0], Some("owner".into()),
//!     )],
//!     player_state: Some(PlayerState::new(
//!         GalacticAddress::new(3, 0, 0, 1, 5, 9), 9,
//!         Some(GalacticAddress::new(3, 0, 0, 1, 2, 0)),
//!         Some(GalacticAddress::new(3, 0, 0, 1, 0, 7)), 100, 20, 5,
//!     )),
//!     active_galaxy: 7,
//!     save_version: 91,
//!     cached_at: 123,
//! };
//!
//! let bytes = serialize(&cache)?;
//! assert_eq!(&bytes[..16], b"NMSCACHE\x04\x00\x00\x00\x06\x00\x00\x00");
//! let decoded = deserialize(&bytes)?;
//! let rebuilt = rebuild_model(&decoded);
//! let extracted = extract_cache_data(&rebuilt, 91);
//!
//! fn rows(data: &CacheData) -> Vec<(GalacticAddress, DiscoverySource, Option<String>, Option<String>, Option<i64>, Vec<Planet>, bool)> {
//!     let mut rows: Vec<_> = data.systems.iter().map(|s| (
//!         s.address, s.source.clone(), s.name.clone(), s.discoverer.clone(),
//!         s.timestamp_secs, s.planets.clone(), s.is_player_placeholder,
//!     )).collect();
//!     rows.sort_by_key(|r| (r.0.reality_index, r.0.packed()));
//!     rows
//! }
//! let expected_rows = vec![
//!     (GalacticAddress::new(3, 0, 0, 1, 2, 0), DiscoverySource::Community { source: "NMSCE".into() }, Some("Atlas".into()), Some("Explorer".into()), Some(1_700_000_000), vec![Planet::new(2, Some(nms_core::Biome::Lush), None, true, Some("Garden".into()), Some(42))], false),
//!     (GalacticAddress::new(3, 0, 0, 1, 0, 7), DiscoverySource::Personal, None, None, None, vec![], false),
//!     (GalacticAddress::new(3, 0, 0, 1, 0, 9), DiscoverySource::Personal, None, None, None, vec![], true),
//! ];
//! assert_eq!(rows(&decoded), expected_rows);
//! assert_eq!(rows(&extracted), expected_rows);
//! assert_eq!(decoded.cached_at, 123);
//! assert_eq!(decoded.active_galaxy, 7);
//! assert_eq!(decoded.save_version, 91);
//! assert_eq!(extracted.active_galaxy, 7);
//! assert_eq!(extracted.save_version, 91);
//! let expected_base = PlayerBase::new("Home".into(), BaseType::HomePlanetBase, GalacticAddress::new(3, 0, 0, 1, 2, 0), [1.0, 2.0, 3.0], Some("owner".into()));
//! assert_eq!(decoded.bases, vec![expected_base.clone()]);
//! assert_eq!(extracted.bases, vec![expected_base]);
//! let expected_player = PlayerState::new(GalacticAddress::new(3, 0, 0, 1, 5, 9), 9, Some(GalacticAddress::new(3, 0, 0, 1, 2, 0)), Some(GalacticAddress::new(3, 0, 0, 1, 0, 7)), 100, 20, 5);
//! assert_eq!(decoded.player_state, Some(expected_player.clone()));
//! assert_eq!(extracted.player_state, Some(expected_player));
//! assert_eq!(rebuilt.player_state, Some(PlayerState::new(GalacticAddress::new(3, 0, 0, 1, 5, 9), 9, Some(GalacticAddress::new(3, 0, 0, 1, 2, 0)), Some(GalacticAddress::new(3, 0, 0, 1, 0, 7)), 100, 20, 5)));
//!
//! let mut expected_community = System::new(GalacticAddress::new(3, 0, 0, 1, 2, 0), Some("Atlas".into()), Some("Explorer".into()), Some(chrono::Utc.timestamp_opt(1_700_000_000, 0).single().ok_or("literal timestamp is invalid")?), vec![Planet::new(2, Some(nms_core::Biome::Lush), None, true, Some("Garden".into()), Some(42))]);
//! expected_community.source = DiscoverySource::Community { source: "NMSCE".into() };
//! let expected_systems = std::collections::HashMap::from([
//!     (SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 2, 0)), expected_community),
//!     (SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 0, 7)), System::new(GalacticAddress::new(3, 0, 0, 1, 0, 7), None, None, None, vec![])),
//!     (SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 0, 9)), System::new(GalacticAddress::new(3, 0, 0, 1, 0, 9), None, None, None, vec![])),
//! ]);
//! assert_eq!(rebuilt.systems, expected_systems);
//! let mut key_projection: Vec<_> = rebuilt.systems.keys()
//!     .map(|id| (id.reality_index(), id.system_address()))
//!     .collect();
//! key_projection.sort_unstable();
//! assert_eq!(key_projection, vec![
//!     (0, 0x0001_0000_0003),
//!     (7, 0x0001_0000_0003),
//!     (9, 0x0001_0000_0003),
//! ]);
//! let id0 = SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 2, 0));
//! let id7 = SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 0, 7));
//! let id9 = SystemId::from_address(&GalacticAddress::new(3, 0, 0, 1, 0, 9));
//! assert_eq!(rebuilt.system_count(), 3);
//! assert_eq!(rebuilt.planet_count(), 1);
//! assert_eq!(rebuilt.base_count(), 1);
//! assert_eq!(rebuilt.graph.node_count(), 3);
//! assert_eq!(rebuilt.graph.edge_count(), 0);
//! assert!(rebuilt.is_player_placeholder(&id9));
//! assert!(!rebuilt.is_player_placeholder(&id7));
//! assert!(!rebuilt.is_player_placeholder(&id0));
//! assert_eq!(rebuilt.planets, std::collections::HashMap::from([((id0, 2), Planet::new(2, Some(nms_core::Biome::Lush), None, true, Some("Garden".into()), Some(42)))]));
//! assert_eq!(rebuilt.bases, std::collections::HashMap::from([("home".into(), PlayerBase::new("Home".into(), BaseType::HomePlanetBase, GalacticAddress::new(3, 0, 0, 1, 2, 0), [1.0, 2.0, 3.0], Some("owner".into())))]));
//! assert_eq!(rebuilt.active_galaxy, 7);
//!
//! let mut inconsistent = deserialize(&bytes)?;
//! inconsistent.player_state.as_mut().ok_or("player state missing")?.current_reality = 3;
//! assert!(matches!(serialize(&inconsistent), Err(nms_cache::CacheError::Serialize(message)) if message == "player state reality mismatch: current_reality=3 but current_address.reality_index=9"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;
use rkyv::rancor::Error as RkyvError;
use rkyv::util::AlignedVec;

use nms_core::system::{DiscoverySource, System};
use nms_graph::{EdgeStrategy, GalaxyModel};

use crate::data::{CacheData, CachedSystem};
use crate::error::{CacheCompatibilityError, CacheError};

// Bump the schema revision when the archive layout or codec changes. Bump the
// semantic revision when the meaning or identity rules of CacheData change.
const CACHE_MAGIC: &[u8; 8] = b"NMSCACHE";
const CACHE_SCHEMA_REVISION: u32 = 4;
const CACHE_SEMANTIC_REVISION: u32 = 6;
const CACHE_HEADER_LEN: usize = 16;

/// Validate duplicated physical reality fields without changing cache data.
fn validate_cache_data(data: &CacheData) -> Result<(), String> {
    if let Some(player) = &data.player_state {
        let address_reality = player.current_address.reality_index;
        if player.current_reality != address_reality {
            return Err(format!(
                "player state reality mismatch: current_reality={} but current_address.reality_index={address_reality}",
                player.current_reality
            ));
        }
    }

    for system in &data.systems {
        if system.is_player_placeholder
            && (system.source != DiscoverySource::Personal
                || system.address.planet_index() != 0
                || system.name.is_some()
                || system.discoverer.is_some()
                || system.timestamp_secs.is_some()
                || !system.planets.is_empty())
        {
            let id = nms_core::system::SystemId::from_address(&system.address);
            return Err(format!(
                "player placeholder {id} must have a planet-zero address and no discovery metadata or planets"
            ));
        }
    }

    Ok(())
}

/// Copy the supported galaxy-model subset into cache data.
///
/// Systems, bases, player state, and the selected galaxy are copied from the
/// model; planets remain embedded in their systems. The caller supplies the
/// save version. The cache timestamp is generated at whole-second precision.
/// Hash-map iteration makes the returned system and base vector order
/// unspecified. Custom edges, derived indexes, and flat orphan-planet entries
/// are not serialized.
///
/// # Examples
///
/// See the [complete codec and rebuild example](crate::serialize#examples).
pub fn extract_cache_data(model: &GalaxyModel, save_version: u32) -> CacheData {
    let systems: Vec<CachedSystem> = model
        .systems
        .values()
        .map(|system| CachedSystem {
            address: system.address,
            source: system.source.clone(),
            name: system.name.clone(),
            discoverer: system.discoverer.clone(),
            timestamp_secs: system.timestamp.map(|ts| ts.timestamp()),
            planets: system.planets.clone(),
            is_player_placeholder: model
                .is_player_placeholder(&nms_core::system::SystemId::from_address(&system.address)),
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
        active_galaxy: model.active_galaxy,
        save_version,
        cached_at,
    }
}

/// Encode cache data with the current 16-byte compatibility envelope.
///
/// The envelope contains the ASCII `NMSCACHE` magic, then the schema and
/// semantic revisions as little-endian `u32` values, followed by the rkyv
/// payload. The revisions are independent of [`CacheData::save_version`].
///
/// # Examples
///
/// See the [complete codec and rebuild example](crate::serialize#examples).
///
/// # Errors
///
/// Returns [`CacheError::Serialize`] if player state or placeholder provenance
/// is inconsistent, or rkyv cannot encode the cache data.
pub fn serialize(data: &CacheData) -> Result<Vec<u8>, CacheError> {
    validate_cache_data(data).map_err(CacheError::Serialize)?;
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
/// Both revisions must exactly equal the current schema and semantic revisions.
/// Envelope and revision errors are returned before the payload is inspected.
/// A headerless archive is incompatible; this function returns its typed error
/// without rebuilding from the source save. A 16-byte current header with an
/// empty payload reaches payload decoding and returns [`CacheError::Deserialize`].
///
/// # Examples
///
/// See the [complete codec and rebuild example](crate::serialize#examples) and
/// the [typed envelope errors](crate::error#examples).
///
/// # Errors
///
/// Returns [`CacheError::Incompatible`] for a truncated header, invalid magic,
/// unsupported schema revision, or unsupported semantic revision. Returns
/// [`CacheError::Deserialize`] when the current envelope contains an invalid
/// rkyv payload, contradictory player reality fields, or invalid placeholder
/// provenance.
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
    let data = rkyv::from_bytes::<CacheData, RkyvError>(&aligned)
        .map_err(|e| CacheError::Deserialize(e.to_string()))?;
    validate_cache_data(&data).map_err(CacheError::Deserialize)?;
    Ok(data)
}

/// Serialize cache data and replace the file at `path`.
///
/// The parent directory is created when needed. The implementation writes to
/// a sibling path with extension `rkyv.tmp` and then renames it over the
/// destination; it does not promise fsync durability or isolation between
/// concurrent writers. Serialization and cache-data validation happen before
/// filesystem mutation.
///
/// # Errors
///
/// Returns [`CacheError::Serialize`] before file access when data validation
/// or encoding fails. Returns [`CacheError::Io`] when directory creation,
/// temporary-file writing, or rename fails. A failed I/O operation may leave
/// the created directory or temporary file behind.
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

/// Read a cache file and validate its envelope and payload.
///
/// This direct API does not check freshness, rebuild from a source save, or
/// repair the cache. Use [`crate::load_or_rebuild`] when fallback behavior is
/// desired.
///
/// # Errors
///
/// Returns [`CacheError::Io`] when the file cannot be read, or the same typed
/// envelope and payload errors as [`deserialize`].
pub fn read_cache(path: &Path) -> Result<CacheData, CacheError> {
    let bytes = fs::read(path).map_err(CacheError::Io)?;
    deserialize(&bytes)
}

/// Rebuild a GalaxyModel from cache data.
///
/// Reconstructs the graph, R-tree, and derived HashMap indices from trusted
/// in-memory [`CacheData`]. This infallible function does not validate persisted
/// data or fall back to a source save. Invalid timestamp seconds convert to
/// `None`. Systems are inserted in vector order: the first genuine duplicate
/// key wins, while a later genuine record can promote an earlier placeholder.
/// A malformed marked record is inserted as an ordinary, unmarked system; its
/// other modeled data is retained after timestamp conversion. Persisted archives
/// must instead pass the checked codec, which rejects malformed flags.
/// Nonempty base names are lowercased without trimming, and a later base with
/// the same normalized key replaces the earlier one. Player state and selected
/// galaxy are restored independently. Graph edges use the default per-galaxy
/// KNN-10 strategy, not custom topology. This function does not ensure a missing
/// player-system key; the loader performs that separate operation.
///
/// # Examples
///
/// See the [complete codec and rebuild example](crate::serialize#examples).
pub fn rebuild_model(data: &CacheData) -> GalaxyModel {
    let mut model = GalaxyModel::new();

    for cached in &data.systems {
        let timestamp = cached
            .timestamp_secs
            .and_then(|secs| chrono::Utc.timestamp_opt(secs, 0).single());

        let mut system = System::new(
            cached.address,
            cached.name.clone(),
            cached.discoverer.clone(),
            timestamp,
            cached.planets.clone(),
        );
        system.source = cached.source.clone();
        if cached.is_player_placeholder
            && cached.source == DiscoverySource::Personal
            && cached.address.planet_index() == 0
            && cached.name.is_none()
            && cached.discoverer.is_none()
            && cached.timestamp_secs.is_none()
            && cached.planets.is_empty()
        {
            model.insert_player_placeholder(cached.address);
        } else {
            // This infallible API accepts trusted in-memory data. Preserve
            // malformed marked records as genuine systems instead of dropping
            // metadata or panicking; checked persisted entrypoints reject them.
            model.insert_system(system);
        }
    }

    for base in &data.bases {
        model.insert_base(base.clone());
    }

    model.player_state = data.player_state.clone();
    model.set_active_galaxy(data.active_galaxy);
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
    fn test_direct_cache_transport_preserves_two_galaxies_and_selected_galaxy() {
        for reverse in [false, true] {
            for with_player in [false, true] {
                let mut model = GalaxyModel::new();
                if with_player {
                    model.player_state = test_model().player_state;
                }
                let addresses = [
                    nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 0),
                    nms_core::address::GalacticAddress::from_packed(0x150003AB8C07, 9),
                ];
                let systems = [
                    System::new(
                        addresses[0],
                        Some("Zero".into()),
                        None,
                        None,
                        vec![nms_core::system::Planet::new(
                            1,
                            Some(nms_core::biome::Biome::Lush),
                            None,
                            false,
                            Some("Zero marker".into()),
                            None,
                        )],
                    ),
                    System::new(
                        addresses[1],
                        Some("Nine".into()),
                        None,
                        None,
                        vec![nms_core::system::Planet::new(
                            1,
                            Some(nms_core::biome::Biome::Toxic),
                            None,
                            false,
                            Some("Nine marker".into()),
                            None,
                        )],
                    ),
                ];
                for index in if reverse { [1, 0] } else { [0, 1] } {
                    model.insert_system(systems[index].clone());
                }
                model.set_active_galaxy(9);

                let data = extract_cache_data(&model, 4720);
                assert_eq!(data.active_galaxy, 9);
                let bytes = serialize(&data).unwrap();
                let decoded = deserialize(&bytes).unwrap();
                let rebuilt = rebuild_model(&decoded);

                let id0 = nms_core::system::SystemId::from_address(&addresses[0]);
                let id9 = nms_core::system::SystemId::from_address(&addresses[1]);
                assert_eq!(rebuilt.active_galaxy, 9);
                assert_eq!(rebuilt.system_count(), 2);
                assert_eq!(rebuilt.planet_count(), 2);
                assert_eq!(rebuilt.spatial_for(0).unwrap().size(), 1);
                assert_eq!(rebuilt.spatial_for(9).unwrap().size(), 1);
                assert_eq!(rebuilt.address_to_id.get(&id0), Some(&id0));
                assert_eq!(rebuilt.address_to_id.get(&id9), Some(&id9));
                assert_eq!(rebuilt.system(&id0).unwrap().name.as_deref(), Some("Zero"));
                assert_eq!(rebuilt.system(&id9).unwrap().name.as_deref(), Some("Nine"));
                assert_eq!(
                    rebuilt.planets.get(&(id0, 1)).unwrap().name.as_deref(),
                    Some("Zero marker")
                );
                assert_eq!(
                    rebuilt.planets.get(&(id9, 1)).unwrap().name.as_deref(),
                    Some("Nine marker")
                );
                assert_eq!(
                    rebuilt.planets.get(&(id0, 1)).unwrap().biome,
                    Some(nms_core::biome::Biome::Lush)
                );
                assert_eq!(
                    rebuilt.planets.get(&(id9, 1)).unwrap().biome,
                    Some(nms_core::biome::Biome::Toxic)
                );
                assert_eq!(rebuilt.player_state.is_some(), with_player);
                if with_player {
                    assert_eq!(rebuilt.player_position().unwrap().reality_index, 0);
                }
            }
        }
    }

    #[test]
    fn test_rebuild_preserves_base_lookup() {
        let model = test_model();
        let data = extract_cache_data(&model, 4720);
        let rebuilt = rebuild_model(&data);
        assert!(rebuilt.base("Test Base").is_some());
    }

    #[test]
    fn test_marked_community_source_is_rejected_but_trusted_rebuild_preserves_it() {
        let address = nms_core::address::GalacticAddress::new(2, 0, 0, 3, 0, 0);
        let mut model = GalaxyModel::new();
        model.insert_player_placeholder(address);
        let positive = extract_cache_data(&model, 4720);
        assert_eq!(positive.systems.len(), 1);
        assert_eq!(positive.systems[0].source, DiscoverySource::Personal);
        assert!(positive.systems[0].is_player_placeholder);
        let bytes = serialize(&positive).unwrap();
        let checked = deserialize(&bytes).unwrap();
        assert_eq!(checked.systems[0].source, DiscoverySource::Personal);
        assert!(checked.systems[0].is_player_placeholder);

        // Decode the same valid positive payload, then mutate only source.
        let mut marked_community = deserialize(&bytes).unwrap();
        marked_community.systems[0].source = DiscoverySource::Community {
            source: " NMSCE α ".to_string(),
        };
        assert_eq!(marked_community.systems.len(), checked.systems.len());
        assert_eq!(
            marked_community.systems[0].address,
            checked.systems[0].address
        );
        assert_eq!(marked_community.systems[0].name, checked.systems[0].name);
        assert_eq!(
            marked_community.systems[0].discoverer,
            checked.systems[0].discoverer
        );
        assert_eq!(
            marked_community.systems[0].timestamp_secs,
            checked.systems[0].timestamp_secs
        );
        assert_eq!(
            marked_community.systems[0].planets,
            checked.systems[0].planets
        );
        assert_eq!(
            marked_community.systems[0].is_player_placeholder,
            checked.systems[0].is_player_placeholder
        );
        assert_eq!(marked_community.bases, checked.bases);
        assert_eq!(marked_community.player_state, checked.player_state);
        assert_eq!(marked_community.active_galaxy, checked.active_galaxy);
        assert_eq!(marked_community.save_version, checked.save_version);
        assert_eq!(marked_community.cached_at, checked.cached_at);
        assert!(matches!(
            serialize(&marked_community),
            Err(CacheError::Serialize(_))
        ));

        // The trusted in-memory rebuild boundary preserves all supplied record
        // data, but a marked non-Personal record is treated as genuine.
        let rebuilt = rebuild_model(&marked_community);
        let id = nms_core::system::SystemId::from_address(&address);
        let system = rebuilt.system(&id).unwrap();
        let mut expected_system = System::new(address, None, None, None, vec![]);
        expected_system.source = DiscoverySource::Community {
            source: " NMSCE α ".to_string(),
        };
        let mut expected = GalaxyModel::new();
        expected.insert_system(expected_system.clone());
        expected.build_edges(EdgeStrategy::default());

        assert_eq!(system, &expected_system);
        assert_eq!(
            system.source,
            DiscoverySource::Community {
                source: " NMSCE α ".to_string()
            }
        );
        assert_eq!(rebuilt.systems, expected.systems);
        assert_eq!(rebuilt.planets, expected.planets);
        assert_eq!(rebuilt.bases, expected.bases);
        assert_eq!(rebuilt.name_index, expected.name_index);
        assert_eq!(rebuilt.biome_index, expected.biome_index);
        assert_eq!(rebuilt.address_to_id, expected.address_to_id);
        assert_eq!(rebuilt.node_map, expected.node_map);
        assert_eq!(rebuilt.active_galaxy, 0);
        assert!(rebuilt.player_state.is_none());
        assert_eq!(rebuilt.graph.node_count(), 1);
        assert_eq!(rebuilt.graph.edge_count(), 0);
        let spatial_rows = |model: &GalaxyModel| {
            let mut rows = model
                .spatial
                .iter()
                .flat_map(|(galaxy, tree)| {
                    tree.iter().map(|point| {
                        (
                            *galaxy,
                            format!("{:?}", point.id),
                            point.point.map(f64::to_bits),
                        )
                    })
                })
                .collect::<Vec<_>>();
            rows.sort();
            rows
        };
        assert_eq!(spatial_rows(&rebuilt), spatial_rows(&expected));
        assert!(!rebuilt.spatial.contains_key(&9));
        assert!(!rebuilt.is_player_placeholder(&id));
    }

    #[test]
    fn test_envelope_has_literal_header_and_roundtrips() {
        let data = extract_cache_data(&test_model(), 4720);
        let bytes = serialize(&data).unwrap();

        assert_eq!(
            &bytes[..CACHE_HEADER_LEN],
            &[
                b'N', b'M', b'S', b'C', b'A', b'C', b'H', b'E', 4, 0, 0, 0, 6, 0, 0, 0
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
        for size in [0, 7, 8, 15] {
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

        for schema in [0_u32, 1, 2, 3, 5] {
            bytes[..8].copy_from_slice(CACHE_MAGIC);
            bytes[8..12].copy_from_slice(&schema.to_le_bytes());
            bytes[12..16].copy_from_slice(&3_u32.to_le_bytes());
            assert!(matches!(
                deserialize(&bytes),
                Err(CacheError::Incompatible(CacheCompatibilityError::SchemaMismatch {
                    found,
                    expected: 4
                })) if found == schema
            ));
        }

        for semantic in [0_u32, 1, 2, 3, 4, 5, 7] {
            bytes[8..12].copy_from_slice(&4_u32.to_le_bytes());
            bytes[12..16].copy_from_slice(&semantic.to_le_bytes());
            assert!(matches!(
                deserialize(&bytes),
                Err(CacheError::Incompatible(CacheCompatibilityError::SemanticMismatch {
                    found,
                    expected: 6
                })) if found == semantic
            ));
        }

        // Mismatches are returned before inspecting an invalid payload, with
        // schema precedence when both revision fields differ.
        bytes[8..12].copy_from_slice(&3_u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&5_u32.to_le_bytes());
        assert!(matches!(
            deserialize(&bytes),
            Err(CacheError::Incompatible(
                CacheCompatibilityError::SchemaMismatch {
                    found: 3,
                    expected: 4
                }
            ))
        ));
    }

    #[test]
    fn test_current_header_with_bad_payload_is_deserialize_error() {
        let mut bytes = CACHE_MAGIC.to_vec();
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&6_u32.to_le_bytes());
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
            expected: 6,
        });
        assert!(error.to_string().contains("semantic revision 9"));
        assert!(error.to_string().contains("expected 6"));
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "cache model semantic revision 9 is incompatible (expected 6)"
        );
    }

    #[test]
    fn test_read_missing_cache_preserves_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let error = read_cache(&dir.path().join("missing.rkyv")).unwrap_err();
        assert!(matches!(error, CacheError::Io(_)));
    }
}
