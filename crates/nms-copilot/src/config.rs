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
    /// DEPRECATED: Old combined path. Treated as `file` for backward compat.
    pub path: Option<PathBuf>,

    /// Path to the NMS save directory (account dir containing `save*.hg`).
    pub dir: Option<PathBuf>,

    /// Path to a specific NMS save file.
    pub file: Option<PathBuf>,

    /// Save format: "auto", "raw", "goatfungus".
    pub format: String,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            path: None,
            dir: None,
            file: None,
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

    /// Custom banner text. `None` = use default embedded banner.
    /// Empty string = disable banner.
    pub banner: Option<String>,

    /// Whether to show the art banner at startup (default: true).
    pub show_banner: bool,

    /// Whether to show the system info line after the banner (default: true).
    pub show_system_banner: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            emoji_glyphs: true,
            color: true,
            table_style: "rounded".into(),
            banner: None,
            show_banner: true,
            show_system_banner: true,
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
        let mut config = if !path.exists() {
            Self::default()
        } else {
            let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
            toml::from_str(&content).map_err(ConfigError::Parse)?
        };
        config.apply_env_overrides();
        Ok(config)
    }

    /// Apply environment variable overrides to the config.
    ///
    /// Reads:
    /// - `NMS_SAVE_DIR` -> `self.save.dir`
    /// - `NMS_SAVE_FILE` -> `self.save.file`
    /// - `NMS_SAVE_FORMAT` -> `self.save.format`
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("NMS_SAVE_DIR") {
            self.save.dir = Some(PathBuf::from(val));
        }
        if let Ok(val) = std::env::var("NMS_SAVE_FILE") {
            self.save.file = Some(PathBuf::from(val));
        }
        if let Ok(val) = std::env::var("NMS_SAVE_FORMAT") {
            self.save.format = val;
        }
    }

    /// Resolve the effective save file path from all configured sources.
    ///
    /// Priority:
    /// 1. `save.file` (explicit file path)
    /// 2. `save.path` if it points to a file (backward compat)
    /// 3. `save.dir` — find most recent save in that directory
    /// 4. `save.path` if it points to a directory (backward compat)
    /// 5. `None` if nothing is configured or resolvable
    pub fn effective_save_file(&self) -> Option<PathBuf> {
        // 1. Explicit file
        if let Some(ref file) = self.save.file {
            return Some(file.clone());
        }

        // 2. Legacy path as file
        if let Some(ref path) = self.save.path {
            if path.is_file() {
                return Some(path.clone());
            }
        }

        // 3. Explicit dir — find most recent save in it
        if let Some(ref dir) = self.save.dir {
            if let Ok(save) = nms_save::locate::find_most_recent_save_in(dir) {
                return Some(save.path().to_path_buf());
            }
        }

        // 4. Legacy path as directory
        if let Some(ref path) = self.save.path {
            if path.is_dir() {
                if let Ok(save) = nms_save::locate::find_most_recent_save_in(path) {
                    return Some(save.path().to_path_buf());
                }
            }
        }

        None
    }

    /// Resolve the effective cache path.
    pub fn cache_path(&self) -> PathBuf {
        self.cache.path.clone().unwrap_or_else(paths::cache_path)
    }

    /// Resolve the effective save path (if configured).
    ///
    /// Delegates to [`effective_save_file`]. For backward compatibility,
    /// also checks the legacy `save.path` field.
    pub fn save_path(&self) -> Option<PathBuf> {
        self.effective_save_file()
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
        assert!(config.save.dir.is_none());
        assert!(config.save.file.is_none());
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
            banner = "My Custom Banner"
            show_banner = false
            show_system_banner = false

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
        assert_eq!(config.display.banner.as_deref(), Some("My Custom Banner"));
        assert!(!config.display.show_banner);
        assert!(!config.display.show_system_banner);
        assert_eq!(config.defaults.galaxy, 1);
        assert_eq!(config.defaults.warp_range, Some(2500.0));
        assert_eq!(config.defaults.find_limit, Some(10));
        assert!(!config.cache.enabled);
    }

    #[test]
    fn test_parse_config_with_new_save_fields() {
        let toml = r#"
            [save]
            dir = "/Users/test/NMS/st_123"
            file = "/Users/test/NMS/st_123/save.hg"
            format = "raw"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.save.dir.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS/st_123"
        );
        assert_eq!(
            config.save.file.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS/st_123/save.hg"
        );
        assert_eq!(config.save.format, "raw");
        // Legacy path should be None
        assert!(config.save.path.is_none());
    }

    #[test]
    fn test_parse_config_backward_compat_path_only() {
        let toml = r#"
            [save]
            path = "/Users/test/NMS/save.hg"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.save.path.is_some());
        assert!(config.save.dir.is_none());
        assert!(config.save.file.is_none());
    }

    #[test]
    fn test_effective_save_file_prefers_file_over_path() {
        let dir = tempfile::tempdir().unwrap();
        let save_file = dir.path().join("save.hg");
        let legacy_file = dir.path().join("legacy.hg");
        fs::write(&save_file, b"data").unwrap();
        fs::write(&legacy_file, b"data").unwrap();

        let mut config = Config::default();
        config.save.file = Some(save_file.clone());
        config.save.path = Some(legacy_file);

        assert_eq!(config.effective_save_file(), Some(save_file));
    }

    #[test]
    fn test_effective_save_file_falls_back_to_path_file() {
        let dir = tempfile::tempdir().unwrap();
        let save_file = dir.path().join("save.hg");
        fs::write(&save_file, b"data").unwrap();

        let mut config = Config::default();
        config.save.path = Some(save_file.clone());

        assert_eq!(config.effective_save_file(), Some(save_file));
    }

    #[test]
    fn test_effective_save_file_dir_with_saves() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("save.hg"), b"data").unwrap();

        let mut config = Config::default();
        config.save.dir = Some(dir.path().to_path_buf());

        let result = config.effective_save_file();
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("save.hg"));
    }

    #[test]
    fn test_effective_save_file_none_when_empty() {
        let config = Config::default();
        assert!(config.effective_save_file().is_none());
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

    #[test]
    fn test_parse_config_banner_custom_text() {
        let toml = r#"
            [display]
            banner = "Welcome to NMS!"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.display.banner.as_deref(), Some("Welcome to NMS!"));
        // show_banner defaults to true when not specified
        assert!(config.display.show_banner);
    }

    #[test]
    fn test_parse_config_banner_empty_disables() {
        let toml = r#"
            [display]
            banner = ""
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.display.banner.as_deref(), Some(""));
    }

    #[test]
    fn test_parse_config_show_banner_false() {
        let toml = r#"
            [display]
            show_banner = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.display.show_banner);
        // banner field defaults to None
        assert!(config.display.banner.is_none());
    }

    #[test]
    fn test_parse_config_show_system_banner_false() {
        let toml = r#"
            [display]
            show_system_banner = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.display.show_system_banner);
        // show_banner defaults to true independently
        assert!(config.display.show_banner);
    }
}
