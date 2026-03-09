//! Output formatting for query results.
//!
//! All formatters return `String` -- the caller prints to stdout.
//! Each formatter accepts a [`Theme`] for API compatibility but uses
//! `nms_theme()` / `nms_theme_no_color()` for table styling internally.

use crate::find::FindResult;
use crate::route::RouteResult;
use crate::show::{ShowBaseResult, ShowResult, ShowSystemResult};
use crate::stats::StatsResult;
use crate::table::{Builder, build_table, nms_theme, nms_theme_no_color};
use crate::theme::Theme;

/// Format a distance in light-years for display.
///
/// - 0.0 (same system): `"< 1 ly"`
/// - ~265 ly (same voxel): `"~265 ly (±100)"` — SD from Robbins' variance
/// - Cross-voxel: `"~800 ly (±163)"` — projection SD, √(2/12) × 400
/// - Large distances: `"~40.0K ly"` — ± dropped when < 5% of distance
///
/// All non-zero distances are prefixed with `~` to indicate approximation.
/// The `±` uncertainty (1σ) is shown when it represents ≥ 5% of the distance.
pub fn format_distance(ly: f64) -> String {
    use nms_core::address::{CROSS_VOXEL_SD, SAME_VOXEL_SD, VOXEL_UNCERTAINTY};

    if ly == 0.0 {
        return "< 1 ly".to_string();
    }

    // Same-voxel estimates use their own SD; cross-voxel uses the projection SD.
    let sd = if (ly - VOXEL_UNCERTAINTY).abs() < 1.0 {
        SAME_VOXEL_SD
    } else {
        CROSS_VOXEL_SD
    };
    let show_error = sd / ly >= 0.05; // Drop ± when < 5% of distance

    let dist_str = if ly < 1_000.0 {
        format!("{:.0} ly", ly)
    } else if ly < 1_000_000.0 {
        format!("{:.1}K ly", ly / 1_000.0)
    } else {
        format!("{:.1}M ly", ly / 1_000_000.0)
    };

    if show_error {
        format!("~{dist_str} (\u{00B1}{:.0})", sd)
    } else {
        format!("~{dist_str}")
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

/// Select the appropriate table theme based on whether the `Theme` has colors.
fn table_theme_for(theme: &Theme) -> crate::table::TableStyleConfig {
    if theme.header.fg.is_some() || theme.header.bold {
        nms_theme()
    } else {
        nms_theme_no_color()
    }
}

/// Format find results as a themed table.
pub fn format_find_results(results: &[FindResult], theme: &Theme) -> String {
    if results.is_empty() {
        return "  No results found.\n".to_string();
    }

    let table_theme = table_theme_for(theme);
    let mut builder = Builder::default();
    builder.push_record([
        "#",
        "Planet",
        "Biome",
        "System",
        "Distance",
        "Address",
        "Portal Glyphs",
    ]);

    for (i, r) in results.iter().enumerate() {
        let planet_name = r.planet.name.as_deref().unwrap_or("-");
        let biome_str = r
            .planet
            .biome
            .map(|b| {
                let mut s = b.to_string();
                if let Some(sub) = r.planet.biome_subtype {
                    let sub_name = format!("{sub:?}");
                    if let Some(variant) = sub_name.strip_prefix(&s) {
                        s.push_str(&format!(" ({variant})"));
                    }
                }
                if r.planet.infested {
                    s.push('*');
                }
                s
            })
            .unwrap_or_else(|| "?".to_string());
        let system_name = r.system.name.as_deref().unwrap_or("-");
        let distance = format_distance(r.distance_ly);
        let address = r.portal_hex.clone();
        let glyphs = hex_to_emoji(&r.portal_hex);

        builder.push_record([
            (i + 1).to_string(),
            truncate(planet_name, 20),
            truncate(&biome_str, 22),
            truncate(system_name, 22),
            distance,
            address,
            glyphs,
        ]);
    }
    builder.push_record(["", "", "", "", "", "", ""]);

    build_table(builder, &["SEARCH RESULTS"], &table_theme, "Results")
}

/// Format a system detail view.
pub fn format_show_system(result: &ShowSystemResult, theme: &Theme) -> String {
    let table_theme = table_theme_for(theme);
    let sys = &result.system;
    let name = sys.name.as_deref().unwrap_or("-");

    let mut builder = Builder::default();
    builder.push_record(["Property", "Detail"]);
    builder.push_record(["Name", name]);
    builder.push_record(["Galaxy", &result.galaxy_name]);
    builder.push_record(["Portal Glyphs", &hex_to_emoji(&result.portal_hex)]);
    builder.push_record(["Hex Address", &result.portal_hex]);
    builder.push_record(["Discoverer", sys.discoverer.as_deref().unwrap_or("unknown")]);
    if let Some(dist) = result.distance_from_player {
        builder.push_record(["Distance", &format_distance(dist)]);
    }
    builder.push_record([
        "Voxel Position",
        &format!(
            "X={}, Y={}, Z={}",
            sys.address.voxel_x(),
            sys.address.voxel_y(),
            sys.address.voxel_z()
        ),
    ]);
    builder.push_record([
        "System Index",
        &format!(
            "{} (0x{:03X})",
            sys.address.solar_system_index(),
            sys.address.solar_system_index()
        ),
    ]);
    builder.push_record(["", ""]);

    let mut out = String::new();
    out.push_str(&build_table(builder, &["SYSTEM DETAIL"], &table_theme, ""));

    if sys.planets.is_empty() {
        out.push_str("\n  No planets discovered.\n");
    } else {
        out.push('\n');
        let mut pbuilder = Builder::default();
        pbuilder.push_record(["Index", "Name", "Biome", "Flags"]);
        for p in &sys.planets {
            let pname = p.name.as_deref().unwrap_or("-");
            let biome_str = p
                .biome
                .map(|b| {
                    let mut s = b.to_string();
                    if let Some(sub) = p.biome_subtype {
                        let sub_name = format!("{sub:?}");
                        if let Some(variant) = sub_name.strip_prefix(&s) {
                            s.push_str(&format!(" ({variant})"));
                        }
                    }
                    s
                })
                .unwrap_or_else(|| "?".to_string());
            let flags = if p.infested { "infested" } else { "" };
            pbuilder.push_record([&p.index.to_string(), pname, &biome_str, flags]);
        }
        pbuilder.push_record(["", "", "", ""]);
        out.push_str(&build_table(
            pbuilder,
            &[&format!("PLANETS ({})", sys.planets.len())],
            &table_theme,
            "Planets",
        ));
    }

    out
}

/// Format a base detail view.
pub fn format_show_base(result: &ShowBaseResult, theme: &Theme) -> String {
    let table_theme = table_theme_for(theme);
    let base = &result.base;

    let mut builder = Builder::default();
    builder.push_record(["Property", "Detail"]);
    builder.push_record(["Name", &base.name]);
    builder.push_record(["Type", &base.base_type.to_string()]);
    builder.push_record(["Galaxy", &result.galaxy_name]);
    builder.push_record(["Portal Glyphs", &hex_to_emoji(&result.portal_hex)]);
    builder.push_record(["Hex Address", &result.portal_hex]);
    if let Some(dist) = result.distance_from_player {
        builder.push_record(["Distance", &format_distance(dist)]);
    }
    if let Some(ref system) = result.system {
        builder.push_record(["System", system.name.as_deref().unwrap_or("-")]);
        builder.push_record(["Planets", &system.planets.len().to_string()]);
    }
    builder.push_record(["", ""]);

    let mut out = String::new();
    out.push_str(&build_table(builder, &["BASE DETAIL"], &table_theme, ""));
    out
}

/// Format a show result (dispatches to system or base).
pub fn format_show_result(result: &ShowResult, theme: &Theme) -> String {
    match result {
        ShowResult::System(s) => format_show_system(s, theme),
        ShowResult::Base(b) => format_show_base(b, theme),
    }
}

/// Format statistics output.
pub fn format_stats(result: &StatsResult, theme: &Theme) -> String {
    let table_theme = table_theme_for(theme);

    let mut builder = Builder::default();
    builder.push_record(["Metric", "Count"]);
    builder.push_record(["Systems", &result.system_count.to_string()]);
    builder.push_record(["Planets", &result.planet_count.to_string()]);
    builder.push_record(["Bases", &result.base_count.to_string()]);
    builder.push_record(["Named Systems", &result.named_system_count.to_string()]);
    builder.push_record(["Named Planets", &result.named_planet_count.to_string()]);
    builder.push_record(["Infested", &result.infested_count.to_string()]);
    builder.push_record(["", ""]);

    let mut out = String::new();
    out.push_str(&build_table(
        builder,
        &["GALAXY STATISTICS"],
        &table_theme,
        "",
    ));

    // Biome distribution table
    if !result.biome_counts.is_empty() || result.unknown_biome_count > 0 {
        out.push('\n');
        let mut bbuilder = Builder::default();
        bbuilder.push_record(["Name", "Count"]);

        let mut biomes: Vec<_> = result.biome_counts.iter().collect();
        biomes.sort_by(|a, b| b.1.cmp(a.1));

        for (biome, count) in biomes {
            bbuilder.push_record([biome.to_string(), count.to_string()]);
        }

        if result.unknown_biome_count > 0 {
            bbuilder.push_record([
                "(unknown)".to_string(),
                result.unknown_biome_count.to_string(),
            ]);
        }
        bbuilder.push_record(["".to_string(), "".to_string()]);
        out.push_str(&build_table(
            bbuilder,
            &["BIOME DISTRIBUTION"],
            &table_theme,
            "Biomes",
        ));
    }

    out
}

/// Format a route result as an itinerary table.
pub fn format_route(result: &RouteResult, model: &nms_graph::GalaxyModel, theme: &Theme) -> String {
    if result.route.hops.is_empty() {
        return "  No route computed.\n".to_string();
    }

    let table_theme = table_theme_for(theme);
    let mut builder = Builder::default();
    builder.push_record([
        "Hop",
        "System",
        "Distance",
        "Cumulative",
        "Address",
        "Portal Glyphs",
    ]);

    let mut hop_number = 0u32;
    for hop in &result.route.hops {
        let system_name = model
            .system(&hop.system_id)
            .and_then(|s| s.name.as_deref())
            .unwrap_or("-");

        let portal_hex = model
            .system(&hop.system_id)
            .map(|s| format!("{:012X}", s.address.packed()))
            .unwrap_or_else(|| format!("{:012X}", hop.system_id.0));
        let glyphs = hex_to_emoji(&portal_hex);

        let distance = format_distance(hop.leg_distance_ly);
        let cumulative = format_distance(hop.cumulative_ly);

        if hop.is_waypoint {
            builder.push_record([
                "*".to_string(),
                format!("\u{21B3} {}", truncate(system_name, 19)),
                distance,
                cumulative,
                portal_hex,
                glyphs,
            ]);
        } else {
            hop_number += 1;
            builder.push_record([
                hop_number.to_string(),
                truncate(system_name, 21),
                distance,
                cumulative,
                portal_hex,
                glyphs,
            ]);
        }
    }
    builder.push_record(["", "", "", "", "", ""]);

    let mut out = build_table(builder, &["ROUTE ITINERARY"], &table_theme, "Hops");

    // Summary line
    out.push('\n');
    let total = format_distance(result.route.total_distance_ly);
    match (result.warp_jumps, result.warp_range) {
        (Some(jumps), Some(range)) => {
            out.push_str(&format!(
                "  Route: {} targets, {} total ({} warp jumps at {} range)\n",
                result.targets_visited,
                total,
                jumps,
                format_distance(range),
            ));
        }
        _ => {
            out.push_str(&format!(
                "  Route: {} targets, {} total\n",
                result.targets_visited, total,
            ));
        }
    }

    let algo_name = match result.algorithm {
        nms_graph::RoutingAlgorithm::NearestNeighbor => "nearest-neighbor",
        nms_graph::RoutingAlgorithm::TwoOpt => "2-opt",
    };
    out.push_str(&format!("  Algorithm: {algo_name}\n"));

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

    fn plain() -> Theme {
        Theme::none()
    }

    #[test]
    fn test_format_distance_zero() {
        assert_eq!(format_distance(0.0), "< 1 ly");
    }

    #[test]
    fn test_format_distance_small_with_uncertainty() {
        // 42 ly (cross-voxel): SD=163, 163/42 > 5%, ± shown
        assert_eq!(format_distance(42.0), "~42 ly (\u{00B1}163)");
        assert_eq!(format_distance(999.0), "~999 ly (\u{00B1}163)");
    }

    #[test]
    fn test_format_distance_same_voxel() {
        // ~265 ly (same-voxel Robbins' estimate): SD=100, ± shown
        let voxel = nms_core::address::VOXEL_UNCERTAINTY;
        assert_eq!(format_distance(voxel), "~265 ly (\u{00B1}100)");
    }

    #[test]
    fn test_format_distance_thousands() {
        // 1000 ly (cross-voxel): SD=163, 163/1000 = 16% >= 5%, ± shown
        assert_eq!(format_distance(1000.0), "~1.0K ly (\u{00B1}163)");
        // 3000 ly: 163/3000 = 5.4% >= 5%, ± shown
        assert_eq!(format_distance(3000.0), "~3.0K ly (\u{00B1}163)");
        // 3200 ly: 163/3200 ≈ 5.1% >= 5%, ± shown
        assert_eq!(format_distance(3200.0), "~3.2K ly (\u{00B1}163)");
        // 3300 ly: 163/3300 < 5%, ± dropped
        assert_eq!(format_distance(3300.0), "~3.3K ly");
        // Large thousands: no ±
        assert_eq!(format_distance(40_000.0), "~40.0K ly");
    }

    #[test]
    fn test_format_distance_millions() {
        assert_eq!(format_distance(1_000_000.0), "~1.0M ly");
        assert_eq!(format_distance(1_500_000.0), "~1.5M ly");
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
        let output = format_find_results(&[], &plain());
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
        let output = format_find_results(&results, &plain());
        assert!(output.contains("Eden"));
        assert!(output.contains("Lush"));
        assert!(output.contains("Sol"));
        assert!(output.contains("~42.0K ly"));
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
        let output = format_find_results(&results, &plain());
        assert!(output.contains("Toxic*"));
    }

    #[test]
    fn test_format_find_results_with_dark_theme_contains_ansi() {
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
        let output = format_find_results(&results, &Theme::default_dark());
        // Should contain ANSI escape sequences (from hex color theme)
        assert!(output.contains("\x1b["));
        // But still contain the data
        assert!(output.contains("Eden"));
        assert!(output.contains("Sol"));
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
        let output = format_show_system(&result, &plain());
        assert!(output.contains("Test System"));
        assert!(output.contains("Euclid"));
        assert!(output.contains("Explorer"));
        assert!(output.contains("~5.0K ly"));
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
        let output = format_stats(&result, &plain());
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

    fn test_route_model() -> nms_graph::GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| nms_graph::GalaxyModel::from_save(&save))
            .unwrap()
    }

    #[test]
    fn test_format_route_empty() {
        let model = test_route_model();
        let result = RouteResult {
            route: nms_graph::Route {
                hops: vec![],
                total_distance_ly: 0.0,
            },
            warp_range: None,
            warp_jumps: None,
            algorithm: nms_graph::RoutingAlgorithm::TwoOpt,
            targets_visited: 0,
        };
        let output = format_route(&result, &model, &plain());
        assert!(output.contains("No route computed"));
    }

    #[test]
    fn test_format_route_header_present() {
        let model = test_route_model();
        let result = RouteResult {
            route: nms_graph::Route {
                hops: vec![nms_graph::RouteHop {
                    system_id: nms_graph::SystemId(0x001000000064),
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                }],
                total_distance_ly: 0.0,
            },
            warp_range: None,
            warp_jumps: None,
            algorithm: nms_graph::RoutingAlgorithm::TwoOpt,
            targets_visited: 0,
        };
        let output = format_route(&result, &model, &plain());
        assert!(output.contains("Hop"));
        assert!(output.contains("System"));
        assert!(output.contains("Distance"));
        assert!(output.contains("Cumulative"));
        assert!(output.contains("Portal Glyphs"));
    }

    #[test]
    fn test_format_route_algorithm_shown() {
        let model = test_route_model();
        let result = RouteResult {
            route: nms_graph::Route {
                hops: vec![nms_graph::RouteHop {
                    system_id: nms_graph::SystemId(0x001000000064),
                    leg_distance_ly: 0.0,
                    cumulative_ly: 0.0,
                    is_waypoint: false,
                }],
                total_distance_ly: 0.0,
            },
            warp_range: None,
            warp_jumps: None,
            algorithm: nms_graph::RoutingAlgorithm::NearestNeighbor,
            targets_visited: 0,
        };
        let output = format_route(&result, &model, &plain());
        assert!(output.contains("Algorithm: nearest-neighbor"));

        let result_2opt = RouteResult {
            algorithm: nms_graph::RoutingAlgorithm::TwoOpt,
            ..result
        };
        let output_2opt = format_route(&result_2opt, &model, &plain());
        assert!(output_2opt.contains("Algorithm: 2-opt"));
    }

    #[test]
    fn test_format_route_warp_jumps_shown() {
        let model = test_route_model();
        let result = RouteResult {
            route: nms_graph::Route {
                hops: vec![
                    nms_graph::RouteHop {
                        system_id: nms_graph::SystemId(0x001000000064),
                        leg_distance_ly: 0.0,
                        cumulative_ly: 0.0,
                        is_waypoint: false,
                    },
                    nms_graph::RouteHop {
                        system_id: nms_graph::SystemId(0x002000000C80),
                        leg_distance_ly: 5000.0,
                        cumulative_ly: 5000.0,
                        is_waypoint: false,
                    },
                ],
                total_distance_ly: 5000.0,
            },
            warp_range: Some(2000.0),
            warp_jumps: Some(3),
            algorithm: nms_graph::RoutingAlgorithm::TwoOpt,
            targets_visited: 1,
        };
        let output = format_route(&result, &model, &plain());
        assert!(output.contains("3 warp jumps"));
        assert!(output.contains("~2.0K ly") && output.contains("range"));
    }

    #[test]
    fn test_format_route_waypoint_marker() {
        let model = test_route_model();
        let result = RouteResult {
            route: nms_graph::Route {
                hops: vec![
                    nms_graph::RouteHop {
                        system_id: nms_graph::SystemId(0x001000000064),
                        leg_distance_ly: 0.0,
                        cumulative_ly: 0.0,
                        is_waypoint: false,
                    },
                    nms_graph::RouteHop {
                        system_id: nms_graph::SystemId(0x001000000064),
                        leg_distance_ly: 1000.0,
                        cumulative_ly: 1000.0,
                        is_waypoint: true,
                    },
                    nms_graph::RouteHop {
                        system_id: nms_graph::SystemId(0x001000000064),
                        leg_distance_ly: 1000.0,
                        cumulative_ly: 2000.0,
                        is_waypoint: false,
                    },
                ],
                total_distance_ly: 2000.0,
            },
            warp_range: None,
            warp_jumps: None,
            algorithm: nms_graph::RoutingAlgorithm::TwoOpt,
            targets_visited: 1,
        };
        let output = format_route(&result, &model, &plain());
        assert!(output.contains("*"));
        assert!(output.contains("\u{21B3}"));
    }
}
