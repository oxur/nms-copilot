//! Core types, enums, address math, and portal glyph display for NMS Copilot.

pub mod address;
pub mod biome;
pub mod discovery;
pub mod galaxy;
pub mod glyph;
pub mod player;
pub mod system;

pub use address::{AddressParseError, GalacticAddress, PortalAddress, PortalParseError};
pub use biome::{Biome, BiomeParseError, BiomeSubType};
pub use discovery::{Discovery, DiscoveryParseError, DiscoveryRecord};
pub use galaxy::{Galaxy, GalaxyType, GalaxyTypeParseError};
pub use glyph::{Glyph, GlyphParseError};
pub use player::{BaseType, BaseTypeParseError, PlayerBase, PlayerState};
pub use system::{Planet, System};
