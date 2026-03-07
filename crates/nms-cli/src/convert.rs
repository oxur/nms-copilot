//! `nms convert` command — coordinate format converter.

use nms_core::address::GalacticAddress;
use nms_core::galaxy::Galaxy;

pub fn run(
    glyphs: Option<String>,
    coords: Option<String>,
    ga: Option<String>,
    voxel: Option<String>,
    ssi: Option<u16>,
    planet: u8,
    galaxy: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let reality_index = resolve_galaxy(&galaxy)?;

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

    print_all_formats(&addr);
    Ok(())
}

fn parse_glyphs(
    input: &str,
    reality_index: u8,
) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let hex = input.trim();
    let hex = hex
        .strip_prefix("0x")
        .or_else(|| hex.strip_prefix("0X"))
        .unwrap_or(hex);

    if hex.len() != 12 {
        return Err(format!(
            "Portal glyphs must be exactly 12 hex digits, got {} (\"{}\")",
            hex.len(),
            hex
        )
        .into());
    }

    let packed = u64::from_str_radix(hex, 16)
        .map_err(|_| format!("Invalid hex in portal glyphs: \"{hex}\""))?;

    Ok(GalacticAddress::from_packed(packed, reality_index))
}

fn parse_signal_booster(
    input: &str,
    planet: u8,
    reality_index: u8,
) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    GalacticAddress::from_signal_booster(input.trim(), planet, reality_index)
        .map_err(|e| format!("Invalid signal booster coordinates: {e}").into())
}

fn parse_galactic_address(
    input: &str,
    reality_index: u8,
) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let hex = input.trim();
    let hex = hex
        .strip_prefix("0x")
        .or_else(|| hex.strip_prefix("0X"))
        .unwrap_or(hex);

    let packed = u64::from_str_radix(hex, 16)
        .map_err(|_| format!("Invalid hex in galactic address: \"{hex}\""))?;

    Ok(GalacticAddress::from_packed(
        packed & 0xFFFF_FFFF_FFFF,
        reality_index,
    ))
}

fn parse_voxel(
    input: &str,
    solar_system_index: u16,
    planet_index: u8,
    reality_index: u8,
) -> Result<GalacticAddress, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.trim().split(',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Voxel position must be X,Y,Z (3 comma-separated integers), got \"{input}\""
        )
        .into());
    }

    let x: i16 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid voxel X: \"{}\"", parts[0].trim()))?;
    let y: i8 = parts[1].trim().parse().map_err(|_| {
        format!(
            "Invalid voxel Y: \"{}\" (must be -128..127)",
            parts[1].trim()
        )
    })?;
    let z: i16 = parts[2]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid voxel Z: \"{}\"", parts[2].trim()))?;

    if !(-2048..=2047).contains(&x) {
        return Err(format!("Voxel X out of range: {x} (must be -2048..2047)").into());
    }
    if !(-2048..=2047).contains(&z) {
        return Err(format!("Voxel Z out of range: {z} (must be -2048..2047)").into());
    }
    if solar_system_index > 0xFFE {
        return Err(
            format!("System index out of range: {solar_system_index} (must be 0..4094)").into(),
        );
    }

    Ok(GalacticAddress::new(
        x,
        y,
        z,
        solar_system_index,
        planet_index,
        reality_index,
    ))
}

fn resolve_galaxy(input: &str) -> Result<u8, Box<dyn std::error::Error>> {
    let trimmed = input.trim();

    if let Ok(idx) = trimmed.parse::<u16>() {
        if idx > 255 {
            return Err(format!("Galaxy index out of range: {idx} (must be 0-255)").into());
        }
        return Ok(idx as u8);
    }

    let lower = trimmed.to_lowercase();
    for i in 0..=255u8 {
        let galaxy = Galaxy::by_index(i);
        if galaxy.name.to_lowercase() == lower {
            return Ok(i);
        }
    }

    Err(format!(
        "Unknown galaxy: \"{trimmed}\". Use a number 0-255 or a galaxy name like \"Euclid\"."
    )
    .into())
}

