//! Logging configuration used by the NMS Copilot MCP server.
//!
//! [`McpConfig`] reads only `[logging]` from the shared
//! `~/.nms-copilot/config.toml`; unrelated sections are ignored. It does not
//! configure save paths, watchers, or the HTTP bind address, and it does not
//! install a logger or mutate environment variables. See the
//! [configuration example](crate::mcp::config) for defaults and option mapping.
//!
//! # Examples
//!
//! ```
//! use nms_copilot::mcp::config::LoggingConfig;
//! use twyg::{LogLevel, Output, TSFormat};
//!
//! let defaults = LoggingConfig::default();
//! assert_eq!(defaults.level, "info");
//! assert!(defaults.coloured);
//! assert_eq!(defaults.output, "stderr");
//! assert!(!defaults.report_caller);
//! let opts = defaults.to_twyg_opts();
//! assert!(opts.coloured());
//! assert!(!opts.report_caller());
//! assert_eq!(opts.level(), LogLevel::Info);
//! assert_eq!(opts.output(), &Output::Stderr);
//! assert_eq!(opts.timestamp_format(), &TSFormat::Simple);
//!
//! let mut debug = LoggingConfig { level: "debug".into(), coloured: false,
//!     output: "stdout".into(), report_caller: true };
//! let opts = debug.to_twyg_opts();
//! assert_eq!(opts.level(), LogLevel::Debug);
//! assert_eq!(opts.output(), &Output::Stdout);
//! assert!(!opts.coloured());
//! assert!(opts.report_caller());
//! assert_eq!(opts.timestamp_format(), &TSFormat::Simple);
//!
//! debug.level = "DEBUG".into();
//! debug.output = "STDOUT".into();
//! let opts = debug.to_twyg_opts();
//! assert_eq!(opts.level(), LogLevel::Info);
//! assert_eq!(opts.output(), &Output::Stderr);
//! assert!(!opts.coloured());
//! assert!(opts.report_caller());
//! assert_eq!(opts.timestamp_format(), &TSFormat::Simple);
//! ```
//!

use std::path::Path;

use serde::Deserialize;

/// MCP logging configuration read from the shared application config file.
///
/// Only `[logging]` is used. Missing sections and fields use
/// [`LoggingConfig::default`]; unrelated sections are ignored. Missing files,
/// read/UTF-8 errors, TOML errors, and typed-deserialization errors silently
/// return the entire default config; no partial parse is retained. If a home
/// directory cannot be determined, [`Self::load`] uses an empty path and
/// follows the same default-return route. This is not a general MCP server
/// configuration.
///
/// # Examples
///
/// File loading is I/O-bearing, so this is compile-only:
///
/// ```no_run
/// let config = nms_copilot::mcp::config::McpConfig::load();
/// let _options = config.logging.to_twyg_opts();
/// ```
///
/// See the [module examples](crate::mcp::config) for logging options.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct McpConfig {
    /// Logging configuration (twyg options).
    pub logging: LoggingConfig,
}

/// Logging settings translated into twyg options.
///
/// Defaults are `level = "info"`, `coloured = true`, `output = "stderr"`, and
/// `report_caller = false`.
/// Exact lowercase `trace`, `debug`, `warn`, and `error` select those levels;
/// every other value selects Info. Only exact `stdout` selects stdout; other
/// destinations select stderr. [`Self::to_twyg_opts`] returns options without
/// initializing a logger and falls back to [`twyg::Opts::default`] if its
/// builder fails. See the [module examples](crate::mcp::config).
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
    /// Convert values into twyg logging options without initializing a logger.
    ///
    /// Exact lowercase levels `trace`, `debug`, `warn`, and `error` map to
    /// those levels; all other strings map to Info. Only exact `stdout` selects
    /// stdout. Timestamp format is Simple with a `hi_black` timestamp color;
    /// other colors use twyg defaults. Builder failure falls back to default
    /// options. See the [module example](crate::mcp::config).
    ///
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
    /// Load logging settings from `~/.nms-copilot/config.toml`.
    ///
    /// Only `[logging]` is deserialized; other sections are ignored. Missing
    /// files and read, UTF-8, TOML, or typed-deserialization errors silently
    /// return the entire default config. There is no partial merge or write. If the
    /// home directory cannot be determined, this uses an empty path and follows
    /// the same default-return route. See the
    /// [loading example](crate::mcp::config::McpConfig).
    ///
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .map(|h| h.join(".nms-copilot/config.toml"))
            .unwrap_or_default();
        Self::load_from(&path)
    }

    /// Load logging settings from a specific path.
    ///
    /// Missing paths, read failures, invalid UTF-8, TOML errors, and typed
    /// deserialization errors silently return the entire default config.
    /// Unknown sections are ignored; no partial merge or write occurs. See the
    /// [loading example](crate::mcp::config::McpConfig).
    ///
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
