//! Mesh hexagonal-prism caps and sides with per-face physically scaled material UVs.
//! This derived geometry does not own world/material resource state.

use crate::materials_p0;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use hexel_survival_game::material_contract::HexFace;
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
    material: u32,
    color: [f32; 3],
) {
    for (position, uv) in points {
        vertices.push(Vertex {
            position: position.to_array(),
            color,
            uv,
            material,
        });
    }
}

// The image is a square, but the projected top/bottom surface is a real hex.
// Top/bottom caps use X/Z measured in metres. They are real hexagons,
// but sampling them requires no hexagonal image file or cube UV layout.
fn cap_uv(position: Vec3) -> [f32; 2] {
    [position.x, position.z]
}

/// Meshes visible faces of a section. Neighbor checks query the world, so
/// future adjacent chunks can hide the shared faces correctly.
/// Temporary red east/west and green north/south seam guides, placed using
/// the current integer global frame (never a fixed northeast-world origin).
pub fn torus_guides(frame: hexel_survival_game::torus_frame::TorusFrame) -> Vec<Vertex> {
    let (east, north) = hexel_survival_game::torus_frame::nearest_seams(frame);
    let mut out = Vec::new();
    let marker = materials_p0::Material::Checker as u32;
    let quad = |out: &mut Vec<Vertex>, p: [Vec3; 4], tint: [f32; 3]| {
        for v in [p[0], p[1], p[2], p[0], p[2], p[3]] {
            out.push(Vertex {
                position: v.to_array(),
                color: tint,
                uv: [v.x * 0.06, v.y * 0.06],
                material: marker,
            });
        }
    };
    let y0 = 175.0;
    let y1 = 285.0;
    if east.abs() < 140.0 {
        quad(
            &mut out,
            [
                Vec3::new(east, y0, -130.0),
                Vec3::new(east, y0, 130.0),
                Vec3::new(east, y1, 130.0),
                Vec3::new(east, y1, -130.0),
            ],
            [1.0, 0.12, 0.08],
        );
    }
    if north.abs() < 140.0 {
        quad(
            &mut out,
            [
                Vec3::new(-130.0, y0, north),
                Vec3::new(130.0, y0, north),
                Vec3::new(130.0, y1, north),
                Vec3::new(-130.0, y1, north),
            ],
            [0.13, 1.0, 0.15],
        );
    }
    out
}

pub fn build_chunk_mesh(world: &World, chunk: ChunkPos) -> Vec<Vertex> {
    build_chunk_mesh_relative(world, chunk, None)
}

