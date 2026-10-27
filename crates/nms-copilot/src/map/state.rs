//! Map state: zoom levels, viewport math, cursor logic.
//!
//! [`MapState`] stores a mutable view of a model/session snapshot and does
//! not enforce cross-field invariants or refresh from either source. Map
//! geometry projects voxel X/Z and ignores Y. [`ZoomLevel`] defines the
//! viewport scales; [`density_char`] maps cell counts to display glyphs.

use std::collections::HashMap;

use nms_core::galaxy::Galaxy;
use nms_graph::GalaxyModel;

use crate::session::SessionState;

/// One of the map's three X/Z viewport scales.
///
/// The per-axis extents are 4096 for Galaxy, 512 for Region, and 64 for Local.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomLevel {
    /// Full galaxy view — 4096×4096 voxel extent.
    Galaxy,
    /// Region view — 512×512 voxel extent.
    Region,
    /// Local view — 64×64 voxel extent.
    Local,
}

impl ZoomLevel {
    /// Voxel extent covered by one axis at this zoom level.
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_copilot::map::state::ZoomLevel;
    /// let levels = [ZoomLevel::Galaxy, ZoomLevel::Region, ZoomLevel::Local];
    /// let actual: Vec<_> = levels.into_iter().map(|z| (z.extent(), z.label(), z.zoom_in(), z.zoom_out())).collect();
    /// assert_eq!(actual, vec![
    ///     (4096.0, "Galaxy", Some(ZoomLevel::Region), None),
    ///     (512.0, "Region", Some(ZoomLevel::Local), Some(ZoomLevel::Galaxy)),
    ///     (64.0, "Local", None, Some(ZoomLevel::Region)),
    /// ]);
    /// ```
    pub fn extent(self) -> f64 {
        match self {
            Self::Galaxy => 4096.0,
            Self::Region => 512.0,
            Self::Local => 64.0,
        }
    }

    /// Zoom in one level, if possible.
    pub fn zoom_in(self) -> Option<Self> {
        match self {
            Self::Galaxy => Some(Self::Region),
            Self::Region => Some(Self::Local),
            Self::Local => None,
        }
    }

    /// Zoom out one level, if possible.
    pub fn zoom_out(self) -> Option<Self> {
        match self {
            Self::Galaxy => None,
            Self::Region => Some(Self::Galaxy),
            Self::Local => Some(Self::Region),
        }
    }

    /// Display name for the status bar.
    pub fn label(self) -> &'static str {
        match self {
            Self::Galaxy => "Galaxy",
            Self::Region => "Region",
            Self::Local => "Local",
        }
    }
}

/// Snapshot label and X/Z location for one map base.
///
/// [`MapState::new`] sorts selected-galaxy bases by case-sensitive name and
/// labels at most 26. This value owns copied data and does not track changes.
#[derive(Debug, Clone)]
pub struct BaseLabel {
    /// Display letter assigned from `A` through `Z`.
    pub letter: char,
    /// Copied base name used by the Local view and legend.
    pub name: String,
    /// Base voxel X coordinate in the selected galaxy.
    pub voxel_x: i16,
    /// Base voxel Z coordinate in the selected galaxy.
    pub voxel_z: i16,
}

/// A marker that remains visible after overlay collisions are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OverlayMarker {
    Base(usize),
    Player,
}

/// Mutable viewport and display snapshot for the interactive map.
///
/// Fields are public and independently mutable; direct changes can create
/// combinations methods do not produce. Galaxy, labels, and player position
/// are copied on construction, not live model views. Geometry uses voxel X/Z.
#[derive(Debug)]
pub struct MapState {
    /// Current X/Z viewport scale.
    pub zoom: ZoomLevel,
    /// Viewport center as `(voxel_x, voxel_z)`.
    pub center: (f64, f64),
    /// Cursor grid cell as `(column, row)`.
    pub cursor: (u16, u16),
    /// Usable map dimensions as `(columns, rows)`.
    pub grid_size: (u16, u16),
    /// Model active-galaxy index captured at construction.
    pub galaxy: u8,
    /// Display name for the captured model galaxy.
    pub galaxy_name: String,
    /// Up to 26 captured base labels for the model galaxy, ordered by name.
    pub base_labels: Vec<BaseLabel>,
    /// Captured player `(voxel_x, voxel_z)`, if known.
    pub player_pos: Option<(i16, i16)>,
    /// Saved `(zoom, center_x, center_z)` entries used by zoom-out.
    pub zoom_stack: Vec<(ZoomLevel, f64, f64)>,
    /// Whether rendering should paint the help overlay.
    pub show_help: bool,
    /// Whether the loop should exit after input handling.
    pub should_quit: bool,
}

