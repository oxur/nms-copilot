use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub struct GlyphInfo {
    pub index: u8,
    pub hex_char: char,
    pub name: &'static str,
    pub emoji: &'static str,
    pub abbrev: &'static str,
}

/// All 16 portal glyph definitions, indexed by glyph value (0-15).
pub const GLYPH_TABLE: [GlyphInfo; 16] = [
    GlyphInfo {
        index: 0,
        hex_char: '0',
        name: "Sunset",
        emoji: "\u{1F305}",
        abbrev: "sset",
    },
    GlyphInfo {
        index: 1,
        hex_char: '1',
        name: "Bird",
        emoji: "\u{1F54A}",
        abbrev: "bird",
    },
    GlyphInfo {
        index: 2,
        hex_char: '2',
        name: "Face",
        emoji: "\u{1F611}",
        abbrev: "face",
    },
    GlyphInfo {
        index: 3,
        hex_char: '3',
        name: "Diplo",
        emoji: "\u{1F995}",
        abbrev: "dipl",
    },
    GlyphInfo {
        index: 4,
        hex_char: '4',
        name: "Eclipse",
        emoji: "\u{1F31C}",
        abbrev: "eclp",
    },
    GlyphInfo {
        index: 5,
        hex_char: '5',
        name: "Balloon",
        emoji: "\u{1F388}",
        abbrev: "blln",
    },
    GlyphInfo {
        index: 6,
        hex_char: '6',
        name: "Boat",
        emoji: "\u{26F5}",
        abbrev: "boat",
    },
    GlyphInfo {
        index: 7,
        hex_char: '7',
        name: "Bug",
        emoji: "\u{1F41C}",
        abbrev: "abug",
    },
    GlyphInfo {
        index: 8,
        hex_char: '8',
        name: "Dragonfly",
        emoji: "\u{1F98B}",
        abbrev: "dfly",
    },
    GlyphInfo {
        index: 9,
        hex_char: '9',
        name: "Galaxy",
        emoji: "\u{1F300}",
        abbrev: "glxy",
    },
    GlyphInfo {
        index: 10,
        hex_char: 'A',
        name: "Voxel",
        emoji: "\u{1F54B}",
        abbrev: "voxl",
    },
    GlyphInfo {
        index: 11,
        hex_char: 'B',
        name: "Whale",
        emoji: "\u{1F40B}",
        abbrev: "whle",
    },
    GlyphInfo {
        index: 12,
        hex_char: 'C',
        name: "Tent",
        emoji: "\u{26FA}",
        abbrev: "tent",
    },
    GlyphInfo {
        index: 13,
        hex_char: 'D',
        name: "Rocket",
        emoji: "\u{1F680}",
        abbrev: "rckt",
    },
    GlyphInfo {
        index: 14,
        hex_char: 'E',
        name: "Tree",
        emoji: "\u{1F333}",
        abbrev: "tree",
    },
    GlyphInfo {
        index: 15,
        hex_char: 'F',
        name: "Atlas",
        emoji: "\u{1F53A}",
        abbrev: "atls",
    },
];

/// A single portal glyph (value 0-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Glyph(u8);

/// Error returned when parsing a glyph string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GlyphParseError(pub String);

impl fmt::Display for GlyphParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid glyph: {}", self.0)
    }
}

impl std::error::Error for GlyphParseError {}

impl Glyph {
    /// Create from index 0-15. Panics if out of range.
    pub fn new(index: u8) -> Self {
        assert!(index < 16, "glyph index must be 0-15, got {index}");
        Self(index)
    }

    /// Try to create from index. Returns None if >= 16.
    pub fn try_new(index: u8) -> Option<Self> {
        if index < 16 { Some(Self(index)) } else { None }
    }

    /// The numeric index (0-15).
    pub fn index(self) -> u8 {
        self.0
    }

    /// The hex character ('0'-'9', 'A'-'F').
    pub fn hex_char(self) -> char {
        GLYPH_TABLE[self.0 as usize].hex_char
    }

    /// The glyph name ("Sunset", "Bird", etc.).
    pub fn name(self) -> &'static str {
        GLYPH_TABLE[self.0 as usize].name
    }

    /// The emoji string for this glyph.
    pub fn emoji(self) -> &'static str {
        GLYPH_TABLE[self.0 as usize].emoji
    }

    /// The 4-character abbreviation for this glyph (e.g., "sset", "bird").
    pub fn abbrev(self) -> &'static str {
        GLYPH_TABLE[self.0 as usize].abbrev
    }
}

