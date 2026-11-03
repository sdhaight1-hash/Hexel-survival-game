//! Periodic addressing of the *rectangular global geography raster*.
//!
//! This does not define the playable hexagonal Q/R/Y lattice or its seam
//! translations. Those are a separate, unresolved integration decision.

/// Proposed side length of the finite square geographic domain, in metres.
pub const WORLD_SIDE_METRES: u32 = 1_024_000;

/// Provisional number of elevation samples along each horizontal axis.
pub const ELEVATION_SAMPLES_PER_AXIS: u32 = 32_768;

/// Provisional edge length of a separately addressable elevation tile.
pub const ELEVATION_TILE_SIDE: u32 = 512;

/// Number of elevation tiles along each horizontal axis.
pub const ELEVATION_TILES_PER_AXIS: u32 = ELEVATION_SAMPLES_PER_AXIS / ELEVATION_TILE_SIDE;

/// Physical spacing between adjacent elevation samples in the global raster.
pub const ELEVATION_SAMPLE_SPACING_METRES: f64 =
    WORLD_SIDE_METRES as f64 / ELEVATION_SAMPLES_PER_AXIS as f64;

/// A canonical sample coordinate, always in [0, ELEVATION_SAMPLES_PER_AXIS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterSample {
    pub x: u32,
    pub z: u32,
}

/// Address of a sample within a globally tiled elevation raster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileAddress {
    pub tile_x: u32,
    pub tile_z: u32,
    pub local_x: u32,
    pub local_z: u32,
}

/// Map arbitrary signed raster coordinates onto the periodic world.
///
/// `rem_euclid` handles negative samples correctly; ordinary signed remainder
/// would produce negative results at world seams.
pub fn wrap_sample(x: i64, z: i64) -> RasterSample {
    let side = i64::from(ELEVATION_SAMPLES_PER_AXIS);
    RasterSample {
        x: x.rem_euclid(side) as u32,
        z: z.rem_euclid(side) as u32,
    }
}

/// Locate a canonical raster sample in its tile.
///
/// Panics for non-canonical samples; use `wrap_sample` first when an input
/// may lie outside the world boundaries.
pub fn tile_address(sample: RasterSample) -> TileAddress {
    assert!(sample.x < ELEVATION_SAMPLES_PER_AXIS);
    assert!(sample.z < ELEVATION_SAMPLES_PER_AXIS);
    TileAddress {
        tile_x: sample.x / ELEVATION_TILE_SIDE,
        tile_z: sample.z / ELEVATION_TILE_SIDE,
        local_x: sample.x % ELEVATION_TILE_SIDE,
        local_z: sample.z % ELEVATION_TILE_SIDE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provisional_grid_dimensions_are_consistent() {
        assert_eq!(ELEVATION_TILES_PER_AXIS, 64);
        assert_eq!(
            ELEVATION_TILES_PER_AXIS * ELEVATION_TILE_SIDE,
            ELEVATION_SAMPLES_PER_AXIS
        );
        assert_eq!(ELEVATION_SAMPLE_SPACING_METRES, 31.25);
    }

    #[test]
    fn both_axes_wrap_for_positive_and_negative_indices() {
        assert_eq!(wrap_sample(-1, 32_768), RasterSample { x: 32_767, z: 0 });
        assert_eq!(
            wrap_sample(32_768 + 7, -32_768 - 8),
            RasterSample { x: 7, z: 32_760 }
        );
        assert_eq!(wrap_sample(0, 0), wrap_sample(32_768, -32_768));
    }

    #[test]
    fn tile_boundaries_are_addressed_without_off_by_one_errors() {
        assert_eq!(
            tile_address(wrap_sample(511, 512)),
            TileAddress {
                tile_x: 0,
                tile_z: 1,
                local_x: 511,
                local_z: 0
            }
        );
        assert_eq!(
            tile_address(wrap_sample(-1, -1)),
            TileAddress {
                tile_x: 63,
                tile_z: 63,
                local_x: 511,
                local_z: 511
            }
        );
    }

    #[test]
    #[should_panic]
    fn invalid_noncanonical_sample_is_rejected() {
        tile_address(RasterSample { x: 32_768, z: 0 });
    }
}