impl MapState {
    /// Capture initial map state from the model and session.
    ///
    /// The model's active galaxy selects the display name and base labels;
    /// `session.galaxy` does not. Player X/Z is copied without a galaxy
    /// comparison. A session position, when present, supplies the center even
    /// if its address names another galaxy; otherwise player position or the
    /// origin is used. Names sort case-sensitively, and only the first 26
    /// selected-galaxy bases receive `A`–`Z`. Equal-name order is unspecified
    /// because source order comes from a hash map. Values are snapshots.
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_copilot::{map::state::{MapState, ZoomLevel}, session::{PositionContext, SessionState}};
    /// use nms_core::{BaseType, GalacticAddress, PlayerBase, PlayerState};
    /// use nms_graph::GalaxyModel;
    /// # fn main() {
    /// let mut model = GalaxyModel::new();
    /// model.set_active_galaxy(9);
    /// model.player_state = Some(PlayerState::new(GalacticAddress::new(100, 0, 200, 1, 0, 9), 9, None, None, 0, 0, 0));
    /// for (name, x, z, galaxy) in [("Beta", 30, 40, 9), ("Alpha", 10, 20, 9), ("Other", 50, 60, 0)] {
    ///     model.insert_base(PlayerBase::new(name.into(), BaseType::HomePlanetBase, GalacticAddress::new(x, 0, z, 1, 0, galaxy), [0.0; 3], None));
    /// }
    /// let mut session = SessionState::from_model(&model);
    /// session.galaxy = nms_core::galaxy::Galaxy::by_index(0);
    /// session.position = Some(PositionContext::Address(GalacticAddress::new(16, 0, -8, 1, 0, 0)));
    /// let state = MapState::new(&model, &session);
    /// assert_eq!((state.zoom, state.center, state.cursor, state.grid_size, state.galaxy, state.galaxy_name.as_str(), state.player_pos, state.zoom_stack.clone(), state.show_help, state.should_quit),
    ///     (ZoomLevel::Galaxy, (16.0, -8.0), (0, 0), (78, 19), 9, "Eissentam", Some((100, 200)), vec![], false, false));
    /// let labels: Vec<_> = state.base_labels.iter().map(|b| (b.letter, b.name.as_str(), b.voxel_x, b.voxel_z)).collect();
    /// assert_eq!(labels, vec![('A', "Alpha", 10, 20), ('B', "Beta", 30, 40)]);
    /// # }
    /// ```
    pub fn new(model: &GalaxyModel, session: &SessionState) -> Self {
        let galaxy = model.active_galaxy;
        let galaxy_name = Galaxy::by_index(galaxy).name.to_string();

        let player_pos = model.player_position().map(|a| (a.voxel_x(), a.voxel_z()));

        // Center on player or origin
        let center = player_pos
            .map(|(x, z)| (f64::from(x), f64::from(z)))
            .unwrap_or((0.0, 0.0));

        // Build base labels (A-Z), sorted by name for stable letter assignment
        let mut bases_in_galaxy: Vec<_> = model
            .bases
            .values()
            .filter(|b| b.address.reality_index == galaxy)
            .collect();
        bases_in_galaxy.sort_by(|a, b| a.name.cmp(&b.name));
        let base_labels: Vec<BaseLabel> = bases_in_galaxy
            .into_iter()
            .take(26)
            .enumerate()
            .map(|(i, b)| BaseLabel {
                letter: (b'A' + i as u8) as char,
                name: b.name.clone(),
                voxel_x: b.address.voxel_x(),
                voxel_z: b.address.voxel_z(),
            })
            .collect();

        // Use session position if available, otherwise player position
        let effective_center = session
            .position
            .as_ref()
            .map(|p| {
                let a = p.address();
                (f64::from(a.voxel_x()), f64::from(a.voxel_z()))
            })
            .unwrap_or(center);

        Self {
            zoom: ZoomLevel::Galaxy,
            center: effective_center,
            cursor: (0, 0),
            grid_size: (78, 19), // 80-2 cols, 24-3-2 rows (borders)
            galaxy,
            galaxy_name,
            base_labels,
            player_pos,
            zoom_stack: Vec::new(),
            show_help: false,
            should_quit: false,
        }
    }