impl From<u8> for Glyph {
    /// Panics if value >= 16.
    fn from(v: u8) -> Self {
        Self::new(v)
    }
}

impl From<Glyph> for u8 {
    fn from(g: Glyph) -> u8 {
        g.0
    }
}

/// Display as the hex character.
impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex_char())
    }
}

/// Parse from a single hex character, an emoji, or a glyph name (case-insensitive).
impl FromStr for Glyph {
    type Err = GlyphParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(GlyphParseError(s.to_string()));
        }

        // Try single hex character
        if s.len() == 1 {
            if let Some(v) = s.chars().next().and_then(|c| c.to_digit(16)) {
                return Ok(Self(v as u8));
            }
        }

        // Try emoji match (strip trailing variation selector U+FE0F for comparison)
        let stripped = s.trim_end_matches('\u{FE0F}');
        for info in &GLYPH_TABLE {
            let table_stripped = info.emoji.trim_end_matches('\u{FE0F}');
            if stripped == table_stripped || s == info.emoji {
                return Ok(Self(info.index));
            }
        }

        // Try name match (case-insensitive)
        let lower = s.to_lowercase();
        for info in &GLYPH_TABLE {
            if info.name.to_lowercase() == lower {
                return Ok(Self(info.index));
            }
        }

        // Try abbreviation match (case-insensitive)
        for info in &GLYPH_TABLE {
            if info.abbrev == lower {
                return Ok(Self(info.index));
            }
        }

        Err(GlyphParseError(s.to_string()))
    }
}

