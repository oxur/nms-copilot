//! Output formatting for query results.
//!
//! All formatters return `String` -- the caller prints to stdout.
//! No ANSI color codes yet (added in Phase 7 polish).

use crate::find::FindResult;
use crate::show::{ShowBaseResult, ShowResult, ShowSystemResult};
use crate::stats::StatsResult;

/// Format a distance in light-years for display.
///
/// - Under 1,000: "42 ly"
/// - 1,000 to 999,999: "127K ly"
/// - 1,000,000+: "1.2M ly"
pub fn format_distance(ly: f64) -> String {
    if ly < 1_000.0 {
        format!("{:.0} ly", ly)
    } else if ly < 1_000_000.0 {
        format!("{:.0}K ly", ly / 1_000.0)
    } else {
        format!("{:.1}M ly", ly / 1_000_000.0)
    }
}

/// Convert a 12-digit hex portal address to emoji string.
///
/// Uses `nms_core::glyph::Glyph::new()` and `emoji()`.
pub fn hex_to_emoji(hex: &str) -> String {
    use nms_core::glyph::Glyph;
    hex.chars()
        .filter_map(|ch| {
            let idx = ch.to_digit(16)? as u8;
            Some(Glyph::new(idx).emoji().to_string())
        })
        .collect()
}

/// Format find results as a numbered table.
///
/// ```text
///   #  Planet            Biome      System             Distance   Portal Glyphs
///   1  Metok-Kalpa       Lush       Gugestor Colony       0 ly    [emoji glyphs]
///   2  (unnamed)         Scorched   Esurad               18K ly   [emoji glyphs]
/// ```
pub fn format_find_results(results: &[FindResult]) -> String {
    if results.is_empty() {
        return "  No results found.\n".to_string();
    }

    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "  {:<3} {:<18} {:<11} {:<20} {:<11} {}\n",
        "#", "Planet", "Biome", "System", "Distance", "Portal Glyphs"
    ));

    for (i, r) in results.iter().enumerate() {
        let planet_name = r.planet.name.as_deref().unwrap_or("(unnamed)");
        let biome = r
            .planet
            .biome
            .map(|b| {
                let mut s = b.to_string();
                if r.planet.infested {
                    s.push('*');
                }
                s
            })
            .unwrap_or_else(|| "?".to_string());
        let system_name = r.system.name.as_deref().unwrap_or("(unnamed)");
        let distance = format_distance(r.distance_ly);
        let glyphs = hex_to_emoji(&r.portal_hex);

        out.push_str(&format!(
            "  {:<3} {:<18} {:<11} {:<20} {:>11} {}\n",
            i + 1,
            truncate(planet_name, 17),
            truncate(&biome, 10),
            truncate(system_name, 19),
            distance,
            glyphs,
        ));
    }

    out
}

/// Format a system detail view.
pub fn format_show_system(result: &ShowSystemResult) -> String {
    let mut out = String::new();
    let sys = &result.system;

    out.push_str("NMS Copilot -- System Detail\n");
    out.push_str("============================\n\n");

    out.push_str(&format!(
        "  Name:            {}\n",
        sys.name.as_deref().unwrap_or("(unnamed)")
    ));
    out.push_str(&format!("  Galaxy:          {}\n", result.galaxy_name));
    out.push_str(&format!(
        "  Portal Glyphs:   {}\n",
        hex_to_emoji(&result.portal_hex)
    ));
    out.push_str(&format!("  Hex Address:     {}\n", result.portal_hex));
    out.push_str(&format!(
        "  Discoverer:      {}\n",
        sys.discoverer.as_deref().unwrap_or("unknown")
    ));
    if let Some(dist) = result.distance_from_player {
        out.push_str(&format!("  Distance:        {}\n", format_distance(dist)));
    }

    out.push_str(&format!(
        "  Voxel Position:  X={}, Y={}, Z={}\n",
        sys.address.voxel_x(),
        sys.address.voxel_y(),
        sys.address.voxel_z(),
    ));
    out.push_str(&format!(
        "  System Index:    {} (0x{:03X})\n",
        sys.address.solar_system_index(),
        sys.address.solar_system_index(),
    ));

    if sys.planets.is_empty() {
        out.push_str("\n  No planets discovered.\n");
    } else {
        out.push_str(&format!("\n  Planets ({}):\n", sys.planets.len()));
        out.push_str(&format!(
            "  {:<5} {:<18} {:<12} {}\n",
            "Idx", "Name", "Biome", "Flags"
        ));
        for p in &sys.planets {
            let name = p.name.as_deref().unwrap_or("(unnamed)");
            let biome = p
                .biome
                .map(|b| b.to_string())
                .unwrap_or_else(|| "?".to_string());
            let flags = if p.infested { "infested" } else { "" };
            out.push_str(&format!(
                "  {:<5} {:<18} {:<12} {}\n",
                p.index, name, biome, flags
            ));
        }
    }

    out
}

