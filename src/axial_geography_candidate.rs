//! P3.1B: headless, *unapproved* mapping between the candidate axial torus
//! and the 1,024 km periodic geography raster.
//!
//! Exact integer arithmetic owns equivalent global addresses. The geographic
//! projection deliberately scales the north/south direction by ~11.56 metres
//! per 1,024 km period. This tiny affine distortion is NOT adopted gameplay
//! geometry or a saved-world schema. No code in this module is called by the
//! live terrain generator, renderer, collision, or streamer.

use glam::{DVec2, Vec3};

use crate::{
    geography::{RasterSample, ELEVATION_SAMPLES_PER_AXIS, WORLD_SIDE_METRES},
    hex_torus_candidate::{
        Axial64, PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET, HEX_ROW_METRES,
    },
};

/// Stable candidate three-dimensional cell identity. Y does not wrap.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CandidateCellId {
    pub q: i64,
    pub r: i64,
    pub y: i64,
}

impl CandidateCellId {
    pub const fn new(q: i64, r: i64, y: i64) -> Self {
        Self { q, r, y }
    }

    pub fn equivalent(self) -> Self {
        let wrapped = Axial64::new(self.q, self.r).candidate_wrap();
        Self::new(wrapped.q, wrapped.r, self.y)
    }

    /// Chunk coordinates stay exact integers and preserve negative-cell
    /// Euclidean division. Canonicalize the CELL before deriving a chunk ID.
    pub fn candidate_chunk(self, side: i64) -> CandidateChunkId {
        assert!(side > 0);
        let canonical = self.equivalent();
        CandidateChunkId {
            q: canonical.q.div_euclid(side),
            r: canonical.r.div_euclid(side),
            y: canonical.y.div_euclid(side),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CandidateChunkId {
    pub q: i64,
    pub r: i64,
    pub y: i64,
}

/// A rational geography projection, exact for raster index ownership.
/// Horizontal position in metres: (q + r/2) * 1m.
/// Vertical plan position: r * (1,024,000m / 1,182,400 cells).
/// This is NOT the standard regular-hex physical z spacing.
#[derive(Clone, Copy, Debug, Default)]
pub struct CandidateProjection;

impl CandidateProjection {
    /// Integer floor mapping into the geographic 32,768-square raster.
    /// Equivalent candidate axial cells always identify the same sample.
    /// Never derive authoritative raster ownership from rounded floats.
    pub fn raster_sample(cell: Axial64) -> RasterSample {
        let x_numerator = (2 * i128::from(cell.q) + i128::from(cell.r))
            * i128::from(ELEVATION_SAMPLES_PER_AXIS);
        let x_denominator = 2 * i128::from(PERIOD_Q);
        let z_numerator = i128::from(cell.r) * i128::from(ELEVATION_SAMPLES_PER_AXIS);
        let z_denominator = i128::from(PERIOD_R);
        let axis = i128::from(ELEVATION_SAMPLES_PER_AXIS);
        RasterSample {
            x: x_numerator.div_euclid(x_denominator).rem_euclid(axis) as u32,
            z: z_numerator.div_euclid(z_denominator).rem_euclid(axis) as u32,
        }
    }

    /// Candidate planar metre coordinates. Explicit physical distortion is
    /// confined here, never silently applied to regular-hex renderer geometry.
    pub fn geographic_metres(cell: Axial64) -> DVec2 {
        DVec2::new(
            cell.q as f64 + cell.r as f64 * 0.5,
            cell.r as f64 * Self::z_metres_per_r(),
        )
    }

    pub fn z_metres_per_r() -> f64 {
        WORLD_SIDE_METRES as f64 / PERIOD_R as f64
    }

    pub fn north_scale_ratio() -> f64 {
        Self::z_metres_per_r() / HEX_ROW_METRES
    }

    /// Difference is evaluated in signed integers before conversion to
    /// floating point. Suitable for nearby geometry, not a world-owned ID.
    pub fn relative_metres(cell: CandidateCellId, origin: CandidateCellId) -> Vec3 {
        let dq = i128::from(cell.q) - i128::from(origin.q);
        let dr = i128::from(cell.r) - i128::from(origin.r);
        let dy = i128::from(cell.y) - i128::from(origin.y);
        Vec3::new(
            (dq as f64 + dr as f64 * 0.5) as f32,
            dy as f32,
            (dr as f64 * Self::z_metres_per_r()) as f32,
        )
    }

    /// Shortest candidate periodic image by exact axial displacement.
    /// Search the 3x3 neighboring period translations, never wrap Q/R
    /// independently. Intended for already-canonicalized local endpoints.
    /// Ties are resolved by fixed period iteration order.
    pub fn nearest_periodic_image(from: Axial64, to: Axial64) -> Axial64 {
        let mut nearest = to;
        let mut best = f64::INFINITY;
        for nz in -1_i64..=1 {
            for nx in -1_i64..=1 {
                let candidate = Axial64::new(
                    to.q + nx * PERIOD_Q + nz * R_PERIOD_Q_OFFSET,
                    to.r + nz * PERIOD_R,
                );
                let dq = (i128::from(candidate.q) - i128::from(from.q)) as f64;
                let dr = (i128::from(candidate.r) - i128::from(from.r)) as f64;
                let x = dq + dr * 0.5;
                let z = dr * Self::z_metres_per_r();
                let distance_squared = x * x + z * z;
                if distance_squared < best {
                    nearest = candidate;
                    best = distance_squared;
                }
            }
        }
        nearest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geography::wrap_sample, world::CHUNK_SIDE};

    #[test]
    fn square_geography_is_exact_but_regular_hex_metric_requires_a_choice() {
        let origin = Axial64::new(12, -17);
        let east = Axial64::new(origin.q + PERIOD_Q, origin.r);
        let north = Axial64::new(origin.q + R_PERIOD_Q_OFFSET, origin.r + PERIOD_R);
        let base = CandidateProjection::geographic_metres(origin);
        let dx = CandidateProjection::geographic_metres(east) - base;
        let dz = CandidateProjection::geographic_metres(north) - base;
        assert!((dx.x - 1_024_000.0).abs() < 1e-6);
        assert!(dx.y.abs() < 1e-6);
        assert!(dz.x.abs() < 1e-6);
        assert!((dz.y - 1_024_000.0).abs() < 1e-6);
        assert!(CandidateProjection::north_scale_ratio() > 1.0);
        assert!((CandidateProjection::north_scale_ratio() - 1.0) < 0.000_012);
    }

    #[test]
    fn raster_ownership_matches_periodic_geography_across_both_seams_and_corners() {
        for cell in [
            Axial64::new(0, 0),
            Axial64::new(-1, -1),
            Axial64::new(PERIOD_Q - 1, PERIOD_R - 1),
            Axial64::new(-1_234_567_890, 1_234_567_890),
            Axial64::new(i64::MAX, i64::MIN),
        ] {
            let actual = CandidateProjection::raster_sample(cell);
            assert!(actual.x < ELEVATION_SAMPLES_PER_AXIS);
            assert!(actual.z < ELEVATION_SAMPLES_PER_AXIS);
            assert_eq!(actual, CandidateProjection::raster_sample(cell.candidate_wrap()));
            for nz in -2_i64..=2 {
                for nx in -2_i64..=2 {
                    if (-2_000_000_000..2_000_000_000).contains(&cell.q)
                        && (-2_000_000_000..2_000_000_000).contains(&cell.r)
                    {
                        let shifted = cell.candidate_translated(nx, nz);
                        assert_eq!(actual, CandidateProjection::raster_sample(shifted));
                    }
                }
            }
        }
        assert_eq!(
            CandidateProjection::raster_sample(Axial64::new(0, 0)),
            wrap_sample(0, 0)
        );
        assert_eq!(
            CandidateProjection::raster_sample(Axial64::new(PERIOD_Q, 0)),
            wrap_sample(0, 0)
        );
    }

    #[test]
    fn equivalent_cells_have_identical_candidate_chunk_ownership_including_vertical() {
        for p in [
            CandidateCellId::new(-1, -1, -1),
            CandidateCellId::new(0, 0, 220),
            CandidateCellId::new(15, 16, 511),
            CandidateCellId::new(1_024_000, -1_182_400, 512),
        ] {
            let base = p.candidate_chunk(i64::from(CHUNK_SIDE));
            for nx in -1_i64..=1 {
                for nz in -1_i64..=1 {
                    let shifted = CandidateCellId::new(
                        p.q + nx * PERIOD_Q + nz * R_PERIOD_Q_OFFSET,
                        p.r + nz * PERIOD_R,
                        p.y,
                    );
                    assert_eq!(shifted.candidate_chunk(i64::from(CHUNK_SIDE)), base);
                }
            }
            assert_ne!(
                CandidateCellId::new(p.q, p.r, p.y + 16).candidate_chunk(16).y,
                base.y
            );
        }
    }

    #[test]
    fn nearest_periodic_image_is_local_at_all_four_seam_corners() {
        let neighbors = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
        for base in [
            Axial64::new(0, 0),
            Axial64::new(PERIOD_Q - 1, 0),
            Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1),
            Axial64::new(PERIOD_Q - 1 + R_PERIOD_Q_OFFSET, PERIOD_R - 1),
        ] {
            let from = base.candidate_wrap();
            for (dq, dr) in neighbors {
                let target = Axial64::new(base.q + dq, base.r + dr).candidate_wrap();
                let near = CandidateProjection::nearest_periodic_image(from, target);
                let d = CandidateProjection::geographic_metres(near)
                    - CandidateProjection::geographic_metres(from);
                assert!(d.length() < 1.1, "nonlocal neighbor from {base:?} via {target:?}: {d:?}");
                assert_eq!(near.candidate_wrap(), target);
            }
        }
    }

    #[test]
    fn local_f32_delta_remains_exact_at_large_global_addresses() {
        let origin = CandidateCellId::new(1_000_000_000_000, -1_000_000_000_000, 200);
        let offset = CandidateCellId::new(origin.q + 1, origin.r - 2, origin.y + 1);
        let local = CandidateProjection::relative_metres(offset, origin);
        assert_eq!(local.x, 0.0);
        assert_eq!(local.y, 1.0);
        assert!((local.z + 2.0 * CandidateProjection::z_metres_per_r() as f32).abs() < 1e-6);
        assert_eq!(
            CandidateProjection::geographic_metres(Axial64::new(offset.q, offset.r)).x as f32,
            CandidateProjection::geographic_metres(Axial64::new(origin.q, origin.r)).x as f32
        );
    }
}
