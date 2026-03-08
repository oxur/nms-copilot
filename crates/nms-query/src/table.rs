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

/// Dummy type to satisfy the `Tabled` trait bound on `apply_to_table`.
#[derive(tabled::Tabled)]
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
padding_left = 1
padding_right = 1
padding_top = 0
padding_bottom = 0

[title]
enabled = true
bg_color = "#1E3A5F"
fg_color = "#1E3A5F"
justification_char = " "
vertical_fg_color = "#1E3A5F"
vertical_bg_color = "#1E3A5F"

[header]
bg_color = "#2C5F8A"
fg_color = "#E0F0FF"
justification_char = " "
vertical_char = "|"
vertical_bg_color = "#2C5F8A"
vertical_fg_color = "#2C5F8A"

[rows]
colors = [
    { bg = "#0A1929", fg = "#B0D0E8" },
    { bg = "#0F2236", fg = "#8BBBD0" },
]
justification_char = " "

[style]
vertical_bg_color = "#0A1929"
vertical_fg_color = "#1E3A5F"

[footer]
enabled = true
bg_color = "#1E3A5F"
fg_color = "#4A9BC7"
vertical_bg_color = "#1E3A5F"
vertical_fg_color = "#1E3A5F"
"##;

/// No-color theme for piped output and MCP.
///
/// Title is enabled but invisible (black on black) so the row layout matches.
const NMS_THEME_NO_COLOR: &str = r##"
[table]
padding_left = 1
padding_right = 1
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
/// The caller pushes records in the order: `[header, data..., footer]`.
/// This function automatically inserts an invisible title row at position 0
/// so the `oxur-cli` theme styling aligns correctly (title at row 0, header
/// at row 1, data at rows 2+, footer at last row).
pub fn build_table(mut builder: Builder, theme: &TableStyleConfig) -> String {
    let col_count = builder.count_columns();
    let title_row: Vec<String> = std::iter::repeat_n(String::new(), col_count).collect();
    builder.insert_record(0, title_row);

    let mut table = builder.build();
    theme.apply_to_table::<Dummy>(&mut table);
    table.to_string()
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
        let output = build_table(builder, &nms_theme());
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
        let output = build_table(builder, &nms_theme_no_color());
        assert!(output.contains("Col"));
        assert!(output.contains("data"));
    }

    #[test]
    fn test_build_table_contains_ansi_with_nms_theme() {
        let mut builder = Builder::default();
        builder.push_record(["Header"]);
        builder.push_record(["value"]);
        builder.push_record([""]);
        let output = build_table(builder, &nms_theme());
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
        let output = build_table(builder, &nms_theme());
        for val in ["A", "B", "C", "1", "2", "3", "x", "y", "z"] {
            assert!(output.contains(val), "Missing '{val}' in output");
        }
    }
}