/// Try to parse a single glyph from the start of `input`.
/// Returns `(Glyph, remaining_str)` on success.
///
/// Attempts in order:
/// 1. Emoji match (longest match first, to handle multi-codepoint emoji)
/// 2. Glyph name match (case-insensitive, longest name first)
/// 3. Abbreviation match (case-insensitive, 4-char abbreviations)
/// 4. Single hex digit
pub fn parse_next_glyph(input: &str) -> Result<(Glyph, &str), GlyphParseError> {
    if input.is_empty() {
        return Err(GlyphParseError("empty input".to_string()));
    }

    // Strip a leading colon separator (used in abbreviated format).
    let input = input.strip_prefix(':').unwrap_or(input);
    if input.is_empty() {
        return Err(GlyphParseError("empty input after separator".to_string()));
    }

    // 1. Try emoji match — check each glyph's emoji as a prefix
    //    Sort by emoji byte length descending to match longest first
    let mut by_len: Vec<(usize, &GlyphInfo)> =
        GLYPH_TABLE.iter().map(|g| (g.emoji.len(), g)).collect();
    by_len.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, info) in &by_len {
        if let Some(rest) = input.strip_prefix(info.emoji) {
            // Consume a trailing VS16 if present (input may include it even
            // though the table emoji does not).
            let rest = rest.strip_prefix('\u{FE0F}').unwrap_or(rest);
            return Ok((Glyph(info.index), rest));
        }
    }

    // 2. Try glyph name match (case-insensitive prefix, longest first)
    let mut names: Vec<&GlyphInfo> = GLYPH_TABLE.iter().collect();
    names.sort_by(|a, b| b.name.len().cmp(&a.name.len()));

    let input_lower = input.to_lowercase();
    for info in &names {
        let name_lower = info.name.to_lowercase();
        if input_lower.starts_with(&name_lower) {
            return Ok((Glyph(info.index), &input[info.name.len()..]));
        }
    }

    // 3. Try abbreviation match (case-insensitive, all 4 chars)
    if input_lower.len() >= 4 {
        for info in &GLYPH_TABLE {
            if input_lower.starts_with(info.abbrev) {
                return Ok((Glyph(info.index), &input[info.abbrev.len()..]));
            }
        }
    }

    // 4. Try single hex digit
    let c = input.chars().next().unwrap();
    if let Some(v) = c.to_digit(16) {
        return Ok((Glyph(v as u8), &input[c.len_utf8()..]));
    }

    Err(GlyphParseError(format!(
        "unrecognized glyph at: {}",
        &input[..input.len().min(20)]
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_16_glyphs_index_roundtrip() {
        for i in 0..16u8 {
            let g = Glyph::new(i);
            assert_eq!(g.index(), i);
        }
    }

    #[test]
    fn all_16_glyphs_hex_roundtrip() {
        let hex_chars = "0123456789ABCDEF";
        for (i, c) in hex_chars.chars().enumerate() {
            let g = Glyph::new(i as u8);
            assert_eq!(g.hex_char(), c);
            let parsed: Glyph = c.to_string().parse().unwrap();
            assert_eq!(parsed.index(), i as u8);
        }
    }

    #[test]
    fn all_16_glyphs_emoji_roundtrip() {
        for i in 0..16u8 {
            let g = Glyph::new(i);
            let emoji = g.emoji();
            let parsed: Glyph = emoji.parse().unwrap();
            assert_eq!(parsed.index(), i);
        }
    }

    #[test]
    fn all_16_glyphs_name_roundtrip() {
        for i in 0..16u8 {
            let g = Glyph::new(i);
            let name = g.name();
            let parsed: Glyph = name.parse().unwrap();
            assert_eq!(parsed.index(), i);
        }
    }

    #[test]
    fn name_case_insensitive() {
        assert_eq!("sunset".parse::<Glyph>().unwrap().index(), 0);
        assert_eq!("SUNSET".parse::<Glyph>().unwrap().index(), 0);
        assert_eq!("Sunset".parse::<Glyph>().unwrap().index(), 0);
    }

    #[test]
    fn bird_with_and_without_variation_selector() {
        let with_vs: Glyph = "\u{1F54A}\u{FE0F}".parse().unwrap();
        assert_eq!(with_vs.index(), 1);

        let without_vs: Glyph = "\u{1F54A}".parse().unwrap();
        assert_eq!(without_vs.index(), 1);
    }

    #[test]
    fn hex_lowercase() {
        assert_eq!("a".parse::<Glyph>().unwrap().index(), 10);
        assert_eq!("f".parse::<Glyph>().unwrap().index(), 15);
    }

    #[test]
    fn invalid_glyph_errors() {
        assert!("X".parse::<Glyph>().is_err());
        assert!("hello".parse::<Glyph>().is_err());
        assert!("".parse::<Glyph>().is_err());
    }

    #[test]
    fn parse_next_glyph_emoji_sequence() {
        let input = "\u{1F305}\u{1F54A}\u{FE0F}\u{1F41C}"; // Sunset Bird Bug
        let (g1, rest) = parse_next_glyph(input).unwrap();
        assert_eq!(g1.index(), 0); // Sunset
        let (g2, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g2.index(), 1); // Bird
        let (g3, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g3.index(), 7); // Bug
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_next_glyph_hex() {
        let (g, rest) = parse_next_glyph("A5").unwrap();
        assert_eq!(g.index(), 10); // A
        let (g2, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g2.index(), 5); // 5
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_next_glyph_name() {
        let (g, rest) = parse_next_glyph("DragonflySunset").unwrap();
        assert_eq!(g.index(), 8); // Dragonfly
        let (g2, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g2.index(), 0); // Sunset
        assert!(rest.is_empty());
    }

    #[test]
    fn test_all_16_glyphs_abbrev_roundtrip() {
        let expected_abbrevs = [
            "sset", "bird", "face", "dipl", "eclp", "blln", "boat", "abug", "dfly", "glxy", "voxl",
            "whle", "tent", "rckt", "tree", "atls",
        ];
        for (i, expected) in expected_abbrevs.iter().enumerate() {
            let g = Glyph::new(i as u8);
            assert_eq!(g.abbrev(), *expected, "abbrev mismatch for index {i}");
            let parsed: Glyph = expected.parse().unwrap();
            assert_eq!(
                parsed.index(),
                i as u8,
                "roundtrip failed for abbrev \"{expected}\""
            );
        }
    }

    #[test]
    fn test_abbrev_case_insensitive() {
        assert_eq!("sset".parse::<Glyph>().unwrap().index(), 0);
        assert_eq!("SSET".parse::<Glyph>().unwrap().index(), 0);
        assert_eq!("Sset".parse::<Glyph>().unwrap().index(), 0);
    }

    #[test]
    fn test_parse_next_glyph_abbrev_sequence() {
        let input = "ssetbirdface";
        let (g1, rest) = parse_next_glyph(input).unwrap();
        assert_eq!(g1.index(), 0); // sset = Sunset
        let (g2, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g2.index(), 1); // bird = Bird
        let (g3, rest) = parse_next_glyph(rest).unwrap();
        assert_eq!(g3.index(), 2); // face = Face
        assert!(rest.is_empty());
    }
}
