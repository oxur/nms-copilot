# Milestone 1.3 — Portal Glyph Converter (nms-core)

Bidirectional conversion between hex, emoji, name, index, and coordinate representations of portal glyphs. All code lives in the `nms-core` crate.

## File Organization

```
crates/nms-core/src/
  glyph.rs    — Glyph type, glyph table constant, single-glyph conversions
  address.rs  — Add PortalAddress type and GalacticAddress <-> PortalAddress conversions
  lib.rs      — Add `pub mod glyph;` and re-export `Glyph, PortalAddress`
```

---

## Glyph Table (constant data)

Define a static array of glyph metadata used for all lookups:

```rust
// In src/glyph.rs

pub(crate) struct GlyphInfo {
    pub index: u8,
    pub hex_char: char,    // '0'–'9', 'A'–'F'
    pub name: &'static str,
    pub emoji: &'static str,
}

pub(crate) const GLYPH_TABLE: [GlyphInfo; 16] = [
    GlyphInfo { index: 0,  hex_char: '0', name: "Sunset",    emoji: "\u{1F305}" },           // sunset
    GlyphInfo { index: 1,  hex_char: '1', name: "Bird",      emoji: "\u{1F54A}\u{FE0F}" },   // dove (bird) + variation selector
    GlyphInfo { index: 2,  hex_char: '2', name: "Face",      emoji: "\u{1F611}" },           // expressionless face
    GlyphInfo { index: 3,  hex_char: '3', name: "Diplo",     emoji: "\u{1F995}" },           // sauropod (diplo)
    GlyphInfo { index: 4,  hex_char: '4', name: "Eclipse",   emoji: "\u{1F31C}" },           // last quarter moon face
    GlyphInfo { index: 5,  hex_char: '5', name: "Balloon",   emoji: "\u{1F388}" },           // balloon
    GlyphInfo { index: 6,  hex_char: '6', name: "Boat",      emoji: "\u{26F5}" },            // sailboat
    GlyphInfo { index: 7,  hex_char: '7', name: "Bug",       emoji: "\u{1F41C}" },           // ant (bug)
    GlyphInfo { index: 8,  hex_char: '8', name: "Dragonfly", emoji: "\u{1F98B}" },           // butterfly (dragonfly)
    GlyphInfo { index: 9,  hex_char: '9', name: "Galaxy",    emoji: "\u{1F300}" },           // cyclone (galaxy)
    GlyphInfo { index: 10, hex_char: 'A', name: "Voxel",     emoji: "\u{1F54B}" },           // kaaba (voxel)
    GlyphInfo { index: 11, hex_char: 'B', name: "Whale",     emoji: "\u{1F40B}" },           // whale
    GlyphInfo { index: 12, hex_char: 'C', name: "Tent",      emoji: "\u{26FA}" },            // tent
    GlyphInfo { index: 13, hex_char: 'D', name: "Rocket",    emoji: "\u{1F680}" },           // rocket
    GlyphInfo { index: 14, hex_char: 'E', name: "Tree",      emoji: "\u{1F333}" },           // deciduous tree
    GlyphInfo { index: 15, hex_char: 'F', name: "Atlas",     emoji: "\u{1F53A}" },           // red triangle pointed up
];
```

---

## Glyph Type (`src/glyph.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A single portal glyph (value 0–15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Glyph(u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphParseError(pub String);

impl fmt::Display for GlyphParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid glyph: {}", self.0)
    }
}

impl std::error::Error for GlyphParseError {}
```

### Glyph Methods

```rust
impl Glyph {
    /// Create from index 0–15. Panics if out of range.
    pub fn new(index: u8) -> Self {
        assert!(index < 16, "glyph index must be 0–15, got {}", index);
        Self(index)
    }

    /// Try to create from index. Returns None if >= 16.
    pub fn try_new(index: u8) -> Option<Self> {
        if index < 16 { Some(Self(index)) } else { None }
    }

    /// The numeric index (0–15).
    pub fn index(self) -> u8 {
        self.0
    }

