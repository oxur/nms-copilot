# Adopt oxur-cli tables for all tabular output

## Context

All tabular and list output currently uses manual `format!` with padding (`{:<30}`, `{:>5}`), which produces misaligned columns — especially with variable-length data like base names, system names, and emoji glyphs. The oxur project's `oxur-cli` crate (published on crates.io as v0.2.0) provides a polished table system built on `tabled` with TOML-driven theming. We'll adopt it with an NMS-specific color theme (deep space blues/cyans instead of oxur's sunset oranges).

## Approach

### 1. Add dependencies

Add to workspace `Cargo.toml`:

```toml
oxur-cli = "0.2"
tabled = "0.17"
```

Add `oxur-cli` and `tabled` to these crate Cargo.toml files:

- `nms-query` — for `display.rs` format functions
- `nms-copilot` — for `dispatch.rs` list commands
- `nms-cli` — for `info.rs` and `saves.rs`

### 2. Create NMS table theme

Create `crates/nms-query/src/table.rs` — NMS-specific theme and helpers.

Defines:

- `NMS_THEME` const TOML string (deep space palette: dark navy backgrounds, cyan/blue headers, soft blue-white alternating rows)
- `pub fn nms_theme() -> TableStyleConfig` — parses `NMS_THEME`
- `pub fn nms_theme_no_color() -> TableStyleConfig` — plain theme for no-color mode and MCP
- Re-exports `tabled::builder::Builder` and `oxur_cli::table::TableStyleConfig` for convenience

Color palette (deep space theme):

```toml
[title]
bg_color = "#1E3A5F"      # deep navy
fg_color = "#E0F0FF"      # ice white

[header]
bg_color = "#2C5F8A"      # medium blue
fg_color = "#E0F0FF"      # ice white

[rows]
colors = [
    { bg = "#0A1929", fg = "#B0D0E8" },   # dark navy / soft blue
    { bg = "#0A1929", fg = "#8BBBD0" },   # dark navy / muted cyan
]

[style]
vertical_bg_color = "#0A1929"
vertical_fg_color = "#1E3A5F"

[footer]
bg_color = "#1E3A5F"
fg_color = "#4A9BC7"
```

### 3. Convert tables (3 scopes)

#### Scope A: REPL list commands (`crates/nms-copilot/src/dispatch.rs`)

5 tables in `dispatch_list`:

| Table | Columns | Notes |
|-------|---------|-------|
| `list galaxies` | Index, Name, Type | 256 rows, optional --type filter |
| `list biomes` | Biome (+ subtypes) | 15 or 46 rows |
| `list glyphs` | Hex, Emoji, Name | 16 rows |
| `list bases` | Name, Type, Galaxy, Address | Dynamic |
| `list systems` | Name, Address, Planets | Dynamic with --limit |

Each converts from manual `format!` lines to `Builder::default()` + `push_record()` + `nms_theme().apply_to_table()`.

#### Scope B: Query results (`crates/nms-query/src/display.rs`)

6 tables/views:

| Function | Type | Notes |
|----------|------|-------|
| `format_find_results` | Table (6 cols) | #, Planet, Biome, System, Distance, Portal Glyphs |
| `format_route` | Table (5 cols) | Hop, System, Distance, Cumulative, Portal Glyphs |
| `format_stats` | Key-value + subtable | Summary stats + biome distribution table |
| `format_show_system` | Key-value + planets table | System detail + planets subtable |
| `format_show_base` | Key-value | Base detail view |

Key consideration: These functions currently embed per-cell ANSI colors via `theme.biome_style()`, `theme.system_name.paint()`, etc. The `tabled` library applies its own ANSI colors via `Colorization`. We have two options:

- **Option A**: Drop the existing `Theme` color system in favor of tabled's theme — simpler but loses per-cell semantic coloring (e.g., biome-specific colors).
- **Option B**: Build plain-text cells with `Builder`, apply `TableStyleConfig` for structure/headers, then use `helpers::apply_cell_color()` for semantic coloring on specific cells.

**Recommendation**: Option A for now — use the table theme for uniform coloring. The NMS theme with alternating row colors will look polished. Per-cell biome colors can be added later as an enhancement. This keeps the refactor scope manageable.

This means `display.rs` functions drop the `Theme` parameter (or ignore it), replacing it with `TableStyleConfig`. The existing `theme.rs` module can be deprecated or kept for non-table uses (e.g., the convert key-value output which is out of scope).

**Note on tests**: The existing display tests check for data presence in output strings (e.g., `assert!(output.contains("Eden"))`). These will still pass since the table builder preserves the data. Tests checking for ANSI codes (`\x1b[`) may need adjustment.

#### Scope C: CLI info/saves (`crates/nms-cli/src/`)

4 tables:

| File | Table | Notes |
|------|-------|-------|
| `info.rs` | Save summary | Key-value pairs (name, platform, version, etc.) |
| `info.rs` | Discoveries | Label + count rows |
| `info.rs` | Bases list | Name, Type, Address |
| `saves.rs` | Save slots | Slot, Manual, Auto, Most Recent |

These currently use `println!` directly. Convert to build table string and print once.

## Files to modify

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Add `oxur-cli = "0.2"`, `tabled = "0.17"` |
| `crates/nms-query/Cargo.toml` | Add `oxur-cli`, `tabled` |
| `crates/nms-query/src/table.rs` | **Create** — NMS theme + helpers |
| `crates/nms-query/src/lib.rs` | Add `pub mod table;` |
| `crates/nms-query/src/display.rs` | Rewrite all format functions to use Builder |
| `crates/nms-copilot/Cargo.toml` | Add `tabled` (already has nms-query for theme) |
| `crates/nms-copilot/src/dispatch.rs` | Rewrite `dispatch_list` to use Builder |
| `crates/nms-cli/Cargo.toml` | Add `oxur-cli`, `tabled` |
| `crates/nms-cli/src/info.rs` | Rewrite to use Builder |
| `crates/nms-cli/src/saves.rs` | Rewrite to use Builder |

## Implementation order

1. Add deps + create `nms-query/src/table.rs` with NMS theme
2. Convert `dispatch_list` in nms-copilot (simplest, isolated)
3. Convert `display.rs` in nms-query (largest, most tests)
4. Convert `info.rs` and `saves.rs` in nms-cli
5. `make format && make lint && make test`

## Verification

1. `make format && make lint && make test` — all existing tests pass
2. `cargo run -p nms-copilot -- --save data/test/multi_system_save.json`
   - `list galaxies --type Lush` — proper table with NMS colors
   - `list glyphs` — emoji column aligned
   - `find --nearest 5` — table output
   - `stats --biomes` — biome distribution table
   - `show system <name>` — detail view with planets subtable
   - `route --biome Lush --nearest 3` — route itinerary table
3. `cargo run -p nms-cli -- --save data/test/multi_system_save.json info`
4. `cargo run -p nms-cli -- saves` (if save dir exists)
