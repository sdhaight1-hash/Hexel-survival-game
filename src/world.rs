//! A single, deterministic 16x16x16 demo chunk on an axial hexagonal grid.
//!
//! This is a locally flat chunk. Geographic torus periods and playable
//! hex-lattice seam translations have NOT been chosen or implemented.

use glam::Vec3;
use std::collections::HashMap;

pub const CHUNK_SIDE: i32 = 16;
pub const CHUNK_VOLUME: usize = 16 * 16 * 16;
pub const HEX_RADIUS: f32 = 0.577_350_26; // 1 metre across opposing flats
pub const HEX_ROW_SPACING: f32 = 0.866_025_4;
pub const HEXEL_HEIGHT: f32 = 1.0; // authored Foliage hexel height

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellPos {
    pub q: i32,
    pub r: i32,
    pub y: i32,
}

impl CellPos {
    pub const fn new(q: i32, r: i32, y: i32) -> Self {
        Self { q, r, y }
    }

    pub fn chunk_and_local(self) -> (ChunkPos, [usize; 3]) {
        (
            ChunkPos {
                q: self.q.div_euclid(CHUNK_SIDE),
                r: self.r.div_euclid(CHUNK_SIDE),
                y: self.y.div_euclid(CHUNK_SIDE),
            },
            [
                self.q.rem_euclid(CHUNK_SIDE) as usize,
                self.r.rem_euclid(CHUNK_SIDE) as usize,
                self.y.rem_euclid(CHUNK_SIDE) as usize,
            ],
        )
    }

    pub fn neighbors(self) -> [Self; 8] {
        [
            Self::new(self.q + 1, self.r, self.y),
            Self::new(self.q + 1, self.r - 1, self.y),
            Self::new(self.q, self.r - 1, self.y),
            Self::new(self.q - 1, self.r, self.y),
            Self::new(self.q - 1, self.r + 1, self.y),
            Self::new(self.q, self.r + 1, self.y),
            Self::new(self.q, self.r, self.y + 1),
            Self::new(self.q, self.r, self.y - 1),
        ]
    }

    pub fn center(self) -> Vec3 {
        Vec3::new(
            self.q as f32 + self.r as f32 * 0.5,
            (self.y as f32 + 0.5) * HEXEL_HEIGHT,
            self.r as f32 * HEX_ROW_SPACING,
        )
    }

    pub fn at_world_xz(x: f32, z: f32, y: i32) -> Self {
        let rf = z / HEX_ROW_SPACING;
        let qf = x - rf * 0.5;
        let xf = qf;
        let zf = rf;
        let yf = -xf - zf;

        // Cube-rounding preserves valid axial coordinates near hex boundaries.
        let mut rx = xf.round();
        let mut ry = yf.round();
        let mut rz = zf.round();
        let dx = (rx - xf).abs();
        let dy = (ry - yf).abs();
        let dz = (rz - zf).abs();
        if dx > dy && dx > dz {
            rx = -ry - rz;
        } else if dy > dz {
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }
        let _ = ry;
        Self::new(rx as i32, rz as i32, y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub q: i32,
    pub r: i32,
    pub y: i32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Soil,
    Rock,
}

impl Block {
    pub const fn is_solid(self) -> bool {
        !matches!(self, Self::Air)
    }
}

#[derive(Clone)]
pub struct Chunk {
    cells: [Block; CHUNK_VOLUME],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: [Block::Air; CHUNK_VOLUME],
        }
    }
}

impl Chunk {
    fn index(local: [usize; 3]) -> usize {
        assert!(local.iter().all(|&axis| axis < CHUNK_SIDE as usize));
        (local[2] * CHUNK_SIDE as usize + local[1]) * CHUNK_SIDE as usize + local[0]
    }

    pub fn get(&self, local: [usize; 3]) -> Block {
        self.cells[Self::index(local)]
    }

    pub fn set(&mut self, local: [usize; 3], block: Block) {
        self.cells[Self::index(local)] = block;
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }
}

#[derive(Default)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    benchmark: bool,
}

