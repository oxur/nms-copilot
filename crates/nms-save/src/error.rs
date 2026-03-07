//! Error types for save file parsing.

/// Error returned by save file decompression and parsing operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SaveError {
    /// Block header magic mismatch.
    #[error("invalid block magic at offset {offset:#x}: expected 0xFEEDA1E5, found {found:#010x}")]
    InvalidMagic { offset: usize, found: u32 },

    /// LZ4 decompression failed for a block.
    #[error("LZ4 decompression failed at offset {offset:#x}: {message}")]
    DecompressionFailed { offset: usize, message: String },

    /// File ended before the expected number of bytes could be read.
    #[error("unexpected EOF at offset {offset:#x}: needed {expected} more bytes")]
    UnexpectedEof { offset: usize, expected: usize },

    /// A block's decompressed_size exceeds the maximum chunk size.
    #[error(
        "block at offset {offset:#x} declares decompressed size {declared}, exceeds max 0x80000"
    )]
    ChunkTooLarge { offset: usize, declared: u32 },

    /// Wrapper for std::io::Error (file I/O).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
