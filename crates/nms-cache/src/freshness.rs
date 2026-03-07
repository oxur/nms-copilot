//! Cache freshness checking.
//!
//! Compares the modification time of the save file against the cache file
//! to determine if the cache is still valid.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Check if the cache file is fresh relative to the save file.
///
/// Returns `true` if the cache exists and is newer than the save file.
/// Returns `false` if the cache is missing, older than the save, or if
/// timestamps can't be read.
pub fn is_cache_fresh(cache_path: &Path, save_path: &Path) -> bool {
    let cache_mtime = match file_mtime(cache_path) {
        Some(t) => t,
        None => return false,
    };
    let save_mtime = match file_mtime(save_path) {
        Some(t) => t,
        None => return false,
    };
    cache_mtime > save_mtime
}

/// Get the modification time of a file.
fn file_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

/// Load a model from cache if fresh, otherwise parse the save file.
///
/// This is the primary entry point for the startup path.
/// Returns `(GalaxyModel, was_cached)`.
pub fn load_or_rebuild(
    cache_path: &Path,
    save_path: &Path,
    no_cache: bool,
) -> Result<(nms_graph::GalaxyModel, bool), Box<dyn std::error::Error>> {
    // Try cache first (unless --no-cache)
    if !no_cache && is_cache_fresh(cache_path, save_path) {
        match crate::read_cache(cache_path) {
            Ok(data) => {
                let model = crate::rebuild_model(&data);
                return Ok((model, true));
            }
            Err(e) => {
                eprintln!("Warning: cache read failed ({e}), rebuilding from save");
            }
        }
    }

    // Parse save file
    let save = nms_save::parse_save_file(save_path)?;
    let model = nms_graph::GalaxyModel::from_save(&save);

    // Write cache for next time
    if !no_cache {
        let data = crate::extract_cache_data(&model, save.version);
        if let Err(e) = crate::write_cache(&data, cache_path) {
            eprintln!("Warning: could not write cache ({e})");
        }
    }

    Ok((model, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_fresh_cache_returns_true() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&save_path, "save data").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&cache_path, "cache data").unwrap();

        assert!(is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_stale_cache_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&cache_path, "cache data").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&save_path, "updated save").unwrap();

        assert!(!is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_missing_cache_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&save_path, "save data").unwrap();
        assert!(!is_cache_fresh(&cache_path, &save_path));
    }

    #[test]
    fn test_missing_save_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let save_path = dir.path().join("save.json");
        let cache_path = dir.path().join("galaxy.rkyv");

        fs::write(&cache_path, "cache data").unwrap();
        assert!(!is_cache_fresh(&cache_path, &save_path));
    }
}