    /// Set the usable grid from complete outer frame or terminal dimensions.
    ///
    /// This method itself reserves two columns and five rows for borders and
    /// status/legend. Dimensions saturate at zero; a zero-sized axis leaves
    /// its previous cursor coordinate unchanged. Callers should not subtract
    /// those reservations before calling this method.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        // Reserve 3 rows for status + legend, and 2 cols + 2 rows for map border
        let inner_cols = cols.saturating_sub(2);
        let inner_rows = rows.saturating_sub(3).saturating_sub(2);
        self.grid_size = (inner_cols, inner_rows);
        self.clamp_cursor();
    }

    /// Resolve displayed base and player markers at the supplied inner size.
    ///
    /// Base labels retain their stored order, spread down rows on collisions,
    /// and preserve the renderer's final-row replacement behavior. The player
    /// marker is placed last and can replace a base at the same cell.
    pub(super) fn visible_markers(
        &self,
        cols: u16,
        rows: u16,
    ) -> HashMap<(u16, u16), OverlayMarker> {
        let mut markers = HashMap::new();

        for (index, base) in self.base_labels.iter().enumerate() {
            if let Some((col, mut row)) = self.voxel_to_grid_for_size(
                f64::from(base.voxel_x),
                f64::from(base.voxel_z),
                cols,
                rows,
            ) {
                while markers.contains_key(&(col, row)) && row + 1 < rows {
                    row += 1;
                }
                markers.insert((col, row), OverlayMarker::Base(index));
            }
        }

        if let Some((player_x, player_z)) = self.player_pos
            && let Some(cell) =
                self.voxel_to_grid_for_size(f64::from(player_x), f64::from(player_z), cols, rows)
        {
            markers.insert(cell, OverlayMarker::Player);
        }

        markers
    }

    /// Move by signed `(dx, dy)` and clamp the cursor to nonempty grid axes.
    ///
    /// The stored `u16` coordinates are cast to `i16` for addition; this is
    /// intended for ordinary terminal dimensions and small input deltas.
    ///
    /// # Panics
    ///
    /// Intermediate `i16` addition can overflow with overflow checks enabled.
    /// Wrapping builds may differ for extreme public cursor values or deltas.
    pub fn move_cursor(&mut self, dx: i16, dy: i16) {
        let (cx, cy) = self.cursor;
        let new_x = (cx as i16 + dx).max(0) as u16;
        let new_y = (cy as i16 + dy).max(0) as u16;
        self.cursor = (new_x, new_y);
        self.clamp_cursor();
    }

    /// Clamp cursor to grid bounds.
    fn clamp_cursor(&mut self) {
        let (cols, rows) = self.grid_size;
        if cols > 0 {
            self.cursor.0 = self.cursor.0.min(cols.saturating_sub(1));
        }
        if rows > 0 {
            self.cursor.1 = self.cursor.1.min(rows.saturating_sub(1));
        }
    }

    /// Zoom in one tier around the voxel under the cursor, when possible.
    ///
    /// Saves the prior zoom and center (not cursor), centers on the visible
    /// marker under the cursor or the cursor voxel when no marker is present,
    /// and places the cursor at the grid midpoint. At Local zoom the state is
    /// unchanged. Public stack edits are not validated.
    pub fn zoom_in(&mut self) {
        if let Some(next_zoom) = self.zoom.zoom_in() {
            let center = self
                .visible_markers(self.grid_size.0, self.grid_size.1)
                .get(&self.cursor)
                .copied()
                .and_then(|marker| match marker {
                    OverlayMarker::Base(index) => {
                        let base = &self.base_labels[index];
                        Some((f64::from(base.voxel_x), f64::from(base.voxel_z)))
                    }
                    OverlayMarker::Player => self
                        .player_pos
                        .map(|(player_x, player_z)| (f64::from(player_x), f64::from(player_z))),
                })
                .unwrap_or_else(|| self.cursor_voxel());

            // Save current state for zoom-out
            self.zoom_stack
                .push((self.zoom, self.center.0, self.center.1));
            self.center = center;
            self.zoom = next_zoom;
            // Reset cursor to center of grid
            self.cursor = (self.grid_size.0 / 2, self.grid_size.1 / 2);
        }
    }

    /// Restore the saved zoom and center at the top of the stack, if present.
    ///
    /// Returns `true` after restoring and centering the cursor. An empty stack
    /// returns `false` without changing `should_quit`; the input layer applies
    /// the exit policy.
    pub fn zoom_out(&mut self) -> bool {
        if let Some((prev_zoom, cx, cz)) = self.zoom_stack.pop() {
            self.zoom = prev_zoom;
            self.center = (cx, cz);
            self.cursor = (self.grid_size.0 / 2, self.grid_size.1 / 2);
            true
        } else {
            // At galaxy level — signal quit
            false
        }
    }

    /// Center on the captured player position when known.
    ///
    /// Changes center and cursor midpoint while preserving zoom and stack; no
    /// player position leaves state unchanged.
    pub fn center_on_player(&mut self) {
        if let Some((px, pz)) = self.player_pos {
            self.center = (f64::from(px), f64::from(pz));
            self.cursor = (self.grid_size.0 / 2, self.grid_size.1 / 2);
        }
    }

    /// Return the voxel `(x, z)` coordinate under the cursor.
    ///
    /// Each cell size is the zoom extent divided by its grid dimension,
    /// treating zero dimensions as one. For a positive dimension, the origin
    /// is center minus half the viewport extent on that axis. When a dimension
    /// is zero, its divisor is one and its half-dimension is zero; a retained
    /// nonzero cursor coordinate can still offset the result on that axis.
    pub fn cursor_voxel(&self) -> (f64, f64) {
        let (cols, rows) = self.grid_size;
        let extent = self.zoom.extent();
        let cell_size_x = extent / f64::from(cols.max(1));
        let cell_size_z = extent / f64::from(rows.max(1));

        let half_cols = f64::from(cols) / 2.0;
        let half_rows = f64::from(rows) / 2.0;

        let vx = self.center.0 + (f64::from(self.cursor.0) - half_cols) * cell_size_x;
        let vz = self.center.1 + (f64::from(self.cursor.1) - half_rows) * cell_size_z;
        (vx, vz)
    }

    /// Convert voxel `(x, z)` coordinates to a grid `(column, row)`.
    ///
    /// Returns `None` when the resulting integer cell is outside the grid.
    /// Float-to-integer casts truncate toward zero, so this is not strict
    /// continuous-rectangle containment: a slightly negative fractional cell
    /// can map to zero. Nonfinite and extreme inputs follow Rust cast rules.
    /// A zero grid dimension yields no in-grid cell.
    pub fn voxel_to_grid(&self, vx: f64, vz: f64) -> Option<(u16, u16)> {
        self.voxel_to_grid_for_size(vx, vz, self.grid_size.0, self.grid_size.1)
    }

    /// Project voxel coordinates using a caller-supplied inner grid size.
    pub(super) fn voxel_to_grid_for_size(
        &self,
        vx: f64,
        vz: f64,
        cols: u16,
        rows: u16,
    ) -> Option<(u16, u16)> {
        let extent = self.zoom.extent();
        let cell_size_x = extent / f64::from(cols.max(1));
        let cell_size_z = extent / f64::from(rows.max(1));

        let half_cols = f64::from(cols) / 2.0;
        let half_rows = f64::from(rows) / 2.0;

        let col = ((vx - self.center.0) / cell_size_x + half_cols) as i32;
        let row = ((vz - self.center.1) / cell_size_z + half_rows) as i32;

        if col >= 0 && col < cols as i32 && row >= 0 && row < rows as i32 {
            Some((col as u16, row as u16))
        } else {
            None
        }
    }

    /// Return bounds as `((min_x, min_z), (max_x, max_z))`.
    ///
    /// A zero-sized axis has equal minimum and maximum coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use nms_copilot::map::state::{MapState, ZoomLevel};
    /// use nms_copilot::session::SessionState;
    /// use nms_graph::GalaxyModel;
    /// # fn main() {
    /// let model = GalaxyModel::new(); let session = SessionState::from_model(&model);
    /// let mut state = MapState::new(&model, &session); state.center = (16.0, -8.0); state.resize(10, 9);
    /// assert_eq!((state.grid_size, state.cursor), ((8, 4), (0, 0)));
    /// assert_eq!(state.cursor_voxel(), (-2032.0, -2056.0));
    /// assert_eq!(state.viewport_bounds(), ((-2032.0, -2056.0), (2064.0, 2040.0)));
    /// assert_eq!(state.voxel_to_grid(16.0, -8.0), Some((4, 2)));
    /// assert_eq!(state.voxel_to_grid(2064.0, -8.0), None);
    /// state.move_cursor(2, 1); assert_eq!((state.cursor, state.cursor_voxel()), ((2, 1), (-1008.0, -1032.0)));
    /// state.zoom_in(); assert_eq!((state.zoom, state.center, state.cursor, state.zoom_stack.clone()), (ZoomLevel::Region, (-1008.0, -1032.0), (4, 2), vec![(ZoomLevel::Galaxy, 16.0, -8.0)]));
    /// assert!(state.zoom_out()); assert_eq!((state.zoom, state.center, state.cursor, state.zoom_stack.clone()), (ZoomLevel::Galaxy, (16.0, -8.0), (4, 2), vec![]));
    /// assert!(!state.zoom_out()); assert!(!state.should_quit);
    /// state.player_pos = Some((100, 200)); state.center_on_player();
    /// assert_eq!((state.center, state.cursor, state.zoom, state.zoom_stack.clone()), ((100.0, 200.0), (4, 2), ZoomLevel::Galaxy, vec![]));
    /// state.resize(1, 3); assert_eq!((state.grid_size, state.cursor), ((0, 0), (4, 2)));
    /// assert_eq!(state.voxel_to_grid(100.0, 200.0), None);
    /// # }
    /// ```
    pub fn viewport_bounds(&self) -> ((f64, f64), (f64, f64)) {
        let (cols, rows) = self.grid_size;
        let extent = self.zoom.extent();
        let cell_size_x = extent / f64::from(cols.max(1));
        let cell_size_z = extent / f64::from(rows.max(1));

        let half_w = f64::from(cols) / 2.0 * cell_size_x;
        let half_h = f64::from(rows) / 2.0 * cell_size_z;

        (
            (self.center.0 - half_w, self.center.1 - half_h),
            (self.center.0 + half_w, self.center.1 + half_h),
        )
    }
}

