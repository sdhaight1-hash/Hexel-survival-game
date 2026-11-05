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
pub const PERIODIC_TERRAIN_CANDIDATE_REVISION: u32 = 4;

/// Elevation is sampled from the same periodic raster at either near or
/// coarse resolution. The 32,768² raster is addressed but never allocated.
#[derive(Clone, Copy, Debug)]
pub struct PeriodicTerrainCandidate {
    seed: u64,
}

/// Deterministic source-of-truth for one vertical hex column. Allows a
/// resident 16x16x16 section to generate its cells without resampling the
/// same height for each of its sixteen vertical blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnProfile {
    pub top: i32,
    pub cover: Block,
    pub deep: Block,
}

/// Seed-derived local terrain diagnostics. Material slots are
/// Grass, Rock, Shale, Gravel, Bare Soil (surface only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainSurvey {
    pub min_height: i32,
    pub max_height: i32,
    pub materials: [u32; 5],
}

impl ColumnProfile {
    pub fn sample(self, y: i64) -> Block {
        if y > i64::from(self.top) {
            Block::Air
        } else if y == i64::from(self.top) {
            self.cover
        } else if y >= i64::from(self.top - 3) {
            Block::Soil
        } else {
            self.deep
        }
    }
}

impl PeriodicTerrainCandidate {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// World-owned regional structure. Fields repeat on the canonical torus,
    /// with coherent lowland, plateau and mountain domains instead of a
    /// globally uniform stack of noise amplitudes.
    pub fn surface_y(&self, pos: Axial64) -> i32 {
        let (x, z) = Self::raster_coords(pos);
        let axis = f64::from(crate::geography::ELEVATION_SAMPLES_PER_AXIS);
        let macro_height = self.periodic_noise_at(x, z, 64, 0xE2A0_0001);
        let uplift = self.periodic_noise_at(x, z, 256, 0xE2A0_0002);
        let plateau_mask = self.periodic_noise_at(x, z, 128, 0xE2A0_0003);
        let basin = smoothstep(0.57, 0.80, 1.0 - macro_height);
        let mountain = smoothstep(0.48, 0.75, uplift) * (1.0 - basin * 0.80);
        let plateau = smoothstep(0.62, 0.83, plateau_mask) * (1.0 - mountain) * (1.0 - basin);

        // Domain warping bends mountain ridgelines without breaking torus
        // periodicity. The warp is evaluated from a lower-frequency field.
        let wx = (self.periodic_noise_at(x, z, 512, 0xE2A0_0004) - 0.5) * 20.0;
        let wz = (self.periodic_noise_at(x, z, 512, 0xE2A0_0005) - 0.5) * 20.0;
        let rx = (x + wx).rem_euclid(axis);
        let rz = (z + wz).rem_euclid(axis);
        let ridge_signal = self.periodic_noise_at(rx, rz, 1024, 0xE2A0_0006);
        let ridge = (1.0 - (ridge_signal * 2.0 - 1.0).abs()).powi(3);
        let foothills = self.periodic_noise_at(rx, rz, 2048, 0xE2A0_0007);
        let rolling = self.periodic_noise_at(x, z, 4096, 0xE2A0_0008);
        let details = self.periodic_noise_at(x, z, 8192, 0xE2A0_0009);
        // Narrow depression corridors suggest drainage and connected
        // valleys. This is analytical *valley structure*, not a claim of
        // simulated hydraulic erosion or validated river flow.
        let channel = 1.0 - (self.periodic_noise_at(rx, rz, 2048, 0xE2A0_000A) * 2.0 - 1.0).abs();
        let valley = channel.powi(10);

        let base = 205.0 + (macro_height - 0.5) * 190.0 + (uplift - 0.5) * 58.0;
        let hills = (foothills - 0.5) * 88.0 + (rolling - 0.5) * 32.0 + (details - 0.5) * 9.0;
        let plains = 0.23 + 0.77 * (1.0 - basin);
        // Flatten local relief continuously: never quantize the regional
        // height to bands, which would create artificial 20m seam walls.
        let plateau_level = base + 42.0;
        let elevation =
            base + hills * plains * (1.0 - plateau * 0.83) + mountain * (68.0 + ridge * 320.0)
                - valley * (22.0 + mountain * 65.0)
                - basin * 36.0;
        lerp(elevation, plateau_level + hills * 0.12, plateau * 0.85).round() as i32
    }

