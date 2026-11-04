//! First terrain-source groundwork for a full-address-space world.
//!
//! Terrain is sampled from absolute axial Q/R/Y coordinates and a world seed,
//! not from chunk-local RNG. Any chunk can be generated independently, in any
//! order, without allocating the rest of the planet or needing a GPU adapter.
//!
//! This intentionally does NOT implement a toroidal hex-lattice seam mapping,
//! streamed residency, persistent edits, geology, caves, or biome simulation.
//! Those are separate gates before this becomes live game terrain.

use crate::world::{Block, CellPos, Chunk, ChunkPos, CHUNK_SIDE};

/// Changes to the shape algorithm must be explicitly versioned before any
/// generated terrain or player edits are saved.
pub const PROTOTYPE_TERRAIN_REVISION: u32 = 1;

/// A pure deterministic generator. No mutable RNG or loaded-chunk state.
#[derive(Debug, Clone, Copy)]
pub struct PrototypeTerrain {
    seed: u64,
}

impl PrototypeTerrain {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Provisional elevation in 1-metre hexel layers. The final vertical
    /// mapping and geology are not designed yet; sea level 220 is a planned
    /// default, not a simulated ocean level here.
    pub fn surface_y(&self, q: i32, r: i32) -> i32 {
        let regional = self.value_noise(q, r, 256, 0xB1A0_0001);
        let local = self.value_noise(q, r, 48, 0xB1A0_0002);
        220 + ((regional - 0.5) * 48.0 + (local - 0.5) * 12.0).round() as i32
    }

    /// Stable original terrain for any absolute hex-lattice cell, independent
    /// of resident sections. A persisted edit layer must override this later.
    pub fn sample(&self, cell: CellPos) -> Block {
        let top = self.surface_y(cell.q, cell.r);
        if cell.y > top {
            Block::Air
        } else if cell.y == top {
            Block::Grass
        } else if cell.y >= top - 3 {
            Block::Soil
        } else {
            Block::Rock
        }
    }

