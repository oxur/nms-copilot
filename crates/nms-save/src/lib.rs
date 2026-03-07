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

pub mod decompress;
pub mod error;
pub mod locate;
pub mod metadata;
pub mod xxtea;

pub use decompress::{SaveFormat, decompress_save, decompress_save_file, detect_format};
pub use error::SaveError;
pub use metadata::{SaveMetadata, StorageSlot, read_metadata, verify_sha256};
