//! Read and resolve NMS Copilot configuration.
//!
//! Configuration is read from `~/.nms-copilot/config.toml`; this module does
//! not write configuration files. Missing fields use the defaults described
//! on [`Config`] and its nested types. TOML strings such as save formats,
//! table styles, and routing algorithms are stored as supplied and are not
//! validated here. Unknown TOML keys are ignored by deserialization.
//!
//! [`Config::load_from`] is useful when an application already has a config
//! path. Environment overrides are read after the file or defaults, and only
//! the three documented `NMS_SAVE_*` variables are recognized. Cache-path
//! construction and the explicit-save-file fast path return configured paths
//! literally. [`Config::effective_save_file`] and [`Config::save_path`] also
//! inspect legacy paths and search configured save directories. No getter
//! creates a file or canonicalizes a returned path.
//!
//! # Examples
//!
//! ```
//! use nms_copilot::config::Config;
//! use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
//! use std::path::{Path, PathBuf};
//! use std::time::Duration;
//!
//! let mut config = Config::default();
//! assert_eq!(config.save.path, None);
//! assert_eq!(config.save.dir, None);
//! assert_eq!(config.save.file, None);
//! assert_eq!(config.save.format, "auto");
//! assert!(config.display.emoji_glyphs);
//! assert!(config.display.color);
//! assert_eq!(config.display.table_style, "rounded");
//! assert_eq!(config.display.banner, None);
//! assert!(config.display.show_banner);
//! assert!(config.display.show_system_banner);
//! assert_eq!(config.defaults.galaxy, 0);
//! assert_eq!(config.defaults.warp_range, None);
//! assert_eq!(config.defaults.tsp_algorithm, "2opt");
//! assert_eq!(config.defaults.find_limit, None);
//! assert!(config.cache.enabled);
//! assert_eq!(config.cache.path, None);
//! assert!(config.watch.enabled);
//! assert_eq!(config.watch.debounce_ms, 500);
//! assert_eq!(config.mcp.host, IpAddr::V4(Ipv4Addr::LOCALHOST));
//! assert_eq!(config.mcp.port, 5099);
//! assert_eq!(config.mcp_http_addr(), SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5099));
//! assert!(config.cache_enabled());
//! assert!(config.watch_enabled());
//! assert_eq!(config.watch_debounce(), Duration::from_millis(500));
//!
//! config.save.file = Some(PathBuf::from("chosen.hg"));
//! config.save.path = Some(PathBuf::from("legacy.hg"));
//! config.save.dir = Some(PathBuf::from("account"));
//! assert_eq!(config.effective_save_file(), Some(PathBuf::from("chosen.hg")));
//! assert_eq!(config.save_path(), Some(PathBuf::from("chosen.hg")));
//! config.cache.path = Some(PathBuf::from("override.rkyv"));
//! assert_eq!(config.cache_path_for(Some(Path::new("account/save3.hg"))), PathBuf::from("override.rkyv"));
//! assert_eq!(config.cache_path_for(None), PathBuf::from("override.rkyv"));
//!
//! config.cache.enabled = false;
//! config.watch.enabled = false;
//! config.watch.debounce_ms = 0;
//! config.mcp.host = IpAddr::V6(Ipv6Addr::LOCALHOST);
//! config.mcp.port = 0;
//! assert!(!config.cache_enabled());
//! assert!(!config.watch_enabled());
//! assert_eq!(config.watch_debounce(), Duration::ZERO);
//! assert_eq!(config.mcp_http_addr(), SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0));
//! ```
//!
//! # Reading configuration
//!
//! This `no_run` example compiles but does not run. If run, these calls read
//! configuration sources and process the current environment; they do not
//! modify the configuration file or environment.
//!
//! ```no_run
//! # fn main() -> Result<(), nms_copilot::config::ConfigError> {
//! use nms_copilot::config::Config;
//! use std::path::Path;
//!
//! let _specific = Config::load_from(Path::new("config.toml"))?;
//! let _default_path = Config::load()?;
//! let mut defaults = Config::default();
//! defaults.apply_env_overrides();
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::paths;

/// Top-level configuration loaded from TOML and environment overrides.
///
/// Each nested section has serde defaults, so omitted fields use that section's
/// [`Default`] implementation. Unknown TOML keys are ignored. Values such as
/// format and algorithm names remain strings and are not validated here.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Save paths and the save format (`"auto"` by default).
    pub save: SaveConfig,

    /// Display preferences; defaults enable glyph emoji, color, and banners.
    pub display: DisplayConfig,

    /// Command defaults: galaxy 0, `"2opt"`, and no range or result limit.
    pub defaults: DefaultsConfig,

    /// Cache enablement and an optional explicit cache path.
    pub cache: CacheConfig,

    /// File watcher enablement and debounce milliseconds (default 500).
    pub watch: WatchConfig,

    /// MCP HTTP host and port; constructing the address does not bind a socket.
    pub mcp: McpConfig,
}