fn print_all_formats(addr: &GalacticAddress) {
    let galaxy = Galaxy::by_index(addr.reality_index);

    println!("NMS Copilot -- Coordinate Conversion");
    println!("=====================================");
    println!();
    println!("  Portal Glyphs:     {:012X}", addr.packed());
    println!("  Signal Booster:    {}", addr.to_signal_booster());
    println!("  Galactic Address:  0x{:012X}", addr.packed());
    println!(
        "  Voxel Position:    X={}, Y={}, Z={}",
        addr.voxel_x(),
        addr.voxel_y(),
        addr.voxel_z()
    );
    println!(
        "  System Index:      {} (0x{:03X})",
        addr.solar_system_index(),
        addr.solar_system_index()
    );
    println!("  Planet Index:      {}", addr.planet_index());
    println!(
        "  Galaxy:            {} ({})",
        galaxy.name, addr.reality_index
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_signal_booster_valid() {
        let addr = parse_signal_booster("0EA2:007D:08A4:0171", 0, 0).unwrap();
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
        let addr = parse_galactic_address("0xFFFF01717D8A4EA2", 0).unwrap();
        assert_eq!(addr.packed(), 0x01717D8A4EA2);
    }

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

    #[test]
    fn roundtrip_glyphs_to_signal_booster() {
        let addr = parse_glyphs("01717D8A4EA2", 0).unwrap();
        let sb = addr.to_signal_booster();
        let addr2 = parse_signal_booster(&sb, addr.planet_index(), 0).unwrap();
        assert_eq!(addr.solar_system_index(), addr2.solar_system_index());
        assert_eq!(addr.voxel_x(), addr2.voxel_x());
        assert_eq!(addr.voxel_y(), addr2.voxel_y());
        assert_eq!(addr.voxel_z(), addr2.voxel_z());
    }

    #[test]
    fn roundtrip_glyphs_to_ga_to_voxel() {
        let addr = parse_glyphs("01717D8A4EA2", 0).unwrap();
        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr2 = parse_voxel(
            &voxel_str,
            addr.solar_system_index(),
            addr.planet_index(),
            0,
        )
        .unwrap();
        assert_eq!(addr.packed(), addr2.packed());
    }

    #[test]
    fn roundtrip_ga_to_all_formats() {
        let addr = parse_galactic_address("0x01717D8A4EA2", 0).unwrap();

        let hex = format!("{:012X}", addr.packed());
        let addr2 = parse_glyphs(&hex, 0).unwrap();
        assert_eq!(addr.packed(), addr2.packed());

        let sb = addr.to_signal_booster();
        let addr3 = parse_signal_booster(&sb, addr.planet_index(), 0).unwrap();
        assert_eq!(addr.solar_system_index(), addr3.solar_system_index());
        assert_eq!(addr.voxel_x(), addr3.voxel_x());
        assert_eq!(addr.voxel_y(), addr3.voxel_y());
        assert_eq!(addr.voxel_z(), addr3.voxel_z());

        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr4 = parse_voxel(
            &voxel_str,
            addr.solar_system_index(),
            addr.planet_index(),
            0,
        )
        .unwrap();
        assert_eq!(addr.packed(), addr4.packed());
    }

    #[test]
    fn roundtrip_from_actual_save_address() {
        // 0x40050003AB8C07 is 14 hex digits (56 bits); from_packed masks to 48 bits
        // Masked: 0x050003AB8C07
        let addr = parse_galactic_address("0x40050003AB8C07", 0).unwrap();
        assert_eq!(addr.packed(), 0x050003AB8C07);

        let hex = format!("{:012X}", addr.packed());
        assert_eq!(hex, "050003AB8C07");

        // Round-trip through voxel
        let voxel_str = format!("{},{},{}", addr.voxel_x(), addr.voxel_y(), addr.voxel_z());
        let addr2 = parse_voxel(
            &voxel_str,
            addr.solar_system_index(),
            addr.planet_index(),
            0,
        )
        .unwrap();
        assert_eq!(addr.packed(), addr2.packed());
    }

    #[test]
    fn run_with_no_input_returns_error() {
        let result = run(None, None, None, None, None, 0, "0".into());
        assert!(result.is_err());
    }

    #[test]
    fn run_voxel_without_ssi_returns_error() {
        let result = run(
            None,
            None,
            None,
            Some("100,50,-200".into()),
            None,
            0,
            "0".into(),
        );
        assert!(result.is_err());
    }
}
