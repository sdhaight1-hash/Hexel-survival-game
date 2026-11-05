//! A deterministic 16x16x16 demo chunk or opt-in, streamed axial hex terrain.
//!
//! This is a locally flat chunk. Geographic torus periods and playable
//! hex-lattice seam translations have NOT been chosen or implemented.

use crate::axial_geography_candidate::CandidateCellId;
use crate::generated_terrain::PrototypeTerrain;
use crate::hex_torus_candidate::Axial64;
use crate::periodic_terrain_candidate::PeriodicTerrainCandidate;
use crate::torus_frame::{local_cell_from_metres, FrameShift, GlobalCell, TorusFrame};
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
    pub pending: usize,
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
    periodic: Option<PeriodicTerrainCandidate>,
    torus_frame: Option<TorusFrame>,
    stream_center: Option<ChunkPos>,
    horizontal_radius: i32,
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
            periodic: None,
            torus_frame: None,
            stream_center: None,
            horizontal_radius: STREAM_HORIZONTAL_RADIUS,
        }
    }

    /// The original terrain source is global, even when no chunk is resident.
    /// This mode is unbounded for now, **not** the future approved torus.
    pub fn streamed(seed: u64) -> Self {
        Self {
            chunks: HashMap::new(),
            terrain: Some(PrototypeTerrain::new(seed)),
            periodic: None,
            torus_frame: None,
            stream_center: None,
            horizontal_radius: STREAM_HORIZONTAL_RADIUS,
        }
    }

    /// Runtime periodic surface with integer global origin and bounded local frame.
    /// Resident CPU/GPU sections are local caches, not saved-world identities.
    pub fn torus(seed: u64, spawn: Axial64) -> (Self, glam::DVec2) {
        let (frame, position) = TorusFrame::at_spawn(spawn);
        (
            Self {
                chunks: HashMap::new(),
                terrain: None,
                periodic: Some(PeriodicTerrainCandidate::new(seed)),
                torus_frame: Some(frame),
                stream_center: None,
                horizontal_radius: STREAM_HORIZONTAL_RADIUS,
            },
            position,
        )
    }

    /// Immutable procedural source for seed-first distant LOD workers.
    /// The worker samples the exact torus elevation source without generating
    /// resident voxel sections. No World reference crosses threads.
    pub fn distant_surface_source(&self) -> Option<(PeriodicTerrainCandidate, TorusFrame)> {
        Some((self.periodic?, self.torus_frame?))
    }

    pub fn torus_frame(&self) -> Option<TorusFrame> {
        self.torus_frame
    }

    pub fn torus_cell(&self, x: f32, z: f32) -> Option<GlobalCell> {
        let cell = local_cell_from_metres(x, z);
        self.torus_frame.map(|frame| frame.cell(cell))
    }

    /// Change the whole local coordinate frame and invalidate resident sections.
    /// Caller MUST rebuild CPU sections and ALL GPU meshes before drawing again.
    pub fn rebase_torus(&mut self, feet: &mut Vec3) -> Option<FrameShift> {
        let frame = self.torus_frame.as_mut()?;
        let shift = frame.rebase_if_needed(&mut feet.x, &mut feet.z)?;
        self.chunks.clear();
        self.stream_center = None;
        Some(shift)
    }

    fn periodic_sample(&self, source: PeriodicTerrainCandidate, cell: CellPos) -> Block {
        let global = self.torus_frame.expect("torus frame").cell(cell);
        source.sample(CandidateCellId::new(
            global.canonical.q,
            global.canonical.r,
            i64::from(cell.y),
        ))
    }

    fn periodic_chunk(&self, source: PeriodicTerrainCandidate, section: ChunkPos) -> Chunk {
        let mut chunk = Chunk::default();
        for y in 0..CHUNK_SIDE {
            for r in 0..CHUNK_SIDE {
                for q in 0..CHUNK_SIDE {
                    let cell = CellPos::new(
                        section.q * CHUNK_SIDE + q,
                        section.r * CHUNK_SIDE + r,
                        section.y * CHUNK_SIDE + y,
                    );
                    chunk.set(
                        [q as usize, r as usize, y as usize],
                        self.periodic_sample(source, cell),
                    );
                }
            }
        }
        chunk
    }

    pub fn is_streamed(&self) -> bool {
        self.terrain.is_some() || self.periodic.is_some()
    }

    /// Full-resolution resident-window radius, NOT a LOD level.
    pub fn horizontal_radius(&self) -> i32 {
        self.horizontal_radius
    }

    /// Explicit diagnostic control; normal gameplay remains radius 2.
    pub fn set_horizontal_radius(&mut self, radius: i32) {
        assert!((2..=5).contains(&radius), "render radius must be 2..=5");
        self.horizontal_radius = radius;
    }

    pub fn chunk_positions(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Loading/evicting changes *residency*, not original cell identity.
    /// A missing resident neighbor must query the deterministic base and
    /// must never be interpreted as empty terrain during meshing.
    pub fn stream_around(&mut self, player_feet: Vec3) -> StreamDelta {
        self.stream_around_budgeted(player_feet, usize::MAX)
    }

    /// Replace at most `max_new_chunks` resident CPU sections per call.
    /// Keep old sections until replacements are ready; absent terrain still
    /// samples the procedural source, including for collision and meshing.
    pub fn stream_around_budgeted(
        &mut self,
        player_feet: Vec3,
        max_new_chunks: usize,
    ) -> StreamDelta {
        if !self.is_streamed() {
            return StreamDelta::default();
        }
        let player_cell =
            CellPos::at_world_xz(player_feet.x, player_feet.z, player_feet.y.floor() as i32);
        let (center, _) = player_cell.chunk_and_local();
        let started = Instant::now();
        let mut requested = Vec::new();
        for dq in -self.horizontal_radius..=self.horizontal_radius {
            for dr in -self.horizontal_radius..=self.horizontal_radius {
                let hex_distance = dq.abs().max(dr.abs()).max((dq + dr).abs());
                if hex_distance > self.horizontal_radius {
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
        if self.stream_center == Some(center)
            && self.chunks.len() == wanted.len()
            && wanted.iter().all(|pos| self.chunks.contains_key(pos))
        {
            return StreamDelta::default();
        }
        let missing: Vec<_> = requested
            .iter()
            .map(|entry| entry.2)
            .filter(|pos| !self.chunks.contains_key(pos))
            .collect();
        let mut obsolete: Vec<_> = self
            .chunks
            .keys()
            .copied()
            .filter(|pos| !wanted.contains(pos))
            .collect();
        obsolete.sort_by_key(|pos| (pos.y, pos.r, pos.q));
        let count = missing.len().min(max_new_chunks);
        // Shrinking a window has no missing sections: evict obsolete
        // sections within the same per-tick budget rather than retaining
        // invisible old geometry forever.
        let remove_count = if missing.is_empty() {
            obsolete.len().min(max_new_chunks)
        } else {
            count
        };
        let removed: Vec<_> = obsolete.into_iter().take(remove_count).collect();
        for &pos in &removed {
            self.chunks.remove(&pos);
        }
        let mut added = Vec::with_capacity(count);
        for pos in missing.into_iter().take(count) {
            let chunk = if let Some(source) = self.terrain {
                source.generate_chunk(pos)
            } else {
                self.periodic_chunk(self.periodic.expect("periodic terrain"), pos)
            };
            self.chunks.insert(pos, chunk);
            added.push(pos);
        }
        self.stream_center = Some(center);
        StreamDelta {
            added,
            removed,
            generation_cpu_ms: started.elapsed().as_secs_f64() * 1000.0,
            pending: wanted
                .symmetric_difference(&self.chunks.keys().copied().collect())
                .count(),
        }
    }

    pub fn get(&self, pos: CellPos) -> Block {
        let (chunk, local) = pos.chunk_and_local();
        if let Some(resident) = self.chunks.get(&chunk) {
            return resident.get(local);
        }
        if let Some(source) = self.periodic {
            return self.periodic_sample(source, pos);
        }
        self.terrain.map_or(Block::Air, |source| source.sample(pos))
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
        if let Some(source) = self.periodic {
            let cell = local_cell_from_metres(x, z);
            let global = self.torus_frame.expect("torus frame").cell(cell).canonical;
            return Some((source.surface_y(global) + 1) as f32 * HEXEL_HEIGHT);
        }
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
    fn global_torus_runtime_survives_rebasing_with_same_periodic_terrain() {
        let spawn = Axial64::new(1_023_984, 1_182_384);
        let (mut world, local) = World::torus(42, spawn);
        let mut feet = Vec3::new(local.x as f32, 0.0, local.y as f32);
        feet.y = world.support_top(feet.x, feet.z).unwrap();
        world.stream_around(feet);
        assert_eq!(world.chunk_count(), 95);
        let original = world.torus_cell(feet.x, feet.z).unwrap();
        let sample = world.get(CellPos::at_world_xz(feet.x, feet.z, feet.y.floor() as i32));
        // Travel 1088m east and 928m north in local metre coordinates.
        feet.x += 1088.0;
        feet.z += 928.0;
        let before = world.torus_cell(feet.x, feet.z).unwrap();
        assert!(world.rebase_torus(&mut feet).is_some());
        let after = world.torus_cell(feet.x, feet.z).unwrap();
        assert_eq!(before.raw, after.raw);
        assert_eq!(before.canonical, after.canonical);
        assert!(feet.x.abs() < 32.0 && feet.z.abs() < 32.0);
        assert_eq!(world.chunk_count(), 0);
        assert_eq!(world.stream_around(feet).added.len(), 95);
        assert_eq!(world.chunk_count(), 95);
        assert_eq!(world.torus_cell(feet.x, feet.z).unwrap(), after);
        assert_eq!(original.canonical, spawn.candidate_wrap());
        assert!(sample.is_solid() || sample == Block::Air);
    }

    #[test]
    fn changing_render_radius_grows_and_shrinks_resident_set_without_stale_sections() {
        let mut world = World::streamed(42);
        let at = CellPos::new(7, 7, 220).center();
        world.stream_around(at);
        assert_eq!(world.chunk_count(), 95);
        world.set_horizontal_radius(4);
        let mut max_pending = 0;
        for _ in 0..310 {
            let d = world.stream_around_budgeted(at, 1);
            max_pending = max_pending.max(d.pending);
            if d.pending == 0 {
                break;
            }
        }
        assert!(max_pending >= 200);
        assert_eq!(world.chunk_count(), 305);
        world.set_horizontal_radius(2);
        for _ in 0..310 {
            let d = world.stream_around_budgeted(at, 1);
            if d.pending == 0 {
                break;
            }
        }
        // Obsolete resident sections are removed on each tick even when
        // there are no missing sections, until the exact target is reached.
        for _ in 0..305 {
            world.stream_around_budgeted(at, 1);
            if world.chunk_count() == 95 {
                break;
            }
        }
        assert_eq!(world.chunk_count(), 95);
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
    fn budgeted_streaming_replaces_one_section_per_tick_without_losing_residency() {
        let mut world = World::streamed(42);
        let origin = CellPos::new(7, 7, 220).center();
        world.stream_around(origin);
        let destination = CellPos::new(40, -20, 255).center();
        let mut exchanged = 0;
        for _ in 0..100 {
            let delta = world.stream_around_budgeted(destination, 1);
            assert!(delta.added.len() <= 1);
            assert!(delta.removed.len() <= 1);
            assert_eq!(delta.added.len(), delta.removed.len());
            assert_eq!(world.chunk_count(), 95);
            exchanged += delta.added.len();
            if delta.pending == 0 {
                break;
            }
        }
        assert!(exchanged > 1);
        assert_eq!(world.stream_around_budgeted(destination, 1).pending, 0);
        let expected = {
            let mut reference = World::streamed(42);
            reference.stream_around(destination);
            reference.chunk_positions().collect::<HashSet<_>>()
        };
        assert_eq!(world.chunk_positions().collect::<HashSet<_>>(), expected);
    }

    #[test]
    fn budgeted_streaming_can_reverse_direction_without_stale_sections() {
        let mut world = World::streamed(97);
        let origin = CellPos::new(-1, -1, 223).center();
        let destination = CellPos::new(200, -300, 223).center();
        world.stream_around(origin);
        for _ in 0..5 {
            let delta = world.stream_around_budgeted(destination, 1);
            assert_eq!(delta.added.len(), 1);
            assert_eq!(world.chunk_count(), 95);
        }
        for _ in 0..100 {
            let delta = world.stream_around_budgeted(origin, 1);
            if delta.pending == 0 {
                break;
            }
        }
        let mut reference = World::streamed(97);
        reference.stream_around(origin);
        assert_eq!(
            world.chunk_positions().collect::<HashSet<_>>(),
            reference.chunk_positions().collect::<HashSet<_>>()
        );
        for cell in [
            CellPos::new(-1, -1, 223),
            CellPos::new(0, 0, 223),
            CellPos::new(200, -300, 223),
        ] {
            assert_eq!(world.get(cell), reference.get(cell));
        }
    }

    #[test]
    fn rapid_target_changes_preserve_cpu_gpu_delta_contract_and_center_priority() {
        let mut world = World::streamed(123);
        let origin = CellPos::new(1, 1, 220).center();
        world.stream_around(origin);
        // Mirror only residency membership: real GPU entries intentionally omit
        // all-air sections, and render::sync_stream asserts mesh subset ownership.
        let mut mirrored_residents: HashSet<_> = world.chunk_positions().collect();
        let targets = [
            CellPos::new(200, -120, 245),
            CellPos::new(-80, 160, 195),
            CellPos::new(200, -120, 245),
            CellPos::new(1, 1, 220),
        ];
        for target in targets {
            let center = target.chunk_and_local().0;
            let mut previous_pending = usize::MAX;
            for tick in 0..4 {
                let center_was_missing = world.chunk(center).is_none();
                let delta = world.stream_around_budgeted(target.center(), 1);
                assert!(delta.added.len() <= 1);
                assert_eq!(delta.added.len(), delta.removed.len());
                assert!(delta.pending <= previous_pending);
                previous_pending = delta.pending;
                for pos in delta.removed {
                    assert!(mirrored_residents.remove(&pos));
                    assert!(world.chunk(pos).is_none());
                }
                for pos in delta.added {
                    assert!(mirrored_residents.insert(pos));
                    assert!(world.chunk(pos).is_some());
                    if tick == 0 && center_was_missing {
                        assert_eq!(pos, center); // nearest missing player section first
                    }
                }
                assert_eq!(mirrored_residents.len(), 95);
                assert_eq!(
                    mirrored_residents,
                    world.chunk_positions().collect::<HashSet<_>>()
                );
            }
        }
        let destination = CellPos::new(-80, 160, 195).center();
        for _ in 0..95 {
            let delta = world.stream_around_budgeted(destination, 1);
            for pos in delta.removed {
                assert!(mirrored_residents.remove(&pos));
            }
            for pos in delta.added {
                assert!(mirrored_residents.insert(pos));
            }
            if delta.pending == 0 {
                break;
            }
        }
        let mut reference = World::streamed(123);
        reference.stream_around(destination);
        assert_eq!(
            mirrored_residents,
            reference.chunk_positions().collect::<HashSet<_>>()
        );
        assert_eq!(mirrored_residents.len(), 95);
        for cell in [
            CellPos::new(-80, 160, 195),
            CellPos::new(1, 1, 220),
            CellPos::new(200, -120, 245),
        ] {
            assert_eq!(world.get(cell), reference.get(cell));
        }
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