/// Format a base detail view.
pub fn format_show_base(result: &ShowBaseResult) -> String {
    let mut out = String::new();
    let base = &result.base;

    out.push_str("NMS Copilot -- Base Detail\n");
    out.push_str("=========================\n\n");

    out.push_str(&format!("  Name:            {}\n", base.name));
    out.push_str(&format!("  Type:            {}\n", base.base_type));
    out.push_str(&format!("  Galaxy:          {}\n", result.galaxy_name));
    out.push_str(&format!(
        "  Portal Glyphs:   {}\n",
        hex_to_emoji(&result.portal_hex)
    ));
    out.push_str(&format!("  Hex Address:     {}\n", result.portal_hex));
    if let Some(dist) = result.distance_from_player {
        out.push_str(&format!("  Distance:        {}\n", format_distance(dist)));
    }

    if let Some(ref system) = result.system {
        out.push_str(&format!(
            "  System:          {}\n",
            system.name.as_deref().unwrap_or("(unnamed)")
        ));
        out.push_str(&format!("  Planets:         {}\n", system.planets.len()));
    }

    out
}

/// Format a show result (dispatches to system or base).
pub fn format_show_result(result: &ShowResult) -> String {
    match result {
        ShowResult::System(s) => format_show_system(s),
        ShowResult::Base(b) => format_show_base(b),
    }
}

/// Format statistics output.
pub fn format_stats(result: &StatsResult) -> String {
    let mut out = String::new();

    out.push_str("NMS Copilot -- Galaxy Statistics\n");
    out.push_str("================================\n\n");

    out.push_str(&format!("  Systems:         {}\n", result.system_count));
    out.push_str(&format!("  Planets:         {}\n", result.planet_count));
    out.push_str(&format!("  Bases:           {}\n", result.base_count));
    out.push_str(&format!(
        "  Named Systems:   {}\n",
        result.named_system_count
    ));
    out.push_str(&format!(
        "  Named Planets:   {}\n",
        result.named_planet_count
    ));
    out.push_str(&format!("  Infested:        {}\n", result.infested_count));
    out.push('\n');

    // Biome distribution table
    if !result.biome_counts.is_empty() || result.unknown_biome_count > 0 {
        out.push_str("  Biome Distribution:\n");
        out.push_str(&format!("  {:<16} {:>6}\n", "Biome", "Count"));
        out.push_str(&format!("  {:<16} {:>6}\n", "-----", "-----"));

        // Sort biomes by count descending
        let mut biomes: Vec<_> = result.biome_counts.iter().collect();
        biomes.sort_by(|a, b| b.1.cmp(a.1));

        for (biome, count) in biomes {
            out.push_str(&format!("  {:<16} {:>6}\n", biome.to_string(), count));
        }

        if result.unknown_biome_count > 0 {
            out.push_str(&format!(
                "  {:<16} {:>6}\n",
                "(unknown)", result.unknown_biome_count
            ));
        }
    }

    out
}

