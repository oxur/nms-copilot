//! Color themes for terminal output.
//!
//! Provides semantic styling for NMS Copilot output using raw ANSI escape codes.
//! No external color crates are used. The [`Theme`] struct maps semantic elements
//! (headers, system names, biome types) to [`Style`] values. Two presets are
//! provided: [`Theme::default_dark`] for interactive terminal use and
//! [`Theme::none`] for piped or machine-consumed output.

use nms_core::biome::Biome;

// ANSI escape constants.
const ESC: &str = "\x1b[";
const RESET: &str = "\x1b[0m";

/// Terminal color identifiers mapped to ANSI SGR codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Gray,
}

impl Color {
    /// Return the ANSI SGR foreground code number as a string slice.
    pub fn ansi_code(&self) -> &str {
        match self {
            Self::Red => "31",
            Self::Green => "32",
            Self::Yellow => "33",
            Self::Blue => "34",
            Self::Magenta => "35",
            Self::Cyan => "36",
            Self::White => "37",
            Self::BrightRed => "91",
            Self::BrightGreen => "92",
            Self::BrightYellow => "93",
            Self::BrightBlue => "94",
            Self::BrightMagenta => "95",
            Self::BrightCyan => "96",
            Self::BrightWhite => "97",
            Self::Gray => "90",
        }
    }
}

/// A text style combining an optional foreground color and bold flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
}

impl Style {
    /// A no-op style that passes text through unchanged.
    pub const fn plain() -> Self {
        Self {
            fg: None,
            bold: false,
        }
    }

    /// Wrap `s` in ANSI escape sequences according to this style.
    ///
    /// Returns the input unchanged when no foreground color or bold is set.
    pub fn paint(&self, s: &str) -> String {
        if self.fg.is_none() && !self.bold {
            return s.to_string();
        }

        let mut codes = String::new();
        if self.bold {
            codes.push('1');
        }
        if let Some(ref color) = self.fg {
            if !codes.is_empty() {
                codes.push(';');
            }
            codes.push_str(color.ansi_code());
        }

        format!("{ESC}{codes}m{s}{RESET}")
    }
}

/// Semantic color theme for NMS Copilot terminal output.
///
/// Each field corresponds to a semantic element in the output (headers, system
/// names, biome categories, etc.). Use [`Theme::default_dark`] for dark
/// terminal backgrounds or [`Theme::none`] for plain text.
#[derive(Debug, Clone)]
pub struct Theme {
    pub header: Style,
    pub system_name: Style,
    pub planet_name: Style,
    pub distance: Style,
    pub glyphs: Style,
    pub muted: Style,

    // Biome-specific styles.
    pub biome_lush: Style,
    pub biome_toxic: Style,
    pub biome_scorched: Style,
    pub biome_frozen: Style,
    pub biome_barren: Style,
    pub biome_dead: Style,
    pub biome_exotic: Style,
    pub biome_other: Style,
}

impl Theme {
    /// A vibrant theme suited for dark terminal backgrounds.
    pub fn default_dark() -> Self {
        Self {
            header: Style {
                fg: Some(Color::BrightWhite),
                bold: true,
            },
            system_name: Style {
                fg: Some(Color::BrightCyan),
                bold: false,
            },
            planet_name: Style {
                fg: Some(Color::Cyan),
                bold: false,
            },
            distance: Style {
                fg: Some(Color::Yellow),
                bold: false,
            },
            glyphs: Style {
                fg: None,
                bold: false,
            },
            muted: Style {
                fg: Some(Color::Gray),
                bold: false,
            },

            biome_lush: Style {
                fg: Some(Color::BrightGreen),
                bold: false,
            },
            biome_toxic: Style {
                fg: Some(Color::Magenta),
                bold: false,
            },
            biome_scorched: Style {
                fg: Some(Color::BrightRed),
                bold: false,
            },
            biome_frozen: Style {
                fg: Some(Color::BrightBlue),
                bold: false,
            },
            biome_barren: Style {
                fg: Some(Color::Yellow),
                bold: false,
            },
            biome_dead: Style {
                fg: Some(Color::Gray),
                bold: false,
            },
            biome_exotic: Style {
                fg: Some(Color::BrightMagenta),
                bold: false,
            },
            biome_other: Style {
                fg: Some(Color::White),
                bold: false,
            },
        }
    }

    /// A no-op theme that produces plain, uncolored text.
    ///
    /// Suitable for MCP output, piped commands, or contexts where ANSI escapes
    /// are unwanted.
    pub fn none() -> Self {
        Self {
            header: Style::plain(),
            system_name: Style::plain(),
            planet_name: Style::plain(),
            distance: Style::plain(),
            glyphs: Style::plain(),
            muted: Style::plain(),
            biome_lush: Style::plain(),
            biome_toxic: Style::plain(),
            biome_scorched: Style::plain(),
            biome_frozen: Style::plain(),
            biome_barren: Style::plain(),
            biome_dead: Style::plain(),
            biome_exotic: Style::plain(),
            biome_other: Style::plain(),
        }
    }

