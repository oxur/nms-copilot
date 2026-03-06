# Milestone 1.10 -- `nms convert` Command (nms-cli)

Full coordinate converter CLI: translate between portal glyphs, signal booster coords, galactic addresses, and voxel positions. This command requires NO save file -- it is pure coordinate math from nms-core.

## Crate: `nms-cli`

Path: `crates/nms-cli/`

No additional dependencies beyond what milestone 1.10 added.

---

## Command Syntax

```
nms convert --glyphs "01717D8A4EA2"
nms convert --glyphs 01717D8A4EA2
nms convert --coords 0EA2:007D:08A4:0171
nms convert --ga 0x01717D8A4EA2
nms convert --voxel 1186,-131,1156 --ssi 369 --planet 0
nms convert --ga 0x01717D8A4EA2 --galaxy Euclid
nms convert --ga 0x01717D8A4EA2 --galaxy 0
```

Exactly one of `--glyphs`, `--coords`, `--ga`, or `--voxel` must be provided. If `--voxel` is used, `--ssi` is required and `--planet` defaults to 0.

The `--galaxy` flag is optional (defaults to `"0"` = Euclid). It accepts either a galaxy name (case-insensitive) or a numeric index (0-255).

---

## Output Format

```
NMS Copilot -- Coordinate Conversion
=====================================

  Portal Glyphs:     01717D8A4EA2
  Signal Booster:    0EA2:007D:08A4:0171
  Galactic Address:  0x01717D8A4EA2
  Voxel Position:    X=1186, Y=-131, Z=1156
  System Index:      369 (0x171)
  Planet Index:      0
  Galaxy:            Euclid (0)
```

All output formats are always shown regardless of which input format was used.

---

## CLI Integration

### Additions to `crates/nms-cli/src/main.rs`

Add the Convert variant to the Commands enum:

```rust
mod convert;

#[derive(Subcommand)]
enum Commands {
    /// Display save file summary
    Info {
        #[arg(long)]
        save: Option<PathBuf>,
    },

    /// Convert between NMS coordinate formats
    Convert {
        /// Portal glyphs as 12 hex digits (e.g., 01717D8A4EA2)
        #[arg(long, group = "input")]
        glyphs: Option<String>,

        /// Signal booster coordinates (XXXX:YYYY:ZZZZ:SSSS)
        #[arg(long, group = "input")]
        coords: Option<String>,

        /// Galactic address as hex (0x01717D8A4EA2)
        #[arg(long, group = "input")]
        ga: Option<String>,

        /// Voxel position as X,Y,Z (requires --ssi)
        #[arg(long, group = "input")]
        voxel: Option<String>,

        /// Solar system index (required with --voxel)
        #[arg(long)]
        ssi: Option<u16>,

        /// Planet index (0-15, defaults to 0)
        #[arg(long, default_value = "0")]
        planet: u8,

        /// Galaxy index (0-255) or name (e.g., "Euclid")
        #[arg(long, default_value = "0")]
        galaxy: String,
    },
}
```

In `main()`, add the dispatch:

```rust
Commands::Convert { glyphs, coords, ga, voxel, ssi, planet, galaxy } => {
    convert::run(glyphs, coords, ga, voxel, ssi, planet, galaxy)
}
```

---

## Convert Command Implementation

### File: `crates/nms-cli/src/convert.rs`

