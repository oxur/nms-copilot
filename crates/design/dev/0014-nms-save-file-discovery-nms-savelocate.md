# NMS Save File Discovery (`nms-save::locate`)

## Context

The user wants reusable functions for discovering NMS save files on disk — finding the platform-specific save directory, listing account directories (`st_*` for Steam, `DefaultUser` for GOG), listing save files within an account, and selecting saves by recency. The milestone 1.9 design doc placed this in `nms-cli/src/resolve.rs`, but it belongs in `nms-save` so both `nms-cli` and `nms-watch` can reuse it.

## Files to Modify

| File | Action |
|------|--------|
| `crates/nms-save/Cargo.toml` | Add `dirs = "6"`, `thiserror = "2"`, `tempfile = "3"` (dev) |
| `crates/nms-save/src/locate.rs` | **New** — all discovery types and functions |
| `crates/nms-save/src/lib.rs` | Add `pub mod locate;` |

## Platform-Specific Save Paths (confirmed via web research)

- **macOS**: `dirs::data_dir()` + `HelloGames/NMS/` → `~/Library/Application Support/HelloGames/NMS/`
- **Windows**: `dirs::data_dir()` + `HelloGames/NMS/` → `%APPDATA%\HelloGames\NMS\`
- **Linux (Proton)**: `dirs::home_dir()` + `.local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/AppData/Roaming/HelloGames/NMS/`

Note: Linux path goes through Steam's Proton compatdata — cannot use `data_dir()` there.

## Types

```rust
// --- Errors ---
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LocateError {
    NoHomeDir, SaveDirNotFound(PathBuf), NoAccountDirs(PathBuf),
    NoSaveFiles(PathBuf), UnsupportedPlatform, Io(#[from] io::Error),
}

// --- Account directory ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDir { path: PathBuf, kind: AccountKind }

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccountKind { Steam(u64), Gog, Unknown(String) }

// --- Save file ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFile { path: PathBuf, slot: u8, save_type: SaveType, modified: SystemTime }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaveType { Manual, Auto }

// --- Save slot (paired manual + auto) ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSlot { slot: u8, manual: Option<SaveFile>, auto: Option<SaveFile> }
```

## Public API

```rust
/// Platform-specific NMS save root (does NOT check existence).
pub fn nms_save_dir() -> Result<PathBuf, LocateError>

/// Like nms_save_dir() but verifies directory exists.
pub fn nms_save_dir_checked() -> Result<PathBuf, LocateError>

/// List account directories (st_*, DefaultUser, etc.).
pub fn list_accounts(save_dir: &Path) -> Result<Vec<AccountDir>, LocateError>

/// List save*.hg files in an account dir, sorted newest-first.
pub fn list_saves(account_dir: &Path) -> Result<Vec<SaveFile>, LocateError>

/// Group saves into slot pairs (manual + auto).
pub fn group_into_slots(saves: &[SaveFile]) -> Vec<SaveSlot>

/// Find the most recent save across all accounts.
pub fn find_most_recent_save() -> Result<SaveFile, LocateError>

/// Find the most recent save in a specific account dir.
pub fn find_most_recent_save_in(account_dir: &Path) -> Result<SaveFile, LocateError>
```

All "select" operations return data — no interactive I/O in the library.

## Save Filename Parsing

NMS save slot numbering: `save.hg` = slot 1 manual (index 1), `save2.hg` = slot 1 auto (index 2), `save3.hg` = slot 2 manual, `save4.hg` = slot 2 auto, etc. Formula: slot = `(index + 1) / 2`, type = odd→Manual, even→Auto. Files prefixed `mf_` are metadata and excluded.

## Implementation Steps

1. Add dependencies to `crates/nms-save/Cargo.toml`
2. Create `crates/nms-save/src/locate.rs` with error type and all data types
3. Implement `nms_save_dir()` with platform-specific `#[cfg]` blocks
4. Implement `nms_save_dir_checked()`, `list_accounts()`, `list_saves()`
5. Implement `parse_save_filename()` helper, `group_into_slots()`
6. Implement `find_most_recent_save()` and `find_most_recent_save_in()`
7. Add `pub mod locate;` to `lib.rs`
8. Write unit tests (filename parsing, account kind parsing, path construction)
9. Write integration tests with `tempfile` (list/sort/group with real temp dirs)
10. `cargo fmt`, `cargo clippy -p nms-save -- -D warnings`, `cargo test -p nms-save`

## Tests (all work without NMS installed)

**Unit tests:**

- `nms_save_dir` returns path containing `HelloGames/NMS`
- `parse_save_filename`: save.hg→(1,Manual), save2.hg→(1,Auto), save3.hg→(2,Manual), save30.hg→(15,Auto), mf_save.hg→None, readme.txt→None
- Account kind parsing: `st_76561198025707979`→Steam(id), `DefaultUser`→Gog, other→Unknown

**Integration tests (tempfile):**

- `list_accounts` finds Steam + GOG dirs
- `list_saves` finds files sorted by mtime, excludes mf_* files
- `group_into_slots` pairs manual/auto correctly
- `find_most_recent_save_in` picks newest
- Empty dir → `NoSaveFiles` error

## Verification

```bash
cargo fmt -p nms-save
cargo clippy -p nms-save -- -D warnings
cargo test -p nms-save
cargo test --workspace  # ensure nothing else broke
```