    /// Look up the style for a given biome.
    pub fn biome_style(&self, biome: &Biome) -> &Style {
        match biome {
            Biome::Lush | Biome::Swamp => &self.biome_lush,
            Biome::Toxic => &self.biome_toxic,
            Biome::Scorched | Biome::Lava => &self.biome_scorched,
            Biome::Frozen => &self.biome_frozen,
            Biome::Barren => &self.biome_barren,
            Biome::Dead => &self.biome_dead,
            Biome::Weird | Biome::Red | Biome::Green | Biome::Blue => &self.biome_exotic,
            Biome::Radioactive | Biome::Waterworld | Biome::GasGiant => &self.biome_other,
            _ => &self.biome_other,
        }
    }
}

/// Decide whether to emit ANSI colors based on config preference and terminal
/// detection.
///
/// Returns `true` only when `config_colors` is `true` **and** stdout is a
/// terminal (as determined by [`std::io::IsTerminal`]).
pub fn should_use_colors(config_colors: bool) -> bool {
    use std::io::IsTerminal;
    config_colors && std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_paint_with_color_returns_ansi_wrapped() {
        let style = Style {
            fg: Some(Color::Red),
            bold: false,
        };
        let painted = style.paint("hello");
        assert_eq!(painted, "\x1b[31mhello\x1b[0m");
    }

    #[test]
    fn test_style_paint_no_color_returns_plain() {
        let style = Style::plain();
        let painted = style.paint("hello");
        assert_eq!(painted, "hello");
    }

    #[test]
    fn test_style_paint_bold_returns_bold_escape() {
        let style = Style {
            fg: None,
            bold: true,
        };
        let painted = style.paint("hello");
        assert_eq!(painted, "\x1b[1mhello\x1b[0m");
    }

    #[test]
    fn test_style_paint_bold_and_color_combines_codes() {
        let style = Style {
            fg: Some(Color::Green),
            bold: true,
        };
        let painted = style.paint("hello");
        assert_eq!(painted, "\x1b[1;32mhello\x1b[0m");
    }

    #[test]
    fn test_theme_biome_style_lush_returns_lush_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Lush), &theme.biome_lush);
    }

    #[test]
    fn test_theme_biome_style_toxic_returns_toxic_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Toxic), &theme.biome_toxic);
    }

    #[test]
    fn test_theme_biome_style_scorched_returns_scorched_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Scorched), &theme.biome_scorched);
    }

    #[test]
    fn test_theme_biome_style_frozen_returns_frozen_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Frozen), &theme.biome_frozen);
    }

    #[test]
    fn test_theme_biome_style_barren_returns_barren_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Barren), &theme.biome_barren);
    }

    #[test]
    fn test_theme_biome_style_dead_returns_dead_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Dead), &theme.biome_dead);
    }

    #[test]
    fn test_theme_biome_style_weird_returns_exotic_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Weird), &theme.biome_exotic);
    }

    #[test]
    fn test_theme_biome_style_swamp_returns_lush_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Swamp), &theme.biome_lush);
    }

    #[test]
    fn test_theme_biome_style_lava_returns_scorched_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Lava), &theme.biome_scorched);
    }

    #[test]
    fn test_theme_biome_style_red_returns_exotic_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Red), &theme.biome_exotic);
    }

    #[test]
    fn test_theme_biome_style_radioactive_returns_other_style() {
        let theme = Theme::default_dark();
        assert_eq!(theme.biome_style(&Biome::Radioactive), &theme.biome_other);
    }

    #[test]
    fn test_theme_none_no_escapes_in_painted_output() {
        let theme = Theme::none();
        let painted = theme.header.paint("Title");
        assert_eq!(painted, "Title");
        assert!(!painted.contains('\x1b'));

        let painted = theme.system_name.paint("Sol");
        assert_eq!(painted, "Sol");
        assert!(!painted.contains('\x1b'));

        let painted = theme.biome_lush.paint("Lush");
        assert_eq!(painted, "Lush");
        assert!(!painted.contains('\x1b'));
    }

    #[test]
    fn test_should_use_colors_false_when_config_disabled() {
        // When config says no colors, result is always false regardless of terminal.
        assert!(!should_use_colors(false));
    }

    #[test]
    fn test_color_ansi_code_completeness() {
        // Verify every variant returns a non-empty code.
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightYellow,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
            Color::BrightWhite,
            Color::Gray,
        ];
        for c in colors {
            assert!(!c.ansi_code().is_empty());
        }
    }

    #[test]
    fn test_style_plain_is_noop() {
        let s = Style::plain();
        assert!(s.fg.is_none());
        assert!(!s.bold);
    }

    #[test]
    fn test_theme_default_dark_header_is_bold() {
        let theme = Theme::default_dark();
        assert!(theme.header.bold);
    }
}