/// Save path settings and the unvalidated format string.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SaveConfig {
    /// Legacy combined save path for file and directory fallback.
    ///
    /// Effective resolution checks `save.file`, this path as a file, `save.dir`
    /// for its newest current save, then this path as a directory for its
    /// newest current save. Searches are limited to the supplied directory.
    pub path: Option<PathBuf>,

    /// Directory searched for its most recent save after the legacy file check.
    pub dir: Option<PathBuf>,

    /// Explicit save file returned literally, even when it does not exist.
    pub file: Option<PathBuf>,

    /// Unvalidated save format string; defaults to `"auto"`.
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

/// Display settings with serde defaults; strings are not validated here.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Use emoji for portal glyphs (true) or hex digits (false).
    pub emoji_glyphs: bool,

    /// Whether consumers should enable ANSI color output; defaults to `true`.
    pub color: bool,

    /// Unvalidated table border style string; defaults to `"rounded"`.
    pub table_style: String,

    /// Custom banner text.
    ///
    /// `None` = use default embedded banner. Empty string = disable banner.
    pub banner: Option<String>,

    /// Whether a consumer should show its startup art banner (default `true`).
    pub show_banner: bool,

    /// Whether a consumer should show the system line after its banner (default `true`).
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

/// Defaults consumed by command frontends; values are stored without validation.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    /// Default galaxy index (0 = Euclid).
    pub galaxy: u8,

    /// Default warp range in light-years for routing.
    pub warp_range: Option<f64>,

    /// Unvalidated TSP algorithm name; defaults to `"2opt"`.
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

/// Cache settings. An explicit `path` overrides derived and legacy cache paths.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Whether a consumer should use its cache (default `true`).
    pub enabled: bool,

    /// Optional explicit cache-path override.
    ///
    /// When absent, a save path derives an account/slot cache path; without a
    /// save path, resolution uses the legacy `galaxy.rkyv` path.
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

/// File watcher settings; a zero-millisecond debounce is accepted.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    /// Whether a consumer should watch for file changes (default `true`).
    pub enabled: bool,
    /// Debounce duration in milliseconds, converted directly to [`Duration`].
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

/// MCP HTTP listener settings. These values form an address but do not start a listener.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// Host/IP address for the MCP HTTP server; defaults to IPv4 localhost.
    pub host: IpAddr,

    /// Port for the MCP HTTP server; defaults to 5099 and is not bound here.
    pub port: u16,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 5099,
        }
    }
}

impl Config {
    /// Load configuration from [`paths::config_path`].
    ///
    /// A missing file selects defaults; an existing file is read as UTF-8 and
    /// parsed before the supported environment overrides are applied. Resolving
    /// the default path inherits [`paths::data_dir`]'s panic if no home directory
    /// is available. See the [configuration module](crate::config) for its
    /// compiled loading example. This function does not write configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] when reading the file fails and
    /// [`ConfigError::Parse`] when its contents cannot be deserialized.
    ///
    /// # Panics
    ///
    /// Panics if the home directory cannot be determined while resolving the default path.
    pub fn load() -> Result<Self, ConfigError> {
        let path = paths::config_path();
        Self::load_from(&path)
    }

