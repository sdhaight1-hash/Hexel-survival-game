//! Mesh hexagonal-prism caps and sides with per-face 32x32 material UVs.
//! This derived geometry does not own world/material resource state.

use crate::materials::Material;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use hexel_survival_game::world::{Block, CellPos, ChunkPos, World, CHUNK_SIDE, HEX_RADIUS};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
    pub material: u32,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Uint32];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// Six side edges, ordered counter-clockwise from a 30-degree corner.
const SIDE_NEIGHBORS: [(i32, i32); 6] = [(0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];

fn triangle(
    vertices: &mut Vec<Vertex>,
    points: [(Vec3, [f32; 2]); 3],
    material: Material,
    color: [f32; 3],
) {
    for (position, uv) in points {
        vertices.push(Vertex {
            position: position.to_array(),
            color,
            uv,
            material: material as u32,
        });
    }
}

// The image is a square, but the projected top/bottom surface is a real hex.
fn cap_uv(position: Vec3, center: Vec3) -> [f32; 2] {
    [
        0.5 + (position.x - center.x) / (2.0 * HEX_RADIUS),
        0.5 + (position.z - center.z) / (2.0 * HEX_RADIUS),
    ]
}

fn material_for(block: Block, cell: CellPos, top: bool) -> Material {
    match block {
        // This forest-floor patch is a visual demo, not ecology or biome data.
        Block::Grass if top && cell.q < 5 && cell.r >= 10 => Material::ForestFloor,
        Block::Grass if top => Material::LowSoil,
        Block::Grass | Block::Soil => Material::MediumSoil,
        Block::Rock => Material::Granite,
        Block::Air => unreachable!("Only solid cells generate faces"),
    }
}

/// Meshes visible faces of a section. Neighbor checks query the world, so
/// future adjacent chunks can hide the shared faces correctly.
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
                let uv = |pos| cap_uv(pos, center);
                if !world
                    .get(CellPos::new(cell.q, cell.r, cell.y + 1))
                    .is_solid()
                {
                    let cap = Vec3::new(center.x, top, center.z);
                    for i in 0..6 {
                        let j = (i + 1) % 6;
                        let a = Vec3::new(ring[i].x, top, ring[i].z);
                        let b = Vec3::new(ring[j].x, top, ring[j].z);
                        triangle(
                            &mut vertices,
                            [(cap, uv(cap)), (a, uv(a)), (b, uv(b))],
                            material_for(block, cell, true),
                            [1.0, 1.0, 1.0],
                        );
                    }
                }
                if !world
                    .get(CellPos::new(cell.q, cell.r, cell.y - 1))
                    .is_solid()
                {
                    let cap = Vec3::new(center.x, bottom, center.z);
                    for i in 0..6 {
                        let j = (i + 1) % 6;
                        let a = Vec3::new(ring[j].x, bottom, ring[j].z);
                        let b = Vec3::new(ring[i].x, bottom, ring[i].z);
                        triangle(
                            &mut vertices,
                            [(cap, uv(cap)), (a, uv(a)), (b, uv(b))],
                            material_for(block, cell, false),
                            [0.77, 0.77, 0.77],
                        );
                    }
                }
                for (side, &(dq, dr)) in SIDE_NEIGHBORS.iter().enumerate() {
                    if world
                        .get(CellPos::new(cell.q + dq, cell.r + dr, cell.y))
                        .is_solid()
                    {
                        continue;
                    }
                    let j = (side + 1) % 6;
                    let a = Vec3::new(ring[side].x, bottom, ring[side].z);
                    let b = Vec3::new(ring[j].x, bottom, ring[j].z);
                    let c = Vec3::new(ring[j].x, top, ring[j].z);
                    let d = Vec3::new(ring[side].x, top, ring[side].z);
                    let shade = 0.82 + side as f32 * 0.025;
                    let material = material_for(block, cell, false);
                    let tint = [shade, shade, shade];
                    triangle(
                        &mut vertices,
                        [(a, [0.0, 1.0]), (b, [1.0, 1.0]), (c, [1.0, 0.0])],
                        material,
                        tint,
                    );
                    triangle(
                        &mut vertices,
                        [(a, [0.0, 1.0]), (c, [1.0, 0.0]), (d, [0.0, 0.0])],
                        material,
                        tint,
                    );
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
    fn generated_hex_faces_have_valid_materials_and_uvs() {
        let world = World::demo(42);
        let vertices = build_chunk_mesh(&world, ChunkPos::default());
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 3, 0);
        assert!(vertices.iter().all(|v| {
            v.position.iter().all(|x| x.is_finite())
                && v.color.iter().all(|x| (0.0..=1.0).contains(x))
                && v.uv.iter().all(|x| (0.0..=1.0).contains(x))
                && v.material < crate::materials::TEXTURES.len() as u32
        }));
        assert!(build_chunk_mesh(&world, ChunkPos { q: 1, r: 0, y: 0 }).is_empty());
    }

    #[test]
    fn demo_surface_soil_and_rock_have_distinct_materials() {
        let c = CellPos::new(8, 8, 0);
        assert_eq!(material_for(Block::Grass, c, true), Material::LowSoil);
        assert_eq!(material_for(Block::Grass, c, false), Material::MediumSoil);
        assert_eq!(material_for(Block::Rock, c, false), Material::Granite);
        assert_eq!(
            material_for(Block::Grass, CellPos::new(2, 12, 0), true),
            Material::ForestFloor
        );
    }
}
