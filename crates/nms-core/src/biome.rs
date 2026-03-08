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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum Biome {
    Lush,
    Toxic,
    Scorched,
    Radioactive,
    Frozen,
    Barren,
    Dead,
    Weird,
    Red,
    Green,
    Blue,
    Swamp,
    Lava,
    Waterworld,
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
pub struct BiomeParseError(pub String);

impl fmt::Display for BiomeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown biome: {}", self.0)
    }
}

impl std::error::Error for BiomeParseError {}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[non_exhaustive]
pub enum BiomeSubType {
    LushRoomTemp,
    LushHumid,
    LushInactive,
    ToxicTentacles,
    ToxicFungus,
    ToxicEggs,
    ScorchedSinged,
    ScorchedCharred,
    ScorchedBlasted,
    RadioactiveFungal,
    RadioactiveContaminated,
    RadioactiveIrradiated,
    FrozenIce,
    FrozenSnow,
    FrozenGlacial,
    BarrenDusty,
    BarrenRocky,
    BarrenMountainous,
    DeadEmpty,
    DeadCorroded,
    DeadVoid,
    WeirdHexagonal,
    WeirdCabled,
    WeirdBubbling,
    WeirdFractured,
    WeirdShattered,
    WeirdContorted,
    WeirdWireCell,
    SwampMurky,
    LavaVolcanic,
    WaterworldOcean,
}

impl fmt::Display for BiomeSubType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

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
}