/// Return the density glyph for a system count in one map cell.
///
/// The exact mapping is `0 => ' '`, `1 => '·'`, `2..=3 => '+'`,
/// `4..=7 => '*'`, and `8 or more => '#'`.
///
/// # Examples
///
/// ```
/// use nms_copilot::map::state::density_char;
/// let counts = [0, 1, 2, 3, 4, 7, 8, 100];
/// let glyphs: Vec<_> = counts.into_iter().map(density_char).collect();
/// assert_eq!(glyphs, [' ', '·', '+', '+', '*', '*', '#', '#']);
/// ```
pub fn density_char(count: usize) -> char {
    match count {
        0 => ' ',
        1 => '·',
        2..=3 => '+',
        4..=7 => '*',
        _ => '#',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_level_extent_galaxy() {
        assert_eq!(ZoomLevel::Galaxy.extent(), 4096.0);
    }

    #[test]
    fn test_zoom_level_extent_region() {
        assert_eq!(ZoomLevel::Region.extent(), 512.0);
    }

    #[test]
    fn test_zoom_level_extent_local() {
        assert_eq!(ZoomLevel::Local.extent(), 64.0);
    }

    #[test]
    fn test_zoom_in_galaxy_to_region() {
        assert_eq!(ZoomLevel::Galaxy.zoom_in(), Some(ZoomLevel::Region));
    }

    #[test]
    fn test_zoom_in_region_to_local() {
        assert_eq!(ZoomLevel::Region.zoom_in(), Some(ZoomLevel::Local));
    }

    #[test]
    fn test_zoom_in_local_returns_none() {
        assert_eq!(ZoomLevel::Local.zoom_in(), None);
    }

    #[test]
    fn test_zoom_out_galaxy_returns_none() {
        assert_eq!(ZoomLevel::Galaxy.zoom_out(), None);
    }

    #[test]
    fn test_zoom_out_region_to_galaxy() {
        assert_eq!(ZoomLevel::Region.zoom_out(), Some(ZoomLevel::Galaxy));
    }

    #[test]
    fn test_zoom_out_local_to_region() {
        assert_eq!(ZoomLevel::Local.zoom_out(), Some(ZoomLevel::Region));
    }

    #[test]
    fn test_density_char_empty() {
        assert_eq!(density_char(0), ' ');
    }

    #[test]
    fn test_density_char_single() {
        assert_eq!(density_char(1), '·');
    }

    #[test]
    fn test_density_char_few() {
        assert_eq!(density_char(2), '+');
        assert_eq!(density_char(3), '+');
    }

    #[test]
    fn test_density_char_several() {
        assert_eq!(density_char(4), '*');
        assert_eq!(density_char(7), '*');
    }

    #[test]
    fn test_density_char_many() {
        assert_eq!(density_char(8), '#');
        assert_eq!(density_char(100), '#');
    }

    #[test]
    fn test_voxel_to_grid_center_maps_to_center() {
        let state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (40, 12),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        // Center voxel should map to center of grid
        let pos = state.voxel_to_grid(0.0, 0.0);
        assert_eq!(pos, Some((40, 12)));
    }

    #[test]
    fn test_voxel_to_grid_outside_returns_none() {
        let state = MapState {
            zoom: ZoomLevel::Local,
            center: (0.0, 0.0),
            cursor: (0, 0),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        // Far away voxel should be outside viewport
        assert!(state.voxel_to_grid(2000.0, 2000.0).is_none());
    }

    #[test]
    fn test_move_cursor_positive() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (10, 10),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.move_cursor(5, 3);
        assert_eq!(state.cursor, (15, 13));
    }

    #[test]
    fn test_move_cursor_clamps_negative() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (2, 2),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.move_cursor(-10, -10);
        assert_eq!(state.cursor, (0, 0));
    }

    #[test]
    fn test_move_cursor_clamps_to_grid_bounds() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (78, 22),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.move_cursor(10, 10);
        assert_eq!(state.cursor, (79, 23));
    }

    #[test]
    fn test_zoom_in_pushes_stack() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (100.0, -200.0),
            cursor: (40, 12),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.zoom_in();
        assert_eq!(state.zoom, ZoomLevel::Region);
        assert_eq!(state.zoom_stack.len(), 1);
        assert_eq!(state.zoom_stack[0].0, ZoomLevel::Galaxy);
    }

    #[test]
    fn test_zoom_out_pops_stack() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (100.0, -200.0),
            cursor: (40, 12),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.zoom_in();
        assert!(state.zoom_out());
        assert_eq!(state.zoom, ZoomLevel::Galaxy);
        assert!(state.zoom_stack.is_empty());
    }

    #[test]
    fn test_zoom_out_at_galaxy_returns_false() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (0, 0),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        assert!(!state.zoom_out());
    }

    #[test]
    fn test_center_on_player_updates_center() {
        let mut state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (10, 10),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: Some((100, -200)),
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        state.center_on_player();
        assert_eq!(state.center, (100.0, -200.0));
        assert_eq!(state.cursor, (40, 12));
    }

    #[test]
    fn test_viewport_bounds_symmetry() {
        let state = MapState {
            zoom: ZoomLevel::Galaxy,
            center: (0.0, 0.0),
            cursor: (0, 0),
            grid_size: (80, 24),
            galaxy: 0,
            galaxy_name: "Euclid".into(),
            base_labels: vec![],
            player_pos: None,
            zoom_stack: vec![],
            show_help: false,
            should_quit: false,
        };
        let ((min_x, min_z), (max_x, max_z)) = state.viewport_bounds();
        // Should be symmetric around center (0, 0)
        assert!((min_x + max_x).abs() < 0.01);
        assert!((min_z + max_z).abs() < 0.01);
    }
}