    /// The hex character ('0'–'9', 'A'–'F').
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
}
```

### Glyph Conversions

```rust
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

/// Parse from:
/// - A single hex character ('0'–'9', 'a'–'f', 'A'–'F')
/// - An emoji string matching one of the 16 glyphs
/// - A glyph name (case-insensitive: "sunset", "Sunset", "SUNSET")
impl FromStr for Glyph {
    type Err = GlyphParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

        Err(GlyphParseError(s.to_string()))
    }
}
```

### Helper: Parse Next Glyph from a String Slice

This function is needed for parsing mixed-format portal address strings. It tries to consume the next glyph from the beginning of the input and returns the glyph plus the remaining unconsumed string.

```rust
/// Try to parse a single glyph from the start of `input`.
/// Returns `(Glyph, remaining_str)` on success.
/// Attempts in order:
/// 1. Emoji match (longest match first, to handle multi-codepoint emoji)
/// 2. Glyph name match (case-insensitive, word boundary: tries longest name first)
/// 3. Single hex digit
pub fn parse_next_glyph(input: &str) -> Result<(Glyph, &str), GlyphParseError> {
    if input.is_empty() {
        return Err(GlyphParseError("empty input".to_string()));
    }

    // 1. Try emoji match — check each glyph's emoji as a prefix
    //    Sort by emoji byte length descending to match longest first
    //    (handles Bird emoji with/without variation selector)
    let mut by_len: Vec<(usize, &GlyphInfo)> = GLYPH_TABLE.iter().map(|g| (g.emoji.len(), g)).collect();
    by_len.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, info) in &by_len {
        if input.starts_with(info.emoji) {
            return Ok((Glyph(info.index), &input[info.emoji.len()..]));
        }
        // Also try without trailing variation selector
        let emoji_no_vs = info.emoji.trim_end_matches('\u{FE0F}');
        if emoji_no_vs != info.emoji && input.starts_with(emoji_no_vs) {
            // Check that the next char isn't the variation selector (already handled above)
            let rest = &input[emoji_no_vs.len()..];
            if rest.starts_with('\u{FE0F}') {
                // Consume the variation selector too
                let vs_len = '\u{FE0F}'.len_utf8();
                return Ok((Glyph(info.index), &rest[vs_len..]));
            }
            return Ok((Glyph(info.index), rest));
        }
    }

    // 2. Try glyph name match (case-insensitive prefix)
    //    Sort names by length descending to match "Dragonfly" before "D"
    let mut names: Vec<&GlyphInfo> = GLYPH_TABLE.iter().collect();
    names.sort_by(|a, b| b.name.len().cmp(&a.name.len()));

    let input_lower = input.to_lowercase();
    for info in &names {
        let name_lower = info.name.to_lowercase();
        if input_lower.starts_with(&name_lower) {
            return Ok((Glyph(info.index), &input[info.name.len()..]));
        }
    }

    // 3. Try single hex digit
    let c = input.chars().next().unwrap();
    if let Some(v) = c.to_digit(16) {
        return Ok((Glyph(v as u8), &input[c.len_utf8()..]));
    }

    Err(GlyphParseError(format!("unrecognized glyph at: {}", &input[..input.len().min(20)])))
}
```

---

## PortalAddress Type (`src/address.rs`)

Add to the existing `address.rs` file, below `GalacticAddress`.

```rust
use crate::glyph::{Glyph, GlyphParseError, parse_next_glyph};

/// A 12-glyph portal address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortalAddress {
    glyphs: [u8; 12], // each value 0–15
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalParseError {
    WrongLength(usize),
    InvalidGlyph(GlyphParseError),
}