impl World {
    /// Bootstrap only: exactly one authored-size playable chunk, with no wrap.
    pub fn demo(seed: u32) -> Self {
        let mut chunk = Chunk::default();
        for q in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                // P0 gallery cells are deliberately flat and identified by
                // material_lab, never by geology or an ore-generation rule.
                let fixture = crate::material_lab::sample(q, r);
                let top = if fixture.is_some() {
                    crate::material_lab::surface_y(r)
                } else {
                    demo_surface(seed, q, r)
                };
                for y in 0..=top {
                    let block = if y == top {
                        match fixture {
                            Some(crate::material_lab::LabSample::Soil(
                                crate::material_contract::CoverDensity::Bare,
                            )) => Block::Soil,
                            Some(crate::material_lab::LabSample::Rock { .. }) => Block::Rock,
                            _ => Block::Grass,
                        }
                    } else if y >= top - 2 {
                        Block::Soil
                    } else {
                        Block::Rock
                    };
                    chunk.set([q as usize, r as usize, y as usize], block);
                }
            }
        }
        let mut chunks = HashMap::new();
        chunks.insert(ChunkPos::default(), chunk);
        Self {
            chunks,
            benchmark: false,
        }
    }

    /// Optional static 20x10x1 chunk surface for visual/CPU benchmarks.
    /// This is not streaming, persisted worldgen, or a toroidal boundary.
    /// Keep the original material-reference chunk at the origin unchanged.
    pub fn benchmark(seed: u32) -> Self {
        let mut world = Self::demo(seed);
        world.benchmark = true;
        for cr in -5..5 {
            for cq in -10..10 {
                if cq == 0 && cr == 0 {
                    continue;
                }
                let mut chunk = Chunk::default();
                for r in 0..CHUNK_SIDE {
                    for q in 0..CHUNK_SIDE {
                        let global_q = cq * CHUNK_SIDE + q;
                        let global_r = cr * CHUNK_SIDE + r;
                        let (top, exposed_rock) = benchmark_surface(seed, global_q, global_r);
                        for y in 0..=top {
                            let block = if y == top {
                                if exposed_rock {
                                    Block::Rock
                                } else {
                                    Block::Grass
                                }
                            } else if y >= top - 2 {
                                Block::Soil
                            } else {
                                Block::Rock
                            };
                            chunk.set([q as usize, r as usize, y as usize], block);
                        }
                    }
                }
                world.chunks.insert(ChunkPos { q: cq, r: cr, y: 0 }, chunk);
            }
        }
        world
    }

    pub fn is_benchmark(&self) -> bool {
        self.benchmark
    }

    /// Deterministic iteration order for stable mesh and benchmark builds.
    pub fn chunk_positions(&self) -> Vec<ChunkPos> {
        let mut positions: Vec<_> = self.chunks.keys().copied().collect();
        positions.sort_by_key(|pos| (pos.r, pos.q, pos.y));
        positions
    }

    pub fn get(&self, pos: CellPos) -> Block {
        let (chunk, local) = pos.chunk_and_local();
        self.chunks
            .get(&chunk)
            .map_or(Block::Air, |chunk| chunk.get(local))
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Demo boundary, not a global world boundary or a torus implementation.
    pub fn contains_demo_footprint(&self, x: f32, z: f32) -> bool {
        let (chunk, _) = CellPos::at_world_xz(x, z, 0).chunk_and_local();
        self.chunks.contains_key(&chunk)
    }

    /// Highest solid top at this horizontal sample in the first vertical section.
    pub fn support_top(&self, x: f32, z: f32) -> Option<f32> {
        let cell = CellPos::at_world_xz(x, z, 0);
        let (chunk, local) = cell.chunk_and_local();
        let data = self.chunks.get(&chunk)?;
        (0..CHUNK_SIDE as usize)
            .rev()
            .find(|&y| data.get([local[0], local[1], y]).is_solid())
            .map(|y| (y + 1) as f32 * HEXEL_HEIGHT)
    }
}

fn benchmark_surface(seed: u32, q: i32, r: i32) -> (i32, bool) {
    let qf = q as f32;
    let rf = r as f32;
    // Low-frequency, continuous authored-test topography with large
    // rock/grass regions. No claim of realistic geology or climate.
    let phase = seed as f32 * 0.004;
    let wave = (qf * 0.036 + phase).sin() * 2.4
        + (rf * 0.053 - phase).cos() * 1.8
        + ((qf + rf) * 0.018 + phase).sin() * 1.3;
    let top = (5.0 + wave).round() as i32;
    let rock_signal = (qf * 0.029 + phase).sin()
        + (rf * 0.041).cos()
        + ((qf - rf) * 0.021).sin();
    (top.clamp(2, CHUNK_SIDE - 3), rock_signal > 0.7)
}

