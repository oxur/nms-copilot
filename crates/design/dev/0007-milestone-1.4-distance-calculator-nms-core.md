# Milestone 1.4 — Distance Calculator (nms-core)

Distance calculation between galactic coordinates using Euclidean voxel distance. All code is added to the existing `GalacticAddress` impl in `src/address.rs` — no new files needed.

## Formula

The galaxy is a 4096 x 256 x 4096 voxel grid (a flattened disc shape). The center of the galaxy (galactic core) is at coordinate (0, 0, 0).

- **Voxel distance** = sqrt((x1 - x2)^2 + (y1 - y2)^2 + (z1 - z2)^2)
- **Distance in light-years** = voxel_distance * 400.0
- 1 voxel = 400 light-years

Coordinates used are the signed center-origin values from the portal coordinate frame:

- VoxelX: i16, range -2048..2047 (12-bit signed)
- VoxelY: i8, range -128..127 (8-bit signed)
- VoxelZ: i16, range -2048..2047 (12-bit signed)

These are extracted via `GalacticAddress::voxel_x()`, `voxel_y()`, `voxel_z()` (defined in Milestone 1.2).

## Cross-Galaxy Note

Distances between addresses in different galaxies (different `reality_index`) are physically meaningless. The methods should still compute the math without checking `reality_index` — it is the caller's responsibility to only compare addresses within the same galaxy. Optionally, add a doc comment noting this.

---

## Methods to Add to GalacticAddress

Add these to the existing `impl GalacticAddress` block in `crates/nms-core/src/address.rs`:

```rust
impl GalacticAddress {
    /// Euclidean distance in light-years to another address.
    ///
    /// Uses signed voxel coordinates (center-origin). Each voxel is 400 ly.
    /// Note: comparing addresses across different galaxies (reality_index)
    /// is not physically meaningful.
    pub fn distance_ly(&self, other: &GalacticAddress) -> f64 {
        let (x1, y1, z1) = self.voxel_position();
        let (x2, y2, z2) = other.voxel_position();

        let dx = (x1 as f64) - (x2 as f64);
        let dy = (y1 as f64) - (y2 as f64);
        let dz = (z1 as f64) - (z2 as f64);

        (dx * dx + dy * dy + dz * dz).sqrt() * 400.0
    }

    /// Whether two addresses are in the same region (same VoxelX, VoxelY, VoxelZ).
    /// Two systems in the same region share the same voxel but may have different
    /// solar system indices and planet indices.
    pub fn same_region(&self, other: &GalacticAddress) -> bool {
        self.voxel_x() == other.voxel_x()
            && self.voxel_y() == other.voxel_y()
            && self.voxel_z() == other.voxel_z()
    }

    /// Whether two addresses are in the same system.
    /// Same region AND same solar system index. Planet index may differ.
    pub fn same_system(&self, other: &GalacticAddress) -> bool {
        self.same_region(other)
            && self.solar_system_index() == other.solar_system_index()
    }

    /// Whether another address is within `ly` light-years of this address.
    pub fn within(&self, other: &GalacticAddress, ly: f64) -> bool {
        self.distance_ly(other) <= ly
    }

    /// Voxel coordinates as (x, y, z) signed integers (center-origin).
    /// Already defined in Milestone 1.2 — listed here for reference only.
    /// pub fn voxel_position(&self) -> (i16, i8, i16)
}
```

### Implementation Details

- All arithmetic uses `f64` to avoid overflow. VoxelX and VoxelZ are i16 (max magnitude 2048), VoxelY is i8 (max magnitude 128). The maximum squared distance before sqrt is approximately `2 * 4096^2 + 256^2 = ~33.6M`, well within f64 range.
- `distance_ly` returns `0.0` for identical addresses.
- `within` uses `<=` (inclusive boundary).
- `same_region` and `same_system` compare the raw signed values, not the packed bits, to be semantically clear (though for these fields comparing packed bits would give the same result).

### Distance to Galactic Core

A useful derived method:

```rust
impl GalacticAddress {
    /// Distance in light-years from this address to the galactic core (0, 0, 0).
    pub fn distance_to_core_ly(&self) -> f64 {
        let (x, y, z) = self.voxel_position();
        let dx = x as f64;
        let dy = y as f64;
        let dz = z as f64;
        (dx * dx + dy * dy + dz * dz).sqrt() * 400.0
    }
}
```

---

## Tests

Add to the existing `#[cfg(test)] mod tests` block in `crates/nms-core/src/address.rs`:

```rust
#[cfg(test)]
mod distance_tests {
    use super::*;

    #[test]
    fn identical_addresses_distance_zero() {
        let addr = GalacticAddress::new(100, 50, -200, 0x123, 3, 0);
        assert_eq!(addr.distance_ly(&addr), 0.0);
    }

    #[test]
    fn one_voxel_apart_x_axis() {
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let b = GalacticAddress::new(1, 0, 0, 0x100, 0, 0);
        let dist = a.distance_ly(&b);
        assert!((dist - 400.0).abs() < 0.001, "expected 400.0, got {}", dist);
    }

    #[test]
    fn one_voxel_apart_y_axis() {
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let b = GalacticAddress::new(0, 1, 0, 0x100, 0, 0);
        let dist = a.distance_ly(&b);
        assert!((dist - 400.0).abs() < 0.001, "expected 400.0, got {}", dist);
    }

    #[test]
    fn one_voxel_apart_z_axis() {
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let b = GalacticAddress::new(0, 0, 1, 0x100, 0, 0);
        let dist = a.distance_ly(&b);
        assert!((dist - 400.0).abs() < 0.001, "expected 400.0, got {}", dist);
    }

    #[test]
    fn diagonal_distance_3_4_5_triangle() {
        // 3-4-5 right triangle in voxel space: dx=3, dy=4, dz=0
        // distance = 5 voxels = 2000 ly
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let b = GalacticAddress::new(3, 4, 0, 0x100, 0, 0);
        let dist = a.distance_ly(&b);
        assert!((dist - 2000.0).abs() < 0.001, "expected 2000.0, got {}", dist);
    }

    #[test]
    fn negative_coordinates() {
        // One address at (-100, -50, -200), another at (100, 50, 200)
        // dx=200, dy=100, dz=400
        // dist = sqrt(200^2 + 100^2 + 400^2) = sqrt(40000 + 10000 + 160000) = sqrt(210000)
        // sqrt(210000) ~= 458.2575695...
        // in ly: 458.2575695 * 400 = 183303.0278
        let a = GalacticAddress::new(-100, -50, -200, 0x100, 0, 0);
        let b = GalacticAddress::new(100, 50, 200, 0x100, 0, 0);
        let dist = a.distance_ly(&b);
        let expected = (210000.0_f64).sqrt() * 400.0;
        assert!(
            (dist - expected).abs() < 0.01,
            "expected {}, got {}",
            expected,
            dist
        );
    }

    #[test]
    fn max_distance_across_galaxy() {
        // Opposite corners: (-2048, -128, -2048) to (2047, 127, 2047)
        // dx=4095, dy=255, dz=4095
        // dist = sqrt(4095^2 + 255^2 + 4095^2) = sqrt(16769025 + 65025 + 16769025)
        //      = sqrt(33603075) ~= 5797.0
        // in ly: ~2318800
        let a = GalacticAddress::new(-2048, -128, -2048, 0x000, 0, 0);
        let b = GalacticAddress::new(2047, 127, 2047, 0x000, 0, 0);
        let dist = a.distance_ly(&b);
        let expected = ((4095.0_f64).powi(2) + (255.0_f64).powi(2) + (4095.0_f64).powi(2)).sqrt() * 400.0;
        assert!(
            (dist - expected).abs() < 1.0,
            "expected {}, got {}",
            expected,
            dist
        );
    }

    #[test]
    fn distance_to_core() {
        // Address at (3, 4, 0): distance to core = 5 voxels = 2000 ly
        let addr = GalacticAddress::new(3, 4, 0, 0x100, 0, 0);
        let dist = addr.distance_to_core_ly();
        assert!((dist - 2000.0).abs() < 0.001, "expected 2000.0, got {}", dist);
    }

    #[test]
    fn distance_to_core_at_origin() {
        let addr = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        assert_eq!(addr.distance_to_core_ly(), 0.0);
    }

    #[test]
    fn same_region_same_voxels_different_ssi() {
        let a = GalacticAddress::new(100, 50, -200, 0x001, 0, 0);
        let b = GalacticAddress::new(100, 50, -200, 0x002, 0, 0);
        assert!(a.same_region(&b));
        assert!(!a.same_system(&b));
    }

    #[test]
    fn same_system_same_voxels_same_ssi() {
        let a = GalacticAddress::new(100, 50, -200, 0x123, 3, 0);
        let b = GalacticAddress::new(100, 50, -200, 0x123, 5, 0);
        assert!(a.same_region(&b));
        assert!(a.same_system(&b));
    }

    #[test]
    fn different_region() {
        let a = GalacticAddress::new(100, 50, -200, 0x123, 0, 0);
        let b = GalacticAddress::new(101, 50, -200, 0x123, 0, 0);
        assert!(!a.same_region(&b));
        assert!(!a.same_system(&b));
    }

    #[test]
    fn within_boundary_inclusive() {
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        let b = GalacticAddress::new(5, 0, 0, 0x100, 0, 0);
        // Distance = 5 * 400 = 2000 ly
        assert!(a.within(&b, 2000.0));  // exactly at boundary
        assert!(a.within(&b, 2001.0));  // just over
        assert!(!a.within(&b, 1999.0)); // just under
    }

    #[test]
    fn within_zero_distance() {
        let a = GalacticAddress::new(0, 0, 0, 0x100, 0, 0);
        assert!(a.within(&a, 0.0));
    }

    #[test]
    fn distance_is_symmetric() {
        let a = GalacticAddress::new(-500, 42, 1000, 0x123, 0, 0);
        let b = GalacticAddress::new(300, -100, -800, 0x456, 0, 0);
        assert_eq!(a.distance_ly(&b), b.distance_ly(&a));
    }
}
```

---

## Summary of Changes to `src/address.rs`

This milestone adds the following to the existing `GalacticAddress` impl block:

| Method | Signature | Description |
|--------|-----------|-------------|
| `distance_ly` | `(&self, &GalacticAddress) -> f64` | Euclidean distance in light-years |
| `distance_to_core_ly` | `(&self) -> f64` | Distance to galactic center |
| `same_region` | `(&self, &GalacticAddress) -> bool` | Same voxel coordinates |
| `same_system` | `(&self, &GalacticAddress) -> bool` | Same region + same SSI |
| `within` | `(&self, &GalacticAddress, f64) -> bool` | Within N light-years |

No new dependencies are needed. No new files are created. The `voxel_position()` method from Milestone 1.2 is a prerequisite.