```rust
use nms_core::address::{GalacticAddress, AddressParseError};
use nms_core::galaxy::galaxy_by_index;

pub fn run(
    glyphs: Option<String>,
    coords: Option<String>,
    ga: Option<String>,
    voxel: Option<String>,
    ssi: Option<u16>,
    planet: u8,
    galaxy: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve galaxy
    let reality_index = resolve_galaxy(&galaxy)?;

    // 2. Parse whichever input was provided into a GalacticAddress
    let addr = if let Some(g) = glyphs {
        parse_glyphs(&g, reality_index)?
    } else if let Some(c) = coords {
        parse_signal_booster(&c, planet, reality_index)?
    } else if let Some(a) = ga {
        parse_galactic_address(&a, reality_index)?
    } else if let Some(v) = voxel {
        let solar_system_index = ssi.ok_or("--ssi is required when using --voxel")?;
        parse_voxel(&v, solar_system_index, planet, reality_index)?
    } else {
        return Err("One of --glyphs, --coords, --ga, or --voxel must be provided".into());
    };

    // 3. Display all formats
    print_all_formats(&addr);

    Ok(())
}

/// Parse portal glyph hex string (12 hex digits, with or without "0x" prefix).
fn parse_glyphs(input: &str, reality_index: u8) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let hex = input.trim();
    // Strip 0x prefix if present
    let hex = hex.strip_prefix("0x").or_else(|| hex.strip_prefix("0X")).unwrap_or(hex);

    if hex.len() != 12 {
        return Err(format!(
            "Portal glyphs must be exactly 12 hex digits, got {} (\"{}\")",
            hex.len(), hex
        ).into());
    }

    let packed = u64::from_str_radix(hex, 16)
        .map_err(|_| format!("Invalid hex in portal glyphs: \"{}\"", hex))?;

    Ok(GalacticAddress::from_packed(packed, reality_index))
}

/// Parse signal booster format "XXXX:YYYY:ZZZZ:SSSS".
fn parse_signal_booster(input: &str, planet: u8, reality_index: u8) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    GalacticAddress::from_signal_booster(input.trim(), planet, reality_index)
        .map_err(|e| format!("Invalid signal booster coordinates: {}", e).into())
}

/// Parse galactic address hex string (with or without "0x" prefix, 12-14 hex digits).
fn parse_galactic_address(input: &str, reality_index: u8) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let hex = input.trim();
    let hex = hex.strip_prefix("0x").or_else(|| hex.strip_prefix("0X")).unwrap_or(hex);

    let packed = u64::from_str_radix(hex, 16)
        .map_err(|_| format!("Invalid hex in galactic address: \"{}\"", hex))?;

    // Mask to 48 bits (the packed galactic address)
    Ok(GalacticAddress::from_packed(packed & 0xFFFF_FFFF_FFFF, reality_index))
}

/// Parse voxel position "X,Y,Z" with signed integers.
fn parse_voxel(
    input: &str,
    solar_system_index: u16,
    planet_index: u8,
    reality_index: u8,
) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.trim().split(',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Voxel position must be X,Y,Z (3 comma-separated integers), got \"{}\"",
            input
        ).into());
    }

    let x: i16 = parts[0].trim().parse()
        .map_err(|_| format!("Invalid voxel X: \"{}\"", parts[0].trim()))?;
    let y: i8 = parts[1].trim().parse()
        .map_err(|_| format!("Invalid voxel Y: \"{}\" (must be -128..127)", parts[1].trim()))?;
    let z: i16 = parts[2].trim().parse()
        .map_err(|_| format!("Invalid voxel Z: \"{}\"", parts[2].trim()))?;

    // Validate ranges
    if x < -2048 || x > 2047 {
        return Err(format!("Voxel X out of range: {} (must be -2048..2047)", x).into());
    }
    if z < -2048 || z > 2047 {
        return Err(format!("Voxel Z out of range: {} (must be -2048..2047)", z).into());
    }
    if solar_system_index > 0xFFE {
        return Err(format!("System index out of range: {} (must be 0..4094)", solar_system_index).into());
    }

    Ok(GalacticAddress::new(x, y, z, solar_system_index, planet_index, reality_index))
}

/// Resolve galaxy name or index string to a reality_index u8.
fn resolve_galaxy(input: &str) -> Result<u8, Box<dyn std::error::Error>> {
    let trimmed = input.trim();

    // Try parsing as a number first
    if let Ok(idx) = trimmed.parse::<u16>() {
        if idx > 255 {
            return Err(format!("Galaxy index out of range: {} (must be 0-255)", idx).into());
        }
        return Ok(idx as u8);
    }

    // Try matching by name (case-insensitive)
    let lower = trimmed.to_lowercase();
    for i in 0..=255u8 {
        let galaxy = galaxy_by_index(i);
        if galaxy.name.to_lowercase() == lower {
            return Ok(i);
        }
    }

    Err(format!("Unknown galaxy: \"{}\". Use a number 0-255 or a galaxy name like \"Euclid\".", trimmed).into())
}

/// Print all coordinate formats for a GalacticAddress.
fn print_all_formats(addr: &GalacticAddress) {
    let galaxy = galaxy_by_index(addr.reality_index);

    println!("NMS Copilot -- Coordinate Conversion");
    println!("=====================================");
    println!();
    println!("  Portal Glyphs:     {:012X}", addr.packed());
    println!("  Signal Booster:    {}", addr.to_signal_booster());
    println!("  Galactic Address:  0x{:012X}", addr.packed());
    println!("  Voxel Position:    X={}, Y={}, Z={}",
        addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
    println!("  System Index:      {} (0x{:03X})",
        addr.solar_system_index(), addr.solar_system_index());
    println!("  Planet Index:      {}", addr.planet_index());
    println!("  Galaxy:            {} ({})", galaxy.name, addr.reality_index);
}
```

---

## Tests

