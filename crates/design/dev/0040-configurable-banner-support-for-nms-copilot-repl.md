# Configurable Banner Support for nms-copilot REPL

## Context

The nms-copilot REPL currently has a hardcoded startup message in `main.rs` (lines 44-52):

```
NMS Copilot v0.2.0
Loaded 293 systems, 644 planets, 12 bases (from save file)
Type 'help' for commands, 'exit' to quit.
```

We want to add a configurable ASCII art banner (the "user banner") that prints **before** the existing system info message. The existing message becomes the "system banner" — a separate, independently controllable function that prints model stats after the art banner. Both are enabled by default.

Pattern modeled on oxur's implementation (`~/lab/oxur/oxur/crates/oxur-cli/`): `include_str!()` for compile-time embedding, version placeholder substitution with ANSI-aware width preservation, config override/disable.

## Files to Modify

| File | Action |
|------|--------|
| `crates/nms-copilot/assets/banners/banner.txt` | **Create** — default ASCII art banner with ANSI colors |
| `crates/nms-copilot/src/banner.rs` | **Create** — banner loading, placeholder substitution, ANSI stripping, system banner |
| `crates/nms-copilot/src/config.rs` | **Edit** — add banner config fields to `DisplayConfig` |
| `crates/nms-copilot/src/lib.rs` | **Edit** — add `pub mod banner;` |
| `crates/nms-copilot/src/main.rs` | **Edit** — replace hardcoded output with banner module calls |

## Implementation Steps

### 1. Create banner asset: `crates/nms-copilot/assets/banners/banner.txt`

NMS/space-themed ASCII art with embedded ANSI true-color codes (`\x1b[38;2;R;G;Bm`). Include:

- NMS Copilot title in ASCII art
- `{version}` placeholder on a version line
- Color palette: deep space blues, nebula purples, star golds
- Does NOT include model stats or help hints (those go in the system banner)

### 2. Add config fields: `crates/nms-copilot/src/config.rs`

Add to `DisplayConfig`:

```rust
/// Custom banner text. None = use default embedded banner.
/// Empty string = disable banner.
pub banner: Option<String>,
/// Whether to show the art banner at startup (default: true).
pub show_banner: bool,
/// Whether to show the system info line after the banner (default: true).
/// "Loaded N systems, N planets, N bases (source)"
pub show_system_banner: bool,
```

Update `Default for DisplayConfig` with `banner: None, show_banner: true, show_system_banner: true`.
Update `test_parse_full_config` and add new config parsing tests.

### 3. Create banner module: `crates/nms-copilot/src/banner.rs`

Two distinct banner functions:

**Art banner** (the new ASCII art):

- `const DEFAULT_BANNER: &str = include_str!("../assets/banners/banner.txt");`
- `visible_width(s: &str) -> usize` — character width ignoring ANSI escapes (ported from oxur `terminal.rs:41-66`)
- `strip_ansi(s: &str) -> String` — remove all ANSI escape sequences
- `substitute_placeholder_in_line(line, placeholder, value) -> String` — replace preserving visual width (ported from oxur `terminal.rs:72-128`)
- `substitute_placeholders(banner, version) -> String` — replace `{version}` across all lines
- `resolve_banner(custom, show, color) -> Option<String>` — resolve config to final text
- `print_banner(custom, show, color)` — print art banner to stdout

**System banner** (the existing model stats, as a function):

- `print_system_banner(show, systems, planets, bases, source)` — prints the "Loaded N systems..." line plus "Type 'help'..." hint
- When `show` is false, prints nothing

Placeholder style: `{version}` (curly-brace, self-documenting — clearer than oxur's `N.N.N`)

Tests (~18):

- `test_visible_width_*` (plain text, with ANSI codes)
- `test_strip_ansi_*` (removes codes, preserves plain text)
- `test_substitute_placeholder_in_line_*` (no match, same length, shorter value, longer value, with ANSI)
- `test_resolve_banner_*` (default returns embedded, custom returns custom, empty=disabled, show_false=None, no_color strips ANSI)
- `test_print_system_banner_*` (show=true includes stats, show=false is empty)

### 4. Register module: `crates/nms-copilot/src/lib.rs`

Add `pub mod banner;`

### 5. Update startup: `crates/nms-copilot/src/main.rs`

Replace lines 44-52 with:

```rust
// Art banner (ASCII art, configurable)
banner::print_banner(
    config.display.banner.as_deref(),
    config.display.show_banner,
    config.display.color,
);

// System banner (model stats + help hint, independently configurable)
banner::print_system_banner(
    config.display.show_system_banner,
    model.systems.len(),
    model.planets.len(),
    model.bases.len(),
    source,
);
```

## Startup Output Sequence

```
┌─────────────────────────────────────────┐
│  [ASCII art banner with colors]         │  ← art banner (show_banner)
│  NMS Copilot v0.2.0                     │
└─────────────────────────────────────────┘
Loaded 293 systems, 644 planets, 12 bases   ← system banner (show_system_banner)
Type 'help' for commands, 'exit' to quit.
Watching save file for live updates.        ← existing watcher line (unchanged)
```

## Config Example

```toml
[display]
show_banner = true              # art banner (default: true)
show_system_banner = true       # model stats line (default: true)
# banner = "Custom welcome!"   # override default art
# banner = ""                  # disable art banner only
```

## Verification

1. `make format && make lint && make test`
2. Run `cargo run -p nms-copilot -- --save data/test/minimal_save.json` to see both banners
3. Test with `show_banner = false` — should skip art, still show system stats
4. Test with `show_system_banner = false` — should show art, skip stats
5. Test with `color = false` — art banner should have ANSI codes stripped
