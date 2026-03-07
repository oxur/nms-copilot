//! Cache error types.

use std::io;

#[derive(Debug)]
pub enum CacheError {
    /// rkyv serialization failed.
    Serialize(String),
    /// rkyv deserialization failed.
    Deserialize(String),
    /// File I/O error.
    Io(io::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "cache serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "cache deserialization error: {e}"),
            Self::Io(e) => write!(f, "cache I/O error: {e}"),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