    /// Materialize only the requested 16³ chunk. Does not depend on or mutate
    /// any neighboring chunk, so generation order cannot affect its contents.
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let base_q = pos.q.checked_mul(CHUNK_SIDE).expect("chunk Q exceeds i32 cell range");
        let base_r = pos.r.checked_mul(CHUNK_SIDE).expect("chunk R exceeds i32 cell range");
        let base_y = pos.y.checked_mul(CHUNK_SIDE).expect("chunk Y exceeds i32 cell range");
        let mut chunk = Chunk::default();
        for y in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                for q in 0..CHUNK_SIDE {
                    let cell = CellPos::new(
                        base_q.checked_add(q).expect("cell Q outside i32 range"),
                        base_r.checked_add(r).expect("cell R outside i32 range"),
                        base_y.checked_add(y).expect("cell Y outside i32 range"),
                    );
                    chunk.set([q as usize, r as usize, y as usize], self.sample(cell));
                }
            }
        }
        chunk
    }

    fn value_noise(&self, q: i32, r: i32, span: i32, domain: u64) -> f64 {
        let gx = q.div_euclid(span) as i64;
        let gz = r.div_euclid(span) as i64;
        let fx = f64::from(q.rem_euclid(span)) / f64::from(span);
        let fz = f64::from(r.rem_euclid(span)) / f64::from(span);
        let u = fx * fx * (3.0 - 2.0 * fx);
        let v = fz * fz * (3.0 - 2.0 * fz);
        let low = lerp(
            self.lattice(gx, gz, domain),
            self.lattice(gx + 1, gz, domain),
            u,
        );
        let high = lerp(
            self.lattice(gx, gz + 1, domain),
            self.lattice(gx + 1, gz + 1, domain),
            u,
        );
        lerp(low, high, v)
    }

    fn lattice(&self, q: i64, r: i64, domain: u64) -> f64 {
        let input = self.seed
            ^ domain
            ^ (q as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (r as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
        let mut bits = input.wrapping_add(0x9E37_79B9_7F4A_7C15);
        bits = (bits ^ (bits >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        bits = (bits ^ (bits >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        bits ^= bits >> 31;
        (bits >> 11) as f64 / (1_u64 << 53) as f64
    }
}

fn lerp(a: f64, b: f64, weight: f64) -> f64 {
    a + (b - a) * weight
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_chunk_matches_absolute_cells(generator: &PrototypeTerrain, section: ChunkPos) {
        let chunk = generator.generate_chunk(section);
        for y in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                for q in 0..CHUNK_SIDE {
                    let local = [q as usize, r as usize, y as usize];
                    let absolute = CellPos::new(
                        section.q * CHUNK_SIDE + q,
                        section.r * CHUNK_SIDE + r,
                        section.y * CHUNK_SIDE + y,
                    );
                    assert_eq!(chunk.get(local), generator.sample(absolute));
                }
            }
        }
    }

    #[test]
    fn absolute_sampling_matches_arbitrary_horizontal_and_vertical_sections() {
        let generator = PrototypeTerrain::new(42);
        // Origin; both sides of q/r/y boundaries; negative sections and
        // far-away addresses roughly a thousand kilometres from origin.
        for pos in [
            ChunkPos { q: 0, r: 0, y: 13 },
            ChunkPos { q: 1, r: 1, y: 14 },
            ChunkPos { q: -1, r: -1, y: 13 },
            ChunkPos { q: 1, r: -1, y: 15 },
            ChunkPos { q: 62_500, r: -62_500, y: 13 },
            ChunkPos { q: -62_500, r: 62_500, y: 15 },
        ] {
            assert_chunk_matches_absolute_cells(&generator, pos);
        }
    }

    #[test]
    fn generation_order_and_repeated_loads_are_identical() {
        let generator = PrototypeTerrain::new(42);
        let sections = [
            ChunkPos { q: -1, r: 0, y: 13 },
            ChunkPos { q: 0, r: 0, y: 13 },
            ChunkPos { q: 0, r: 1, y: 14 },
            ChunkPos { q: 1, r: 1, y: 14 },
            ChunkPos { q: 6_400, r: -9_000, y: 13 },
        ];
        let original: Vec<_> = sections
            .iter()
            .map(|&pos| generator.generate_chunk(pos))
            .collect();
        for (pos, expected) in sections.into_iter().rev().zip(original.iter().rev()) {
            let regenerated = generator.generate_chunk(pos);
            for y in 0..CHUNK_SIDE as usize {
                for r in 0..CHUNK_SIDE as usize {
                    for q in 0..CHUNK_SIDE as usize {
                        assert_eq!(regenerated.get([q, r, y]), expected.get([q, r, y]));
                    }
                }
            }
        }
    }

    #[test]
    fn exact_boundaries_are_not_independently_seeded() {
        let generator = PrototypeTerrain::new(99);
        for (q, r) in [(15, 15), (16, 16), (-1, -1), (0, 0), (1_000_015, -1_000_016)] {
            let elevation = generator.surface_y(q, r);
            for y in [elevation - 3, elevation - 1, elevation, elevation + 1] {
                let cell = CellPos::new(q, r, y);
                let (section, local) = cell.chunk_and_local();
                assert_eq!(
                    generator.generate_chunk(section).get(local),
                    generator.sample(cell),
                );
            }
        }
    }

    #[test]
    fn seed_is_part_of_world_terrain_identity() {
        let first = PrototypeTerrain::new(42);
        let same = PrototypeTerrain::new(42);
        let different = PrototypeTerrain::new(99);
        let positions = [(0, 0), (255, -1), (-1_020, 531), (500_000, -500_000)];
        assert!(positions.iter().all(|&(q, r)| first.surface_y(q, r) == same.surface_y(q, r)));
        assert!(positions.iter().any(|&(q, r)| first.surface_y(q, r) != different.surface_y(q, r)));
        assert_eq!(PROTOTYPE_TERRAIN_REVISION, 1);
    }
}
