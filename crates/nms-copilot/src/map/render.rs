//! Ratatui rendering for the interactive galaxy map.

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use nms_graph::GalaxyModel;

use super::state::{MapState, ZoomLevel, density_char};

/// Bin systems into grid cells and render the full map frame.
pub fn render(frame: &mut Frame, state: &MapState, model: &GalaxyModel) {
    let area = frame.area();

    // Layout: map area | status bar | legend
    let chunks = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_map(frame, chunks[0], state, model);
    render_status(frame, chunks[1], state);
    render_legend(frame, chunks[2], state);

    // Help overlay on top
    if state.show_help {
        render_help(frame, area);
    }
}

/// Render the map grid into the given area.
fn render_map(frame: &mut Frame, area: Rect, state: &MapState, model: &GalaxyModel) {
    let block = Block::default().borders(Borders::ALL).title(format!(
        " {} — {} ",
        state.galaxy_name,
        state.zoom.label()
    ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Build a density grid by binning systems into cells
    let grid = bin_systems(state, model, inner.width, inner.height);

    // Overlay base labels and player position.
    // Each overlay entry maps (col, row) -> (label_string, color).
    // At Local zoom, base labels include the name; otherwise just the letter.
    // When multiple bases share the same cell, spread them to adjacent rows.
    let mut overlays: HashMap<(u16, u16), (String, Color)> = HashMap::new();

    // Bases
    let show_names = state.zoom == ZoomLevel::Local;
    for bl in &state.base_labels {
        if let Some((col, mut row)) = voxel_to_cell(
            f64::from(bl.voxel_x),
            f64::from(bl.voxel_z),
            state,
            inner.width,
            inner.height,
        ) {
            // Spread stacked bases to adjacent rows
            while overlays.contains_key(&(col, row)) && row + 1 < inner.height {
                row += 1;
            }
            let label = if show_names {
                let max_len = (inner.width - col) as usize;
                let full = format!("{} {}", bl.letter, bl.name);
                if full.len() > max_len {
                    full[..max_len].to_string()
                } else {
                    full
                }
            } else {
                bl.letter.to_string()
            };
            overlays.insert((col, row), (label, Color::Yellow));
        }
    }

    // Player position
    if let Some((px, pz)) = state.player_pos {
        if let Some((col, row)) = voxel_to_cell(
            f64::from(px),
            f64::from(pz),
            state,
            inner.width,
            inner.height,
        ) {
            overlays.insert((col, row), ("@".to_string(), Color::Green));
        }
    }

    // Render each row as a Line of Spans
    let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
    for row in 0..inner.height {
        let mut spans: Vec<Span> = Vec::with_capacity(inner.width as usize);
        let mut skip: u16 = 0; // columns to skip (consumed by multi-char label)
        for col in 0..inner.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            let is_cursor = col == state.cursor.0 && row == state.cursor.1;

            if let Some((label, color)) = overlays.get(&(col, row)) {
                let style = Style::default().fg(*color).add_modifier(Modifier::BOLD);
                if label.len() > 1 {
                    // Multi-char label: first char may get cursor highlight
                    let mut chars = label.chars();
                    let first = chars.next().unwrap();
                    let first_style = if is_cursor {
                        style.add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };
                    spans.push(Span::styled(first.to_string(), first_style));
                    let rest: String = chars.collect();
                    skip = rest.len() as u16;
                    spans.push(Span::styled(rest, style));
                } else {
                    let style = if is_cursor {
                        style.add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };
                    spans.push(Span::styled(label.clone(), style));
                }
            } else {
                let count = grid.get(&(col, row)).copied().unwrap_or(0);
                let ch = density_char(count);
                let base_style = if count > 0 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let style = if is_cursor {
                    base_style.add_modifier(Modifier::REVERSED)
                } else {
                    base_style
                };
                spans.push(Span::styled(ch.to_string(), style));
            };
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Render the status bar.
fn render_status(frame: &mut Frame, area: Rect, state: &MapState) {
    let (vx, vz) = state.cursor_voxel();
    let scale = state.zoom.extent() / f64::from(state.grid_size.0.max(1));

    let status = format!(
        " [{galaxy}]  [{zoom}]  Cursor: X={vx:+.0} Z={vz:+.0}  Scale: {scale:.0} vox/cell",
        galaxy = state.galaxy_name,
        zoom = state.zoom.label(),
    );

    let status_line = Paragraph::new(Line::from(vec![Span::styled(
        status,
        Style::default().fg(Color::White),
    )]))
    .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status_line, area);
}

/// Render the legend showing base labels and key hints.
fn render_legend(frame: &mut Frame, area: Rect, state: &MapState) {
    let mut parts: Vec<Span> = Vec::new();

    // Base labels
    for bl in &state.base_labels {
        if !parts.is_empty() {
            parts.push(Span::raw("  "));
        }
        parts.push(Span::styled(
            format!("{}={}", bl.letter, bl.name),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Player marker
    if state.player_pos.is_some() {
        if !parts.is_empty() {
            parts.push(Span::raw("  "));
        }
        parts.push(Span::styled("@=You", Style::default().fg(Color::Green)));
    }

    // Key hints
    if !parts.is_empty() {
        parts.push(Span::raw("  "));
    }
    parts.push(Span::styled(
        "?=Help  q=Quit",
        Style::default().fg(Color::DarkGray),
    ));

    let legend = Paragraph::new(Line::from(parts)).wrap(Wrap { trim: false });
    frame.render_widget(legend, area);
}

/// Render a help overlay centered on the screen.
///
/// Two-column layout: keybindings on the left, density symbols and
/// zoom levels on the right.
fn render_help(frame: &mut Frame, area: Rect) {
    let bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let cyan = Style::default().fg(Color::Cyan);
    let green = Style::default().fg(Color::Green);
    let dim = Style::default().fg(Color::DarkGray);
    let normal = Style::default().fg(Color::White);

    // Column widths: left 36, gap 4, right 28 = 68 total content + 2 border = 70
    let left_w = 36;
    let gap = 4;

    /// Pad or truncate a string to exactly `width` characters.
    fn pad(s: &str, width: usize) -> String {
        if s.len() >= width {
            s[..width].to_string()
        } else {
            format!("{s:<width$}")
        }
    }

    // Build rows as (left_text, right_spans)
    let help_text = vec![
        // Title row
        Line::from(Span::styled(" Galaxy Map ", bold)),
        // Blank
        Line::from(""),
        // Row: keys header | symbols header
        Line::from(vec![
            Span::styled(pad("  Keys:", left_w), bold),
            Span::raw(pad("", gap)),
            Span::styled("Density:", bold),
        ]),
        // Row: arrow keys | · = 1
        Line::from(vec![
            Span::styled(pad("  Arrow keys    Move cursor", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("·", cyan),
            Span::styled("  1 system", normal),
        ]),
        // Row: enter | + = 2-3
        Line::from(vec![
            Span::styled(pad("  Enter / +     Zoom in", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("+", cyan),
            Span::styled("  2-3 systems", normal),
        ]),
        // Row: esc | * = 4-7
        Line::from(vec![
            Span::styled(pad("  Esc / -       Zoom out (exit)", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("*", cyan),
            Span::styled("  4-7 systems", normal),
        ]),
        // Row: c | # = 8+
        Line::from(vec![
            Span::styled(pad("  c             Center on player", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("#", cyan),
            Span::styled("  8+ systems", normal),
        ]),
        // Row: ? | blank
        Line::from(vec![
            Span::styled(pad("  ?             Toggle this help", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("@", green),
            Span::styled("  Your position", normal),
        ]),
        // Row: q | markers header
        Line::from(vec![
            Span::styled(pad("  q / Ctrl+C    Exit map", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("A", Style::default().fg(Color::Yellow)),
            Span::styled("-", normal),
            Span::styled("Z", Style::default().fg(Color::Yellow)),
            Span::styled("  Base locations", normal),
        ]),
        // Blank
        Line::from(""),
        // Zoom levels header
        Line::from(vec![
            Span::styled(pad("", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("Zoom Levels:", bold),
        ]),
        // Galaxy
        Line::from(vec![
            Span::styled(pad("", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("Galaxy  4096\u{00D7}4096 vox", normal),
        ]),
        // Region
        Line::from(vec![
            Span::styled(pad("", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("Region   512\u{00D7}512  vox  8\u{00D7}", normal),
        ]),
        // Local
        Line::from(vec![
            Span::styled(pad("", left_w), normal),
            Span::raw(pad("", gap)),
            Span::styled("Local     64\u{00D7}64   vox 64\u{00D7}", normal),
        ]),
        // Blank
        Line::from(""),
        // Dismiss
        Line::from(Span::styled("  Press any key to close", dim)),
    ];

    let help_height = help_text.len() as u16 + 2; // +2 for borders
    let help_width = 70;

    let x = area.x + area.width.saturating_sub(help_width) / 2;
    let y = area.y + area.height.saturating_sub(help_height) / 2;
    let help_area = Rect::new(
        x,
        y,
        help_width.min(area.width),
        help_height.min(area.height),
    );

    let help_block = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black));

    // Clear the area behind the help (resets all cells to empty)
    frame.render_widget(Clear, help_area);
    frame.render_widget(help_block, help_area);
}

/// Bin systems from the model into grid cells.
///
/// Returns a map from (col, row) -> system count.
fn bin_systems(
    state: &MapState,
    model: &GalaxyModel,
    cols: u16,
    rows: u16,
) -> HashMap<(u16, u16), usize> {
    let mut grid: HashMap<(u16, u16), usize> = HashMap::new();

    // Iterate all systems in the active galaxy
    for sys in model.systems.values() {
        if sys.address.reality_index != state.galaxy {
            continue;
        }

        let vx = f64::from(sys.address.voxel_x());
        let vz = f64::from(sys.address.voxel_z());

        if let Some((col, row)) = voxel_to_cell(vx, vz, state, cols, rows) {
            *grid.entry((col, row)).or_insert(0) += 1;
        }
    }

    grid
}

/// Convert voxel coordinates to grid cell position.
fn voxel_to_cell(vx: f64, vz: f64, state: &MapState, cols: u16, rows: u16) -> Option<(u16, u16)> {
    let extent = state.zoom.extent();
    let cell_size_x = extent / f64::from(cols.max(1));
    let cell_size_z = extent / f64::from(rows.max(1));

    let half_cols = f64::from(cols) / 2.0;
    let half_rows = f64::from(rows) / 2.0;

    let col = ((vx - state.center.0) / cell_size_x + half_cols) as i32;
    let row = ((vz - state.center.1) / cell_size_z + half_rows) as i32;

    if col >= 0 && col < cols as i32 && row >= 0 && row < rows as i32 {
        Some((col as u16, row as u16))
    } else {
        None
    }
}
