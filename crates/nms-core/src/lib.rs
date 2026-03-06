//! Core types, enums, address math, and portal glyph display for NMS Copilot.
//!
//! This crate provides the foundational types used by all other NMS Copilot crates:
//!
//! - `GalacticAddress` — 48-bit packed coordinates with bidirectional conversion
//! - Portal glyph system — hex, emoji, name, and index interconversion
//! - `Biome` / `BiomeSubType` enums matching game data
//! - `System`, `Planet`, `PlayerBase`, `PlayerState` models
//! - Distance calculation (Euclidean voxel distance × 400 ly)
