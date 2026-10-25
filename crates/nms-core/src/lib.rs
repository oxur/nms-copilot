//! Core value types shared by the NMS Copilot crates.
//!
//! This crate models legacy portal coordinates, galaxy-qualified system keys,
//! player and discovery records, biome classifications, and additive save
//! deltas. It does not parse or edit save files; those operations belong to
//! higher-level crates.
//!
//! # Quick start
//!
//! Build a system key from a coordinate, then use the bundled galaxy and
//! parsers to interpret it:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use nms_core::{Biome, GalacticAddress, Galaxy, Glyph, SystemId};
//!
//! let address = GalacticAddress::new(3, 0, 0, 1, 2, 9);
//! let system = SystemId::from_address(&address);
//! assert_eq!(system.reality_index(), 9);
//! assert_eq!(Galaxy::by_index(system.reality_index()).name, "Eissentam");
//! assert_eq!("lush".parse::<Biome>()?, Biome::Lush);
//! assert_eq!("rckt".parse::<Glyph>()?.index(), 13);
//! # Ok(())
//! # }
//! ```
//!
//! # Modules
//!
//! - [address] packs coordinates and converts portal and signal-booster forms.
//! - [glyph] defines the sixteen portal glyphs and prefix parser.
//! - [biome] and [galaxy] contain bundled classifications and lookups.
//! - [discovery] and [player] model observations and player state.
//! - [system] defines planets, systems, and galaxy-qualified [SystemId] values.
//! - [delta] contains additive changes and runtime discovery-name observations.
//!
//! The APIs intentionally retain legacy behavior described on their items:
//! constructors may mask or copy supplied values rather than validate or
//! reconcile them.

pub mod address;
pub mod biome;
pub mod delta;
pub mod discovery;
pub mod galaxy;
pub mod glyph;
pub mod player;
pub mod system;

pub use address::{AddressParseError, GalacticAddress, PortalAddress, PortalParseError};
pub use biome::{Biome, BiomeParseError, BiomeSubType};
pub use delta::{PlayerMoved, SaveDelta, SaveUpdate};
pub use discovery::{Discovery, DiscoveryParseError, DiscoveryRecord};
pub use galaxy::{Galaxy, GalaxyType, GalaxyTypeParseError};
pub use glyph::{Glyph, GlyphParseError};
pub use player::{BaseType, BaseTypeParseError, PlayerBase, PlayerState};
pub use system::{DiscoverySource, Planet, System, SystemId};