impl fmt::Display for PortalParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength(n) => write!(f, "expected 12 glyphs, got {}", n),
            Self::InvalidGlyph(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PortalParseError {}

impl From<GlyphParseError> for PortalParseError {
    fn from(e: GlyphParseError) -> Self {
        Self::InvalidGlyph(e)
    }
}
```

### PortalAddress Methods

```rust
impl PortalAddress {
    /// Create from an array of 12 u8 values (each 0–15).
    pub fn new(glyphs: [u8; 12]) -> Self {
        for (i, &g) in glyphs.iter().enumerate() {
            assert!(g < 16, "glyph[{}] = {} is out of range 0–15", i, g);
        }
        Self { glyphs }
    }

    /// Get the glyph at position `i` (0–11).
    pub fn glyph(&self, i: usize) -> Glyph {
        Glyph::new(self.glyphs[i])
    }

    /// Get all 12 glyphs.
    pub fn glyphs(&self) -> [Glyph; 12] {
        let mut out = [Glyph::new(0); 12];
        for i in 0..12 {
            out[i] = Glyph::new(self.glyphs[i]);
        }
        out
    }

    /// Format as 12 hex digits (uppercase): e.g., "01717D8A4EA2"
    pub fn to_hex_string(&self) -> String {
        self.glyphs.iter().map(|g| format!("{:X}", g)).collect()
    }

    /// Format as emoji string: e.g., "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"
    pub fn to_emoji_string(&self) -> String {
        self.glyphs.iter().map(|&g| Glyph::new(g).emoji()).collect()
    }

    /// Parse a mixed-format string containing 12 glyphs.
    /// Accepts any combination of hex digits, emoji, and glyph names.
    /// Examples:
    /// - "01717D8A4EA2"
    /// - "🌅🕊️🐜🕊️🐜🌳🦋🕋🌜🔺🕋😑"
    /// - "🌅1🐜Bird🐜E🦋A4🔺A2"
    pub fn parse_mixed(s: &str) -> Result<Self, PortalParseError> {
        let mut glyphs = Vec::with_capacity(12);
        let mut remaining = s.trim();

        while !remaining.is_empty() && glyphs.len() < 12 {
            let (glyph, rest) = parse_next_glyph(remaining)?;
            glyphs.push(glyph.index());
            remaining = rest;
        }

        if glyphs.len() != 12 {
            return Err(PortalParseError::WrongLength(glyphs.len()));
        }

        if !remaining.is_empty() {
            return Err(PortalParseError::WrongLength(13)); // too many glyphs
        }

        let mut arr = [0u8; 12];
        arr.copy_from_slice(&glyphs);
        Ok(Self { glyphs: arr })
    }
}
```

### Display and FromStr for PortalAddress

```rust
/// Default display is hex.
impl fmt::Display for PortalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// Parse from any supported format (hex, emoji, mixed).
impl FromStr for PortalAddress {
    type Err = PortalParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_mixed(s)
    }
}
```

---

## GalacticAddress <-> PortalAddress Conversions

Add these impls to `src/address.rs`:

```rust
impl From<PortalAddress> for GalacticAddress {
    /// Convert portal address to galactic address.
    /// Portal glyph layout: P-SSS-YY-ZZZ-XXX
    /// Glyph positions (0-indexed):
    ///   [0]       = P (planet index, 4 bits)
    ///   [1][2][3] = SSS (solar system index, 12 bits)
    ///   [4][5]    = YY (voxel Y, 8 bits)
    ///   [6][7][8] = ZZZ (voxel Z, 12 bits)
    ///   [9][10][11] = XXX (voxel X, 12 bits)
    /// Reality index defaults to 0.
    fn from(pa: PortalAddress) -> Self {
        let g = pa.glyphs; // [u8; 12], each 0–15

        let planet_index = g[0];
        let ssi = ((g[1] as u16) << 8) | ((g[2] as u16) << 4) | (g[3] as u16);
        let y_raw = ((g[4] as u8) << 4) | g[5];
        let z_raw = ((g[6] as u16) << 8) | ((g[7] as u16) << 4) | (g[8] as u16);
        let x_raw = ((g[9] as u16) << 8) | ((g[10] as u16) << 4) | (g[11] as u16);

        let packed = ((planet_index as u64) << 44)
            | ((ssi as u64) << 32)
            | ((y_raw as u64) << 24)
            | ((z_raw as u64) << 12)
            | (x_raw as u64);

        GalacticAddress::from_packed(packed, 0)
    }
}

