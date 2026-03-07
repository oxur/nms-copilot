//! Configuration file support for NMS Copilot.
//!
//! Config file location: `~/.nms-copilot/config.toml`
//!
//! All fields are optional -- sensible defaults are used when not specified.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::paths;

/// Top-level configuration.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Save file configuration.
    pub save: SaveConfig,

    /// Display preferences.
    pub display: DisplayConfig,

    /// Default values for commands.
    pub defaults: DefaultsConfig,

    /// Cache settings.
    pub cache: CacheConfig,

    /// File watcher settings.
    pub watch: WatchConfig,
}

/// Save file location and format.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SaveConfig {
    /// Path to the NMS save directory or specific save file.
    /// If omitted, auto-detected from platform defaults.
    pub path: Option<PathBuf>,

    /// Save format: "auto", "raw", "goatfungus".
    pub format: String,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            path: None,
            format: "auto".into(),
        }
    }
}

/// Display preferences.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Use emoji for portal glyphs (true) or hex digits (false).
    pub emoji_glyphs: bool,

    /// Enable ANSI color output.
    pub color: bool,

    /// Table border style.
    pub table_style: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            emoji_glyphs: true,
            color: true,
            table_style: "rounded".into(),
        }
    }
}

/// Default values for commands.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    /// Default galaxy index (0 = Euclid).
    pub galaxy: u8,

    /// Default warp range in light-years for routing.
    pub warp_range: Option<f64>,

    /// Default TSP algorithm: "nearest-neighbor" or "2opt".
    pub tsp_algorithm: String,

    /// Default number of results for find.
    pub find_limit: Option<usize>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            galaxy: 0,
            warp_range: None,
            tsp_algorithm: "2opt".into(),
            find_limit: None,
        }
    }
}

/// Cache settings.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Enable caching (default: true).
    pub enabled: bool,

    /// Cache file path (default: ~/.nms-copilot/galaxy.rkyv).
    pub path: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
        }
    }
}

/// File watcher settings.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    /// Enable file watching (default: true).
    pub enabled: bool,
    /// Debounce duration in milliseconds (default: 500).
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 500,
        }
    }
}

impl Config {
    /// Load config from the default path (`~/.nms-copilot/config.toml`).
    ///
    /// Returns the default config if the file doesn't exist.
    /// Returns an error if the file exists but can't be parsed.
    pub fn load() -> Result<Self, ConfigError> {
        let path = paths::config_path();
        Self::load_from(&path)
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
        toml::from_str(&content).map_err(ConfigError::Parse)
    }

    /// Resolve the effective cache path.
    pub fn cache_path(&self) -> PathBuf {
        self.cache.path.clone().unwrap_or_else(paths::cache_path)
    }

    /// Resolve the effective save path (if configured).
    pub fn save_path(&self) -> Option<&Path> {
        self.save.path.as_deref()
    }

    /// Whether caching is enabled.
    pub fn cache_enabled(&self) -> bool {
        self.cache.enabled
    }

    /// Whether file watching is enabled.
    pub fn watch_enabled(&self) -> bool {
        self.watch.enabled
    }

    /// The configured debounce duration for file watching.
    pub fn watch_debounce(&self) -> Duration {
        Duration::from_millis(self.watch.debounce_ms)
    }
}

/// Config loading errors.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "config I/O error: {e}"),
            Self::Parse(e) => write!(f, "config parse error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.display.emoji_glyphs);
        assert!(config.display.color);
        assert_eq!(config.defaults.galaxy, 0);
        assert!(config.cache.enabled);
        assert!(config.save.path.is_none());
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.display.emoji_glyphs);
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
            [save]
            path = "/Users/test/NMS"
            format = "raw"

            [display]
            emoji_glyphs = false
            color = false
            table_style = "ascii"

            [defaults]
            galaxy = 1
            warp_range = 2500.0
            tsp_algorithm = "nearest-neighbor"
            find_limit = 10

            [cache]
            enabled = false
            path = "/tmp/nms-cache.rkyv"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.save.path.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS"
        );
        assert_eq!(config.save.format, "raw");
        assert!(!config.display.emoji_glyphs);
        assert!(!config.display.color);
        assert_eq!(config.defaults.galaxy, 1);
        assert_eq!(config.defaults.warp_range, Some(2500.0));
        assert_eq!(config.defaults.find_limit, Some(10));
        assert!(!config.cache.enabled);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
            [defaults]
            warp_range = 1500.0
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.display.emoji_glyphs);
        assert!(config.cache.enabled);
        assert_eq!(config.defaults.warp_range, Some(1500.0));
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let config = Config::load_from(Path::new("/nonexistent/config.toml")).unwrap();
        assert!(config.display.emoji_glyphs);
    }

    #[test]
    fn test_load_invalid_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        fs::write(&path, "not valid toml [[[").unwrap();
        assert!(Config::load_from(&path).is_err());
    }

    #[test]
    fn test_cache_path_default() {
        let config = Config::default();
        let path = config.cache_path();
        assert!(path.ends_with("galaxy.rkyv"));
    }

    #[test]
    fn test_cache_path_override() {
        let toml = r#"
            [cache]
            path = "/tmp/custom-cache.rkyv"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.cache_path(), PathBuf::from("/tmp/custom-cache.rkyv"));
    }

    #[test]
    fn test_save_path_none_when_unset() {
        let config = Config::default();
        assert!(config.save_path().is_none());
    }

    #[test]
    fn test_unknown_fields_are_ignored() {
        let toml = r#"
            [save]
            path = "/tmp"
            unknown_field = "ignored"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.save.path.is_some());
    }

    #[test]
    fn test_watch_config_defaults() {
        let config = Config::default();
        assert!(config.watch_enabled());
        assert_eq!(config.watch_debounce(), Duration::from_millis(500));
    }

    #[test]
    fn test_watch_config_from_toml() {
        let toml = r#"
            [watch]
            enabled = false
            debounce_ms = 1000
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.watch_enabled());
        assert_eq!(config.watch_debounce(), Duration::from_millis(1000));
    }

    #[test]
    fn test_watch_config_partial_toml() {
        let toml = r#"
            [watch]
            debounce_ms = 250
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.watch_enabled());
        assert_eq!(config.watch_debounce(), Duration::from_millis(250));
    }
}
