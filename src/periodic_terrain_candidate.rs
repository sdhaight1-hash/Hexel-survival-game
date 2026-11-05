//! P3.1D: headless *candidate* periodic terrain source on the proposed hex torus.
//!
//! Not wired into the live PrototypeTerrain or streamer. A wrapped address
//! alone is insufficient: this candidate uses periodic lattice values, so
//! elevations stay continuous across the physical east/north seams.
//! No biome, geology, water, persistence, height datum or topology approval.

use crate::{
    axial_geography_candidate::{CandidateCellId, CandidateChunkId},
    hex_torus_candidate::Axial64,
    world::{Block, CHUNK_SIDE},
};

/// New experimental algorithm: deliberately distinct from live terrain revision 1.
/// This is NOT a persistence or world-save generator version.
pub const PERIODIC_TERRAIN_CANDIDATE_REVISION: u32 = 2;

/// Elevation is sampled from the same periodic raster at either near or
/// coarse resolution. The 32,768² raster is addressed but never allocated.
#[derive(Clone, Copy, Debug)]
pub struct PeriodicTerrainCandidate {
    seed: u64,
}

impl PeriodicTerrainCandidate {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn surface_y(&self, pos: Axial64) -> i32 {
        // Continuous periodic raster coordinates, not a floor-snapped 31m
        // sample: adjacent 1m hexels must not inherit 31m elevation steps.
        let canonical = pos.candidate_wrap();
        let axis = f64::from(crate::geography::ELEVATION_SAMPLES_PER_AXIS);
        let x = ((canonical.q as f64 + canonical.r as f64 * 0.5)
            / crate::hex_torus_candidate::PERIOD_Q as f64
            * axis)
            .rem_euclid(axis);
        let z = (canonical.r as f64 / crate::hex_torus_candidate::PERIOD_R as f64 * axis)
            .rem_euclid(axis);

        // Broad plains are maintained by a regional relief mask rather than
        // applying equally strong bumps everywhere. All layers tile exactly
        // on the torus and are seed-deterministic.
        let plains = self.periodic_noise_at(x, z, 128, 0xD1A0_0001);
        let relief = smoothstep(0.30, 0.78, plains);
        // Plains still retain walkable rolling ground. The old 0.20
        // minimum suppressed nearly every local contour near test spawn.
        let amplitude = 0.55 + 0.45 * relief;
        let continent = self.periodic_noise_at(x, z, 32, 0xD1A0_0002);
        let regional = self.periodic_noise_at(x, z, 256, 0xD1A0_0003);
        let hills = self.periodic_noise_at(x, z, 1024, 0xD1A0_0004);
        let slopes = self.periodic_noise_at(x, z, 2048, 0xD1A0_0005);
        let fine = self.periodic_noise_at(x, z, 4096, 0xD1A0_0006);
        let undulation = self.periodic_noise_at(x, z, 8192, 0xD1A0_0007);
        let micro_relief = self.periodic_noise_at(x, z, 16384, 0xD1A0_0008);
        (220.0
            + (continent - 0.5) * 180.0
            + (regional - 0.5) * 85.0
            + (hills - 0.5) * 105.0 * amplitude
            + (slopes - 0.5) * 50.0 * amplitude
            + (fine - 0.5) * 72.0 * amplitude
            + (undulation - 0.5) * 24.0 * amplitude
            + (micro_relief - 0.5) * 6.0 * amplitude)
            .round() as i32
    }

    /// Unloaded cells are derived from the same periodic surface, not air.
    /// This matches the prototype's *simple* material layering only.
    pub fn sample(&self, pos: CandidateCellId) -> Block {
        let top = self.surface_y(Axial64::new(pos.q, pos.r));
        if pos.y > i64::from(top) {
            Block::Air
        } else if pos.y == i64::from(top) {
            Block::Grass
        } else if pos.y >= i64::from(top - 3) {
            Block::Soil
        } else {
            Block::Rock
        }
    }