    /// Load configuration from `path`, using defaults when it does not exist.
    ///
    /// Environment overrides are applied after either defaults or file values.
    /// This method does not create or write the path.
    /// See the [configuration module](crate::config) for its compiled loading example.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] for a read/UTF-8 error and
    /// [`ConfigError::Parse`] when TOML cannot be deserialized. Parsing first
    /// tries direct TOML deserialization, then a TOML value conversion; if both
    /// attempts fail, the original document error is returned.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let mut config = if !path.exists() {
            Self::default()
        } else {
            let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
            parse_config(&content).map_err(ConfigError::Parse)?
        };
        config.apply_env_overrides();
        Ok(config)
    }

    /// Apply supported Unicode environment overrides in place.
    ///
    /// Reads:
    /// - `NMS_SAVE_DIR` -> `self.save.dir`
    /// - `NMS_SAVE_FILE` -> `self.save.file`
    /// - `NMS_SAVE_FORMAT` -> `self.save.format`
    ///
    /// Values are copied literally without trimming, expansion, or validation;
    /// non-Unicode values and all other environment variables are ignored.
    /// See the [configuration module](crate::config) for its compiled loading example.
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

    /// Resolve the effective save path using the documented precedence.
    ///
    /// Priority:
    /// 1. `save.file` (explicit file path)
    /// 2. `save.path` if it points to a file (backward compat)
    /// 3. `save.dir` — find most recent save in that directory
    /// 4. `save.path` if it points to a directory (backward compat)
    /// 5. `None` if nothing is configured or resolvable
    ///
    /// The explicit file is returned literally without checking existence.
    /// Directory searches use the save locator to choose the newest current
    /// save in the supplied account directory. They are not recursive and do
    /// not discover a default account. Search failures fall through to the next
    /// source. Legacy file/directory checks and searches access the filesystem;
    /// the result is not canonicalized.
    pub fn effective_save_file(&self) -> Option<PathBuf> {
        // 1. Explicit file
        if let Some(ref file) = self.save.file {
            return Some(file.clone());
        }

        // 2. Legacy path as file
        if let Some(ref path) = self.save.path
            && path.is_file()
        {
            return Some(path.clone());
        }

        // 3. Explicit dir — find most recent save in it
        if let Some(ref dir) = self.save.dir
            && let Ok(save) = nms_save::locate::find_most_recent_save_in(dir)
        {
            return Some(save.path().to_path_buf());
        }

        // 4. Legacy path as directory
        if let Some(ref path) = self.save.path
            && path.is_dir()
            && let Ok(save) = nms_save::locate::find_most_recent_save_in(path)
        {
            return Some(save.path().to_path_buf());
        }

        None
    }

    /// Resolve an effective cache path without checking or creating it.
    ///
    /// If `cache.path` is set, returns it literally. Otherwise a supplied save
    /// path derives a per-save cache path from its lexical components:
    /// `~/.nms-copilot/<account_dir>/<save_stem>/galaxy.rkyv`.
    /// With no save path, it falls back to the legacy
    /// `~/.nms-copilot/galaxy.rkyv` path. An explicit override avoids resolving
    /// the home directory.
    ///
    /// # Panics
    ///
    /// Panics if no cache override is set and the home directory cannot be
    /// determined while building the fallback path.
    pub fn cache_path_for(&self, save_path: Option<&std::path::Path>) -> PathBuf {
        if let Some(p) = &self.cache.path {
            return p.clone();
        }
        match save_path {
            Some(sp) => paths::cache_path_for_save(sp),
            None => paths::cache_path(),
        }
    }

    /// Return the effective save path, if configured or found.
    ///
    /// Delegates to [`Self::effective_save_file`]. For backward compatibility,
    /// also checks the legacy `save.path` field.
    pub fn save_path(&self) -> Option<PathBuf> {
        self.effective_save_file()
    }

    /// Return the stored cache-enabled flag.
    pub fn cache_enabled(&self) -> bool {
        self.cache.enabled
    }

    /// Return the stored file-watcher enabled flag.
    pub fn watch_enabled(&self) -> bool {
        self.watch.enabled
    }

    /// Convert the stored debounce milliseconds directly to a [`Duration`].
    pub fn watch_debounce(&self) -> Duration {
        Duration::from_millis(self.watch.debounce_ms)
    }

    /// Construct the configured MCP HTTP socket address without binding it.
    pub fn mcp_http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.mcp.host, self.mcp.port)
    }
}

fn parse_config(content: &str) -> Result<Config, toml::de::Error> {
    match toml::from_str(content) {
        Ok(config) => Ok(config),
        Err(document_error) => content
            .parse::<toml::Value>()
            .ok()
            .and_then(|value| value.try_into().ok())
            .ok_or(document_error),
    }
}

/// Errors returned while reading or deserializing configuration.
///
/// Both variants expose their contained error through [`std::error::Error::source`].
#[derive(Debug)]
pub enum ConfigError {
    /// Reading the configuration file or decoding its UTF-8 contents failed.
    Io(
        /// Underlying filesystem or UTF-8 decoding error.
        std::io::Error,
    ),
    /// The TOML document could not be deserialized into configuration values.
    Parse(
        /// Underlying TOML deserialization error.
        toml::de::Error,
    ),
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
        assert_eq!(config.mcp_http_addr().to_string(), "127.0.0.1:5099");
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

            [mcp]
            host = "127.0.0.1"
            port = 5055
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
        assert_eq!(config.mcp_http_addr().to_string(), "127.0.0.1:5055");
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
    fn test_parse_inline_save_with_mcp_config() {
        let toml = r#"
            save = { dir = "/Users/test/NMS/st_123", file = "/Users/test/NMS/st_123/save.hg", format = "auto" }

            [mcp]
            host = "127.0.0.1"
            port = 5055
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.save.file.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS/st_123/save.hg"
        );
        assert_eq!(config.mcp_http_addr().to_string(), "127.0.0.1:5055");
    }

    #[test]
    fn test_parse_legacy_inline_value_save_config() {
        let toml = r#"{ save = { dir = "/Users/test/NMS/st_123", file = "/Users/test/NMS/st_123/save.hg", format = "auto" } }"#;
        let config = parse_config(toml).unwrap();
        assert_eq!(
            config.save.file.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS/st_123/save.hg"
        );
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
    fn test_cache_path_default_no_save() {
        let config = Config::default();
        let path = config.cache_path_for(None);
        assert!(path.ends_with("galaxy.rkyv"));
    }

    #[test]
    fn test_cache_path_per_save() {
        let config = Config::default();
        let save = Path::new("/nms/st_12345/save3.hg");
        let path = config.cache_path_for(Some(save));
        assert!(path.ends_with("st_12345/save3/galaxy.rkyv"));
    }

    #[test]
    fn test_cache_path_override() {
        let toml = r#"
            [cache]
            path = "/tmp/custom-cache.rkyv"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.cache_path_for(Some(Path::new("/nms/st_99/save.hg"))),
            PathBuf::from("/tmp/custom-cache.rkyv")
        );
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
    fn test_mcp_config_partial_toml() {
        let toml = r#"
            [mcp]
            port = 5055
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.mcp_http_addr().to_string(), "127.0.0.1:5055");
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
