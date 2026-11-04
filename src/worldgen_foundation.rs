//! Headless identity and coarse elevation sampling for the proposed world.
//!
//! This does not enable playable torus seams, biomes, geological simulation,
//! persistence, or rendering LOD. It reuses the authoritative prototype
//! height sampler instead of inventing a second distant-terrain algorithm.

use crate::{
    generated_terrain::{PrototypeTerrain, PROTOTYPE_TERRAIN_REVISION},
    geography::{ELEVATION_SAMPLES_PER_AXIS, ELEVATION_TILE_SIDE, WORLD_SIDE_METRES},
};

/// The geographic raster's wrap is not yet a playable-hex seam mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexTopology {
    Unresolved,
}

/// In-memory identity metadata, NOT a stable save-file format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldGenerationIdentity {
    pub seed: u64,
    pub terrain_revision: u32,
    pub geographic_side_metres: u32,
    pub raster_samples_per_axis: u32,
    pub raster_tile_side: u32,
    pub playable_hex_topology: HexTopology,
}

impl WorldGenerationIdentity {
    pub const fn prototype(seed: u64) -> Self {
        Self {
            seed,
            terrain_revision: PROTOTYPE_TERRAIN_REVISION,
            geographic_side_metres: WORLD_SIDE_METRES,
            raster_samples_per_axis: ELEVATION_SAMPLES_PER_AXIS,
            raster_tile_side: ELEVATION_TILE_SIDE,
            playable_hex_topology: HexTopology::Unresolved,
        }
    }

    pub const fn terrain(self) -> PrototypeTerrain {
        PrototypeTerrain::new(self.seed)
    }
}

/// Elevation-only *sample grid*, not a terrain mesh or GPU-ready LOD.
/// Each point is sampled at an absolute signed axial position, including
/// negative positions, using the same generator as the detailed hexels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoarseSurfacePatch {
    pub origin_q: i32,
    pub origin_r: i32,
    pub spacing_cells: i32,
    pub points_per_axis: usize,
    pub heights: Vec<i32>,
}

impl CoarseSurfacePatch {
    /// Sample a square axial grid; points are row-major (R, then Q).
    ///
    /// Checked coordinate arithmetic rejects requests outside the current
    /// i32 cell-address domain. A future i64 world ownership scheme is
    /// deliberately not inferred from the unapproved torus candidate.
    pub fn sample(
        terrain: &PrototypeTerrain,
        origin_q: i32,
        origin_r: i32,
        spacing_cells: i32,
        points_per_axis: usize,
    ) -> Result<Self, &'static str> {
        if spacing_cells <= 0 || !(2..=129).contains(&points_per_axis) {
            return Err("coarse surface requires positive spacing and 2..=129 points per axis");
        }
        let last = (points_per_axis - 1) as i64 * i64::from(spacing_cells);
        for origin in [origin_q, origin_r] {
            let end = i64::from(origin) + last;
            if end > i64::from(i32::MAX) {
                return Err("coarse surface coordinates exceed the i32 cell domain");
            }
        }
        let mut heights = Vec::with_capacity(points_per_axis * points_per_axis);
        for r in 0..points_per_axis {
            for q in 0..points_per_axis {
                let abs_q = i64::from(origin_q) + q as i64 * i64::from(spacing_cells);
                let abs_r = i64::from(origin_r) + r as i64 * i64::from(spacing_cells);
                heights.push(terrain.surface_y(abs_q as i32, abs_r as i32));
            }
        }
        Ok(Self {
            origin_q,
            origin_r,
            spacing_cells,
            points_per_axis,
            heights,
        })
    }

    pub fn height(&self, q: usize, r: usize) -> Option<i32> {
        if q >= self.points_per_axis || r >= self.points_per_axis {
            return None;
        }
        Some(self.heights[r * self.points_per_axis + q])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Block, CellPos};

    #[test]
    fn metadata_is_versioned_and_does_not_claim_a_playable_torus() {
        let identity = WorldGenerationIdentity::prototype(42);
        assert_eq!(identity.terrain_revision, 1);
        assert_eq!(identity.geographic_side_metres, 1_024_000);
        assert_eq!(identity.raster_samples_per_axis, 32_768);
        assert_eq!(identity.raster_tile_side, 512);
        assert_eq!(identity.playable_hex_topology, HexTopology::Unresolved);
        assert_eq!(identity.terrain().seed(), 42);
    }

    #[test]
    fn coarse_grid_matches_near_surface_at_shared_coordinates() {
        let terrain = WorldGenerationIdentity::prototype(42).terrain();
        for (q0, r0) in [(-257, -257), (-16, 15), (0, 0), (65_536, -65_536)] {
            let patch = CoarseSurfacePatch::sample(&terrain, q0, r0, 16, 5).unwrap();
            for r in 0..5 {
                for q in 0..5 {
                    let aq = q0 + q as i32 * 16;
                    let ar = r0 + r as i32 * 16;
                    let top = terrain.surface_y(aq, ar);
                    assert_eq!(patch.height(q, r), Some(top));
                    assert_eq!(terrain.sample(CellPos::new(aq, ar, top)), Block::Grass);
                    assert_eq!(terrain.sample(CellPos::new(aq, ar, top + 1)), Block::Air);
                }
            }
        }
    }

    #[test]
    fn adjacent_patches_share_exact_edge_heights() {
        let terrain = PrototypeTerrain::new(97);
        let west = CoarseSurfacePatch::sample(&terrain, -64, -48, 16, 5).unwrap();
        let east = CoarseSurfacePatch::sample(&terrain, 0, -48, 16, 5).unwrap();
        let north = CoarseSurfacePatch::sample(&terrain, -64, 16, 16, 5).unwrap();
        for i in 0..5 {
            assert_eq!(west.height(4, i), east.height(0, i));
            assert_eq!(west.height(i, 4), north.height(i, 0));
        }
    }

    #[test]
    fn sampled_elevation_is_repeatable_and_seed_specific() {
        let a = PrototypeTerrain::new(42);
        let b = PrototypeTerrain::new(42);
        let c = PrototypeTerrain::new(99);
        let p = CoarseSurfacePatch::sample(&a, -10_000, 20_000, 32, 17).unwrap();
        assert_eq!(
            p,
            CoarseSurfacePatch::sample(&b, -10_000, 20_000, 32, 17).unwrap()
        );
        assert_ne!(
            p.heights,
            CoarseSurfacePatch::sample(&c, -10_000, 20_000, 32, 17)
                .unwrap()
                .heights
        );
    }

    #[test]
    fn invalid_or_overflowing_requests_are_rejected_without_allocating() {
        let terrain = PrototypeTerrain::new(42);
        for (q, r, spacing, count) in [
            (0, 0, 0, 4),
            (0, 0, -1, 4),
            (0, 0, 1, 1),
            (0, 0, 1, 130),
            (i32::MAX, 0, 16, 5),
            (0, i32::MAX, 16, 5),
        ] {
            assert!(CoarseSurfacePatch::sample(&terrain, q, r, spacing, count).is_err());
        }
    }
}
