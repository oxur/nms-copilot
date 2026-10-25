//! Ratatui rendering for the interactive galaxy map.
//!
//! [`render`] draws a model/state snapshot into a caller-owned Ratatui frame.
//! It does not poll input or manage the terminal. Coordinates use voxel X/Z.
//! Call [`MapState::resize`] with the complete
//! outer frame or terminal dimensions; that method derives the usable grid.

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use nms_core::address::GalacticAddress;
use nms_graph::GalaxyModel;

use super::state::{MapState, ZoomLevel, density_char};

/// Draw the map, status bar, legend, and optional help overlay into `frame`.
///
/// Systems are selected by `state.galaxy` and projected on voxel X/Z. The
/// outer frame is divided into map, status, and legend areas, then help is
/// painted over them last. If the map area's inner width or height is zero,
/// its map contents are skipped. The grid used to bin systems comes from the
/// frame's map area, while status and cursor calculations use
/// `state.grid_size`; resize state from the complete outer frame or terminal
/// dimensions to keep them synchronized. Rendering does not resize or validate
/// state and uses fixed colors.
///
/// Base labels are the stored A–Z snapshot and are yellow. At every zoom,
/// colliding base cells are spread down adjacent rows; at Local zoom the
/// labels also include names, which are clipped at the map edge on complete
/// grapheme boundaries. A collision reaching the last row can overwrite an
/// earlier base, so separation is not guaranteed. The player `@` is green and
/// can overwrite a base cell. The cursor uses reverse styling; density glyphs
/// are cyan for occupied cells and dark gray for empty cells. The legend
/// reflects stored labels and whether a player position is present.
///
/// # Examples
///
/// This executing in-memory example checks a player cell and the complete
/// legend without opening a terminal.
///
/// ```
/// use nms_copilot::{map::{render::render, state::MapState}, session::SessionState};
/// use nms_graph::GalaxyModel;
/// use ratatui::{backend::TestBackend, style::{Color, Modifier}, Terminal};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let model = GalaxyModel::new();
/// let session = SessionState::from_model(&model);
/// let mut state = MapState::new(&model, &session);
/// state.resize(80, 24);
/// state.player_pos = Some((0, 0));
/// let mut terminal = Terminal::new(TestBackend::new(80, 24))?;
/// terminal.draw(|frame| render(frame, &state, &model))?;
/// let buffer = terminal.backend().buffer();
/// assert_eq!(
///     buffer.cell((40, 10)).map(|cell| (cell.symbol(), cell.fg, cell.modifier.contains(Modifier::BOLD))),
///     Some(("@", Color::Green, true)),
/// );
/// let legend = buffer.content().chunks(80).nth(22).map(|row|
///     row.iter().map(|cell| cell.symbol()).collect::<String>().trim_end().to_owned()
/// );
/// assert_eq!(legend, Some("@=You  ?=Help  q=Quit".to_owned()));
/// # Ok(())
/// # }
/// ```
pub fn render(frame: &mut Frame, state: &MapState, model: &GalaxyModel) {
    render_with_addresses(
        frame,
        state,
        model.systems.values().map(|system| system.address),
    );
}

/// Draw from the owned address snapshot used by the modal map.
pub(super) fn render_addresses(frame: &mut Frame, state: &MapState, addresses: &[GalacticAddress]) {
    render_with_addresses(frame, state, addresses.iter().copied());
}

fn render_with_addresses(
    frame: &mut Frame,
    state: &MapState,
    addresses: impl IntoIterator<Item = GalacticAddress>,
) {
    let area = frame.area();

    // Layout: map area | status bar | legend
    let chunks = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(area);

    render_map(frame, chunks[0], state, addresses);
    render_status(frame, chunks[1], state);
    render_legend(frame, chunks[2], state);

    // Help overlay on top
    if state.show_help {
        render_help(frame, area);
    }
}

