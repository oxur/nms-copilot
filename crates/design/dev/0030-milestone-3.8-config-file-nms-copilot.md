# Milestone 3.8 -- Config File (nms-copilot)

Load configuration from `~/.nms-copilot/config.toml`. Supports save path, display preferences, default warp range, cache settings. Parsed with `toml` crate.

## Crate: `nms-copilot`

Path: `crates/nms-copilot/`

### Dependencies to add

```toml
# crates/nms-copilot/Cargo.toml
[dependencies]
# ... existing ...
toml = "0.8"
serde = { version = "1", features = ["derive"] }
```

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# ... existing ...
toml = "0.8"
serde = { version = "1", features = ["derive"] }
```

Note: `serde` may already be a workspace dependency via nms-core. Check before adding.

---

## New File: `crates/nms-copilot/src/config.rs`

Configuration types and loading logic.

```rust
//! Configuration file support for NMS Copilot.
//!
//! Config file location: `~/.nms-copilot/config.toml`
//!
//! All fields are optional -- sensible defaults are used when not specified.

use std::fs;
use std::path::{Path, PathBuf};

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
        self.cache
            .path
            .clone()
            .unwrap_or_else(paths::cache_path)
    }

    /// Resolve the effective save path (if configured).
    pub fn save_path(&self) -> Option<&Path> {
        self.save.path.as_deref()
    }

    /// Whether caching is enabled.
    pub fn cache_enabled(&self) -> bool {
        self.cache.enabled
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
```

---

## Modified File: `crates/nms-copilot/src/main.rs`

Load config at startup and use it for save path resolution, cache settings, and session defaults.

```rust
mod config;

use config::Config;

fn main() {
    // Load config
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not load config: {e}");
            Config::default()
        }
    };

    let args: Vec<String> = std::env::args().collect();

    // CLI args override config
    let save_path = parse_save_arg(&args)
        .or_else(|| config.save_path().map(PathBuf::from));

    let no_cache = args.iter().any(|a| a == "--no-cache") || !config.cache_enabled();
    let cache_path = config.cache_path();

    // Load model (with cache support from 3.7)
    let (model, was_cached) = match load_model(save_path, &cache_path, no_cache) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading save: {e}");
            std::process::exit(1);
        }
    };

    // Initialize session with config defaults
    let mut session = session::SessionState::from_model(&model);
    if let Some(warp_range) = config.defaults.warp_range {
        session.set_warp_range(warp_range);
    }

    // ... REPL loop ...
}
```

---

## Modified File: `crates/nms-copilot/src/lib.rs`

```rust
pub mod commands;
pub mod completer;
pub mod config;
pub mod dispatch;
pub mod paths;
pub mod prompt;
pub mod session;
```

---

## Example Config File

This is the reference config that could be generated by `nms-copilot --init-config`:

```toml
# NMS Copilot configuration
# Location: ~/.nms-copilot/config.toml

[save]
# path = "/path/to/NMS/saves/"     # auto-detected if omitted
# format = "auto"                   # auto | raw | goatfungus

[display]
emoji_glyphs = true                 # use emoji for portal glyphs
color = true                        # ANSI color output
# table_style = "rounded"           # table border style

[defaults]
galaxy = 0                          # Euclid
# warp_range = 2500                 # default warp range (ly) for routing
# tsp_algorithm = "2opt"            # nearest-neighbor | 2opt
# find_limit = 20                   # default result limit for find

[cache]
enabled = true
# path = "~/.nms-copilot/galaxy.rkyv"
```

---

## Tests

### File: `crates/nms-copilot/src/config.rs` (inline tests)

```rust
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
        assert_eq!(config.save.path.as_deref().unwrap().to_str().unwrap(), "/Users/test/NMS");
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
        // Unspecified sections use defaults
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
        // With serde's default deny_unknown_fields off, this should parse fine
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.save.path.is_some());
    }
}
```

---

## Implementation Notes

1. **All fields optional**: Every config field has a sensible default. An empty `config.toml` (or no file at all) produces a fully functional configuration.

2. **`#[serde(default)]` on every struct**: Ensures missing sections in TOML use the struct's `Default` impl rather than erroring.

3. **CLI overrides config**: `--save` flag overrides `config.save.path`. `--no-cache` overrides `config.cache.enabled`. Config is the base; CLI args are the override layer.

4. **No `confyg`/`twyg` yet**: The project plan mentions these crates but they're not strictly necessary for this milestone. Plain `toml` + `serde::Deserialize` is sufficient. `confyg` (config management) and `twyg` (logging) can be added later if their features are needed.

5. **Unknown fields tolerated**: `serde` defaults to ignoring unknown fields, which is the right behavior for forward compatibility -- adding new config options in future versions won't break old config files.

6. **Config error handling**: Missing file returns default (not an error). Existing but unparseable file is an error. This matches user expectations -- no config file is fine, a broken one should be reported.

7. **Display preferences**: `emoji_glyphs` and `color` are defined here but not wired into the display layer yet. The `format_*` functions in `nms-query::display` would need to accept a display options struct. This can be done incrementally.

8. **Integration with session**: The config's `defaults.warp_range` is applied to `SessionState` at startup, providing a persistent default that can be overridden with `set warp-range`.
