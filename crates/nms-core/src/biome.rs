//! Biome classifications and the known biome-subtype names.
//!
//! Parsing is case-insensitive and matches the listed full names and aliases.
//! Inputs are not trimmed. Unknown input is retained in [BiomeParseError].
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use nms_core::{Biome, BiomeSubType};
//!
//! assert_eq!("GAS_GIANT".parse::<Biome>()?, Biome::GasGiant);
//! assert_eq!("ocean".parse::<BiomeSubType>()?, BiomeSubType::WaterworldOcean);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// All 15 biome variants, for iteration.
pub const ALL_BIOMES: [Biome; 15] = [
    Biome::Lush,
    Biome::Toxic,
    Biome::Scorched,
    Biome::Radioactive,
    Biome::Frozen,
    Biome::Barren,
    Biome::Dead,
    Biome::Weird,
    Biome::Red,
    Biome::Green,
    Biome::Blue,
    Biome::Swamp,
    Biome::Lava,
    Biome::Waterworld,
    Biome::GasGiant,
];

/// All 31 biome subtype variants, for iteration.
pub const ALL_BIOME_SUBTYPES: [BiomeSubType; 31] = [
    BiomeSubType::LushRoomTemp,
    BiomeSubType::LushHumid,
    BiomeSubType::LushInactive,
    BiomeSubType::ToxicTentacles,
    BiomeSubType::ToxicFungus,
    BiomeSubType::ToxicEggs,
    BiomeSubType::ScorchedSinged,
    BiomeSubType::ScorchedCharred,
    BiomeSubType::ScorchedBlasted,
    BiomeSubType::RadioactiveFungal,
    BiomeSubType::RadioactiveContaminated,
    BiomeSubType::RadioactiveIrradiated,
    BiomeSubType::FrozenIce,
    BiomeSubType::FrozenSnow,
    BiomeSubType::FrozenGlacial,
    BiomeSubType::BarrenDusty,
    BiomeSubType::BarrenRocky,
    BiomeSubType::BarrenMountainous,
    BiomeSubType::DeadEmpty,
    BiomeSubType::DeadCorroded,
    BiomeSubType::DeadVoid,
    BiomeSubType::WeirdHexagonal,
    BiomeSubType::WeirdCabled,
    BiomeSubType::WeirdBubbling,
    BiomeSubType::WeirdFractured,
    BiomeSubType::WeirdShattered,
    BiomeSubType::WeirdContorted,
    BiomeSubType::WeirdWireCell,
    BiomeSubType::SwampMurky,
    BiomeSubType::LavaVolcanic,
    BiomeSubType::WaterworldOcean,
];

/// Planet biome classification matching GcBiomeType from game data.
///
/// The parser accepts the variant names without regard to case, plus
/// gasgiant and gas_giant for [Biome::GasGiant]. It does not trim whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum Biome {
    /// Lush planet biome.
    Lush,
    /// Toxic planet biome.
    Toxic,
    /// Scorched planet biome.
    Scorched,
    /// Radioactive planet biome.
    Radioactive,
    /// Frozen planet biome.
    Frozen,
    /// Barren planet biome.
    Barren,
    /// Dead planet biome.
    Dead,
    /// Weird planet biome.
    Weird,
    /// Red planet biome.
    Red,
    /// Green planet biome.
    Green,
    /// Blue planet biome.
    Blue,
    /// Swamp planet biome.
    Swamp,
    /// Lava planet biome.
    Lava,
    /// Waterworld planet biome.
    Waterworld,
    /// Gas-giant biome classification.
    GasGiant,
}

impl fmt::Display for Biome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Lush => "Lush",
            Self::Toxic => "Toxic",
            Self::Scorched => "Scorched",
            Self::Radioactive => "Radioactive",
            Self::Frozen => "Frozen",
            Self::Barren => "Barren",
            Self::Dead => "Dead",
            Self::Weird => "Weird",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Swamp => "Swamp",
            Self::Lava => "Lava",
            Self::Waterworld => "Waterworld",
            Self::GasGiant => "GasGiant",
        };
        write!(f, "{s}")
    }
}

