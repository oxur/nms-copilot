//! Zero-copy serialization cache for NMS Copilot.
//!
//! Serializes the in-memory `GalaxyModel` discovery data to an rkyv archive
//! for near-instant startup on subsequent runs. Indices are rebuilt on load.

pub mod data;
pub mod error;
pub mod freshness;
pub mod serialize;

pub use data::CacheData;
pub use error::CacheError;
pub use freshness::{is_cache_fresh, load_or_rebuild};
pub use serialize::{extract_cache_data, read_cache, rebuild_model, write_cache};
