//! NMS-themed table formatting using `oxur-cli` tables.
//!
//! Provides a deep-space color palette for all tabular output, plus a no-color
//! variant for piped output and MCP consumption.
//!
//! The `build_table` function handles the `oxur-cli` row layout convention
//! automatically: callers push `[header, data..., footer]` and the helper
//! inserts a title row at position 0 so the theme styling aligns correctly.

pub use oxur_cli::table::Builder;
pub use oxur_cli::table::TableStyleConfig;
use tabled::settings::Color;
use tabled::settings::formatting::Justification;
use tabled::settings::object::Segment;
use tabled::settings::style::BorderColor;

/// Background for every other group in a grouped table: a step lighter than the
/// data-row background (`#0A1929`) so alternating groups read as bands.
const GROUP_SHADE_BG: (u8, u8, u8) = (0x13, 0x2F, 0x4D);

/// Dummy type to satisfy the `Tabled` trait bound on `apply_to_table`.
///
/// Uses `oxur_cli`'s re-exported `Tabled` to match the version expected by
/// `TableStyleConfig::apply_to_table`.
#[derive(oxur_cli::table::Tabled)]
struct Dummy {
    x: String,
}

/// NMS deep-space theme as inline TOML.
///
/// The `[title]` section is enabled so that `apply_to_table` correctly maps:
/// - Row 0 = title (auto-inserted by `build_table`)
/// - Row 1 = header (caller's first `push_record`)
/// - Rows 2..n-1 = data
/// - Row n = footer (caller's last `push_record`, styled by `[footer]`)
const NMS_THEME: &str = r##"
[table]
padding_left = 0
padding_right = 0
padding_top = 0
padding_bottom = 0

[title]
enabled = true
bg_color = "#1E3A5F"
fg_color = "#E0F0FF"
justification_char = " "
vertical_fg_color = "#1E3A5F"
vertical_bg_color = "#1E3A5F"

[header]
bg_color = "#2C5F8A"
fg_color = "#A0C8E0"
justification_char = " "
vertical_char = "│"
vertical_bg_color = "#2C5F8A"
vertical_fg_color = "#2C5F8A"

[rows]
colors = [
    { bg = "#0A1929", fg = "#B0D0E8" },
    { bg = "#0A1929", fg = "#8BBBD0" },
]
justification_char = " "

[style]
vertical_bg_color = "#0A1929"
vertical_fg_color = "#162D45"

[footer]
enabled = true
bg_color = "#1E3A5F"
fg_color = "#4A9BC7"
justification_char = " "
vertical_bg_color = "#1E3A5F"
vertical_fg_color = "#1E3A5F"
"##;

/// No-color theme for piped output and MCP.
///
/// Title is enabled but invisible (black on black) so the row layout matches.
const NMS_THEME_NO_COLOR: &str = r##"
[table]
padding_left = 0
padding_right = 0
padding_top = 0
padding_bottom = 0

[title]
enabled = true
bg_color = "black"
fg_color = "black"
justification_char = " "

[header]
bg_color = "black"
fg_color = "white"
justification_char = " "

[rows]
colors = [
    { bg = "black", fg = "white" },
]

[style]

[footer]
enabled = true
bg_color = "black"
fg_color = "black"
"##;

/// Parse and return the NMS deep-space color theme.
pub fn nms_theme() -> TableStyleConfig {
    toml::from_str(NMS_THEME).expect("NMS_THEME TOML is valid")
}

/// Parse and return the no-color theme.
pub fn nms_theme_no_color() -> TableStyleConfig {
    toml::from_str(NMS_THEME_NO_COLOR).expect("NMS_THEME_NO_COLOR TOML is valid")
}

/// Build a table string from a `Builder` and apply the given theme.
///
/// The caller pushes records in the order: `[header, data..., empty_footer]`.
/// This function automatically:
/// 1. Pads each cell with a leading and trailing space (workaround for
///    `tabled` cell padding not inheriting row background colors in
///    header/footer rows).
/// 2. Inserts a title row at position 0 so the `oxur-cli` theme styling
///    aligns correctly (title at row 0, header at row 1, data at rows 2+,
///    footer at last row). Title words can be spread across columns by
///    passing multiple elements in the `title` slice.
/// 3. If `count_label` is non-empty, replaces the empty footer with a
///    "Total: N <label>" summary. If the combined text fits within the
///    widest column-0 entry, it goes entirely in column 0; otherwise
///    "Total:" goes in column 0 and "N <label>" in column 1.
pub fn build_table(
    builder: Builder,
    title: &[&str],
    theme: &TableStyleConfig,
    count_label: &str,
) -> String {
    build_table_inner(builder, title, theme, count_label, None)
}