impl From<GalacticAddress> for PortalAddress {
    /// Convert galactic address to portal address.
    /// Extracts each nibble from the packed 48-bit value.
    fn from(addr: GalacticAddress) -> Self {
        let p = addr.packed();
        let mut glyphs = [0u8; 12];

        // Extract nibbles from MSB to LSB (48 bits = 12 nibbles)
        for i in 0..12 {
            glyphs[i] = ((p >> (44 - i * 4)) & 0xF) as u8;
        }

        PortalAddress { glyphs }
    }
}
```

### Convenience Methods on GalacticAddress

```rust
impl GalacticAddress {
    /// Convert to PortalAddress.
    pub fn to_portal_address(&self) -> PortalAddress {
        PortalAddress::from(*self)
    }

    /// Create from a portal address string (hex, emoji, or mixed).
    /// Reality index defaults to 0.
    pub fn from_portal_string(s: &str) -> Result<Self, PortalParseError> {
        let pa: PortalAddress = s.parse()?;
        Ok(GalacticAddress::from(pa))
    }
}
```

### Convenience Methods on PortalAddress

```rust
impl PortalAddress {
    /// Convert to GalacticAddress (reality_index = 0).
    pub fn to_galactic_address(&self) -> GalacticAddress {
        GalacticAddress::from(*self)
    }

    /// Create from a GalacticAddress.
    pub fn from_galactic_address(addr: &GalacticAddress) -> Self {
        PortalAddress::from(*addr)
    }

    /// Create from a signal booster string by first parsing to GalacticAddress.
    /// Requires planet_index and reality_index since signal booster format lacks them.
    pub fn from_signal_booster(s: &str, planet_index: u8, reality_index: u8) -> Result<Self, AddressParseError> {
        let addr = GalacticAddress::from_signal_booster(s, planet_index, reality_index)?;
        Ok(PortalAddress::from(addr))
    }
}
```

---

## Important Implementation Notes

1. **Bird glyph variation selector**: The Bird emoji (U+1F54A) is often followed by variation selector U+FE0F. The `parse_next_glyph` function must handle both `"\u{1F54A}"` and `"\u{1F54A}\u{FE0F}"` as valid Bird glyphs. When outputting emoji, always include the variation selector for Bird.

2. **Mixed input parsing**: The `parse_mixed` function processes the input string left-to-right, greedily consuming the longest match at each position. Glyph names are matched before hex digits so that "Atlas" is recognized as glyph F rather than "A" (hex digit) + "tlas" (error). The name sort by descending length ensures "Dragonfly" matches before "D".

3. **Case insensitivity**: All name matching is case-insensitive. Hex digit matching naturally handles both 'a'–'f' and 'A'–'F' via `char::to_digit(16)`.

4. **No separators**: The parser does not expect or skip separators. If the input has spaces or dashes between glyphs, they will cause parse errors. If separator tolerance is desired later, strip whitespace and dashes before calling `parse_mixed`.

---

## Tests

All tests go in `#[cfg(test)] mod tests` blocks in the respective source files.

### Glyph Tests (`src/glyph.rs`)

```rust
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
        // With variation selector
        let with_vs: Glyph = "\u{1F54A}\u{FE0F}".parse().unwrap();
        assert_eq!(with_vs.index(), 1);

        // Without variation selector
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
}
```

### PortalAddress Tests (`src/address.rs`)

