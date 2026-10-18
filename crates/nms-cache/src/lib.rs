//! Versioned, checked serialization cache for NMS Copilot.
//!
//! Cache files contain a 16-byte envelope followed by an rkyv `CacheData`
//! archive. Bytes 0–7 are the ASCII magic `NMSCACHE`, bytes 8–11 are the
//! little-endian archive schema revision, and bytes 12–15 are the little-endian
//! model semantic revision. The checked payload begins at byte 16.
//! The current schema and semantic revision values are both 2.
//!
//! Readers require both revisions to equal the values they support. Headerless
//! legacy caches and caches with mismatched revisions are rejected. The schema
//! revision tracks archive layout, codec, and configuration compatibility;
//! the semantic revision tracks the meaning and identity of decoded model data.
//! A single repair may require changing both. These revisions are independent
//! of the game's `SaveRoot.version`, the crate version, and `CacheData::cached_at`.
//!
//! Schema 2 adds the independently selected galaxy to `CacheData`; semantic 2
//! stores the galaxy-qualified system identity and restores that selection.
//! Older 1/1 archives are rejected and rebuilt from the source save. Public
//! `CacheData` struct literals must now provide `active_galaxy`. These revisions
//! do not establish game-format authority or universal nonzero-galaxy
//! extraction correctness. Future semantic repairs must reject their
//! predecessor cache, rebuild from source, and demonstrate a compatible warm
//! load.
//!
//! [`is_cache_fresh`] checks modification times only; it does not validate the
//! envelope or model semantics. With caching enabled, [`load_or_rebuild`]
//! checks a fresh cache and, when reading it fails, reparses the source and
//! writes a current envelope.
//! Persisted player state is accepted only when its duplicated physical reality
//! fields agree: `current_reality` must equal
//! `current_address.reality_index`. The independently selected galaxy, previous
//! address, and freighter address are not constrained by this check. The cache
//! reader rejects contradictory state and lets the existing loader rebuild from
//! the source save; it never guesses which cached field should be corrected.
//! Direct [`serialize::deserialize`] and [`serialize::read_cache`] calls return
//! errors for legacy or incompatible caches and do not rebuild them.
//! Direct [`rebuild_model`] calls are infallible and consume caller-supplied
//! [`CacheData`] without persisted-byte validation or source fallback; callers
//! constructing that data are responsible for its consistency. The checked
//! persisted-data entrypoints validate their input, while [`load_or_rebuild`]
//! additionally reparses the source save after a cache read failure.
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
