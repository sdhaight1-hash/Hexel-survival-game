//! Build exposed hex-prism faces from simulation-owned world cells.
//!
//! This GPU-ready mesh is derived/cached data, never world authority.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use hexel_survival_game::world::{Block, CellPos, ChunkPos, World, CHUNK_SIDE, HEX_RADIUS};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// Edges i -> i+1 for the regular hexagon with first corner at 30 degrees.
// Each direction is the cell whose occupancy hides that edge.
const SIDE_NEIGHBORS: [(i32, i32); 6] =
    [(0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];

fn triangle(vertices: &mut Vec<Vertex>, a: Vec3, b: Vec3, c: Vec3, color: [f32; 3]) {
    for point in [a, b, c] {
        vertices.push(Vertex {
            position: point.to_array(),
            color,
        });
    }
}

fn color(block: Block, top: bool, side: usize) -> [f32; 3] {
    let base: [f32; 3] = match block {
        Block::Air => [0.0, 0.0, 0.0],
        Block::Grass if top => [0.32, 0.67, 0.22],
        Block::Grass => [0.37, 0.34, 0.19],
        Block::Soil => [0.42, 0.29, 0.18],
        Block::Rock => [0.46, 0.48, 0.51],
    };
    let shade = if top { 1.12 } else { 0.65 + side as f32 * 0.055 };
    base.map(|channel| (channel * shade).min(1.0))
}

/// Neighbor-aware exposed-face mesh for one 16³ section.
///
/// Neighbor queries go through the world, not an isolated chunk, so an eventual
/// multi-chunk renderer can cull shared faces without changing this API.
pub fn build_chunk_mesh(world: &World, chunk: ChunkPos) -> Vec<Vertex> {
    if world.chunk(chunk).is_none() {
        return Vec::new();
    }
    let mut vertices = Vec::new();
    for y in 0..CHUNK_SIDE {
        for r in 0..CHUNK_SIDE {
            for q in 0..CHUNK_SIDE {
                let cell = CellPos::new(
                    chunk.q * CHUNK_SIDE + q,
                    chunk.r * CHUNK_SIDE + r,
                    chunk.y * CHUNK_SIDE + y,
                );
                let block = world.get(cell);
                if !block.is_solid() {
                    continue;
                }

                let center = cell.center();
                let bottom = cell.y as f32;
                let top = bottom + 1.0;
                let ring: [Vec3; 6] = std::array::from_fn(|i| {
                    let angle = (30.0 + i as f32 * 60.0).to_radians();
                    Vec3::new(
                        center.x + HEX_RADIUS * angle.cos(),
                        0.0,
                        center.z + HEX_RADIUS * angle.sin(),
                    )
                });

                if !world.get(CellPos::new(cell.q, cell.r, cell.y + 1)).is_solid() {
                    let cap = Vec3::new(center.x, top, center.z);
                    for i in 0..6 {
                        let j = (i + 1) % 6;
                        triangle(
                            &mut vertices,
                            cap,
                            Vec3::new(ring[i].x, top, ring[i].z),
                            Vec3::new(ring[j].x, top, ring[j].z),
                            color(block, true, 0),
                        );
                    }
                }

                if !world.get(CellPos::new(cell.q, cell.r, cell.y - 1)).is_solid() {
                    let cap = Vec3::new(center.x, bottom, center.z);
                    for i in 0..6 {
                        let j = (i + 1) % 6;
                        triangle(
                            &mut vertices,
                            cap,
                            Vec3::new(ring[j].x, bottom, ring[j].z),
                            Vec3::new(ring[i].x, bottom, ring[i].z),
                            color(block, false, 0),
                        );
                    }
                }

                for (side, &(dq, dr)) in SIDE_NEIGHBORS.iter().enumerate() {
                    if world.get(CellPos::new(cell.q + dq, cell.r + dr, cell.y)).is_solid() {
                        continue;
                    }
                    let j = (side + 1) % 6;
                    let a = Vec3::new(ring[side].x, bottom, ring[side].z);
                    let b = Vec3::new(ring[j].x, bottom, ring[j].z);
                    let c = Vec3::new(ring[j].x, top, ring[j].z);
                    let d = Vec3::new(ring[side].x, top, ring[side].z);
                    let shade = color(block, false, side);
                    triangle(&mut vertices, a, b, c, shade);
                    triangle(&mut vertices, a, c, d, shade);
                }
            }
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_contains_triangles_for_only_the_loaded_chunk() {
        let world = World::demo(42);
        let vertices = build_chunk_mesh(&world, ChunkPos::default());
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 3, 0);
        assert!(vertices.iter().all(|v| {
            v.position.iter().all(|x| x.is_finite())
                && v.color.iter().all(|x| (0.0..=1.0).contains(x))
        }));
        assert!(build_chunk_mesh(&world, ChunkPos { q: 1, r: 0, y: 0 }).is_empty());
    }
}
