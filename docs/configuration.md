# Configuration and save selection

Use an explicit save path when you want to be certain which game the copilot is reading:

```bash
nms info --save /path/to/save.hg
nms-copilot --save /path/to/save.hg
```

The one-shot CLI resolves saves for each command. The REPL and headless server choose a save at startup and keep following that file.

## Selecting a save in the CLI

`nms saves` lists discovered accounts and their occupied save slots. Slots 1–15 correspond to the canonical files `save.hg`, then `save2.hg` through `save30.hg`, paired as manual and auto saves.

```bash
nms saves
nms info --slot 3
nms find --slot 5 --biome Lush
nms list --save /path/to/save.hg bases
nms show --save /path/to/save.hg base "Home"
```

Save selection follows these rules:

1. An explicit `--save` wins, including when `--slot` is also supplied.
2. `--slot N` selects the newest manual or auto file in that slot from the first name-sorted account. Manual wins an equal-modification-time tie. An empty account or missing slot does not fall through to another account.
3. Without either option, the CLI selects the most recently modified discovered save across accounts.

`--slot` accepts only 1–15. Put `--save` before a nested target such as `list bases` or `show base`, as in the examples above. These CLI commands do not use the REPL's configured save path or its running model.

## Selecting a save in the REPL or server

The application checks an explicit `--save`, then configured save paths, then automatic discovery. If no save is found, the interactive REPL offers setup. Headless operation needs a resolvable save and cannot run the questionnaire.

```bash
nms-copilot --setup
nms-copilot --headless --save /path/to/save.hg
```

Configuration lives in `~/.nms-copilot/config.toml`. For a fixed file:

```toml
[save]
file = "/path/to/save.hg"
```

Alternatively, set `dir` to one account directory to select its newest save. `NMS_SAVE_FILE` and `NMS_SAVE_DIR` override their corresponding configuration fields. File selection takes precedence over directory selection, so clear a configured `file` if you want to switch to directory discovery. Paths in TOML and environment variables are taken literally; use an absolute path rather than `~`.

The legacy `save.path` setting is still accepted. Resolution checks `save.file`, legacy `save.path` as a file, `save.dir`, then legacy `save.path` as a directory. An explicit `--save` also takes precedence over `--setup`.

## Watching and caching

Both features are enabled by default:

```toml
[watch]
enabled = true
debounce_ms = 500

[cache]
enabled = true
```

Changes take effect at the next application start. Set `watch.enabled = false` to keep the startup atlas without ingesting later saves. Set `cache.enabled = false`, or pass `--no-cache`, to bypass cache reads and writes. Disabling one feature does not disable the other.

Each save normally has a cache at `~/.nms-copilot/<account-directory>/<save-file-stem>/galaxy.rkyv`. For example, `st_example/save3.hg` maps to `~/.nms-copilot/st_example/save3/galaxy.rkyv`. An explicit override is available:

```toml
[cache]
enabled = true
path = "/path/to/copilot-cache.rkyv"
```

The default mapping uses the immediate directory name and file stem, so distinct source paths with the same pair can collide. Give such sources distinct cache paths. See [live updates and cache behavior](live-atlas.md) for rebuilds and failure handling.

## Terminal and server settings

A few useful application settings are:

```toml
[display]
show_banner = true
show_system_banner = true
color = true

[defaults]
warp_range = 2500

[mcp]
host = "127.0.0.1"
port = 5099
```

`display.color` controls startup art color; it is not a table-palette selector. Configurable table themes and some other stored display/query settings are not wired into the application yet. Use [REPL session commands](exploration.md) for interactive reference, biome, and warp-range settings.

`--http ADDRESS` overrides the configured listener in a standard REPL build or selects HTTP in headless mode. See [HTTP MCP](mcp-http.md) for connection and smoke-test examples, and [shell completions](shell-completions.md) for terminal setup.
