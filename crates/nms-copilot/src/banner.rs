//! Startup banner display for the NMS Copilot REPL.
//!
//! Provides an ASCII art banner with ANSI color support and a system info
//! banner showing loaded model statistics. Both are independently configurable.

/// Default ASCII art banner, embedded at compile time.
const DEFAULT_BANNER: &str = include_str!("../assets/banners/banner.txt");

/// Count visible characters in a string, ignoring ANSI escape sequences.
///
/// Walks through the characters; when hitting `\x1b`, skips the entire
/// escape sequence (up to and including `m`).
pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for esc_ch in chars.by_ref() {
                    if esc_ch == 'm' {
                        break;
                    }
                }
            }
        } else {
            width += 1;
        }
    }

    width
}

/// Remove all ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for esc_ch in chars.by_ref() {
                    if esc_ch == 'm' {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Replace a placeholder in a line while preserving visual column width.
///
/// After replacement, calculates the width difference and adds or removes
/// trailing spaces before the last border character (`║`, `│`, `|`, `]`)
/// to maintain alignment.
pub fn substitute_placeholder_in_line(line: &str, placeholder: &str, value: &str) -> String {
    if !line.contains(placeholder) {
        return line.to_string();
    }

    let original_visible_width = visible_width(line);
    let result = line.replace(placeholder, value);
    let new_visible_width = visible_width(&result);

    if new_visible_width == original_visible_width {
        return result;
    }

    // Find the position where we'll adjust spacing.
    // Look for common border characters or last ANSI escape sequence.
    let border_pos = result
        .rfind('\x1b')
        .or_else(|| result.rfind('║'))
        .or_else(|| result.rfind('│'))
        .or_else(|| result.rfind('|'))
        .or_else(|| result.rfind(']'));

    let (before_border, border_and_after) = if let Some(pos) = border_pos {
        (&result[..pos], &result[pos..])
    } else {
        (result.as_str(), "")
    };

    if new_visible_width < original_visible_width {
        // Need to add spaces
        let spaces_needed = original_visible_width - new_visible_width;
        format!(
            "{}{}{}",
            before_border,
            " ".repeat(spaces_needed),
            border_and_after
        )
    } else {
        // Need to remove spaces (new_visible_width > original_visible_width)
        let spaces_to_remove = new_visible_width - original_visible_width;
        let trimmed = before_border.trim_end();
        let trailing_space_count = before_border.len() - trimmed.len();

        if trailing_space_count >= spaces_to_remove {
            let keep_len = trimmed.len() + (trailing_space_count - spaces_to_remove);
            format!("{}{}", &before_border[..keep_len], border_and_after)
        } else {
            // Not enough trailing spaces -- remove what we have
            format!("{trimmed}{border_and_after}")
        }
    }
}

/// Apply `{version}` substitution across all lines of a banner.
pub fn substitute_placeholders(banner: &str, version: &str) -> String {
    banner
        .lines()
        .map(|line| substitute_placeholder_in_line(line, "{version}", version))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Resolve the banner text to display, or `None` if disabled.
///
/// Returns `None` when:
/// - `show_banner` is `false`
/// - `custom_banner` is `Some("")` (empty string disables the banner)
///
/// When `color_enabled` is `false`, ANSI escape sequences are stripped.
pub fn resolve_banner(
    custom_banner: Option<&str>,
    show_banner: bool,
    color_enabled: bool,
) -> Option<String> {
    if !show_banner {
        return None;
    }

    let raw = match custom_banner {
        Some("") => return None,
        Some(custom) => custom.to_string(),
        None => DEFAULT_BANNER.to_string(),
    };

    let version = env!("CARGO_PKG_VERSION");
    let substituted = substitute_placeholders(&raw, version);

    if color_enabled {
        Some(substituted)
    } else {
        Some(strip_ansi(&substituted))
    }
}

/// Resolve the banner and print it to stdout.
///
/// Prints nothing if the banner is disabled or resolves to `None`.
pub fn print_banner(custom_banner: Option<&str>, show_banner: bool, color_enabled: bool) {
    if let Some(text) = resolve_banner(custom_banner, show_banner, color_enabled) {
        println!("{text}");
    }
}

/// Print the system info banner (model stats and help hint).
///
/// When `show` is `true`, prints the loaded model summary and a help hint.
/// When `show` is `false`, prints nothing.
pub fn print_system_banner(show: bool, systems: usize, planets: usize, bases: usize, source: &str) {
    if show {
        println!(
            "Loaded {systems} systems, {planets} planets, {bases} bases ({source})\n\
             Type 'help' for commands, 'exit' to quit."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_width_plain_text_returns_char_count() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width("hello world"), 11);
    }

    #[test]
    fn test_visible_width_with_ansi_codes_ignores_escapes() {
        // Simple color code
        assert_eq!(visible_width("\x1b[31mhello\x1b[0m"), 5);
        // True color (24-bit)
        assert_eq!(visible_width("\x1b[38;2;100;200;50mtest\x1b[0m"), 4);
    }

    #[test]
    fn test_visible_width_empty_string_returns_zero() {
        assert_eq!(visible_width(""), 0);
    }

    #[test]
    fn test_strip_ansi_removes_escape_sequences() {
        assert_eq!(strip_ansi("\x1b[31mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("\x1b[38;2;100;200;50mtest\x1b[0m"), "test");
    }

    #[test]
    fn test_strip_ansi_preserves_plain_text() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_empty_string() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn test_substitute_placeholder_in_line_no_placeholder_unchanged() {
        let line = "no placeholder here";
        assert_eq!(
            substitute_placeholder_in_line(line, "{version}", "1.0.0"),
            "no placeholder here"
        );
    }

    #[test]
    fn test_substitute_placeholder_in_line_same_length_no_padding_change() {
        // {version} is 9 chars, "123456789" is 9 chars -- same length
        let line = "v{version} |";
        let result = substitute_placeholder_in_line(line, "{version}", "123456789");
        assert_eq!(visible_width(&result), visible_width(line));
    }

    #[test]
    fn test_substitute_placeholder_in_line_shorter_value_adds_spaces() {
        // {version} is 9 chars, "1.0" is 3 chars -- should add 6 spaces
        let line = "v{version}    |";
        let result = substitute_placeholder_in_line(line, "{version}", "1.0");
        assert_eq!(visible_width(&result), visible_width(line));
        assert!(result.contains("1.0"));
    }

    #[test]
    fn test_substitute_placeholder_in_line_longer_value_removes_spaces() {
        // {version} is 9 chars, "12345678901234" is 14 chars -- should remove 5 spaces
        let line = "v{version}          |";
        let result = substitute_placeholder_in_line(line, "{version}", "12345678901234");
        assert_eq!(visible_width(&result), visible_width(line));
        assert!(result.contains("12345678901234"));
    }

    #[test]
    fn test_substitute_placeholders_replaces_version() {
        let banner = "line1 {version} end\nline2 no change";
        let result = substitute_placeholders(banner, "1.2.3");
        assert!(result.contains("1.2.3"));
        assert!(!result.contains("{version}"));
        assert!(result.contains("line2 no change"));
    }

    #[test]
    fn test_resolve_banner_default_returns_embedded_content() {
        let result = resolve_banner(None, true, true);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(!text.is_empty());
        // Should have version substituted (no raw placeholder)
        assert!(!text.contains("{version}"));
    }

    #[test]
    fn test_resolve_banner_custom_returns_custom_text() {
        let result = resolve_banner(Some("Custom Banner {version}"), true, true);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("Custom Banner"));
        assert!(!text.contains("{version}"));
    }

    #[test]
    fn test_resolve_banner_empty_string_returns_none() {
        let result = resolve_banner(Some(""), true, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_banner_show_false_returns_none() {
        let result = resolve_banner(None, false, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_banner_no_color_strips_ansi() {
        let result = resolve_banner(Some("\x1b[31mRed Banner\x1b[0m"), true, false);
        assert!(result.is_some());
        let text = result.unwrap();
        assert_eq!(text, "Red Banner");
        assert!(!text.contains("\x1b"));
    }

    #[test]
    fn test_default_banner_contains_version_placeholder() {
        assert!(DEFAULT_BANNER.contains("{version}"));
    }

    #[test]
    fn test_default_banner_is_not_empty() {
        assert!(!DEFAULT_BANNER.is_empty());
        assert!(DEFAULT_BANNER.len() > 10);
    }
}
