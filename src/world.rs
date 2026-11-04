//! A deterministic 16x16x16 demo chunk or opt-in, streamed axial hex terrain.
//!
//! This is a locally flat chunk. Geographic torus periods and playable
//! hex-lattice seam translations have NOT been chosen or implemented.

use crate::generated_terrain::PrototypeTerrain;
use glam::Vec3;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

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

/// Changes to the resident CPU working set; used for matching GPU uploads.
#[derive(Default, Debug)]
pub struct StreamDelta {
    pub added: Vec<ChunkPos>,
    pub removed: Vec<ChunkPos>,
    pub generation_cpu_ms: f64,
}

/// Opt-in first streamed-world prototype: 19 hexagonal Q/R columns and five
/// Y sections, centered on the player's section. This is a bounded resident
/// window, not the world extent or a render-distance promise.
pub const STREAM_HORIZONTAL_RADIUS: i32 = 2;
pub const STREAM_VERTICAL_RADIUS: i32 = 2;

#[derive(Default)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    terrain: Option<PrototypeTerrain>,
    stream_center: Option<ChunkPos>,
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
            terrain: None,
            stream_center: None,
        }
    }

    /// The original terrain source is global, even when no chunk is resident.
    /// This mode is unbounded for now, **not** the future approved torus.
    pub fn streamed(seed: u64) -> Self {
        Self {
            chunks: HashMap::new(),
            terrain: Some(PrototypeTerrain::new(seed)),
            stream_center: None,
        }
    }

    pub fn is_streamed(&self) -> bool {
        self.terrain.is_some()
    }

    pub fn chunk_positions(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Loading/evicting changes *residency*, not original cell identity.
    /// A missing resident neighbor must query the deterministic base and
    /// must never be interpreted as empty terrain during meshing.
    pub fn stream_around(&mut self, player_feet: Vec3) -> StreamDelta {
        let Some(source) = self.terrain else {
            return StreamDelta::default();
        };
        let player_cell =
            CellPos::at_world_xz(player_feet.x, player_feet.z, player_feet.y.floor() as i32);
        let (center, _) = player_cell.chunk_and_local();
        if self.stream_center == Some(center) {
            return StreamDelta::default();
        }
        let started = Instant::now();
        let mut requested = Vec::new();
        for dq in -STREAM_HORIZONTAL_RADIUS..=STREAM_HORIZONTAL_RADIUS {
            for dr in -STREAM_HORIZONTAL_RADIUS..=STREAM_HORIZONTAL_RADIUS {
                let hex_distance = dq.abs().max(dr.abs()).max((dq + dr).abs());
                if hex_distance > STREAM_HORIZONTAL_RADIUS {
                    continue;
                }
                for dy in -STREAM_VERTICAL_RADIUS..=STREAM_VERTICAL_RADIUS {
                    requested.push((
                        hex_distance,
                        dy.abs(),
                        ChunkPos {
                            q: center.q + dq,
                            r: center.r + dr,
                            y: center.y + dy,
                        },
                    ));
                }
            }
        }
        // Center-first, deterministic generation order. No randomized
        // per-chunk geology or dependence on iteration order of HashMap.
        requested.sort_by_key(|(distance, height, pos)| (*distance, *height, pos.y, pos.q, pos.r));
        let wanted: HashSet<_> = requested.iter().map(|entry| entry.2).collect();
        let mut removed: Vec<_> = self
            .chunks
            .keys()
            .copied()
            .filter(|pos| !wanted.contains(pos))
            .collect();
        removed.sort_by_key(|pos| (pos.y, pos.r, pos.q));
        self.chunks.retain(|pos, _| wanted.contains(pos));
        let mut added = Vec::new();
        for (_, _, pos) in requested {
            if let std::collections::hash_map::Entry::Vacant(entry) = self.chunks.entry(pos) {
                entry.insert(source.generate_chunk(pos));
                added.push(pos);
            }
        }
        self.stream_center = Some(center);
        StreamDelta {
            added,
            removed,
            generation_cpu_ms: started.elapsed().as_secs_f64() * 1000.0,
        }
    }

    pub fn get(&self, pos: CellPos) -> Block {
        let (chunk, local) = pos.chunk_and_local();
        self.chunks.get(&chunk).map_or_else(
            || self.terrain.map_or(Block::Air, |source| source.sample(pos)),
            |chunk| chunk.get(local),
        )
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Demo boundary, not a global world boundary or a torus implementation.
    pub fn contains_demo_footprint(&self, x: f32, z: f32) -> bool {
        if self.is_streamed() {
            return true; // no artificial demo barrier; topology is still provisional
        }
        let (chunk, _) = CellPos::at_world_xz(x, z, 0).chunk_and_local();
        self.chunks.contains_key(&chunk)
    }

    /// Highest solid top at this horizontal sample in the first vertical section.
    pub fn support_top(&self, x: f32, z: f32) -> Option<f32> {
        if let Some(source) = self.terrain {
            let cell = CellPos::at_world_xz(x, z, 0);
            return Some((source.surface_y(cell.q, cell.r) + 1) as f32 * HEXEL_HEIGHT);
        }
        let cell = CellPos::at_world_xz(x, z, 0);
        let (chunk, local) = cell.chunk_and_local();
        let data = self.chunks.get(&chunk)?;
        (0..CHUNK_SIDE as usize)
            .rev()
            .find(|&y| data.get([local[0], local[1], y]).is_solid())
            .map(|y| (y + 1) as f32 * HEXEL_HEIGHT)
    }
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
    fn streamed_window_remains_bounded_and_follows_horizontal_and_vertical_sections() {
        let mut world = World::streamed(42);
        let feet = CellPos::new(7, 7, 220).center();
        let first = world.stream_around(feet);
        assert_eq!(first.added.len(), 95); // 19 axial hex columns × five Y sections
        assert!(first.removed.is_empty());
        assert_eq!(world.chunk_count(), 95);
        assert!(world.stream_around(feet).added.is_empty());
        let moved = world.stream_around(CellPos::new(40, -20, 255).center());
        assert!(!moved.added.is_empty());
        assert!(!moved.removed.is_empty());
        assert_eq!(world.chunk_count(), 95);
        assert_eq!(world.chunk_positions().collect::<HashSet<_>>().len(), 95);
    }

    #[test]
    fn unresident_neighbors_are_real_base_terrain_not_false_air() {
        let source = PrototypeTerrain::new(42);
        let mut world = World::streamed(42);
        let at = CellPos::new(15, 16, source.surface_y(15, 16));
        assert!(world.get(at).is_solid());
        world.stream_around(at.center());
        let chunk_before = world.get(at);
        world.stream_around(CellPos::new(160, 160, 220).center());
        assert_eq!(world.get(at), chunk_before);
        assert_eq!(world.get(at), source.sample(at));
        assert!(world.contains_demo_footprint(-200.0, 200.0));
        assert!(world.support_top(-200.0, 200.0).is_some());
    }

    #[test]
    fn streaming_revisit_restores_unchanged_original_chunks() {
        let mut world = World::streamed(97);
        let origin = CellPos::new(-1, -1, 223).center();
        world.stream_around(origin);
        let first: Vec<_> = (-16..=16)
            .flat_map(|q| (-16..=16).map(move |r| CellPos::new(q, r, 220)))
            .map(|p| world.get(p))
            .collect();
        world.stream_around(CellPos::new(200, -300, 223).center());
        world.stream_around(origin);
        let second: Vec<_> = (-16..=16)
            .flat_map(|q| (-16..=16).map(move |r| CellPos::new(q, r, 220)))
            .map(|p| world.get(p))
            .collect();
        assert_eq!(first, second);
    }

    #[test]
    fn authored_hexel_dimensions_are_explicit() {
        assert!((HEX_RADIUS * 3.0_f32.sqrt() - 1.0).abs() < 0.0001);
        assert_eq!(HEXEL_HEIGHT, 1.0);
    }
}
