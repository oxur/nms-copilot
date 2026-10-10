//! Versioned, checked serialization cache for NMS Copilot.
//!
//! Cache files contain a 16-byte envelope followed by an rkyv `CacheData`
//! archive. Bytes 0–7 are the ASCII magic `NMSCACHE`, bytes 8–11 are the
//! little-endian archive schema revision, and bytes 12–15 are the little-endian
//! model semantic revision. The checked payload begins at byte 16.
//! The current schema and semantic revision values are both 1.
//!
//! Readers require both revisions to equal the values they support. Headerless
//! legacy caches and caches with mismatched revisions are rejected. The schema
//! revision tracks archive layout, codec, and configuration compatibility;
//! the semantic revision tracks the meaning and identity of decoded model data.
//! A single repair may require changing both. These revisions are independent
//! of the game's `SaveRoot.version`, the crate version, and `CacheData::cached_at`.
//!
//! Revision 1 for each field labels the current cache interpretation. It does
//! not repair known address or galaxy-identity defects, establish game-format
//! authority, or make older model meanings safe to reuse. A later semantic
//! repair must reject its predecessor cache, rebuild from the source save, and
//! demonstrate a subsequent compatible warm load.
//!
//! [`is_cache_fresh`] checks modification times only; it does not validate the
//! envelope or model semantics. With caching enabled, [`load_or_rebuild`]
//! checks a fresh cache and, when reading it fails, reparses the source and
//! writes a current envelope.
//! Direct [`serialize::deserialize`] and [`serialize::read_cache`] calls return
//! errors for legacy or incompatible caches and do not rebuild them.
//!
//! Adding [`CacheError::Incompatible`] can require updates to external code
//! that exhaustively matches `CacheError`. The nested
//! [`CacheCompatibilityError`] is non-exhaustive. Public function signatures
//! remain unchanged, while serialized bytes and error matching have
//! compatibility consequences. This crate documentation does not claim a
//! release/version bump or universal compatibility guarantee.
//!
//! Cache decoding uses checked deserialization from owned, aligned payload
//! storage, and model indexes are reconstructed on load. No startup-speed or
//! zero-copy performance guarantee is made.

pub mod data;
pub mod error;
pub mod freshness;
pub mod serialize;

pub use data::CacheData;
pub use error::{CacheCompatibilityError, CacheError};
pub use freshness::{LoadResult, is_cache_fresh, load_or_rebuild};
pub use serialize::{extract_cache_data, read_cache, rebuild_model, write_cache};
