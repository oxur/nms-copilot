//! Read and model No Man's Sky save data.
//!
//! The legacy file path reads bytes, classifies by the first byte, decompresses
//! LZ4 blocks when selected, removes trailing NUL bytes only when a non-NUL
//! byte exists (an all-NUL input is retained), replaces invalid UTF-8,
//! checks a leading `{"F2P"` key after ASCII whitespace, optionally applies the
//! bundled mapping, then deserializes the modeled fields. [`parse_save`] is a
//! narrower compatibility entry point for already-deobfuscated UTF-8 JSON and
//! applies serde defaults without validating original address members.
//!
//! Callers that need source-address validation can use
//! [`parse_validated_save`] while holding original deobfuscated JSON. It checks
//! modeled address values before serde defaults and keeps each physical address
//! paired with the typed row parsed from that same value. Both routes produce a
//! typed subset: unknown source keys and opaque values not represented by the
//! model are not a complete raw-save archive. Metadata parsing is a separate
//! operation through [`read_metadata`] and [`verify_sha256`]; neither save-file
//! entry point reads or verifies metadata automatically.
//!
//! For an in-memory on-disk representation, [`SaveParser`] and
//! [`NmsSaveParser`] accept supported plaintext JSON or standard NMS LZ4 bytes
//! through the same checked pipeline as [`parse_validated_save_file`].
//!
//! The modules provide the lower-level stages: [`decompress`] classifies and
//! decompresses bytes, [`mapping`] handles obfuscated JSON keys, [`model`]
//! contains the legacy typed representation and [`convert`] its compatibility
//! projections, [`address`] decodes checked source addresses, and [`validated`]
//! validates a whole modeled save. [`metadata`] parses separate metadata files,
//! [`xxtea`] exposes the low-level word transform, and [`locate`] lists save
//! paths without reading their contents.
//!
//! # Checked in-memory example
//!
//! ```
//! let json = br#"{
//!   "Version": 4720, "Platform": "Test", "ActiveContext": "Main",
//!   "CommonStateData": {},
//!   "BaseContext": {"PlayerStateData": {"UniverseAddress": {
//!     "RealityIndex": 9, "GalacticAddress": {"VoxelX": 3, "VoxelY": 0,
//!       "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 2}
//!   }}},
//!   "ExpeditionContext": {},
//!   "DiscoveryManagerData": {"DiscoveryData-v1": {"Store": {"Record": [
//!     {"DD": {"UA": "0x20010900000003", "DT": "Planet"}, "OWS": {}}
//!   ]}}}
//! }"#;
//! let checked = nms_save::parse_validated_save(json)?;
//! assert_eq!(checked.root().version, 4720);
//! assert_eq!(checked.player_state().current_reality, 9);
//! assert_eq!(checked.player_state().current_address.planet_index(), 2);
//! assert_eq!(checked.player_state().current_address.voxel_x(), 3);
//! assert_eq!(checked.discoveries().len(), 1);
//! let (record, physical) = checked.discoveries().next().ok_or_else(|| {
//!     std::io::Error::new(std::io::ErrorKind::Other, "missing discovery row")
//! })?;
//! assert_eq!(record.dd.dt, "Planet");
//! assert_eq!(physical.reality_index, 9);
//! assert_eq!(physical.planet_index(), 2);
//! assert_eq!(physical.voxel_x(), 3);
//! assert_eq!(physical.solar_system_index(), 1);
//! assert_eq!(physical.packed(), 0x200100000003);
//! assert!(checked.bases().is_empty());
//! assert!(checked.player_state().previous_address.is_none());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A checked packed address wider than 56 bits is rejected at its source path:
//!
//! ```
//! let too_wide = br#"{
//!   "Version":4720,"Platform":"Test","ActiveContext":"Main",
//!   "CommonStateData":{},"BaseContext":{"PlayerStateData":{"UniverseAddress":{
//!     "RealityIndex":0,"GalacticAddress":{"VoxelX":0,"VoxelY":0,"VoxelZ":0,"SolarSystemIndex":0,"PlanetIndex":0}}}},
//!   "ExpeditionContext":{},"DiscoveryManagerData":{"DiscoveryData-v1":{"Store":{"Record":[
//!     {"DD":{"UA":"0x100000000000000","DT":"Planet"},"OWS":{}}
//!   ]}}}
//! }"#;
//! assert!(matches!(
//!     nms_save::parse_validated_save(too_wide),
//!     Err(nms_save::SaveError::Address(nms_save::address::AddressDecodeError::PackedWidth { path }))
//!         if path == "/DiscoveryManagerData/DiscoveryData-v1/Store/Record/0/DD/UA"
//! ));
//! ```

pub mod address;
pub mod convert;
pub mod decompress;
pub mod error;
pub mod locate;
pub mod mapping;
pub mod metadata;
pub mod model;
pub mod parser;
pub mod validated;
pub mod xxtea;

