use rstar::{AABB, PointDistance, RTreeObject};

// Re-export SystemId from nms-core (canonical definition lives there).
pub use nms_core::system::SystemId;

/// A system's position in 3D voxel space, stored in the R-tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemPoint {
    pub id: SystemId,
    pub point: [f64; 3],
}

impl SystemPoint {
    pub fn new(id: SystemId, x: f64, y: f64, z: f64) -> Self {
        Self {
            id,
            point: [x, y, z],
        }
    }

    /// Create from a `GalacticAddress`.
    pub fn from_address(addr: &nms_core::address::GalacticAddress) -> Self {
        let id = SystemId::from_address(addr);
        Self::new(
            id,
            addr.voxel_x() as f64,
            addr.voxel_y() as f64,
            addr.voxel_z() as f64,
        )
    }
}

impl RTreeObject for SystemPoint {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for SystemPoint {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.point[0] - point[0];
        let dy = self.point[1] - point[1];
        let dz = self.point[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::address::GalacticAddress;

    #[test]
    fn test_system_id_from_address_zeroes_planet() {
        let addr1 = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        let addr2 = GalacticAddress::new(100, 50, 200, 0x123, 5, 0);
        assert_eq!(
            SystemId::from_address(&addr1),
            SystemId::from_address(&addr2)
        );
    }

    #[test]
    fn test_system_id_different_ssi_differs() {
        let addr1 = GalacticAddress::new(100, 50, 200, 0x123, 0, 0);
        let addr2 = GalacticAddress::new(100, 50, 200, 0x456, 0, 0);
        assert_ne!(
            SystemId::from_address(&addr1),
            SystemId::from_address(&addr2)
        );
    }

    #[test]
    fn test_system_point_from_address_coordinates() {
        let addr = GalacticAddress::new(100, -50, 200, 0x123, 3, 0);
        let point = SystemPoint::from_address(&addr);
        assert_eq!(point.point, [100.0, -50.0, 200.0]);
    }

    #[test]
    fn test_system_point_distance_squared() {
        use rstar::PointDistance;
        let p = SystemPoint::new(SystemId(0), 0.0, 0.0, 0.0);
        let target = [3.0, 4.0, 0.0];
        assert!((p.distance_2(&target) - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_rtree_nearest_neighbor() {
        use rstar::RTree;
        let points = vec![
            SystemPoint::new(SystemId(1), 0.0, 0.0, 0.0),
            SystemPoint::new(SystemId(2), 10.0, 0.0, 0.0),
            SystemPoint::new(SystemId(3), 100.0, 0.0, 0.0),
        ];
        let tree = RTree::bulk_load(points);
        let nearest = tree.nearest_neighbor(&[1.0, 0.0, 0.0]).unwrap();
        assert_eq!(nearest.id, SystemId(1));
    }

    #[test]
    fn test_rtree_bulk_load_size() {
        use rstar::RTree;
        let points = vec![
            SystemPoint::new(SystemId(1), 0.0, 0.0, 0.0),
            SystemPoint::new(SystemId(2), 5.0, 5.0, 5.0),
        ];
        let tree = RTree::bulk_load(points);
        assert_eq!(tree.size(), 2);
    }

    #[test]
    fn test_system_point_envelope() {
        use rstar::RTreeObject;
        let p = SystemPoint::new(SystemId(1), 3.0, 4.0, 5.0);
        let env = p.envelope();
        assert_eq!(env.lower(), [3.0, 4.0, 5.0]);
        assert_eq!(env.upper(), [3.0, 4.0, 5.0]);
    }
}