```rust
#[cfg(test)]
mod portal_tests {
    use super::*;

    #[test]
    fn known_address_hex_to_emoji() {
        let pa: PortalAddress = "01717D8A4EA2".parse().unwrap();
        let emoji = pa.to_emoji_string();
        // Glyph indices: 0,1,7,1,7,D(13),8,A(10),4,E(14),A(10),2
        // Emoji: Sunset Bird Bug Bird Bug Rocket Dragonfly Voxel Eclipse Tree Voxel Face
        assert_eq!(
            emoji,
            "\u{1F305}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F680}\u{1F98B}\u{1F54B}\u{1F31C}\u{1F333}\u{1F54B}\u{1F611}"
        );
    }

    #[test]
    fn known_address_emoji_to_hex() {
        let emoji_str = "\u{1F305}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F680}\u{1F98B}\u{1F54B}\u{1F31C}\u{1F333}\u{1F54B}\u{1F611}";
        let pa: PortalAddress = emoji_str.parse().unwrap();
        assert_eq!(pa.to_hex_string(), "01717D8A4EA2");
    }

    #[test]
    fn galactic_address_portal_roundtrip() {
        let ga = GalacticAddress::from_packed(0x01717D8A4EA2, 0);
        let pa = PortalAddress::from(ga);
        assert_eq!(pa.to_hex_string(), "01717D8A4EA2");
        let ga2 = GalacticAddress::from(pa);
        assert_eq!(ga.packed(), ga2.packed());
    }

    #[test]
    fn hex_string_roundtrip() {
        let pa: PortalAddress = "01717D8A4EA2".parse().unwrap();
        let hex = pa.to_hex_string();
        assert_eq!(hex, "01717D8A4EA2");
        let pa2: PortalAddress = hex.parse().unwrap();
        assert_eq!(pa, pa2);
    }

    #[test]
    fn full_roundtrip_ga_pa_hex_pa_ga() {
        let ga1 = GalacticAddress::new(-350, 42, 1000, 0x123, 3, 5);
        let pa1 = ga1.to_portal_address();
        let hex = pa1.to_hex_string();
        let pa2: PortalAddress = hex.parse().unwrap();
        let ga2 = pa2.to_galactic_address();
        // Note: reality_index is lost in portal address conversion (defaults to 0)
        assert_eq!(ga1.packed(), ga2.packed());
    }

    #[test]
    fn parse_emoji_with_variation_selectors() {
        // Bird glyph with variation selector
        let with_vs = "\u{1F305}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F54A}\u{FE0F}\u{1F41C}\u{1F680}\u{1F98B}\u{1F54B}\u{1F31C}\u{1F333}\u{1F54B}\u{1F611}";
        let pa_with: PortalAddress = with_vs.parse().unwrap();

        // Bird glyph without variation selector
        let without_vs = "\u{1F305}\u{1F54A}\u{1F41C}\u{1F54A}\u{1F41C}\u{1F680}\u{1F98B}\u{1F54B}\u{1F31C}\u{1F333}\u{1F54B}\u{1F611}";
        let pa_without: PortalAddress = without_vs.parse().unwrap();

        assert_eq!(pa_with, pa_without);
    }

    #[test]
    fn parse_mixed_input() {
        // Mix of hex, emoji, and names
        // "🌅" (Sunset=0) + "1" (Bird=1) + "🐜" (Bug=7) + "Bird" (=1) ...
        // This test constructs a known 12-glyph mixed string
        // Target: "01717D8A4EA2"
        // Glyphs: 0 1 7 1 7 D 8 A 4 E A 2
        let mixed = "\u{1F305}1\u{1F41C}Bird\u{1F41C}D8A4EA2";
        let pa: PortalAddress = mixed.parse().unwrap();
        assert_eq!(pa.to_hex_string(), "01717D8A4EA2");
    }

    #[test]
    fn wrong_length_errors() {
        // Too few glyphs
        assert!("0171".parse::<PortalAddress>().is_err());
        // Too many glyphs
        assert!("01717D8A4EA20".parse::<PortalAddress>().is_err());
    }

    #[test]
    fn signal_booster_to_portal_address() {
        let ga = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let sb = ga.to_signal_booster();
        let pa = PortalAddress::from_signal_booster(&sb, 0, 0).unwrap();
        assert_eq!(pa.to_galactic_address().packed(), ga.packed());
    }

    #[test]
    fn display_is_hex() {
        let pa: PortalAddress = "01717D8A4EA2".parse().unwrap();
        assert_eq!(format!("{}", pa), "01717D8A4EA2");
    }
}
```