/// Like [`build_table`], but shades alternating groups of data rows.
///
/// `groups[i]` is the group index of data row `i` (the first record after the
/// header is row 0). Consecutive rows with the same group index form one band;
/// bands with an odd group index get a lighter background. Colour is the only
/// cue, so the no-colour theme renders exactly like [`build_table`].
pub fn build_grouped_table(
    builder: Builder,
    title: &[&str],
    theme: &TableStyleConfig,
    count_label: &str,
    groups: &[usize],
) -> String {
    build_table_inner(builder, title, theme, count_label, Some(groups))
}

/// Parse a `#RRGGBB` colour string.
fn hex_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

fn build_table_inner(
    builder: Builder,
    title: &[&str],
    theme: &TableStyleConfig,
    count_label: &str,
    groups: Option<&[usize]>,
) -> String {
    let mut records: Vec<Vec<String>> = builder.into();

    // The last row is the empty footer placeholder — remove it.
    // Data rows = total - 1 (header) - 1 (footer) = records.len() - 2.
    let data_count = if records.len() >= 2 {
        records.len() - 2
    } else {
        0
    };
    let col_count = records.first().map(|r| r.len()).unwrap_or(0);

    // Compute max display width of column 0 across data rows (skip header at 0).
    let max_col0_width = records
        .iter()
        .skip(1) // skip header
        .take(data_count) // only data rows
        .filter_map(|r| r.first())
        .map(|s| s.len())
        .max()
        .unwrap_or(0);

    // Build footer row.
    if !count_label.is_empty() && col_count >= 2 {
        // Remove the empty footer placeholder.
        if records.last().map(|r| r.iter().all(|c| c.is_empty())) == Some(true) {
            records.pop();
        }
        let combined = format!("Total: {data_count} {count_label}");
        let mut footer: Vec<String> = std::iter::repeat_n(String::new(), col_count).collect();
        if combined.len() <= max_col0_width {
            footer[0] = combined;
        } else {
            footer[0] = "Total:".to_string();
            footer[1] = format!("{data_count} {count_label}");
        }
        records.push(footer);
    }

    // Rebuild with space-padded cells.
    let mut padded = Builder::default();
    for row in records {
        let padded_row: Vec<String> = row
            .into_iter()
            .map(|cell| {
                if cell.is_empty() {
                    cell
                } else {
                    format!(" {cell} ")
                }
            })
            .collect();
        padded.push_record(padded_row);
    }

    let final_col_count = padded.count_columns();
    let mut title_row: Vec<String> = std::iter::repeat_n(String::new(), final_col_count).collect();
    for (i, word) in title.iter().enumerate() {
        if i < final_col_count && !word.is_empty() {
            title_row[i] = format!(" {word} ");
        }
    }
    padded.insert_record(0, title_row);

    let mut table = padded.build();
    theme.apply_to_table::<Dummy>(&mut table);

    // Overlay the group shading. The theme only carries hex colours when it is the
    // colour theme, so a theme without a parseable row background gets no overlay.
    if let Some(groups) = groups
        && let Some(row_fg) = theme.rows.colors.first().and_then(|c| hex_rgb(&c.fg))
        && theme
            .rows
            .colors
            .first()
            .and_then(|c| hex_rgb(&c.bg))
            .is_some()
    {
        let (r, g, b) = GROUP_SHADE_BG;
        let just_char = theme
            .rows
            .justification_char
            .as_deref()
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');
        let border_fg = theme
            .style
            .vertical_fg_color
            .as_deref()
            .and_then(hex_rgb)
            .unwrap_or(row_fg);
        for (i, &group) in groups.iter().enumerate().take(data_count) {
            if group % 2 == 0 {
                continue;
            }
            // Data rows start at table row 2 (title at 0, header at 1). Address the
            // cells themselves: the theme sets its colours per cell, and cell-level
            // settings shadow row-level ones.
            let row = i + 2;
            let cell = Color::rgb_fg(row_fg.0, row_fg.1, row_fg.2) | Color::rgb_bg(r, g, b);
            table.modify(Segment::new(row..row + 1, 0..final_col_count), cell);
            table.modify(
                Segment::new(row..row + 1, 0..final_col_count),
                Justification::new(just_char).color(Color::rgb_bg(r, g, b)),
            );
            let border =
                Color::rgb_fg(border_fg.0, border_fg.1, border_fg.2) | Color::rgb_bg(r, g, b);
            table.modify(
                Segment::new(row..row + 1, 0..final_col_count),
                BorderColor::filled(border),
            );
        }
    }
    format!("\n{}\n\n", table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nms_theme_parses_without_panic() {
        let _theme = nms_theme();
    }

    #[test]
    fn test_nms_theme_no_color_parses_without_panic() {
        let _theme = nms_theme_no_color();
    }

    #[test]
    fn test_build_table_produces_output() {
        let mut builder = Builder::default();
        builder.push_record(["Name", "Value"]);
        builder.push_record(["foo", "bar"]);
        builder.push_record(["", ""]);
        let output = build_table(builder, &[], &nms_theme(), "");
        assert!(output.contains("Name"));
        assert!(output.contains("Value"));
        assert!(output.contains("foo"));
        assert!(output.contains("bar"));
    }

    #[test]
    fn test_build_table_no_color_produces_output() {
        let mut builder = Builder::default();
        builder.push_record(["Col"]);
        builder.push_record(["data"]);
        builder.push_record([""]);
        let output = build_table(builder, &[], &nms_theme_no_color(), "");
        assert!(output.contains("Col"));
        assert!(output.contains("data"));
    }

    #[test]
    fn test_build_table_contains_ansi_with_nms_theme() {
        let mut builder = Builder::default();
        builder.push_record(["Header"]);
        builder.push_record(["value"]);
        builder.push_record([""]);
        let output = build_table(builder, &[], &nms_theme(), "");
        // NMS theme uses hex colors which produce ANSI escape codes
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_build_table_preserves_all_data() {
        let mut builder = Builder::default();
        builder.push_record(["A", "B", "C"]);
        builder.push_record(["1", "2", "3"]);
        builder.push_record(["x", "y", "z"]);
        builder.push_record(["", "", ""]);
        let output = build_table(builder, &[], &nms_theme(), "");
        for val in ["A", "B", "C", "1", "2", "3", "x", "y", "z"] {
            assert!(output.contains(val), "Missing '{val}' in output");
        }
    }

    #[test]
    fn test_hex_rgb_parses_theme_colours() {
        assert_eq!(hex_rgb("#0A1929"), Some((0x0A, 0x19, 0x29)));
        assert_eq!(hex_rgb("black"), None);
        assert_eq!(hex_rgb("#123"), None);
    }

    #[test]
    fn test_build_grouped_table_shades_odd_groups_in_colour_theme() {
        let mut builder = Builder::default();
        builder.push_record(["A", "B"]);
        builder.push_record(["g0-1", "x"]);
        builder.push_record(["g0-2 is a much longer cell", "x"]);
        builder.push_record(["g1-1", "x"]);
        builder.push_record(["g2-1", "x"]);
        builder.push_record(["", ""]);
        let output = build_grouped_table(builder, &[], &nms_theme(), "Rows", &[0, 0, 1, 2]);
        let (r, g, b) = GROUP_SHADE_BG;
        let shade = format!("48;2;{r};{g};{b}m");
        let shaded_lines: Vec<&str> = output.lines().filter(|l| l.contains(&shade)).collect();
        assert_eq!(shaded_lines.len(), 1, "{output}");
        assert!(shaded_lines[0].contains("g1-1"));
        // The shade must cover the fill that pads the short cell out to the column
        // width, not just the text, so it appears more than once on the line.
        assert!(shaded_lines[0].matches(&shade).count() >= 2, "{output}");
        assert!(output.contains("Total:"));
        assert!(output.contains("4 Rows"));
    }

    #[test]
    fn test_build_grouped_table_no_colour_theme_has_no_shading() {
        let mut builder = Builder::default();
        builder.push_record(["A"]);
        builder.push_record(["one"]);
        builder.push_record(["two"]);
        builder.push_record([""]);
        let output = build_grouped_table(builder, &[], &nms_theme_no_color(), "", &[0, 1]);
        assert!(output.contains("one") && output.contains("two"));
        assert!(!output.contains("48;2;"), "{output}");
    }
}