    /// A test-only 16×16 surface slice. No 16³ allocation, GPU upload,
    /// generated ocean, or separate algorithm for distant terrain.
    pub fn chunk_surface(&self, pos: CandidateChunkId) -> Result<Vec<i32>, &'static str> {
        let side = i64::from(CHUNK_SIDE);
        let base_q = pos.q.checked_mul(side).ok_or("chunk Q overflows i64")?;
        let base_r = pos.r.checked_mul(side).ok_or("chunk R overflows i64")?;
        let mut result = Vec::with_capacity((CHUNK_SIDE * CHUNK_SIDE) as usize);
        for r in 0..side {
            for q in 0..side {
                let q = base_q.checked_add(q).ok_or("chunk Q cell overflows i64")?;
                let r = base_r.checked_add(r).ok_or("chunk R cell overflows i64")?;
                result.push(self.surface_y(Axial64::new(q, r)));
            }
        }
        Ok(result)
    }

    /// A coarse 2D grid sampled directly from the exact same source as
    /// individual cells and chunk slices. No far-LOD interpolation assumed.
    pub fn coarse_surface(
        &self,
        origin: Axial64,
        spacing: i64,
        points_per_axis: usize,
    ) -> Result<Vec<i32>, &'static str> {
        if spacing <= 0 || !(2..=129).contains(&points_per_axis) {
            return Err("spacing must be positive and points_per_axis in 2..=129");
        }
        let mut heights = Vec::with_capacity(points_per_axis * points_per_axis);
        for r in 0..points_per_axis {
            for q in 0..points_per_axis {
                let q_offset = (q as i64).checked_mul(spacing).ok_or("Q offset overflow")?;
                let r_offset = (r as i64).checked_mul(spacing).ok_or("R offset overflow")?;
                let aq = origin
                    .q
                    .checked_add(q_offset)
                    .ok_or("Q position overflow")?;
                let ar = origin
                    .r
                    .checked_add(r_offset)
                    .ok_or("R position overflow")?;
                heights.push(self.surface_y(Axial64::new(aq, ar)));
            }
        }
        Ok(heights)
    }

    fn periodic_noise_at(&self, x: f64, z: f64, grid_side: u32, domain: u64) -> f64 {
        let span = f64::from(crate::geography::ELEVATION_SAMPLES_PER_AXIS) / f64::from(grid_side);
        let gx = (x / span).floor() as u32 % grid_side;
        let gz = (z / span).floor() as u32 % grid_side;
        let tx = (x / span).fract();
        let tz = (z / span).fract();
        let u = tx * tx * (3.0 - 2.0 * tx);
        let v = tz * tz * (3.0 - 2.0 * tz);
        let x1 = (gx + 1) % grid_side;
        let z1 = (gz + 1) % grid_side;
        lerp(
            lerp(
                self.lattice(gx, gz, domain),
                self.lattice(x1, gz, domain),
                u,
            ),
            lerp(
                self.lattice(gx, z1, domain),
                self.lattice(x1, z1, domain),
                u,
            ),
            v,
        )
    }

    fn lattice(&self, q: u32, r: u32, domain: u64) -> f64 {
        let mut bits = self.seed
            ^ domain
            ^ u64::from(q).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ u64::from(r).wrapping_mul(0xD6E8_FEB_8665_9FD9);
        bits = bits.wrapping_add(0x9E37_79B9_7F4A_7C15);
        bits = (bits ^ (bits >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        bits = (bits ^ (bits >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        bits ^= bits >> 31;
        (bits >> 11) as f64 / (1_u64 << 53) as f64
    }
}

fn smoothstep(low: f64, high: f64, value: f64) -> f64 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex_torus_candidate::{PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET};

    fn translated(p: Axial64, east: i64, north: i64) -> Axial64 {
        Axial64::new(
            p.q + east * PERIOD_Q + north * R_PERIOD_Q_OFFSET,
            p.r + north * PERIOD_R,
        )
    }

    #[test]
    fn local_relief_is_visibly_nonflat_near_test_spawn() {
        let source = PeriodicTerrainCandidate::new(42);
        let centre = Axial64::new(432_776, 1_182_375);
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        // About a 500-metre walkable region, sampled without resident chunks.
        for dq in (-256..=256).step_by(32) {
            for dr in (-256..=256).step_by(32) {
                let y = source.surface_y(Axial64::new(centre.q + dq, centre.r + dr));
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        assert!(
            max_y - min_y >= 15,
            "seed 42 spawn remains visually flat: relief = {} m",
            max_y - min_y
        );
    }

    #[test]
    fn periodic_heights_and_layers_have_identical_translated_identity() {
        for seed in [0, 42, 99] {
            let source = PeriodicTerrainCandidate::new(seed);
            for p in [
                Axial64::new(0, 0),
                Axial64::new(-1, -1),
                Axial64::new(PERIOD_Q - 1, PERIOD_R - 1),
                Axial64::new(-10_000_000, 12_000_000),
            ] {
                let h = source.surface_y(p);
                for east in -2..=2 {
                    for north in -2..=2 {
                        let image = translated(p, east, north);
                        assert_eq!(source.surface_y(image), h);
                        for y in [i64::from(h - 4), i64::from(h), i64::from(h + 1)] {
                            assert_eq!(
                                source.sample(CandidateCellId::new(p.q, p.r, y)),
                                source.sample(CandidateCellId::new(image.q, image.r, y))
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn all_seam_neighbors_have_local_relief_not_a_noise_reset() {
        let source = PeriodicTerrainCandidate::new(42);
        let neighbor_offsets = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
        for start in [
            Axial64::new(0, 0),
            Axial64::new(PERIOD_Q - 1, 0),
            Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1),
            Axial64::new(R_PERIOD_Q_OFFSET + PERIOD_Q - 1, PERIOD_R - 1),
        ] {
            for (dq, dr) in neighbor_offsets {
                let neighbor = Axial64::new(start.q + dq, start.r + dr);
                let near = CandidateProjection::nearest_periodic_image(
                    start.candidate_wrap(),
                    neighbor.candidate_wrap(),
                );
                assert_eq!(near.candidate_wrap(), neighbor.candidate_wrap());
                let delta = source.surface_y(start) - source.surface_y(near);
                // With smooth periodic lattice sampling, a one-hex step near
                // either seam must not jump the heightfield by dozens of cells.
                assert!(delta.abs() <= 2, "seam height jump {delta} at {start:?}");
            }
        }
    }

    #[test]
    fn chunk_boundaries_and_order_independent_regeneration_agree() {
        let source = PeriodicTerrainCandidate::new(97);
        let chunks = [
            CandidateChunkId {
                q: -1,
                r: -1,
                y: 13,
            },
            CandidateChunkId { q: 0, r: 0, y: 13 },
            CandidateChunkId {
                q: PERIOD_Q / 16 - 1,
                r: 0,
                y: 13,
            },
            CandidateChunkId {
                q: R_PERIOD_Q_OFFSET / 16,
                r: PERIOD_R / 16 - 1,
                y: 13,
            },
        ];
        let first: Vec<_> = chunks
            .iter()
            .map(|&p| source.chunk_surface(p).unwrap())
            .collect();
        for (key, original) in chunks.iter().rev().zip(first.iter().rev()) {
            let regenerated = source.chunk_surface(*key).unwrap();
            assert_eq!(&regenerated, original);
            for r in 0..16_i64 {
                for q in 0..16_i64 {
                    let aq = key.q * 16 + q;
                    let ar = key.r * 16 + r;
                    assert_eq!(
                        regenerated[(r * 16 + q) as usize],
                        source.surface_y(Axial64::new(aq, ar))
                    );
                }
            }
        }
    }

    #[test]
    fn coarse_grid_uses_same_source_and_repeats_at_translated_origins() {
        let source = PeriodicTerrainCandidate::new(11);
        let origin = Axial64::new(-80, 97);
        let baseline = source.coarse_surface(origin, 16, 9).unwrap();
        assert_eq!(baseline, source.coarse_surface(origin, 16, 9).unwrap());
        assert_eq!(
            baseline,
            source
                .coarse_surface(translated(origin, 1, -1), 16, 9)
                .unwrap()
        );
        for r in 0..9 {
            for q in 0..9 {
                assert_eq!(
                    baseline[r * 9 + q],
                    source.surface_y(Axial64::new(
                        origin.q + q as i64 * 16,
                        origin.r + r as i64 * 16
                    ))
                );
            }
        }
    }

    #[test]
    fn seed_changes_field_and_extreme_addresses_do_not_overflow() {
        let a = PeriodicTerrainCandidate::new(42);
        let b = PeriodicTerrainCandidate::new(99);
        let mut differences = 0;
        for i in 0..100 {
            let p = Axial64::new(i * 12_345 - 23_000, i * 43_210 - 5_000);
            differences += usize::from(a.surface_y(p) != b.surface_y(p));
        }
        assert!(differences > 0);
        let extremes = [
            Axial64::new(i64::MIN, i64::MAX),
            Axial64::new(i64::MAX, i64::MIN),
        ];
        for p in extremes {
            assert_eq!(a.surface_y(p), a.surface_y(p.candidate_wrap()));
        }
        assert!(a.coarse_surface(Axial64::new(i64::MAX, 0), 16, 3).is_err());
        assert!(a.coarse_surface(Axial64::new(0, 0), 0, 3).is_err());
        assert!(a.coarse_surface(Axial64::new(0, 0), 16, 130).is_err());
        assert!(a
            .chunk_surface(CandidateChunkId {
                q: i64::MAX,
                r: 0,
                y: 0
            })
            .is_err());
    }
}