/// Error returned when parsing a biome or biome subtype string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeParseError(
    /// The original string that did not match a biome or subtype.
    pub String,
);

impl fmt::Display for BiomeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown biome: {}", self.0)
    }
}

impl std::error::Error for BiomeParseError {}

/// Parse a biome name without regard to case.
///
/// Accepted names are the variant spellings, with `gasgiant` and `gas_giant`
/// both accepted for [Biome::GasGiant]. Input is not trimmed.
///
/// # Errors
///
/// Returns [BiomeParseError] containing the original input when no name matches.
impl FromStr for Biome {
    type Err = BiomeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lush" => Ok(Self::Lush),
            "toxic" => Ok(Self::Toxic),
            "scorched" => Ok(Self::Scorched),
            "radioactive" => Ok(Self::Radioactive),
            "frozen" => Ok(Self::Frozen),
            "barren" => Ok(Self::Barren),
            "dead" => Ok(Self::Dead),
            "weird" => Ok(Self::Weird),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "swamp" => Ok(Self::Swamp),
            "lava" => Ok(Self::Lava),
            "waterworld" => Ok(Self::Waterworld),
            "gasgiant" | "gas_giant" => Ok(Self::GasGiant),
            _ => Err(BiomeParseError(s.to_string())),
        }
    }
}

/// Planet biome subtype matching GcBiomeSubType from game data (31 variants).
///
/// Parsing accepts full variant names without regard to case and also accepts
/// the unique suffixes, such as ice or ocean. Inputs are not trimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum BiomeSubType {
    /// Room-temperature Lush subtype.
    LushRoomTemp,
    /// Humid Lush subtype.
    LushHumid,
    /// Inactive Lush subtype.
    LushInactive,
    /// Tentacles Toxic subtype.
    ToxicTentacles,
    /// Fungus Toxic subtype.
    ToxicFungus,
    /// Eggs Toxic subtype.
    ToxicEggs,
    /// Singed Scorched subtype.
    ScorchedSinged,
    /// Charred Scorched subtype.
    ScorchedCharred,
    /// Blasted Scorched subtype.
    ScorchedBlasted,
    /// Fungal Radioactive subtype.
    RadioactiveFungal,
    /// Contaminated Radioactive subtype.
    RadioactiveContaminated,
    /// Irradiated Radioactive subtype.
    RadioactiveIrradiated,
    /// Ice Frozen subtype.
    FrozenIce,
    /// Snow Frozen subtype.
    FrozenSnow,
    /// Glacial Frozen subtype.
    FrozenGlacial,
    /// Dusty Barren subtype.
    BarrenDusty,
    /// Rocky Barren subtype.
    BarrenRocky,
    /// Mountainous Barren subtype.
    BarrenMountainous,
    /// Empty Dead subtype.
    DeadEmpty,
    /// Corroded Dead subtype.
    DeadCorroded,
    /// Void Dead subtype.
    DeadVoid,
    /// Hexagonal Weird subtype.
    WeirdHexagonal,
    /// Cabled Weird subtype.
    WeirdCabled,
    /// Bubbling Weird subtype.
    WeirdBubbling,
    /// Fractured Weird subtype.
    WeirdFractured,
    /// Shattered Weird subtype.
    WeirdShattered,
    /// Contorted Weird subtype.
    WeirdContorted,
    /// Wire-cell Weird subtype.
    WeirdWireCell,
    /// Murky Swamp subtype.
    SwampMurky,
    /// Volcanic Lava subtype.
    LavaVolcanic,
    /// Ocean Waterworld subtype.
    WaterworldOcean,
}

