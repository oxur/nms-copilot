//! `nms info` command -- display save file summary.

use std::collections::HashMap;
use std::path::PathBuf;

use nms_core::galaxy::Galaxy;
use nms_query::display::hex_to_emoji;
use nms_query::table::{Builder, build_table, nms_theme};
use nms_save::model::{PlayerStateData, SaveRoot};

pub fn run(save_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let path = match save_path {
        Some(p) => p,
        None => nms_save::locate::find_most_recent_save()?
            .path()
            .to_path_buf(),
    };

    let save = nms_save::parse_save_file(&path)?;
    print_summary(&save);
    Ok(())
}

fn print_summary(save: &SaveRoot) {
    let theme = nms_theme();

    let ps = save.active_player_state();
    let ua = &ps.universe_address;
    let galaxy = Galaxy::by_index(ua.reality_index);
    let ga = &ua.galactic_address;

    let mut builder = Builder::default();
    builder.push_record(["Property", "Detail"]);
    builder.push_record(["Save Name", &save.common_state_data.save_name]);
    builder.push_record(["Platform", &save.platform]);
    builder.push_record(["Version", &save.version.to_string()]);
    builder.push_record([
        "Play Time",
        &format_play_time(save.common_state_data.total_play_time),
    ]);
    builder.push_record(["Game Mode", &format_game_mode(save.base_context.game_mode)]);
    builder.push_record(["Galaxy", galaxy.name]);
    builder.push_record([
        "Voxel Position",
        &format!("X={}, Y={}, Z={}", ga.voxel_x, ga.voxel_y, ga.voxel_z),
    ]);
    builder.push_record(["System Index", &ga.solar_system_index.to_string()]);
    builder.push_record(["Planet Index", &ga.planet_index.to_string()]);
    builder.push_record(["", ""]);

    println!(
        "{}",
        build_table(builder, &["SAVE FILE SUMMARY"], &theme, "")
    );
    println!();

    print_discoveries(save, &theme);
    print_bases(ps, &theme);
    print_currencies(ps, &theme);
}

fn print_discoveries(save: &SaveRoot, theme: &nms_query::table::TableStyleConfig) {
    let records = &save.discovery_manager_data.discovery_data_v1.store.record;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for rec in records {
        *counts.entry(rec.dd.dt.as_str()).or_insert(0) += 1;
    }

    let mut builder = Builder::default();
    builder.push_record(["Category", "Count"]);
    builder.push_record([
        "Total Discoveries".to_string(),
        format_number(records.len() as i64),
    ]);

    for (label, key) in [
        ("Solar Systems", "SolarSystem"),
        ("Planets", "Planet"),
        ("Sectors", "Sector"),
        ("Animals", "Animal"),
        ("Flora", "Flora"),
        ("Minerals", "Mineral"),
    ] {
        let count = counts.get(key).copied().unwrap_or(0);
        if count > 0 {
            builder.push_record([label.to_string(), format_number(count as i64)]);
        }
    }
    builder.push_record(["".to_string(), "".to_string()]);

    println!(
        "{}",
        build_table(builder, &["DISCOVERIES"], theme, "Categories")
    );
    println!();
}

fn print_bases(ps: &PlayerStateData, theme: &nms_query::table::TableStyleConfig) {
    let bases = &ps.persistent_player_bases;
    if !bases.is_empty() {
        let mut builder = Builder::default();
        builder.push_record(["Name", "Type", "Address", "Portal Glyphs"]);
        for base in bases {
            let name = if base.name.is_empty() {
                "(unnamed)"
            } else {
                &base.name
            };
            let hex = format!("{:012X}", base.galactic_address.0 & 0xFFFF_FFFF_FFFF);
            let glyphs = hex_to_emoji(&hex);
            builder.push_record([
                name.to_string(),
                base.base_type.persistent_base_types.clone(),
                hex,
                glyphs,
            ]);
        }
        builder.push_record(["", "", "", ""]);
        println!("{}", build_table(builder, &["BASES"], theme, "Bases"));
    }
    println!();
}

fn print_currencies(ps: &PlayerStateData, theme: &nms_query::table::TableStyleConfig) {
    let mut builder = Builder::default();
    builder.push_record(["Name", "Balance"]);
    builder.push_record(["Units".to_string(), format_number(ps.units)]);
    builder.push_record(["Nanites".to_string(), format_number(ps.nanites)]);
    builder.push_record(["Quicksilver".to_string(), format_number(ps.specials)]);
    builder.push_record(["".to_string(), "".to_string()]);
    println!(
        "{}",
        build_table(builder, &["CURRENCIES"], theme, "Currencies")
    );
}

/// Format seconds as "Xd Yh Zm" or "Xh Ym" or "Xm Ys".
fn format_play_time(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m {}s", minutes, seconds % 60)
    }
}

/// Format game mode integer to display string.
fn format_game_mode(mode: u32) -> String {
    let name = match mode {
        0 => "Unspecified",
        1 => "Normal",
        2 => "Creative",
        3 => "Survival",
        4 => "Ambient",
        5 => "Permadeath",
        6 => "Seasonal/Expedition",
        _ => "Unknown",
    };
    format!("{name} ({mode})")
}

/// Format an integer with thousands separators (commas).
fn format_number(n: i64) -> String {
    let negative = n < 0;
    let abs = if negative {
        (n as i128).unsigned_abs()
    } else {
        n as u128
    };
    let s = abs.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    let formatted: String = result.chars().rev().collect();
    if negative {
        format!("-{formatted}")
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_play_time_zero() {
        assert_eq!(format_play_time(0), "0m 0s");
    }

    #[test]
    fn format_play_time_seconds_only() {
        assert_eq!(format_play_time(45), "0m 45s");
    }

    #[test]
    fn format_play_time_minutes_and_seconds() {
        assert_eq!(format_play_time(125), "2m 5s");
    }

    #[test]
    fn format_play_time_hours_and_minutes() {
        assert_eq!(format_play_time(3661), "1h 1m");
    }

    #[test]
    fn format_play_time_days() {
        assert_eq!(format_play_time(90061), "1d 1h 1m");
    }

    #[test]
    fn format_play_time_actual_save_value() {
        assert_eq!(format_play_time(2464349), "28d 12h 32m");
    }

    #[test]
    fn format_number_positive() {
        assert_eq!(format_number(1234567890), "1,234,567,890");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(format_number(-919837762), "-919,837,762");
    }

    #[test]
    fn format_number_zero() {
        assert_eq!(format_number(0), "0");
    }

    #[test]
    fn format_number_small() {
        assert_eq!(format_number(42), "42");
    }

    #[test]
    fn format_number_thousands() {
        assert_eq!(format_number(1000), "1,000");
    }

    #[test]
    fn format_game_mode_normal() {
        assert_eq!(format_game_mode(1), "Normal (1)");
    }

    #[test]
    fn format_game_mode_creative() {
        assert_eq!(format_game_mode(2), "Creative (2)");
    }

    #[test]
    fn format_game_mode_expedition() {
        assert_eq!(format_game_mode(6), "Seasonal/Expedition (6)");
    }

    #[test]
    fn format_game_mode_unknown() {
        assert_eq!(format_game_mode(99), "Unknown (99)");
    }
}
