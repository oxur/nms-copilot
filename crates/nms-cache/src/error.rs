//! Cache error types.

use std::io;

/// Why an on-disk cache cannot be interpreted by this version of the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheCompatibilityError {
    /// The cache does not contain a complete compatibility header.
    TruncatedHeader { actual: usize },
    /// The bytes do not identify an NMS Copilot cache.
    InvalidMagic,
    /// The archive layout revision is not supported.
    SchemaMismatch { found: u32, expected: u32 },
    /// The meaning of the cached model is not supported.
    SemanticMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for CacheCompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(
                f,
                "cache compatibility header is truncated: got {actual} bytes, expected at least 16"
            ),
            Self::InvalidMagic => write!(f, "cache compatibility header has invalid magic"),
            Self::SchemaMismatch { found, expected } => write!(
                f,
                "cache schema revision {found} is incompatible (expected {expected})"
            ),
            Self::SemanticMismatch { found, expected } => write!(
                f,
                "cache model semantic revision {found} is incompatible (expected {expected})"
            ),
        }
    }
}

impl std::error::Error for CacheCompatibilityError {}

#[derive(Debug)]
pub enum CacheError {
    /// rkyv serialization failed.
    Serialize(String),
    /// rkyv deserialization failed.
    Deserialize(String),
    /// File I/O error.
    Io(io::Error),
    /// The cache envelope uses an unsupported format or model interpretation.
    Incompatible(CacheCompatibilityError),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "cache serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "cache deserialization error: {e}"),
            Self::Io(e) => write!(f, "cache I/O error: {e}"),
            Self::Incompatible(e) => write!(f, "cache incompatibility: {e}"),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Incompatible(e) => Some(e),
            _ => None,
        }
    }
}