impl fmt::Display for BiomeSubType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Parse a biome subtype name or its unique suffix without regard to case.
///
/// Full variant names and the suffix aliases in the parser are accepted. Input
/// is not trimmed.
///
/// # Errors
///
/// Returns [BiomeParseError] containing the original input when no name or
/// suffix matches.
impl FromStr for BiomeSubType {
    type Err = BiomeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lushroomtemp" => Ok(Self::LushRoomTemp),
            "lushhumid" => Ok(Self::LushHumid),
            "lushinactive" => Ok(Self::LushInactive),
            "toxictentacles" => Ok(Self::ToxicTentacles),
            "toxicfungus" => Ok(Self::ToxicFungus),
            "toxiceggs" => Ok(Self::ToxicEggs),
            "scorchedsinged" => Ok(Self::ScorchedSinged),
            "scorchedcharred" => Ok(Self::ScorchedCharred),
            "scorchedblasted" => Ok(Self::ScorchedBlasted),
            "radioactivefungal" => Ok(Self::RadioactiveFungal),
            "radioactivecontaminated" => Ok(Self::RadioactiveContaminated),
            "radioactiveirradiated" => Ok(Self::RadioactiveIrradiated),
            "frozenice" => Ok(Self::FrozenIce),
            "frozensnow" => Ok(Self::FrozenSnow),
            "frozenglacial" => Ok(Self::FrozenGlacial),
            "barrendusty" => Ok(Self::BarrenDusty),
            "barrenrocky" => Ok(Self::BarrenRocky),
            "barrenmountainous" => Ok(Self::BarrenMountainous),
            "deadempty" => Ok(Self::DeadEmpty),
            "deadcorroded" => Ok(Self::DeadCorroded),
            "deadvoid" => Ok(Self::DeadVoid),
            "weirdhexagonal" => Ok(Self::WeirdHexagonal),
            "weirdcabled" => Ok(Self::WeirdCabled),
            "weirdbubbling" => Ok(Self::WeirdBubbling),
            "weirdfractured" => Ok(Self::WeirdFractured),
            "weirdshattered" => Ok(Self::WeirdShattered),
            "weirdcontorted" => Ok(Self::WeirdContorted),
            "weirdwirecell" => Ok(Self::WeirdWireCell),
            "swampmurky" => Ok(Self::SwampMurky),
            "lavavolcanic" => Ok(Self::LavaVolcanic),
            "waterworldocean" => Ok(Self::WaterworldOcean),

            // Suffix-only matches (all suffixes are unique across biome subtypes)
            "roomtemp" => Ok(Self::LushRoomTemp),
            "humid" => Ok(Self::LushHumid),
            "inactive" => Ok(Self::LushInactive),
            "tentacles" => Ok(Self::ToxicTentacles),
            "fungus" => Ok(Self::ToxicFungus),
            "eggs" => Ok(Self::ToxicEggs),
            "singed" => Ok(Self::ScorchedSinged),
            "charred" => Ok(Self::ScorchedCharred),
            "blasted" => Ok(Self::ScorchedBlasted),
            "fungal" => Ok(Self::RadioactiveFungal),
            "contaminated" => Ok(Self::RadioactiveContaminated),
            "irradiated" => Ok(Self::RadioactiveIrradiated),
            "ice" => Ok(Self::FrozenIce),
            "snow" => Ok(Self::FrozenSnow),
            "glacial" => Ok(Self::FrozenGlacial),
            "dusty" => Ok(Self::BarrenDusty),
            "rocky" => Ok(Self::BarrenRocky),
            "mountainous" => Ok(Self::BarrenMountainous),
            "empty" => Ok(Self::DeadEmpty),
            "corroded" => Ok(Self::DeadCorroded),
            "void" => Ok(Self::DeadVoid),
            "hexagonal" => Ok(Self::WeirdHexagonal),
            "cabled" => Ok(Self::WeirdCabled),
            "bubbling" => Ok(Self::WeirdBubbling),
            "fractured" => Ok(Self::WeirdFractured),
            "shattered" => Ok(Self::WeirdShattered),
            "contorted" => Ok(Self::WeirdContorted),
            "wirecell" => Ok(Self::WeirdWireCell),
            "murky" => Ok(Self::SwampMurky),
            "volcanic" => Ok(Self::LavaVolcanic),
            "ocean" => Ok(Self::WaterworldOcean),