### File: `crates/nms-cli/src/convert.rs` (inline tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ── Glyph parsing ──

    #[test]
    fn parse_glyphs_12_hex_digits() {
        let addr = parse_glyphs("01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
        assert_eq!(addr.reality_index, 0);
    }

    #[test]
    fn parse_glyphs_with_0x_prefix() {
        let addr = parse_glyphs("0x01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    #[test]
    fn parse_glyphs_lowercase() {
        let addr = parse_glyphs("01717d8a4ea2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    #[test]
    fn parse_glyphs_wrong_length() {
        assert!(parse_glyphs("0171", 0).is_err());
    }

    #[test]
    fn parse_glyphs_invalid_hex() {
        assert!(parse_glyphs("01717D8A4EGZ", 0).is_err());
    }

    // ── Signal booster parsing ──

    #[test]
    fn parse_signal_booster_valid() {
        let addr = parse_signal_booster("0EA2:007D:08A4:0171", 0, 0).unwrap();
        // Verify round-trip
        assert_eq!(addr.to_signal_booster(), "0EA2:007D:08A4:0171");
    }

    #[test]
    fn parse_signal_booster_invalid_format() {
        assert!(parse_signal_booster("0EA2:007D", 0, 0).is_err());
    }

    #[test]
    fn parse_signal_booster_invalid_hex() {
        assert!(parse_signal_booster("ZZZZ:007D:08A4:0171", 0, 0).is_err());
    }

    // ── Galactic address parsing ──

    #[test]
    fn parse_ga_with_prefix() {
        let addr = parse_galactic_address("0x01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    #[test]
    fn parse_ga_without_prefix() {
        let addr = parse_galactic_address("01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    #[test]
    fn parse_ga_masks_to_48_bits() {
        // If someone passes a value > 48 bits, mask it
        let addr = parse_galactic_address("0xFFFF01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

    // ── Voxel parsing ──

    #[test]
    fn parse_voxel_positive() {
        let addr = parse_voxel("1186,42,1156", 369, 0, 0).unwrap();
        assert_eq!(addr.voxel_x(), 1186);
        assert_eq!(addr.voxel_y(), 42);
        assert_eq!(addr.voxel_z(), 1156);
        assert_eq!(addr.solar_system_index(), 369);
        assert_eq!(addr.planet_index(), 0);
    }

    #[test]
    fn parse_voxel_negative() {
        let addr = parse_voxel("-350,-2,165", 505, 3, 0).unwrap();
        assert_eq!(addr.voxel_x(), -350);
        assert_eq!(addr.voxel_y(), -2);
        assert_eq!(addr.voxel_z(), 165);
        assert_eq!(addr.solar_system_index(), 505);
        assert_eq!(addr.planet_index(), 3);
    }

    #[test]
    fn parse_voxel_with_spaces() {
        let addr = parse_voxel(" 100 , 50 , -200 ", 42, 0, 0).unwrap();
        assert_eq!(addr.voxel_x(), 100);
        assert_eq!(addr.voxel_y(), 50);
        assert_eq!(addr.voxel_z(), -200);
    }

    #[test]
    fn parse_voxel_x_out_of_range() {
        assert!(parse_voxel("3000,0,0", 0, 0, 0).is_err());
    }

    #[test]
    fn parse_voxel_z_out_of_range() {
        assert!(parse_voxel("0,0,-3000", 0, 0, 0).is_err());
    }

    #[test]
    fn parse_voxel_ssi_out_of_range() {
        assert!(parse_voxel("0,0,0", 0xFFFF, 0, 0).is_err());
    }

    #[test]
    fn parse_voxel_wrong_part_count() {
        assert!(parse_voxel("100,200", 0, 0, 0).is_err());
    }

    // ── Galaxy resolution ──

    #[test]
    fn resolve_galaxy_by_index() {
        assert_eq!(resolve_galaxy("0").unwrap(), 0);
        assert_eq!(resolve_galaxy("1").unwrap(), 1);
        assert_eq!(resolve_galaxy("255").unwrap(), 255);
    }

    #[test]
    fn resolve_galaxy_index_out_of_range() {
        assert!(resolve_galaxy("256").is_err());
    }

    #[test]
    fn resolve_galaxy_by_name_euclid() {
        assert_eq!(resolve_galaxy("Euclid").unwrap(), 0);
    }

    #[test]
    fn resolve_galaxy_by_name_case_insensitive() {
        assert_eq!(resolve_galaxy("euclid").unwrap(), 0);
        assert_eq!(resolve_galaxy("EUCLID").unwrap(), 0);
    }

    #[test]
    fn resolve_galaxy_hilbert() {
        assert_eq!(resolve_galaxy("Hilbert Dimension").unwrap(), 1);
    }

    #[test]
    fn resolve_galaxy_unknown_name() {
        assert!(resolve_galaxy("NotAGalaxy").is_err());
    }

    // ── Round-trip conversions ──

    #[test]
    fn roundtrip_glyphs_to_signal_booster() {
        let addr = parse_glyphs("01717D8A4EA2", 0).unwrap();
        let sb = addr.to_signal_booster();
        let addr2 = parse_signal_booster(&sb, addr.planet_index(), 0).unwrap();
        // Planet index is NOT in signal booster format, so SSI and voxels should match
        assert_eq!(addr.solar_system_index(), addr2.solar_system_index());
        assert_eq!(addr.voxel_x(), addr2.voxel_x());
        assert_eq!(addr.voxel_y(), addr2.voxel_y());
        assert_eq!(addr.voxel_z(), addr2.voxel_z());
    }

    #[test]
    fn roundtrip_glyphs_to_ga_to_voxel() {
        let addr = parse_glyphs("01717D8A4EA2", 0).unwrap();
        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr2 = parse_voxel(&voxel_str, addr.solar_system_index(), addr.planet_index(), 0).unwrap();
        assert_eq!(addr.packed(), addr2.packed());
    }

    #[test]
    fn roundtrip_ga_to_all_formats() {
        let addr = parse_galactic_address("0x01717D8A4EA2", 0).unwrap();

        // To glyphs and back
        let hex = format!("{:012X}", addr.packed());
        let addr2 = parse_glyphs(&hex, 0).unwrap();
        assert_eq!(addr.packed(), addr2.packed());

        // To signal booster and back
        let sb = addr.to_signal_booster();
        let addr3 = parse_signal_booster(&sb, addr.planet_index(), 0).unwrap();
        assert_eq!(addr.solar_system_index(), addr3.solar_system_index());
        assert_eq!(addr.voxel_x(), addr3.voxel_x());
        assert_eq!(addr.voxel_y(), addr3.voxel_y());
        assert_eq!(addr.voxel_z(), addr3.voxel_z());

        // To voxel and back
        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr4 = parse_voxel(&voxel_str, addr.solar_system_index(), addr.planet_index(), 0).unwrap();
        assert_eq!(addr.packed(), addr4.packed());
    }

    #[test]
    fn roundtrip_from_actual_save_address() {
        // From the actual save file: GalacticAddress "0x40050003AB8C07" (a base address)
        let addr = parse_galactic_address("0x40050003AB8C07", 0).unwrap();

        assert_eq!(addr.planet_index(), 4);     // bits 47-44 = 0x4
        assert_eq!(addr.solar_system_index(), 5); // bits 43-32 = 0x005

        let hex = format!("{:012X}", addr.packed());
        assert_eq!(hex, "40050003AB8C07");

        // Round-trip through voxel
        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr2 = parse_voxel(&voxel_str, addr.solar_system_index(), addr.planet_index(), 0).unwrap();
        assert_eq!(addr.packed(), addr2.packed());
    }

    // ── Error messages ──

    #[test]
    fn run_with_no_input_returns_error() {
        let result = run(None, None, None, None, None, 0, "0".into());
        assert!(result.is_err());
    }

    #[test]
    fn run_voxel_without_ssi_returns_error() {
        let result = run(None, None, None, Some("100,50,-200".into()), None, 0, "0".into());
        assert!(result.is_err());
    }
}
```

---

## Implementation Notes

1. **No save file needed.** This command is pure coordinate math. All the conversion logic lives in `nms_core::address::GalacticAddress` (defined in milestone 1.2). This command is a thin CLI wrapper around those methods.

2. **Signal booster format does NOT include planet index.** The `--planet` argument (defaulting to 0) is used when converting FROM signal booster to fill in the planet index. When converting TO signal booster format, the planet index is simply not shown (it is already displayed separately).

3. **The clap `group = "input"` attribute** ensures that exactly one of the four input options is provided. If none or multiple are given, clap will emit an appropriate error message automatically.

4. **Galaxy resolution** scans all 256 galaxies by name. This is a linear scan but only 256 entries, so performance is not a concern.

5. **Galactic address masking:** The packed galactic address is 48 bits. If a user provides more than 48 bits of hex (e.g., from a full save-file address that includes extra metadata), we mask to 48 bits. This is consistent with `GalacticAddress::from_packed()`.

6. **The portal glyph hex format** is the same as the galactic address hex format -- both are 12 hex digits representing the 48-bit packed value. The distinction is conceptual (portal glyphs are what you enter at a portal; galactic address is the coordinate system). The output shows both for clarity.

7. **Future enhancement:** Accept Unicode portal glyph emoji input and convert to hex. The NMS portal uses 16 glyphs mapped to hex digits 0-F. This is deferred to a later milestone.