/// Truncate a string to `max_len` characters, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;
    use nms_core::biome::Biome;
    use nms_core::system::{Planet, System};

    #[test]
    fn test_format_distance_small() {
        assert_eq!(format_distance(42.0), "42 ly");
        assert_eq!(format_distance(0.0), "0 ly");
        assert_eq!(format_distance(999.0), "999 ly");
    }

    #[test]
    fn test_format_distance_thousands() {
        assert_eq!(format_distance(1000.0), "1K ly");
        assert_eq!(format_distance(127_000.0), "127K ly");
        assert_eq!(format_distance(999_999.0), "1000K ly");
    }

    #[test]
    fn test_format_distance_millions() {
        assert_eq!(format_distance(1_000_000.0), "1.0M ly");
        assert_eq!(format_distance(1_500_000.0), "1.5M ly");
    }

    #[test]
    fn test_hex_to_emoji_basic() {
        let emoji = hex_to_emoji("000000000000");
        // All zeros = 12 Sunset glyphs
        assert!(emoji.contains('\u{1F305}')); // Sunset emoji
    }

    #[test]
    fn test_hex_to_emoji_length() {
        let emoji = hex_to_emoji("01717D8A4EA2");
        // Should produce 12 emoji (some multi-byte)
        assert!(!emoji.is_empty());
    }

    #[test]
    fn test_format_find_results_empty() {
        let output = format_find_results(&[]);
        assert!(output.contains("No results found"));
    }

    #[test]
    fn test_format_find_results_with_data() {
        let addr = GalacticAddress::new(100, 50, -200, 0x123, 0, 0);
        let results = vec![FindResult {
            planet: Planet::new(0, Some(Biome::Lush), None, false, Some("Eden".into()), None),
            system: System::new(
                addr,
                Some("Sol".into()),
                Some("Explorer".into()),
                None,
                vec![],
            ),
            distance_ly: 42_000.0,
            portal_hex: format!("{:012X}", addr.packed()),
        }];
        let output = format_find_results(&results);
        assert!(output.contains("Eden"));
        assert!(output.contains("Lush"));
        assert!(output.contains("Sol"));
        assert!(output.contains("42K ly"));
    }

    #[test]
    fn test_format_find_results_infested_marker() {
        let addr = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let results = vec![FindResult {
            planet: Planet::new(0, Some(Biome::Toxic), None, true, None, None),
            system: System::new(addr, None, None, None, vec![]),
            distance_ly: 0.0,
            portal_hex: "000000000001".into(),
        }];
        let output = format_find_results(&results);
        assert!(output.contains("Toxic*"));
    }

    #[test]
    fn test_format_show_system_output() {
        let addr = GalacticAddress::new(100, 50, -200, 0x123, 0, 0);
        let result = ShowSystemResult {
            system: System::new(
                addr,
                Some("Test System".into()),
                Some("Explorer".into()),
                None,
                vec![Planet::new(0, Some(Biome::Lush), None, false, None, None)],
            ),
            portal_hex: format!("{:012X}", addr.packed()),
            galaxy_name: "Euclid".into(),
            distance_from_player: Some(5000.0),
        };
        let output = format_show_system(&result);
        assert!(output.contains("Test System"));
        assert!(output.contains("Euclid"));
        assert!(output.contains("Explorer"));
        assert!(output.contains("5K ly"));
        assert!(output.contains("Lush"));
    }

    #[test]
    fn test_format_stats_output() {
        let mut biome_counts = std::collections::HashMap::new();
        biome_counts.insert(Biome::Lush, 10);
        biome_counts.insert(Biome::Toxic, 5);

        let result = StatsResult {
            system_count: 50,
            planet_count: 120,
            base_count: 3,
            biome_counts,
            unknown_biome_count: 5,
            named_planet_count: 20,
            named_system_count: 30,
            infested_count: 2,
        };
        let output = format_stats(&result);
        assert!(output.contains("50"));
        assert!(output.contains("120"));
        assert!(output.contains("Lush"));
        assert!(output.contains("10"));
        assert!(output.contains("(unknown)"));
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }
}
