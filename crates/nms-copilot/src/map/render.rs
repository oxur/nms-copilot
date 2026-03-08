//! Ratatui rendering for the interactive galaxy map.

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use nms_graph::GalaxyModel;

use super::state::{MapState, density_char};

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

    // Overlay base labels and player position
    let mut overlays: HashMap<(u16, u16), (char, Color)> = HashMap::new();

    // Bases
    for bl in &state.base_labels {
        if let Some((col, row)) = voxel_to_cell(
            f64::from(bl.voxel_x),
            f64::from(bl.voxel_z),
            state,
            inner.width,
            inner.height,
        ) {
            overlays.insert((col, row), (bl.letter, Color::Yellow));
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
            overlays.insert((col, row), ('@', Color::Green));
        }
    }

    // Render each row as a Line of Spans
    let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
    for row in 0..inner.height {
        let mut spans: Vec<Span> = Vec::with_capacity(inner.width as usize);
        for col in 0..inner.width {
            let is_cursor = col == state.cursor.0 && row == state.cursor.1;

            let (ch, base_style) = if let Some(&(overlay_ch, color)) = overlays.get(&(col, row)) {
                (
                    overlay_ch,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )
            } else {
                let count = grid.get(&(col, row)).copied().unwrap_or(0);
                let ch = density_char(count);
                let style = if count > 0 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                (ch, style)
            };

            let style = if is_cursor {
                base_style.add_modifier(Modifier::REVERSED)
            } else {
                base_style
            };

            spans.push(Span::styled(ch.to_string(), style));
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
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            " Galaxy Map — Key Bindings ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Arrow keys    Move cursor"),
        Line::from("  Enter / +     Zoom in"),
        Line::from("  Esc / -       Zoom out (exit at galaxy level)"),
        Line::from("  c             Center on player"),
        Line::from("  ?             Toggle this help"),
        Line::from("  q / Ctrl+C    Exit map"),
        Line::from(""),
        Line::from(Span::styled(
            "  Symbols: · =1  + =2-3  * =4-7  # =8+  @=You",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("  Press any key to close"),
    ];

    let help_height = help_text.len() as u16 + 2; // +2 for borders
    let help_width = 50;

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

    // Clear the area behind the help
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        help_area,
    );
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
