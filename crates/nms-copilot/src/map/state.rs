//! Map state: zoom levels, viewport math, cursor logic.

use nms_core::galaxy::Galaxy;
use nms_graph::GalaxyModel;

use crate::session::SessionState;

/// Zoom tier for the galaxy map.
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

/// Label for a base on the map.
#[derive(Debug, Clone)]
pub struct BaseLabel {
    pub letter: char,
    pub name: String,
    pub voxel_x: i16,
    pub voxel_z: i16,
}

/// Interactive map state.
pub struct MapState {
    /// Current zoom level.
    pub zoom: ZoomLevel,
    /// Viewport center in voxel coordinates (X, Z).
    pub center: (f64, f64),
    /// Cursor position on the grid (col, row).
    pub cursor: (u16, u16),
    /// Usable map grid size (cols, rows).
    pub grid_size: (u16, u16),
    /// Active galaxy index.
    pub galaxy: u8,
    /// Galaxy name for display.
    pub galaxy_name: String,
    /// Base labels (A-Z).
    pub base_labels: Vec<BaseLabel>,
    /// Player voxel position (X, Z), if known.
    pub player_pos: Option<(i16, i16)>,
    /// Stack for zoom-out restoration.
    pub zoom_stack: Vec<(ZoomLevel, f64, f64)>,
    /// Whether to show the help overlay.
    pub show_help: bool,
    /// Whether the map should exit.
    pub should_quit: bool,
}

impl MapState {
    /// Create initial map state from the model and session.
    pub fn new(model: &GalaxyModel, session: &SessionState) -> Self {
        let galaxy = model.active_galaxy;
        let galaxy_name = Galaxy::by_index(galaxy).name.to_string();

        let player_pos = model.player_position().map(|a| (a.voxel_x(), a.voxel_z()));

        // Center on player or origin
        let center = player_pos
            .map(|(x, z)| (f64::from(x), f64::from(z)))
            .unwrap_or((0.0, 0.0));

        // Build base labels (A-Z)
        let mut base_labels: Vec<BaseLabel> = model
            .bases
            .values()
            .filter(|b| b.address.reality_index == galaxy)
            .enumerate()
            .take(26)
            .map(|(i, b)| BaseLabel {
                letter: (b'A' + i as u8) as char,
                name: b.name.clone(),
                voxel_x: b.address.voxel_x(),
                voxel_z: b.address.voxel_z(),
            })
            .collect();
        base_labels.sort_by_key(|b| b.letter);

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
            grid_size: (80, 24),
            galaxy,
            galaxy_name,
            base_labels,
            player_pos,
            zoom_stack: Vec::new(),
            show_help: false,
            should_quit: false,
        }
    }

    /// Update grid size (e.g., on terminal resize).
    pub fn resize(&mut self, cols: u16, rows: u16) {
        // Reserve 3 rows for status + legend
        let map_rows = rows.saturating_sub(3);
        self.grid_size = (cols, map_rows);
        self.clamp_cursor();
    }

    /// Move cursor by (dx, dy), clamping to grid bounds.
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

    /// Zoom in on the cell under the cursor.
    pub fn zoom_in(&mut self) {
        if let Some(next_zoom) = self.zoom.zoom_in() {
            // Save current state for zoom-out
            self.zoom_stack
                .push((self.zoom, self.center.0, self.center.1));
            // New center = voxel coordinate of cursor cell
            self.center = self.cursor_voxel();
            self.zoom = next_zoom;
            // Reset cursor to center of grid
            self.cursor = (self.grid_size.0 / 2, self.grid_size.1 / 2);
        }
    }

    /// Zoom out, restoring previous state.
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

    /// Center the viewport on the player position.
    pub fn center_on_player(&mut self) {
        if let Some((px, pz)) = self.player_pos {
            self.center = (f64::from(px), f64::from(pz));
            self.cursor = (self.grid_size.0 / 2, self.grid_size.1 / 2);
        }
    }

    /// Get the voxel coordinate that the cursor is pointing at.
    pub fn cursor_voxel(&self) -> (f64, f64) {
        let (cols, rows) = self.grid_size;
        let extent = self.zoom.extent();
        let cell_size = extent / f64::from(cols.max(1));

        let half_cols = f64::from(cols) / 2.0;
        let half_rows = f64::from(rows) / 2.0;

        let vx = self.center.0 + (f64::from(self.cursor.0) - half_cols) * cell_size;
        let vz = self.center.1 + (f64::from(self.cursor.1) - half_rows) * cell_size;
        (vx, vz)
    }

    /// Convert a voxel coordinate to grid position.
    /// Returns `None` if outside the viewport.
    pub fn voxel_to_grid(&self, vx: f64, vz: f64) -> Option<(u16, u16)> {
        let (cols, rows) = self.grid_size;
        let extent = self.zoom.extent();
        let cell_size = extent / f64::from(cols.max(1));

        let half_cols = f64::from(cols) / 2.0;
        let half_rows = f64::from(rows) / 2.0;

        let col = ((vx - self.center.0) / cell_size + half_cols) as i32;
        let row = ((vz - self.center.1) / cell_size + half_rows) as i32;

        if col >= 0 && col < cols as i32 && row >= 0 && row < rows as i32 {
            Some((col as u16, row as u16))
        } else {
            None
        }
    }

    /// Get the voxel bounding box for the current viewport.
    /// Returns ((min_x, min_z), (max_x, max_z)).
    pub fn viewport_bounds(&self) -> ((f64, f64), (f64, f64)) {
        let (cols, rows) = self.grid_size;
        let extent = self.zoom.extent();
        let cell_size = extent / f64::from(cols.max(1));

        let half_w = f64::from(cols) / 2.0 * cell_size;
        let half_h = f64::from(rows) / 2.0 * cell_size;

        (
            (self.center.0 - half_w, self.center.1 - half_h),
            (self.center.0 + half_w, self.center.1 + half_h),
        )
    }
}

/// Select a density character based on system count in a cell.
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
