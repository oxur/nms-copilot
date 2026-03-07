//! Configuration for the NMS Copilot MCP server.
//!
//! Reads from `~/.nms-copilot/config.toml`, sharing the same config
//! file as the REPL. Only the `[logging]` section is used by the
//! MCP server; other sections are ignored.

use std::path::Path;

use serde::Deserialize;

/// MCP server configuration.
///
/// Extracts only the sections relevant to the MCP server from the
/// shared `~/.nms-copilot/config.toml`.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct McpConfig {
    /// Logging configuration (twyg options).
    pub logging: LoggingConfig,
}

/// Logging configuration wrapping twyg options.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level: "trace", "debug", "info", "warn", "error".
    pub level: String,
    /// Enable colored output.
    pub coloured: bool,
    /// Output destination: "stdout" or "stderr".
    pub output: String,
    /// Include caller info in log messages.
    pub report_caller: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            coloured: true,
            output: "stderr".into(),
            report_caller: false,
        }
    }
}

impl LoggingConfig {
    /// Convert to twyg::Opts for logger initialization.
    pub fn to_twyg_opts(&self) -> twyg::Opts {
        let output = match self.output.as_str() {
            "stdout" => twyg::Output::Stdout,
            _ => twyg::Output::Stderr,
        };
        let level = match self.level.as_str() {
            "trace" => twyg::LogLevel::Trace,
            "debug" => twyg::LogLevel::Debug,
            "warn" => twyg::LogLevel::Warn,
            "error" => twyg::LogLevel::Error,
            _ => twyg::LogLevel::Info,
        };
        let colors = twyg::Colors {
            timestamp: Some(twyg::Color::hi_black()),
            ..Default::default()
        };
        twyg::OptsBuilder::new()
            .coloured(self.coloured)
            .output(output)
            .level(level)
            .report_caller(self.report_caller)
            .timestamp_format(twyg::TSFormat::Simple)
            .colors(colors)
            .build()
            .unwrap_or_default()
    }
}

impl McpConfig {
    /// Load config from `~/.nms-copilot/config.toml`.
    ///
    /// Returns defaults if the file doesn't exist.
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .map(|h| h.join(".nms-copilot/config.toml"))
            .unwrap_or_default();
        Self::load_from(&path)
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert_eq!(config.logging.level, "info");
        assert!(config.logging.coloured);
        assert_eq!(config.logging.output, "stderr");
        assert!(!config.logging.report_caller);
    }

    #[test]
    fn test_parse_logging_config() {
        let toml = r#"
            [logging]
            level = "debug"
            coloured = false
            output = "stdout"
            report_caller = true
        "#;
        let config: McpConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.logging.level, "debug");
        assert!(!config.logging.coloured);
        assert_eq!(config.logging.output, "stdout");
        assert!(config.logging.report_caller);
    }

    #[test]
    fn test_parse_empty_config() {
        let config: McpConfig = toml::from_str("").unwrap();
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_to_twyg_opts() {
        let config = LoggingConfig::default();
        let opts = config.to_twyg_opts();
        // Just verify it doesn't panic and produces valid opts
        assert!(!format!("{:?}", opts).is_empty());
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let config = McpConfig::load_from(std::path::Path::new("/nonexistent/config.toml"));
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_unknown_sections_ignored() {
        let toml = r#"
            [save]
            path = "/tmp/save"

            [display]
            emoji_glyphs = false

            [logging]
            level = "warn"
        "#;
        let config: McpConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.logging.level, "warn");
    }
}