/// Render the map grid into the given area.
fn render_map(
    frame: &mut Frame,
    area: Rect,
    state: &MapState,
    addresses: impl IntoIterator<Item = GalacticAddress>,
) {
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
    let grid = bin_systems(state, addresses, inner.width, inner.height);

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
                format!("{} {}", bl.letter, bl.name)
            } else {
                bl.letter.to_string()
            };
            overlays.insert((col, row), (label, Color::Yellow));
        }
    }

    // Player position
    if let Some((px, pz)) = state.player_pos
        && let Some((col, row)) = voxel_to_cell(
            f64::from(px),
            f64::from(pz),
            state,
            inner.width,
            inner.height,
        )
    {
        overlays.insert((col, row), ("@".to_string(), Color::Green));
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
                    let available_cols = usize::from(inner.width - col);
                    skip = Span::raw(label.as_str())
                        .width()
                        .min(available_cols)
                        .saturating_sub(1) as u16;
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
    addresses: impl IntoIterator<Item = GalacticAddress>,
    cols: u16,
    rows: u16,
) -> HashMap<(u16, u16), usize> {
    let mut grid: HashMap<(u16, u16), usize> = HashMap::new();

    // Iterate all systems in the active galaxy
    for address in addresses {
        if address.reality_index != state.galaxy {
            continue;
        }

        let vx = f64::from(address.voxel_x());
        let vz = f64::from(address.voxel_z());

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::state::BaseLabel;
    use crate::session::SessionState;
    use nms_core::{GalacticAddress, System};
    use nms_graph::GalaxyModel;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect};

    fn map_state(model: &GalaxyModel, width: u16, height: u16) -> MapState {
        let session = SessionState::from_model(model);
        let mut state = MapState::new(model, &session);
        state.zoom = ZoomLevel::Local;
        state.galaxy = 9;
        state.galaxy_name = "Eissentam".to_owned();
        state.center = (0.0, 0.0);
        state.cursor = (0, 0);
        state.grid_size = (10, 6);
        state.base_labels.clear();
        state.player_pos = None;
        state.resize(width, height);
        state
    }

    fn insert_system(model: &mut GalaxyModel, x: i16, y: i8, z: i16, ssi: u16, galaxy: u8) {
        model.insert_system(System::new(
            GalacticAddress::new(x, y, z, ssi, 0, galaxy),
            None,
            None,
            None,
            vec![],
        ));
    }

    fn draw(state: &MapState, model: &GalaxyModel, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test backend terminal");
        terminal
            .draw(|frame| render(frame, state, model))
            .expect("in-memory render");
        terminal.backend().buffer().clone()
    }

    fn row(buffer: &Buffer, y: u16, width: u16) -> String {
        row_slice(buffer, y, 0, width)
    }

    fn row_slice(buffer: &Buffer, y: u16, start: u16, width: u16) -> String {
        (start..start + width)
            .map(|x| buffer.cell((x, y)).expect("cell in buffer").symbol())
            .collect()
    }

    fn cell(buffer: &Buffer, x: u16, y: u16) -> &ratatui::buffer::Cell {
        buffer.cell((x, y)).expect("cell in buffer")
    }

    fn base(letter: char, name: &str, x: i16, z: i16) -> BaseLabel {
        BaseLabel {
            letter,
            name: name.to_owned(),
            voxel_x: x,
            voxel_z: z,
        }
    }

    fn local_model() -> GalaxyModel {
        let mut model = GalaxyModel::new();
        model.set_active_galaxy(9);
        model
    }

    #[test]
    fn test_render_model_projection_density_and_bounds_are_literal() {
        let mut model = local_model();
        let mut ssi = 1;
        for (x, y, z, count) in [
            (-16, -7, -16, 1),
            (0, 0, -16, 2),
            (16, 4, -16, 4),
            (-16, 0, 16, 8),
        ] {
            for _ in 0..count {
                insert_system(&mut model, x, y, z, ssi, 9);
                ssi += 1;
            }
        }
        insert_system(&mut model, 0, 0, 0, ssi, 0);
        insert_system(&mut model, 60, 0, 0, ssi + 1, 9);
        model.set_active_galaxy(0);
        assert_eq!(model.systems.len(), 17);

        let state = map_state(&model, 12, 11);
        assert_eq!(model.active_galaxy, 0);
        assert_eq!(state.galaxy, 9);
        assert_eq!(state.grid_size, (10, 6));
        let before = model.systems.clone();
        let bins = bin_systems(
            &state,
            model.systems.values().map(|system| system.address),
            10,
            6,
        );
        assert_eq!(
            bins,
            HashMap::from([((2, 1), 1), ((5, 1), 2), ((7, 1), 4), ((2, 4), 8)])
        );
        let buffer = draw(&state, &model, 12, 11);

        let expected_rows = [
            "          ",
            "  ·  + *  ",
            "          ",
            "          ",
            "  #       ",
            "          ",
        ];
        for (row_index, expected) in expected_rows.iter().enumerate() {
            assert_eq!(row_slice(&buffer, row_index as u16 + 1, 1, 10), *expected);
            for (col, symbol) in expected.chars().enumerate() {
                let cell = cell(&buffer, col as u16 + 1, row_index as u16 + 1);
                assert_eq!(cell.symbol(), symbol.to_string());
                assert_eq!(
                    cell.fg,
                    if symbol == ' ' {
                        Color::DarkGray
                    } else {
                        Color::Cyan
                    }
                );
                assert!(!cell.modifier.contains(Modifier::BOLD));
                assert_eq!(
                    cell.modifier.contains(Modifier::REVERSED),
                    col == 0 && row_index == 0
                );
            }
        }
        assert_eq!(row(&buffer, 0, 12), "┌ Eissentam┐");
        assert_eq!(buffer.cell((0, 4)).unwrap().symbol(), "│");
        assert_eq!(buffer.cell((11, 4)).unwrap().symbol(), "│");
        assert_eq!(model.systems, before);
        assert_eq!(model.systems.len(), 17);

        let bounds = [
            ((0.0, 0.0), Some((5, 3))),
            ((-32.0, -32.0), Some((0, 0))),
            ((32.0, 0.0), None),
            ((0.0, 32.0), None),
            ((-40.0, 0.0), None),
            ((-33.0, 0.0), Some((0, 3))),
        ];
        for ((x, z), expected) in bounds {
            assert_eq!(voxel_to_cell(x, z, &state, 10, 6), expected, "({x}, {z})");
        }
        assert_eq!(voxel_to_cell(0.0, 0.0, &state, 0, 6), None);
        assert_eq!(voxel_to_cell(0.0, 0.0, &state, 10, 0), None);
    }

    #[test]
    fn test_render_local_labels_clip_graphemes_and_preserve_ascii() {
        for (name, expected) in [
            ("ééé", vec!["A", " ", "é", "é", "é"]),
            ("界界", vec!["A", " ", "界", " ", " "]),
            ("😀x", vec!["A", " ", "😀", " ", "x"]),
            ("👩\u{200d}🚀x", vec!["A", " ", "👩\u{200d}🚀", " ", "x"]),
            ("ABCDE", vec!["A", " ", "A", "B", "C"]),
            ("abc", vec!["A", " ", "a", "b", "c"]),
        ] {
            let visible_offsets: &[u16] = match name {
                "界界" => &[0, 1, 2],
                "😀x" | "👩\u{200d}🚀x" => &[0, 1, 2, 4],
                _ => &[0, 1, 2, 3, 4],
            };
            let model = local_model();
            let mut state = map_state(&model, 12, 11);
            state.cursor = (5, 3);
            state.base_labels = vec![base('A', name, 0, 0)];
            let original_names: Vec<_> = state
                .base_labels
                .iter()
                .map(|item| item.name.clone())
                .collect();
            let buffer = draw(&state, &model, 12, 11);
            for (offset, symbol) in expected.iter().enumerate() {
                assert_eq!(
                    cell(&buffer, 6 + offset as u16, 4).symbol(),
                    *symbol,
                    "name={name}"
                );
            }
            for offset in visible_offsets {
                let visible = cell(&buffer, 6 + offset, 4);
                assert_eq!(visible.fg, Color::Yellow, "name={name}, offset={offset}");
                assert!(visible.modifier.contains(Modifier::BOLD));
                assert_eq!(visible.modifier.contains(Modifier::REVERSED), *offset == 0);
            }
            assert_eq!(cell(&buffer, 11, 4).symbol(), "│");
            assert_eq!(
                state
                    .base_labels
                    .iter()
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>(),
                original_names
            );
        }

        let mut model = local_model();
        insert_system(&mut model, 0, 0, 0, 1, 9);
        let mut state = map_state(&model, 12, 11);
        state.cursor = (2, 3);
        state.base_labels = vec![base('A', "é", -16, 0)];
        let buffer = draw(&state, &model, 12, 11);
        assert_eq!(row_slice(&buffer, 4, 3, 4), "A é·");
        let marker = cell(&buffer, 3, 4);
        assert_eq!(marker.symbol(), "A");
        assert_eq!(marker.fg, Color::Yellow);
        assert!(marker.modifier.contains(Modifier::BOLD));
        assert!(marker.modifier.contains(Modifier::REVERSED));
        for x in [4, 5] {
            let name_cell = cell(&buffer, x, 4);
            assert_eq!(name_cell.fg, Color::Yellow);
            assert!(name_cell.modifier.contains(Modifier::BOLD));
            assert!(!name_cell.modifier.contains(Modifier::REVERSED));
        }
        let density = cell(&buffer, 6, 4);
        assert_eq!(density.symbol(), "·");
        assert_eq!(density.fg, Color::Cyan);
        assert!(!density.modifier.contains(Modifier::BOLD));
        assert!(!density.modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_render_overlays_keep_collision_and_player_precedence() {
        let model = local_model();
        let mut state = map_state(&model, 12, 11);
        state.cursor = (5, 3);
        state.base_labels = vec![
            base('A', "First", 0, 0),
            base('B', "Second", 0, 0),
            base('C', "Third", 0, 0),
            base('D', "Fourth", 0, 0),
        ];
        let original = state
            .base_labels
            .iter()
            .map(|item| (item.letter, item.name.clone(), item.voxel_x, item.voxel_z))
            .collect::<Vec<_>>();
        let buffer = draw(&state, &model, 12, 11);
        assert_eq!(row_slice(&buffer, 4, 6, 5), "A Fir");
        assert_eq!(row_slice(&buffer, 5, 6, 5), "B Sec");
        assert_eq!(row_slice(&buffer, 6, 6, 5), "D Fou");
        for (y, reversed) in [(4, true), (5, false), (6, false)] {
            for x in 6..11 {
                let visible = cell(&buffer, x, y);
                assert_eq!(visible.fg, Color::Yellow);
                assert!(visible.modifier.contains(Modifier::BOLD));
                assert_eq!(
                    visible.modifier.contains(Modifier::REVERSED),
                    reversed && x == 6
                );
            }
        }
        assert_eq!(
            state
                .base_labels
                .iter()
                .map(|item| (item.letter, item.name.clone(), item.voxel_x, item.voxel_z))
                .collect::<Vec<_>>(),
            original
        );

        state.player_pos = Some((0, 0));
        let buffer = draw(&state, &model, 12, 11);
        assert_eq!(row_slice(&buffer, 4, 6, 5), "@    ");
        let player = cell(&buffer, 6, 4);
        assert_eq!(player.fg, Color::Green);
        assert!(player.modifier.contains(Modifier::BOLD));
        assert!(player.modifier.contains(Modifier::REVERSED));
        assert_eq!(row_slice(&buffer, 5, 6, 5), "B Sec");
        assert_eq!(row_slice(&buffer, 6, 6, 5), "D Fou");
        assert_eq!(
            state
                .base_labels
                .iter()
                .map(|item| (item.letter, item.name.clone(), item.voxel_x, item.voxel_z))
                .collect::<Vec<_>>(),
            original
        );

        for zoom in [ZoomLevel::Galaxy, ZoomLevel::Region] {
            let mut state = map_state(&model, 12, 11);
            state.zoom = zoom;
            state.cursor = (5, 3);
            state.base_labels = vec![base('A', "界 Long name", 0, 0)];
            let buffer = draw(&state, &model, 12, 11);
            assert_eq!(row_slice(&buffer, 4, 6, 5), "A    ");
            for x in 6..11 {
                let visible = cell(&buffer, x, 4);
                if x == 6 {
                    assert_eq!(visible.fg, Color::Yellow);
                    assert!(visible.modifier.contains(Modifier::BOLD));
                    assert!(visible.modifier.contains(Modifier::REVERSED));
                } else {
                    assert_eq!(visible.fg, Color::DarkGray);
                    assert!(!visible.modifier.contains(Modifier::BOLD));
                    assert!(!visible.modifier.contains(Modifier::REVERSED));
                }
            }
        }
    }

    #[test]
    fn test_render_status_legend_and_help_are_complete_and_repeatable() {
        let model = local_model();
        let mut state = map_state(&model, 80, 24);
        state.cursor = (39, 9);
        state.base_labels = vec![base('A', "Alpha", 0, 0), base('B', "Beta", 16, 0)];
        state.player_pos = Some((0, 0));
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test backend terminal");
        terminal
            .draw(|frame| render(frame, &state, &model))
            .unwrap();
        let normal = terminal.backend().buffer().clone();
        assert_eq!(
            row(&normal, 21, 80),
            " [Eissentam]  [Local]  Cursor: X=+0 Z=-2  Scale: 1 vox/cell                     "
        );
        assert_eq!(
            row(&normal, 22, 80),
            "A=Alpha  B=Beta  @=You  ?=Help  q=Quit                                          "
        );
        for x in 0..80 {
            assert_eq!(cell(&normal, x, 21).bg, Color::DarkGray);
        }
        for x in 0..59 {
            assert_eq!(cell(&normal, x, 21).fg, Color::White);
        }
        for x in 0..7 {
            assert_eq!(cell(&normal, x, 22).fg, Color::Yellow);
        }
        for x in 9..15 {
            assert_eq!(cell(&normal, x, 22).fg, Color::Yellow);
        }
        for x in 17..22 {
            assert_eq!(cell(&normal, x, 22).fg, Color::Green);
        }
        for x in 24..38 {
            assert_eq!(cell(&normal, x, 22).fg, Color::DarkGray);
        }

        state.show_help = true;
        terminal
            .draw(|frame| render(frame, &state, &model))
            .unwrap();
        let help = terminal.backend().buffer();
        let expected = [
            "┌────────────────────────────────────────────────────────────────────┐",
            "│ Galaxy Map                                                         │",
            "│                                                                    │",
            "│  Keys:                                 Density:                    │",
            "│  Arrow keys    Move cursor             ·  1 system                 │",
            "│  Enter / +     Zoom in                 +  2-3 systems              │",
            "│  Esc / -       Zoom out (exit)         *  4-7 systems              │",
            "│  c             Center on player        #  8+ systems               │",
            "│  ?             Toggle this help        @  Your position            │",
            "│  q / Ctrl+C    Exit map                A-Z  Base locations         │",
            "│                                                                    │",
            "│                                        Zoom Levels:                │",
            "│                                        Galaxy  4096×4096 vox       │",
            "│                                        Region   512×512  vox  8×   │",
            "│                                        Local     64×64   vox 64×   │",
            "│                                                                    │",
            "│  Press any key to close                                            │",
            "└────────────────────────────────────────────────────────────────────┘",
        ];
        for (offset, expected_row) in expected.iter().enumerate() {
            assert_eq!(
                row_slice(help, offset as u16 + 3, 5, 70),
                *expected_row,
                "help row {offset}"
            );
        }
        for y in 3..21 {
            for x in 5..75 {
                assert_eq!(
                    cell(help, x, y).bg,
                    Color::Black,
                    "help background at ({x}, {y})"
                );
            }
        }
        for (x, y, symbol) in [(7, 4, "G"), (8, 6, "K"), (46, 6, "D"), (46, 14, "Z")] {
            let header = cell(help, x, y);
            assert_eq!(header.symbol(), symbol, "help header at ({x}, {y})");
            assert_eq!(
                header.fg,
                Color::White,
                "help header foreground at ({x}, {y})"
            );
            assert!(
                header.modifier.contains(Modifier::BOLD),
                "help header weight at ({x}, {y})"
            );
        }
        assert_eq!(cell(help, 6, 5).bg, Color::Black);
        assert!(cell(help, 8, 6).modifier.contains(Modifier::BOLD));
        assert_eq!(cell(help, 46, 7).fg, Color::Cyan);
        assert_eq!(cell(help, 46, 11).fg, Color::Green);
        assert_eq!(cell(help, 46, 12).fg, Color::Yellow);
        assert_eq!(cell(help, 48, 12).fg, Color::Yellow);
        assert_eq!(cell(help, 8, 19).fg, Color::DarkGray);

        state.show_help = false;
        terminal
            .draw(|frame| render(frame, &state, &model))
            .unwrap();
        assert_eq!(terminal.backend().buffer(), &normal);
        let restored = terminal.backend().buffer();
        assert_eq!(
            row(restored, 21, 80),
            " [Eissentam]  [Local]  Cursor: X=+0 Z=-2  Scale: 1 vox/cell                     "
        );
        assert_eq!(
            row(restored, 22, 80),
            "A=Alpha  B=Beta  @=You  ?=Help  q=Quit                                          "
        );
    }

    #[test]
    fn test_render_map_zero_inner_axes_keep_only_borders() {
        let mut model = local_model();
        insert_system(&mut model, 0, 0, 0, 1, 9);
        let mut state = map_state(&model, 12, 11);
        state.base_labels = vec![base('A', "Origin", 0, 0)];
        state.player_pos = Some((0, 0));
        state.cursor = (0, 0);

        for (width, height, expected) in [
            (2, 8, vec!["┌┐", "││", "││", "││", "││", "││", "││", "└┘"]),
            (8, 2, vec!["┌ Eisse┐", "└──────┘"]),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    render_map(
                        frame,
                        area,
                        &state,
                        model.systems.values().map(|system| system.address),
                    );
                })
                .unwrap();
            let buffer = terminal.backend().buffer();
            assert_eq!(buffer.area, Rect::new(0, 0, width, height));
            for (y, expected_row) in expected.iter().enumerate() {
                assert_eq!(row(buffer, y as u16, width), *expected_row);
            }
            for y in 0..height {
                for x in 0..width {
                    let symbol = cell(buffer, x, y).symbol();
                    assert!(
                        !["·", "+", "*", "#", "@", "A"].contains(&symbol),
                        "unexpected content {symbol:?} at {x},{y}"
                    );
                }
            }
        }
    }
}
