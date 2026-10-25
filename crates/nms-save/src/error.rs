//! Error types for save file parsing.

use crate::address::AddressDecodeError;

/// Error returned by save file decompression and parsing operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SaveError {
    /// Block header magic mismatch.
    #[error("invalid block magic at offset {offset:#x}: expected 0xFEEDA1E5, found {found:#010x}")]
    InvalidMagic {
        /// Byte offset where the block header began.
        offset: usize,
        /// Little-endian magic value read from that header.
        found: u32,
    },

    /// LZ4 decompression failed for a block.
    #[error("LZ4 decompression failed at offset {offset:#x}: {message}")]
    DecompressionFailed {
        /// Byte offset of the LZ4 block header.
        offset: usize,
        /// Decoder-provided description of the malformed block.
        message: String,
    },

    /// File ended before the expected number of bytes could be read.
    #[error("unexpected EOF at offset {offset:#x}: needed {expected} more bytes")]
    UnexpectedEof {
        /// Byte offset at which the incomplete header or payload begins.
        offset: usize,
        /// Number of bytes requested for the header or payload.
        expected: usize,
    },

    /// A block's decompressed_size exceeds the maximum chunk size.
    #[error(
        "block at offset {offset:#x} declares decompressed size {declared}, exceeds max 0x80000"
    )]
    ChunkTooLarge {
        /// Byte offset of the header that declared the size.
        offset: usize,
        /// Declared decompressed size, greater than the supported chunk limit.
        declared: u32,
    },

    /// Metadata file has an invalid length (not one of the known sizes).
    #[error("invalid metadata file length: {length} bytes")]
    InvalidMetaLength {
        /// Observed metadata-file byte length.
        length: usize,
    },

    /// No candidate same-kind storage key produced the metadata magic sentinel.
    #[error("metadata decryption failed: magic sentinel not found")]
    MetaDecryptionFailed,

    /// Metadata format version is unsupported (e.g., 2000/vanilla).
    #[error("unsupported metadata format version: {version:#x}")]
    UnsupportedMetaFormat {
        /// Metadata format value rejected by the parser (currently `0x7D0`).
        version: u32,
    },

    /// SHA-256 verification failed.
    #[error("SHA-256 mismatch")]
    Sha256Mismatch {
        /// Digest stored in metadata.
        expected: [u8; 32],
        /// Digest computed from the supplied raw save bytes.
        actual: [u8; 32],
    },

    /// Failed to parse a mapping JSON file.
    #[error("mapping parse error: {message}")]
    MappingParseError {
        /// Serde's mapping-file parse diagnostic.
        message: String,
    },

    /// Failed to parse save JSON.
    #[error("JSON parse error: {message}")]
    JsonParseError {
        /// Serde's JSON or typed-model parse diagnostic.
        message: String,
    },

    /// A modeled source address or its containing JSON field is invalid.
    #[error("{0}")]
    Address(#[from] AddressDecodeError),

    /// Wrapper for std::io::Error (file I/O).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
