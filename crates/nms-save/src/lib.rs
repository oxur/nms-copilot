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

/// Parse a deobfuscated save file from disk.
///
/// Reads the file and deserializes it. Assumes the file is already
/// decompressed and deobfuscated JSON.
pub fn parse_save_file(path: &std::path::Path) -> Result<SaveRoot, SaveError> {
    let bytes = std::fs::read(path)?;
    parse_save(&bytes)
}