            _ => Err(BiomeParseError(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_fromstr_roundtrip() {
        for biome in ALL_BIOMES {
            let s = biome.to_string();
            let parsed: Biome = s.parse().unwrap();
            assert_eq!(biome, parsed);
        }
    }

    #[test]
    fn case_insensitive_parse() {
        assert_eq!("lush".parse::<Biome>().unwrap(), Biome::Lush);
        assert_eq!("LUSH".parse::<Biome>().unwrap(), Biome::Lush);
        assert_eq!("Lush".parse::<Biome>().unwrap(), Biome::Lush);
    }

    #[test]
    fn gas_giant_alternate_name() {
        assert_eq!("gas_giant".parse::<Biome>().unwrap(), Biome::GasGiant);
    }

    #[test]
    fn unknown_biome_error() {
        assert!("NotABiome".parse::<Biome>().is_err());
    }

    #[test]
    fn subtype_display_fromstr_roundtrip() {
        for v in ALL_BIOME_SUBTYPES {
            let s = v.to_string();
            let parsed: BiomeSubType = s.parse().unwrap();
            assert_eq!(v, parsed);
        }
    }

    #[test]
    fn test_subtype_suffix_only_parse() {
        // Every suffix-only name should resolve to the correct variant
        let cases = [
            ("rocky", BiomeSubType::BarrenRocky),
            ("Rocky", BiomeSubType::BarrenRocky),
            ("ROCKY", BiomeSubType::BarrenRocky),
            ("roomtemp", BiomeSubType::LushRoomTemp),
            ("humid", BiomeSubType::LushHumid),
            ("inactive", BiomeSubType::LushInactive),
            ("tentacles", BiomeSubType::ToxicTentacles),
            ("fungus", BiomeSubType::ToxicFungus),
            ("eggs", BiomeSubType::ToxicEggs),
            ("singed", BiomeSubType::ScorchedSinged),
            ("charred", BiomeSubType::ScorchedCharred),
            ("blasted", BiomeSubType::ScorchedBlasted),
            ("fungal", BiomeSubType::RadioactiveFungal),
            ("contaminated", BiomeSubType::RadioactiveContaminated),
            ("irradiated", BiomeSubType::RadioactiveIrradiated),
            ("ice", BiomeSubType::FrozenIce),
            ("snow", BiomeSubType::FrozenSnow),
            ("glacial", BiomeSubType::FrozenGlacial),
            ("dusty", BiomeSubType::BarrenDusty),
            ("mountainous", BiomeSubType::BarrenMountainous),
            ("empty", BiomeSubType::DeadEmpty),
            ("corroded", BiomeSubType::DeadCorroded),
            ("void", BiomeSubType::DeadVoid),
            ("hexagonal", BiomeSubType::WeirdHexagonal),
            ("cabled", BiomeSubType::WeirdCabled),
            ("bubbling", BiomeSubType::WeirdBubbling),
            ("fractured", BiomeSubType::WeirdFractured),
            ("shattered", BiomeSubType::WeirdShattered),
            ("contorted", BiomeSubType::WeirdContorted),
            ("wirecell", BiomeSubType::WeirdWireCell),
            ("murky", BiomeSubType::SwampMurky),
            ("volcanic", BiomeSubType::LavaVolcanic),
            ("ocean", BiomeSubType::WaterworldOcean),
        ];
        for (input, expected) in cases {
            let parsed: BiomeSubType = input
                .parse()
                .unwrap_or_else(|_| panic!("failed to parse suffix '{input}'"));
            assert_eq!(
                parsed, expected,
                "suffix '{input}' should parse to {expected:?}"
            );
        }
    }

    #[test]
    fn test_subtype_unknown_suffix_error() {
        assert!("notasubtype".parse::<BiomeSubType>().is_err());
    }
}
