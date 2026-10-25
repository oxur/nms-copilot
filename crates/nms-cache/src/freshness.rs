//! Cache freshness and cache-or-save model loading.
//!
//! Compares save and cache modification times to select a cache candidate;
//! the checked loader separately validates the candidate's envelope and data.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Compare cache and save modification times.
///
/// This check uses only filesystem modification times: it returns `true` when
/// the cache time is strictly later than the save time. Equal times, missing
/// files, and unreadable modification times return `false`. It does not read or
/// validate either file's contents or the cache envelope.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// let fresh = nms_cache::is_cache_fresh(
///     Path::new("galaxy.rkyv"),
///     Path::new("save.hg"),
/// );
/// println!("cache is newer: {fresh}");
/// ```
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

/// Model returned by cache loading or source-save rebuilding.
///
/// `was_cached` identifies the input used to build the returned model. It can
/// remain `true` after a best-effort cache repair write, and is `false` when a
/// parsed save supplied the model even if writing its cache fails.
pub struct LoadResult {
    /// The loaded galaxy model.
    pub model: nms_graph::GalaxyModel,
    /// Whether the model was loaded from cache.
    pub was_cached: bool,
    /// The version from the chosen cache or parsed source save.
    pub save_version: u32,
}

/// Load a fresh compatible cache or rebuild from the checked source save.
///
/// Modification times select a cache candidate; a fresh candidate is still
/// checked for a supported envelope and valid cache data. If its read, decode,
/// or duplicated player-reality consistency check fails, startup warns and
/// rebuilds from the checked source save. This check requires
/// `player.current_reality == player.current_address.reality_index`; it does not
/// compare the independently selected galaxy or previous/freighter addresses.
/// A successful compatible-cache load does not read or compare source contents
/// or source save version. The loader ensures the physical player's system key;
/// it rewrites the cache on a best-effort basis only when that insertion grows
/// the system count. An existing genuine or marked key, or absent player state,
/// does not trigger a repair write. A failed repair warns but still returns the
/// ensured model with `was_cached` set to `true`. Missing or stale caches skip
/// cache reading; cache read failures warn before checked source rebuilding.
/// Cold parsing also ensures the key, and a failed cache write warns without
/// discarding the parsed model. With `no_cache`, all cache I/O is skipped, but
/// the parsed model is still ensured.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// let loaded = nms_cache::load_or_rebuild(
///     Path::new("galaxy.rkyv"),
///     Path::new("save.hg"),
///     false,
/// )?;
/// println!("loaded from cache: {}", loaded.was_cached);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns the boxed [`nms_save::SaveError`] when the source save cannot be read,
/// parsed, or converted to a validated address. Cache read, compatibility,
/// and repair-write failures are warnings followed by source rebuilding or an
/// in-memory success, as applicable.
///
/// After either a cold parse or a compatible warm rebuild, inserting a missing
/// player-system key rebuilds the model's default per-galaxy KNN-10 edges before
/// returning or extracting cache data. Existing keys and saves without player
/// state do not trigger that additional rebuild.
///
/// # Panics
///
/// When source parsing is needed, this can inherit the documented panic conditions
/// of [`nms_save::parse_validated_save_file`], including malformed compressed
/// size offsets on 32-bit targets and an invalid bundled mapping file.
pub fn load_or_rebuild(
    cache_path: &Path,
    save_path: &Path,
    no_cache: bool,
) -> Result<LoadResult, Box<dyn std::error::Error>> {
    // Try cache first (unless --no-cache)
    if !no_cache && is_cache_fresh(cache_path, save_path) {
        match crate::read_cache(cache_path) {
            Ok(data) => {
                let save_version = data.save_version;
                let mut model = crate::rebuild_model(&data);
                let system_count = model.system_count();
                model.ensure_player_system();
                if model.system_count() != system_count {
                    model.build_edges(nms_graph::EdgeStrategy::default());
                    let repaired = crate::extract_cache_data(&model, save_version);
                    if let Err(e) = crate::write_cache(&repaired, cache_path) {
                        eprintln!(
                            "Warning: could not repair cache after ensuring player system ({e})"
                        );
                    }
                }
                return Ok(LoadResult {
                    model,
                    was_cached: true,
                    save_version,
                });
            }
            Err(e) => {
                eprintln!("Warning: cache read failed ({e}), rebuilding from save");
            }
        }
    }

    // Parse save file
    let save = nms_save::parse_validated_save_file(save_path)?;
    let save_version = save.root().version;
    let mut model = nms_graph::GalaxyModel::from_validated_save(&save);
    let system_count = model.system_count();
    model.ensure_player_system();
    if model.system_count() != system_count {
        model.build_edges(nms_graph::EdgeStrategy::default());
    }

    // Write cache for next time
    if !no_cache {
        let data = crate::extract_cache_data(&model, save_version);
        if let Err(e) = crate::write_cache(&data, cache_path) {
            eprintln!("Warning: could not write cache ({e})");
        }
    }

    Ok(LoadResult {
        model,
        was_cached: false,
        save_version,
    })
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