pub use decompress::{SaveFormat, decompress_save, decompress_save_file, detect_format};
pub use error::SaveError;
pub use mapping::{KeyMapping, deobfuscate_json, is_obfuscated};
pub use metadata::{SaveMetadata, StorageSlot, read_metadata, verify_sha256};
pub use model::SaveRoot;
pub use parser::{NmsSaveParser, SaveParser};
pub use validated::{ValidatedSave, parse_validated_save, parse_validated_save_file};

/// Deserialize already-deobfuscated UTF-8 JSON into the legacy [`SaveRoot`] model.
///
/// This does not decompress bytes, map obfuscated keys, or validate source
/// address presence and ranges before serde applies defaults. Use
/// [`parse_validated_save`] when the original JSON value must be checked first.
/// The model retains only fields it defines; unknown JSON members are ignored.
///
/// # Examples
///
/// ```
/// let root = nms_save::parse_save(br#"{"Version":4720,"Platform":"Test","ActiveContext":"Main","CommonStateData":{},"BaseContext":{},"ExpeditionContext":{},"DiscoveryManagerData":{}}"#)?;
/// assert_eq!(root.version, 4720);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
/// # Errors
///
/// Returns [`SaveError::JsonParseError`] when JSON syntax or a modeled typed
/// field cannot be deserialized. It does not return address-validation errors.
///
pub fn parse_save(json: &[u8]) -> Result<SaveRoot, SaveError> {
    serde_json::from_slice(json).map_err(|e| SaveError::JsonParseError {
        message: e.to_string(),
    })
}

/// Read a save file through the legacy byte, mapping, and typed-deserialization path.
///
/// The pipeline reads raw bytes, applies first-byte format detection and
/// decompression, removes trailing NUL bytes only when a non-NUL byte exists
/// (an all-NUL input is retained), replaces invalid UTF-8, checks for the
/// leading obfuscated `F2P` key, applies bundled mappings when that
/// heuristic matches, then deserializes [`SaveRoot`]. This remains a permissive
/// compatibility route; use [`parse_validated_save_file`] to check modeled
/// address members before typed defaults.
///
/// # Examples
///
/// ```no_run
/// # fn read(path: &std::path::Path) -> Result<(), nms_save::SaveError> {
/// let save = nms_save::parse_save_file(path)?;
/// println!("save version {}", save.version);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns I/O and decompression errors, or [`SaveError::JsonParseError`] for
/// malformed JSON or modeled typed fields. The legacy path does not return
/// checked address errors.
///
/// # Panics
///
/// Decompression can panic on a malformed compressed-size offset on 32-bit
/// targets; see [`decompress::decompress_save`]. It also panics if an embedded
/// bundled mapping fails to parse, which indicates invalid compiled-in data.
pub fn parse_save_file(path: &std::path::Path) -> Result<SaveRoot, SaveError> {
    let raw = std::fs::read(path)?;
    let decompressed = decompress_save(&raw)?;

    // NMS saves can contain raw non-UTF-8 bytes in some string values
    // (e.g., item hashes, binary IDs). Sanitize to valid UTF-8 before
    // JSON parsing by replacing invalid sequences with U+FFFD.
    let json_bytes = sanitize_for_json(&decompressed);

    // Quick byte-level check: obfuscated saves start with {"F2P" (the obfuscated
    // "Version" key).
    if is_obfuscated_bytes(json_bytes.as_bytes()) {
        let mapping = KeyMapping::bundled();
        let value = deobfuscate_json(json_bytes.as_bytes(), &mapping)?;
        serde_json::from_value(value).map_err(|e| SaveError::JsonParseError {
            message: e.to_string(),
        })
    } else {
        parse_save(json_bytes.as_bytes())
    }
}

/// Sanitize decompressed save bytes for JSON parsing.
///
/// 1. Removes trailing null bytes after the last non-null byte, when one
///    exists (NMS LZ4 blocks are padded with nulls); all-null input is retained.
/// 2. Replaces invalid UTF-8 sequences with U+FFFD (NMS saves can contain
///    raw non-UTF-8 bytes in some string values like item hashes)
fn sanitize_for_json(data: &[u8]) -> std::borrow::Cow<'_, str> {
    let trimmed = match data.iter().rposition(|&b| b != 0) {
        Some(pos) => &data[..=pos],
        None => data,
    };
    String::from_utf8_lossy(trimmed)
}

/// Check if decompressed save bytes have obfuscated keys.
///
/// Scans the first bytes (skipping whitespace) for the `{"F2P"` pattern,
/// which is the obfuscated form of the `"Version"` key and always appears
/// first in obfuscated NMS saves. Works on raw bytes without requiring
/// valid UTF-8.
fn is_obfuscated_bytes(data: &[u8]) -> bool {
    let marker = b"{\"F2P\"";
    let trimmed = data
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .map(|pos| &data[pos..])
        .unwrap_or(data);
    trimmed.starts_with(marker)
}
