# REPL Setup Wizard & Config Improvements

## Context

Running `nms-copilot` with no NMS save files configured crashes with "Error loading save: JSON parse error: expected value at line 1 column 1". The REPL needs a graceful first-run experience: detect saves automatically, and when that fails, guide the user through an interactive setup wizard.

## Changes

### 1. Add `dialoguer` dependency

**Files:** `Cargo.toml` (workspace), `crates/nms-copilot/Cargo.toml`

- Add `dialoguer = "0.11"` to workspace dependencies
- Add `dialoguer = { workspace = true }` to nms-copilot dependencies

### 2. Split `save.path` into `save.dir` + `save.file` in config

**File:** `crates/nms-copilot/src/config.rs`

- Add `dir: Option<PathBuf>` and `file: Option<PathBuf>` to `SaveConfig`
- Keep `path: Option<PathBuf>` for backward compat (old configs still work)
- Add `Config::effective_save_file() -> Option<PathBuf>` with merge logic:
  - `file` > `path` (if it's a file) > auto-detect from `dir` > auto-detect from `path` (if dir)
- Add `Config::apply_env_overrides(&mut self)`:
  - `NMS_SAVE_DIR` → `save.dir`
  - `NMS_SAVE_FILE` → `save.file`
  - `NMS_SAVE_FORMAT` → `save.format`
- Call `apply_env_overrides()` at the end of `Config::load()`
- Update existing `save_path()` to delegate to `effective_save_file()`
- Add/update tests for new fields, backward compat, and env overrides

### 3. Create setup wizard module

**New file:** `crates/nms-copilot/src/setup.rs`

Public API:

```rust
pub fn run_setup_wizard() -> Result<PathBuf, SetupError>
```

Wizard flow:

1. Print welcome banner explaining first-time setup
2. Call `nms_save::locate::nms_save_dir()` to find platform default
3. If found, call `list_accounts()` on it
   - If not found, prompt user for custom path via `dialoguer::Input`
4. If 1 account → auto-select with message; if multiple → `dialoguer::Select`
5. Call `list_saves()` + `group_into_slots()` on selected account
6. Display slots with metadata (slot #, manual/auto, timestamp) — reuse display pattern from `crates/nms-cli/src/saves.rs`
7. If 1 save → auto-select; if multiple → `dialoguer::Select`
8. Ask user: "Save these settings to ~/.nms-copilot/config.toml?" via `dialoguer::Confirm`
   - If yes: write `[save]` section to config (merge with existing file if present)
   - If no: use for this session only
9. Return the resolved save file path

Error type: `SetupError` with variants `Cancelled`, `NoInstallation`, `Locate(LocateError)`, `Io`

Config writing: parse existing config.toml as `toml::Value` table, update `[save]` section only, write back. This preserves user's `[display]`, `[defaults]`, etc.

### 4. Register setup module

**File:** `crates/nms-copilot/src/lib.rs`

Add `pub mod setup;`

### 5. Restructure `main.rs` startup flow

**File:** `crates/nms-copilot/src/main.rs`

Extract `resolve_save_path(args, config) -> Option<PathBuf>` with priority chain:

1. `--save` CLI arg (highest)
2. ENV vars + config file (via `config.effective_save_file()`, since env overrides already applied)
3. Auto-detect via `nms_save::locate::find_most_recent_save()`
4. Interactive wizard (only if `std::io::stdin().is_terminal()`)
5. If non-TTY and nothing found: print helpful error + exit

Remove the `find_most_recent_save()` fallback from `load_model()` — that logic moves to `resolve_save_path()`.

Update `load_model()` to take `save_path: PathBuf` (not Option) since resolution is done before calling it.

### Key reuse

- **`nms_save::locate`** — `nms_save_dir()`, `list_accounts()`, `list_saves()`, `group_into_slots()`, `find_most_recent_save()`, `find_most_recent_save_in()` — all reused directly, no changes needed
- **`crates/nms-cli/src/saves.rs`** — `format_mtime()` pattern duplicated in setup.rs (8 lines, not worth a shared dep)
- **`std::io::IsTerminal`** — stable since Rust 1.70, no new dependency needed for TTY check

## File Summary

| File | Action |
|------|--------|
| `Cargo.toml` | Add `dialoguer` workspace dep |
| `crates/nms-copilot/Cargo.toml` | Add `dialoguer` dep |
| `crates/nms-copilot/src/config.rs` | Add dir/file fields, env overrides, effective_save_file() |
| `crates/nms-copilot/src/setup.rs` | **New** — wizard module |
| `crates/nms-copilot/src/lib.rs` | Add `pub mod setup` |
| `crates/nms-copilot/src/main.rs` | New resolve_save_path(), wizard fallback, restructure startup |

## Verification

1. `make format && make lint && make test` — all 688+ tests pass
2. Run `nms-copilot` with no config, no NMS install → wizard launches
3. Run `nms-copilot --save /path/to/save.hg` → skips wizard, loads directly
4. Set `NMS_SAVE_FILE=/path/to/save.hg nms-copilot` → loads from env
5. Create `~/.nms-copilot/config.toml` with old `save.path` field → backward compat works
6. Run wizard, save to config → second run loads without wizard
7. Pipe stdin (`echo | nms-copilot`) → no wizard, helpful error message
