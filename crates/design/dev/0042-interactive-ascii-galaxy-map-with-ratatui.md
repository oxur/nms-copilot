# Interactive ASCII Galaxy Map (with ratatui)

## Context

Add a `map` command to the nms-copilot REPL that opens an interactive full-screen ASCII galaxy map. The player can navigate with arrow keys and zoom in/out on regions to explore their discoveries. Uses ratatui for TUI rendering, which handles layout, borders, styled text, and the canvas widget — dramatically reducing manual terminal math.

## Architecture

### Zoom Levels (3 tiers)

| Level | Name | Voxel Extent | Scale |
|-------|------|-------------|-------|
| 0 | Galaxy | 4096×4096 | ~24K ly/cell |
| 1 | Region | 512×512 | ~3.2K ly/cell |
| 2 | Local | 64×64 | ~400 ly/cell |

Grid size adapts to terminal. Viewport centered on cursor position.

### Cell Characters

| Symbol | Meaning |
|--------|---------|
| ` ` | Empty |
| `·` | 1 system |
| `+` | 2-3 systems |
| `*` | 4-7 systems |
| `#` | 8+ systems |
| `A`-`Z` | Named bases |
| `@` | Player position |
| Cursor | Reverse video highlight |

### Key Bindings

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Enter / `+` | Zoom in on cursor cell |
| Escape / `-` | Zoom out (exit at galaxy level) |
| `q` | Exit map |
| `c` | Center on player |
| `?` | Toggle help overlay |

## Dependencies

Add to workspace `Cargo.toml`:

```toml
ratatui = "0.30"
```

Add to `crates/nms-copilot/Cargo.toml`:

```toml
crossterm = { workspace = true }
ratatui = { workspace = true }
```

Note: crossterm 0.28 is already an indirect dep via reedline. Ratatui 0.29 uses crossterm as its backend.

## Files to Create/Modify

| File | Action |
|------|--------|
| `Cargo.toml` (workspace) | **Edit** — add `ratatui = "0.30"`, `crossterm = "0.28"` |
| `crates/nms-copilot/Cargo.toml` | **Edit** — add `crossterm`, `ratatui` |
| `crates/nms-copilot/src/map/mod.rs` | **Create** — `run_map()` entry point, terminal setup/teardown |
| `crates/nms-copilot/src/map/state.rs` | **Create** — `MapState`, `ZoomLevel`, viewport math, cursor logic |
| `crates/nms-copilot/src/map/render.rs` | **Create** — ratatui widget rendering (map grid, legend, status bar) |
| `crates/nms-copilot/src/map/input.rs` | **Create** — crossterm event handling, key → action mapping |
| `crates/nms-copilot/src/commands.rs` | **Edit** — add `Map` to Action enum |
| `crates/nms-copilot/src/main.rs` | **Edit** — intercept `Action::Map` before dispatch |
| `crates/nms-copilot/src/lib.rs` | **Edit** — add `pub mod map;` |

## Key Design Decisions

### Ratatui Usage

**Layout** (vertical split):

```
┌─────────────────────────────────────────┐
│ Map area (Canvas or custom widget)      │  ← 80% of height
│                                         │
├─────────────────────────────────────────┤
│ Status: [Euclid] [Galaxy] X=+100 Z=-200│  ← 1 row
├─────────────────────────────────────────┤
│ Legend: A=Home  B=Mining  @=You  ?=Help │  ← 1-2 rows
└─────────────────────────────────────────┘
```

Use `Layout::vertical([Constraint::Min(5), Constraint::Length(1), Constraint::Length(2)])` for the three areas.

**Map rendering**: Custom widget implementing `ratatui::widgets::Widget`. For each cell, compute density and render a styled `Span`. The cursor cell gets `Style::reversed()`. Bases and player get distinct colors.

**Frame loop**:

```rust
loop {
    terminal.draw(|frame| {
        // Layout split
        // Render map widget
        // Render status bar
        // Render legend
    })?;

    // Poll for events
    if crossterm::event::poll(Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => handle_key(key, &mut state),
            Event::Resize(w, h) => state.resize(w, h),
            _ => {}
        }
    }
}
```

### Terminal Management

```rust
pub fn run_map(model: &GalaxyModel, session: &SessionState) -> Result<()> {
    // Setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_map_loop(&mut terminal, model, session);

    // Teardown (always runs)
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), Show, LeaveAlternateScreen)?;

    result
}
```

The setup/teardown is in the outer function with `result` captured, so cleanup always runs even if the loop errors.

### Main Loop Integration

`Action::Map` intercepted in `main.rs` before `dispatch()`:

```rust
if matches!(action, commands::Action::Map) {
    if let Err(e) = map::run_map(&model, &session) {
        eprintln!("Map error: {e}");
    }
    continue;
}
```

### State

```rust
pub struct MapState {
    pub zoom: ZoomLevel,
    pub center: (f64, f64),         // viewport center (voxel X, Z)
    pub cursor: (u16, u16),         // grid position (col, row)
    pub grid_size: (u16, u16),      // usable map area
    pub galaxy: u8,
    pub base_labels: Vec<BaseLabel>,
    pub player_pos: Option<(i16, i16)>,
    pub zoom_stack: Vec<(ZoomLevel, f64, f64)>,
    pub show_help: bool,
    pub should_quit: bool,
}
```

### Binning Strategy

- **Galaxy level**: Iterate all systems once, bucket into grid cells by voxel coords. O(n), no spatial queries needed.
- **Region/Local**: Use R-tree `locate_in_envelope_intersecting` with a bounding box for each visible cell, or iterate all systems and filter by viewport bounds.

## Testing Strategy

Pure functions (no terminal):

- **state.rs**: zoom push/pop, cursor movement/clamping, viewport math, center-on-player
- **render.rs** (binning logic): voxel-to-grid mapping, density character selection, base label assignment with known model data

## Verification

1. `make format && make lint && make test`
2. `cargo run -p nms-copilot -- --save data/test/multi_system_save.json` → type `map`
3. Arrow keys navigate, Enter zooms in, Escape zooms out, q exits
4. Terminal restores cleanly after exiting map
5. Resize terminal while map is open — should adapt