fn demo_surface(seed: u32, q: i32, r: i32) -> i32 {
    // A simple deterministic demonstration, NOT future tectonic generation.
    let d = (q - 5).pow(2) + (r - 10).pow(2);
    let hill = if d <= 6 {
        3
    } else if d <= 14 {
        2
    } else if d <= 23 {
        1
    } else {
        0
    };
    let ledge = i32::from((11..=14).contains(&q) && (2..=5).contains(&r));
    let variation = i32::from(seed % 3 == 0 && q > 12 && r > 12);
    (2 + hill + ledge + variation).min(CHUNK_SIDE - 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_holds_exactly_4096_cells() {
        assert_eq!(Chunk::default().len(), CHUNK_VOLUME);
        assert_eq!(CHUNK_VOLUME, 4096);
    }

    #[test]
    fn negative_coordinates_are_chunked_using_euclidean_division() {
        assert_eq!(
            CellPos::new(-1, -16, -17).chunk_and_local(),
            (
                ChunkPos {
                    q: -1,
                    r: -1,
                    y: -2
                },
                [15, 0, 15]
            )
        );
    }

    #[test]
    fn coordinates_round_trip_across_hex_centres() {
        for q in -20..=20 {
            for r in -20..=20 {
                let pos = CellPos::new(q, r, 3);
                let centre = pos.center();
                assert_eq!(CellPos::at_world_xz(centre.x, centre.z, 3), pos);
            }
        }
    }

    #[test]
    fn neighbor_offsets_are_reciprocal() {
        let origin = CellPos::new(2, -4, 8);
        for neighbor in origin.neighbors() {
            assert!(neighbor.neighbors().contains(&origin));
        }
    }

    #[test]
    fn terrain_is_deterministic_and_remains_one_chunk() {
        let a = World::demo(42);
        let b = World::demo(42);
        assert_eq!(a.chunk_count(), 1);
        for q in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                for y in 0..CHUNK_SIDE {
                    let pos = CellPos::new(q, r, y);
                    assert_eq!(a.get(pos), b.get(pos));
                }
            }
        }
        assert_eq!(a.get(CellPos::new(-1, 0, 1)), Block::Air);
    }

    #[test]
    fn p0_reference_strip_has_raised_exposed_faces_and_real_soil_below() {
        let world = World::demo(42);
        for q in 2..=14 {
            let y = crate::material_lab::surface_y(10);
            assert!(world.get(CellPos::new(q, 10, y)).is_solid());
            assert_eq!(world.get(CellPos::new(q, 10, y + 1)), Block::Air);
            assert_eq!(world.get(CellPos::new(q, 10, y - 1)), Block::Soil);
        }
        assert_eq!(world.get(CellPos::new(2, 10, 3)), Block::Soil);
        assert_eq!(world.get(CellPos::new(9, 10, 3)), Block::Grass);
        assert_eq!(world.get(CellPos::new(12, 10, 3)), Block::Rock);
        assert_eq!(world.get(CellPos::new(14, 10, 3)), Block::Rock);
    }

    #[test]
    fn benchmark_is_200_horizontal_chunks_with_stable_terrain_and_origin_gallery() {
        let a = World::benchmark(42);
        let b = World::benchmark(42);
        assert!(a.is_benchmark());
        assert!(!World::demo(42).is_benchmark());
        assert_eq!(a.chunk_count(), 200);
        assert_eq!(a.chunk_positions(), b.chunk_positions());
        assert_eq!(a.chunk_positions().len(), 200);
        assert!(a.chunk(ChunkPos::default()).is_some());
        assert!(a.chunk(ChunkPos { q: -10, r: -5, y: 0 }).is_some());
        assert!(a.chunk(ChunkPos { q: 9, r: 4, y: 0 }).is_some());
        assert!(a.chunk(ChunkPos { q: 10, r: 0, y: 0 }).is_none());
        for pos in [CellPos::new(8, 10, 3), CellPos::new(14, 10, 3),
            CellPos::new(-45, 18, 5), CellPos::new(88, -42, 3)] {
            assert_eq!(a.get(pos), b.get(pos));
        }
        let reference = World::demo(42);
        for q in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                for y in 0..CHUNK_SIDE {
                    let p = CellPos::new(q, r, y);
                    assert_eq!(a.get(p), reference.get(p));
                }
            }
        }
    }

    #[test]
    fn benchmark_chunks_are_not_independent_islands() {
        let world = World::benchmark(42);
        for (q, r) in [(15, 8), (16, 8), (-1, 0), (0, -1), (80, 40)] {
            let c = CellPos::new(q, r, 0);
            assert!(world.get(c).is_solid());
            assert!(world.contains_demo_footprint(c.center().x, c.center().z));
            assert!(world.support_top(c.center().x, c.center().z).is_some());
        }
    }

    #[test]
    fn authored_hexel_dimensions_are_explicit() {
        assert!((HEX_RADIUS * 3.0_f32.sqrt() - 1.0).abs() < 0.0001);
        assert_eq!(HEXEL_HEIGHT, 1.0);
    }
}