/// In the opt-in distant spawn, build all resident meshes relative to the
/// same integer axial origin before converting positions to f32. Never add
/// 0.58m hex corners to million-metre f32 coordinates.
pub fn build_chunk_mesh_relative(
    world: &World,
    chunk: ChunkPos,
    origin: Option<(i32, i32)>,
) -> Vec<Vertex> {
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
                let center = if let Some((origin_q, origin_r)) = origin {
                    // Use i64 subtraction to avoid i32 overflow and preserve
                    // the full metre / half-metre detail at distant spawns.
                    let dq = i64::from(cell.q) - i64::from(origin_q);
                    let dr = i64::from(cell.r) - i64::from(origin_r);
                    Vec3::new(
                        dq as f32 + dr as f32 * 0.5,
                        (cell.y as f32 + 0.5),
                        dr as f32 * hexel_survival_game::world::HEX_ROW_SPACING,
                    )
                } else {
                    cell.center()
                };
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
                let uv = cap_uv;
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
                            world.gallery_rock_index(cell).map(|i| i * 3).unwrap_or_else(|| if world.is_rock_gallery() { 69 } else { materials_p0::for_face(block, cell, HexFace::Top) as u32 }),
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
                            world.gallery_rock_index(cell).map(|i| i * 3 + 2).unwrap_or_else(|| if world.is_rock_gallery() { 69 } else { materials_p0::for_face(block, cell, HexFace::Bottom) as u32 }),
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
                    // V5 isolates texture continuity from P0's per-side orientation tint.
                    let shade = if world.is_rock_v5() { 1.0 } else { 0.88 + side as f32 * 0.018 };
                    let material = world.gallery_rock_index(cell).map(|i| i * 3 + 1).unwrap_or_else(|| if world.is_rock_gallery() { 69 } else { materials_p0::for_face(block, cell, HexFace::Side(side as u8)) as u32 });
                    let tint = [shade, shade, shade];
                    // One side is 0.577m wide x 1m tall. Use metres
                    // around the hex perimeter, NOT 0..1 across each side.
                    // Half-texel inset vertically prevents the top-edge
                    // grass rim from wrapping onto the block bottom.
                    let u0 = side as f32 * HEX_RADIUS;
                    let u1 = (side as f32 + 1.0) * HEX_RADIUS;
                    let half_texel = 0.5 / materials_p0::TEXTURE_SIZE as f32;
                    let top_v = half_texel;
                    let bottom_v = 1.0 - half_texel;
                    triangle(
                        &mut vertices,
                        [(a, [u0, bottom_v]), (b, [u1, bottom_v]), (c, [u1, top_v])],
                        material,
                        tint,
                    );
                    triangle(
                        &mut vertices,
                        [(a, [u0, bottom_v]), (c, [u1, top_v]), (d, [u0, top_v])],
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
    use crate::materials_p0::Material;

    #[test]
    fn generated_hex_faces_have_valid_materials_and_uvs() {
        let world = World::demo(42);
        let vertices = build_chunk_mesh(&world, ChunkPos::default());
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 3, 0);
        assert!(vertices.iter().all(|v| {
            v.position.iter().all(|x| x.is_finite())
                && v.color.iter().all(|x| (0.0..=1.0).contains(x))
                && v.uv.iter().all(|x| x.is_finite())
                && v.material < materials_p0::LAYER_COUNT as u32
        }));
        assert!(build_chunk_mesh(&world, ChunkPos { q: 1, r: 0, y: 0 }).is_empty());
    }

    #[test]
    fn distant_origin_keeps_hex_corners_distinct_and_shared_across_chunks() {
        let mut world = World::streamed(42);
        let origin = CellPos::new(1_023_984, 0, 220).center();
        world.stream_around(origin);
        let near = CellPos::at_world_xz(origin.x, origin.z, 220);
        let (section, _) = near.chunk_and_local();
        let vertices = build_chunk_mesh_relative(&world, section, Some((1_023_984, 0)));
        assert!(!vertices.is_empty());
        assert!(vertices.iter().all(|v| v.position[0].abs() < 100.0));
        // A top cap's six triangles must retain their distinct adjacent
        // vertices, rather than snapping to coarse global-f32 increments.
        let mut distinct = std::collections::HashSet::new();
        for vertex in &vertices {
            distinct.insert((vertex.position[0].to_bits(), vertex.position[2].to_bits()));
        }
        assert!(distinct.len() > 100);
    }

    #[test]
    fn side_uv_span_respects_physical_hex_edge_length() {
        let edge = HEX_RADIUS;
        let u0 = 2.0 * edge;
        let u1 = 3.0 * edge;
        let texels = (u1 - u0) * materials_p0::REFERENCE_TEXELS_PER_METRE;
        assert!((texels - 147.8).abs() < 0.1);
    }

    #[test]
    fn preview_materials_vary_by_face_role_and_host() {
        let soil = CellPos::new(9, 10, 3);
        assert_eq!(
            materials_p0::for_face(Block::Grass, soil, HexFace::Top),
            Material::SoilTopFull
        );
        assert_eq!(
            materials_p0::for_face(Block::Grass, soil, HexFace::Side(0)),
            Material::SoilSideFull
        );
        assert_eq!(
            materials_p0::for_face(Block::Grass, soil, HexFace::Bottom),
            Material::SoilBare
        );
        let ore = CellPos::new(14, 10, 3);
        assert_eq!(
            materials_p0::for_face(Block::Rock, ore, HexFace::Top),
            Material::ShaleCopper
        );
    }
}
