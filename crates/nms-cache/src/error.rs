//! Errors returned by cache envelope, payload, and filesystem operations.
//!
//! Envelope incompatibilities are typed separately from serialization,
//! checked-deserialization, and filesystem failures. The compatibility enum is
//! non-exhaustive, so downstream matches should include a wildcard.
//! `CacheError::Io` and `CacheError::Incompatible` expose their nested error
//! as a source; the string-bearing serialization and deserialization variants
//! do not retain an underlying error object.
//!
//! # Examples
//!
//! ```
//! use nms_cache::{CacheCompatibilityError, CacheError};
//! use nms_cache::serialize::deserialize;
//!
//! assert!(matches!(deserialize(b""), Err(CacheError::Incompatible(
//!     CacheCompatibilityError::TruncatedHeader { actual: 0 }
//! ))));
//! assert!(matches!(deserialize(&[0; 16]), Err(CacheError::Incompatible(
//!     CacheCompatibilityError::InvalidMagic
//! ))));
//! let schema_three = b"NMSCACHE\x03\x00\x00\x00\x05\x00\x00\x00";
//! assert!(matches!(deserialize(schema_three), Err(CacheError::Incompatible(
//!     CacheCompatibilityError::SchemaMismatch { found: 3, expected: 4 }
//! ))));
//! let semantic_five = b"NMSCACHE\x04\x00\x00\x00\x05\x00\x00\x00";
//! assert!(matches!(deserialize(semantic_five), Err(CacheError::Incompatible(
//!     CacheCompatibilityError::SemanticMismatch { found: 5, expected: 6 }
//! ))));
//! let current_empty_payload = b"NMSCACHE\x04\x00\x00\x00\x06\x00\x00\x00";
//! assert!(matches!(deserialize(current_empty_payload), Err(CacheError::Deserialize(_))));
//! ```
//!
//! These are synthetic header controls, not archived witnesses from previous
//! releases. A current header without a payload passes envelope checks and
//! fails during payload decoding.

use std::io;

/// Why an on-disk cache envelope cannot be interpreted by this version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheCompatibilityError {
    /// The cache does not contain a complete compatibility header.
    TruncatedHeader {
        /// Number of bytes supplied; a complete envelope requires 16.
        actual: usize,
    },
    /// The bytes do not identify an NMS Copilot cache.
    InvalidMagic,
    /// The archive layout revision is not supported.
    SchemaMismatch {
        /// Archive schema revision read from the envelope.
        found: u32,
        /// Archive schema revision supported by this reader.
        expected: u32,
    },
    /// The meaning of the cached model is not supported.
    SemanticMismatch {
        /// Model semantic revision read from the envelope.
        found: u32,
        /// Model semantic revision supported by this reader.
        expected: u32,
    },
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

/// Errors from cache encoding, decoding, and filesystem operations.
#[derive(Debug)]
pub enum CacheError {
    /// Cache-data validation or rkyv serialization failed.
    Serialize(
        /// Encoding or pre-serialization cache-data validation detail.
        String,
    ),
    /// Checked payload deserialization or cache-data validation failed.
    Deserialize(
        /// Decoding or post-decode cache-data validation detail.
        String,
    ),
    /// File I/O error.
    Io(
        /// Underlying filesystem error.
        io::Error,
    ),
    /// The cache envelope uses an unsupported format or model interpretation.
    Incompatible(
        /// Typed envelope incompatibility, before payload decoding.
        CacheCompatibilityError,
    ),
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
