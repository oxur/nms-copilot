//! Raw binary save file parser for No Man's Sky.
//!
//! Reads `save.hg` files directly from disk:
//!
//! 1. Detect format (plaintext JSON vs LZ4 compressed)
//! 2. Parse sequential LZ4 blocks (magic `0xFEEDA1E5`), decompress, concatenate
//! 3. Deobfuscate JSON keys using MBINCompiler's `mapping.json`
//! 4. Deserialize into typed Rust structs via serde
//!
//! Also handles metadata verification (`mf_save.hg`) via XXTEA + SHA-256.

pub mod convert;
pub mod decompress;
pub mod error;
pub mod locate;
pub mod mapping;
pub mod metadata;
pub mod model;
pub mod xxtea;

pub use decompress::{SaveFormat, decompress_save, decompress_save_file, detect_format};
pub use error::SaveError;
pub use mapping::{KeyMapping, deobfuscate_json, is_obfuscated};
pub use metadata::{SaveMetadata, StorageSlot, read_metadata, verify_sha256};
pub use model::SaveRoot;

/// Parse deobfuscated save file JSON bytes into a [`SaveRoot`] struct.
///
/// The input must be valid UTF-8 JSON with plaintext (deobfuscated) keys.
pub fn parse_save(json: &[u8]) -> Result<SaveRoot, SaveError> {
    serde_json::from_slice(json).map_err(|e| SaveError::JsonParseError {
        message: e.to_string(),
    })
}

/// Parse a save file from disk, handling the full pipeline.
///
/// Runs the complete parsing pipeline:
/// 1. Read raw bytes from disk
/// 2. Decompress LZ4 blocks (passes through plaintext JSON unchanged)
/// 3. Check for obfuscated keys and deobfuscate if needed
/// 4. Deserialize into [`SaveRoot`]
///
/// This is the high-level entry point for reading NMS save files.
/// For already-decompressed, already-deobfuscated JSON bytes, use [`parse_save`] instead.
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
/// 1. Strips trailing null bytes (NMS LZ4 blocks are padded with nulls)
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