    fn raster_coords(pos: Axial64) -> (f64, f64) {
        let canonical = pos.candidate_wrap();
        let axis = f64::from(crate::geography::ELEVATION_SAMPLES_PER_AXIS);
        let x = ((canonical.q as f64 + canonical.r as f64 * 0.5)
            / crate::hex_torus_candidate::PERIOD_Q as f64
            * axis)
            .rem_euclid(axis);
        let z = (canonical.r as f64 / crate::hex_torus_candidate::PERIOD_R as f64 * axis)
            .rem_euclid(axis);
        (x, z)
    }

    fn geology_at(&self, pos: Axial64) -> f64 {
        let (x, z) = Self::raster_coords(pos);
        0.65 * self.periodic_noise_at(x, z, 512, 0xE2A0_0011)
            + 0.35 * self.periodic_noise_at(x, z, 4096, 0xE2A0_0012)
    }

    /// Near blocks and distant LOD use the same region-aware geology.
    /// Distinct rock hosts and gravel deposits make terrain transitions
    /// visible without adding permanent art assets.
    pub fn surface_cover(&self, pos: Axial64) -> Block {
        self.cover_at_height(pos, self.surface_y(pos))
    }

    fn cover_at_height(&self, pos: Axial64, top: i32) -> Block {
        let mut steepest = 0;
        for (dq, dr) in [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)] {
            let neighbor = self.surface_y(Axial64::new(pos.q + dq, pos.r + dr));
            steepest = steepest.max((top - neighbor).abs());
        }
        let geology = self.geology_at(pos);
        if top >= 355 || steepest >= 5 {
            if geology > 0.51 {
                Block::Shale
            } else {
                Block::Rock
            }
        } else if top >= 305 || steepest >= 3 {
            if geology < 0.38 {
                Block::Gravel
            } else if geology > 0.57 {
                Block::Shale
            } else {
                Block::Rock
            }
        } else {
            // Grass is a covering, not the identity of all exposed ground.
            // A slow regional field controls moisture; finer periodic patches
            // create connected bare earth, dry/sparse cover, and stony soil.
            // All coordinates use the canonical torus raster, so unloaded
            // cells and distant geometry agree with resident sections.
            let (x, z) = Self::raster_coords(pos);
            let moisture = self.periodic_noise_at(x, z, 4096, 0xE2A0_0021);
            let patch = self.periodic_noise_at(x, z, 32768, 0xE2A0_0022);
            let detail = self.periodic_noise_at(x, z, 65536, 0xE2A0_0023);
            let cover = moisture * 0.48 + patch * 0.40 + detail * 0.12;
            let thin_soil = geology < 0.39 || (top >= 265 && geology < 0.46);
            if thin_soil && cover < 0.48 {
                Block::Gravel
            } else if cover < 0.46 {
                Block::Soil
            } else if cover < 0.55 {
                Block::GrassSparse
            } else if moisture < 0.49 {
                Block::GrassDry
            } else {
                Block::Grass
            }
        }
    }

    fn deep_block(&self, pos: Axial64) -> Block {
        if self.geology_at(pos) <= 0.52 {
            Block::Rock
        } else {
            Block::Shale
        }
    }

    /// Coarse diagnostic sampling, independent of resident sections/LOD.
    /// A fixed seed and torus-equivalent centre produce identical statistics.
    pub fn survey_region(&self, center: Axial64, radius: i64, step: i64) -> TerrainSurvey {
        assert!(radius > 0 && step > 0 && radius <= 16_384);
        let mut result = TerrainSurvey {
            min_height: i32::MAX,
            max_height: i32::MIN,
            materials: [0; 5],
        };
        let mut dr = -radius;
        while dr <= radius {
            let mut dq = -radius;
            while dq <= radius {
                let pos = Axial64::new(center.q + dq, center.r + dr);
                let column = self.column_profile(pos);
                result.min_height = result.min_height.min(column.top);
                result.max_height = result.max_height.max(column.top);
                let slot = match column.cover {
                    Block::Grass | Block::GrassDry | Block::GrassSparse => 0,
                    Block::Rock => 1,
                    Block::Shale => 2,
                    Block::Gravel => 3,
                    Block::Soil => 4,
                    Block::Air => unreachable!("surface must be solid"),
                };
                result.materials[slot] += 1;
                dq += step;
            }
            dr += step;
        }
        result
    }

    pub fn column_profile(&self, pos: Axial64) -> ColumnProfile {
        let top = self.surface_y(pos);
        ColumnProfile {
            top,
            cover: self.cover_at_height(pos, top),
            deep: self.deep_block(pos),
        }
    }

    /// Unloaded neighbors, collision and mesh visibility use the same
    /// column rules as resident section generation, but do not compute
    /// expensive neighboring slopes for air or deep underground rock.
    pub fn sample(&self, pos: CandidateCellId) -> Block {
        let horizontal = Axial64::new(pos.q, pos.r);
        let top = self.surface_y(horizontal);
        if pos.y > i64::from(top) {
            Block::Air
        } else if pos.y == i64::from(top) {
            self.cover_at_height(horizontal, top)
        } else if pos.y >= i64::from(top - 3) {
            Block::Soil
        } else {
            self.deep_block(horizontal)
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
    use crate::axial_geography_candidate::CandidateProjection;
    use crate::hex_torus_candidate::{PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET};

    fn translated(p: Axial64, east: i64, north: i64) -> Axial64 {
        Axial64::new(
            p.q + east * PERIOD_Q + north * R_PERIOD_Q_OFFSET,
            p.r + north * PERIOD_R,
        )
    }

    #[test]
    fn surface_cover_includes_natural_bare_soil_patches() {
        let source = PeriodicTerrainCandidate::new(42);
        let center = Axial64::new(432_776, 1_182_375);
        let survey = source.survey_region(center, 512, 32);
        assert!(
            survey.materials[4] > 0,
            "expected some bare soil among sampled ground"
        );
        assert!(survey.materials[0] > 0, "expected some vegetated ground");
        assert_eq!(
            survey,
            source.survey_region(Axial64::new(center.q + PERIOD_Q, center.r), 512, 32)
        );
    }

    #[test]
    fn terrain_survey_is_deterministic_and_torus_equivalent() {
        let source = PeriodicTerrainCandidate::new(42);
        let at = Axial64::new(432_776, 1_182_375);
        let baseline = source.survey_region(at, 512, 64);
        assert!(baseline.max_height >= baseline.min_height);
        assert_eq!(baseline.materials.iter().sum::<u32>(), 17 * 17);
        assert_eq!(
            baseline,
            source.survey_region(Axial64::new(at.q + PERIOD_Q, at.r), 512, 64)
        );
    }

    #[test]
    fn cached_column_profile_matches_unloaded_sampling_including_torus_seams() {
        let source = PeriodicTerrainCandidate::new(42);
        for cell in [
            Axial64::new(432_776, 1_182_375),
            Axial64::new(0, 0),
            Axial64::new(PERIOD_Q - 1, 0),
            Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1),
        ] {
            let profile = source.column_profile(cell);
            for offset in [-30, -4, -3, -1, 0, 1, 20] {
                let y = i64::from(profile.top + offset);
                assert_eq!(
                    profile.sample(y),
                    source.sample(CandidateCellId::new(cell.q, cell.r, y))
                );
            }
            for image in [
                Axial64::new(cell.q + PERIOD_Q, cell.r),
                Axial64::new(cell.q + R_PERIOD_Q_OFFSET, cell.r + PERIOD_R),
            ] {
                assert_eq!(source.column_profile(image), profile);
            }
        }
    }

    #[test]
    fn surface_cover_follows_authoritative_top() {
        let source = PeriodicTerrainCandidate::new(42);
        let p = Axial64::new(432_776, 1_182_375);
        let y = source.surface_y(p);
        assert_eq!(
            source.sample(CandidateCellId::new(p.q, p.r, y as i64)),
            source.surface_cover(p)
        );
    }

    #[test]
    fn mountain_belts_reach_substantial_relief_without_a_height_cap() {
        let source = PeriodicTerrainCandidate::new(42);
        let start = Axial64::new(432_776, 1_182_375);
        let mut lowest = i32::MAX;
        let mut highest = i32::MIN;
        for dq in (-8_192..=8_192).step_by(256) {
            for dr in (-8_192..=8_192).step_by(256) {
                let y = source.surface_y(Axial64::new(start.q + dq, start.r + dr));
                lowest = lowest.min(y);
                highest = highest.max(y);
            }
        }
        assert!(
            highest - lowest >= 120,
            "mountain belt too flat: {}m",
            highest - lowest
        );
        assert!(highest > 320, "no substantial highland peak: {highest}m");
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
