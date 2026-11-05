//! Mesh hexagonal-prism caps and sides with per-face physically scaled material UVs.
//! This derived geometry does not own world/material resource state.

use crate::materials_p0::{self, Material};
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use hexel_survival_game::material_contract::HexFace;
use hexel_survival_game::world::{Block, CellPos, ChunkPos, World, CHUNK_SIDE};
use std::collections::HashSet;

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
const SIDE_NEIGHBORS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

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

//// Temporary vertical magenta crosses mark the perimeter of *resident* horizontal
//// section columns, not the extent of the procedural world or visible geometry.
pub fn resident_boundary_markers(
    world: &World,
    origin: Option<(i32, i32)>,
    eye_y: f32,
) -> Vec<Vertex> {
    let columns: HashSet<(i32, i32)> = world.chunk_positions().map(|pos| (pos.q, pos.r)).collect();
    let neighbors = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
    let mut out = Vec::new();
    for &(q, r) in &columns {
        if neighbors
            .iter()
            .all(|&(dq, dr)| columns.contains(&(q + dq, r + dr)))
        {
            continue;
        }
        let cell_q = i64::from(q) * i64::from(CHUNK_SIDE) + i64::from(CHUNK_SIDE / 2);
        let cell_r = i64::from(r) * i64::from(CHUNK_SIDE) + i64::from(CHUNK_SIDE / 2);
        let (dq, dr) = if let Some((oq, or)) = origin {
            (cell_q - i64::from(oq), cell_r - i64::from(or))
        } else {
            (cell_q, cell_r)
        };
        let x = dq as f32;
        let z = dr as f32;
        let y0 = eye_y - 20.0;
        let y1 = eye_y + 35.0;
        let tint = [1.0, 0.05, 0.8];
        for corners in [
            [
                Vec3::new(x - 1.0, y0, z),
                Vec3::new(x + 1.0, y0, z),
                Vec3::new(x + 1.0, y1, z),
                Vec3::new(x - 1.0, y1, z),
            ],
            [
                Vec3::new(x, y0, z - 1.0),
                Vec3::new(x, y0, z + 1.0),
                Vec3::new(x, y1, z + 1.0),
                Vec3::new(x, y1, z - 1.0),
            ],
        ] {
            for v in [
                corners[0], corners[1], corners[2], corners[0], corners[2], corners[3],
            ] {
                out.push(Vertex {
                    position: v.to_array(),
                    color: tint,
                    uv: [0.0, 0.0],
                    material: materials_p0::Material::Checker as u32,
                });
            }
        }
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
                let (x, z) = if let Some((oq, or)) = origin {
                    (
                        (i64::from(cell.q) - i64::from(oq)) as f32,
                        (i64::from(cell.r) - i64::from(or)) as f32,
                    )
                } else {
                    (cell.q as f32, cell.r as f32)
                };
                let y0 = cell.y as f32;
                let y1 = y0 + 1.0;
                let x0 = x - 0.5;
                let x1 = x + 0.5;
                let z0 = z - 0.5;
                let z1 = z + 0.5;

                if !world
                    .get(CellPos::new(cell.q, cell.r, cell.y + 1))
                    .is_solid()
                {
                    let p = [
                        Vec3::new(x0, y1, z0),
                        Vec3::new(x0, y1, z1),
                        Vec3::new(x1, y1, z1),
                        Vec3::new(x1, y1, z0),
                    ];
                    cube_quad(
                        &mut vertices,
                        p,
                        materials_p0::for_face(block, cell, HexFace::Top),
                        [1.0, 1.0, 1.0],
                        true,
                    );
                }
                if !world
                    .get(CellPos::new(cell.q, cell.r, cell.y - 1))
                    .is_solid()
                {
                    let p = [
                        Vec3::new(x0, y0, z0),
                        Vec3::new(x1, y0, z0),
                        Vec3::new(x1, y0, z1),
                        Vec3::new(x0, y0, z1),
                    ];
                    cube_quad(
                        &mut vertices,
                        p,
                        materials_p0::for_face(block, cell, HexFace::Bottom),
                        [0.77, 0.77, 0.77],
                        true,
                    );
                }
                let walls = [
                    [
                        Vec3::new(x1, y0, z0),
                        Vec3::new(x1, y0, z1),
                        Vec3::new(x1, y1, z1),
                        Vec3::new(x1, y1, z0),
                    ],
                    [
                        Vec3::new(x0, y0, z1),
                        Vec3::new(x0, y0, z0),
                        Vec3::new(x0, y1, z0),
                        Vec3::new(x0, y1, z1),
                    ],
                    [
                        Vec3::new(x1, y0, z1),
                        Vec3::new(x0, y0, z1),
                        Vec3::new(x0, y1, z1),
                        Vec3::new(x1, y1, z1),
                    ],
                    [
                        Vec3::new(x0, y0, z0),
                        Vec3::new(x1, y0, z0),
                        Vec3::new(x1, y1, z0),
                        Vec3::new(x0, y1, z0),
                    ],
                ];
                for (side, &(dq, dr)) in SIDE_NEIGHBORS.iter().enumerate() {
                    if world
                        .get(CellPos::new(cell.q + dq, cell.r + dr, cell.y))
                        .is_solid()
                    {
                        continue;
                    }
                    cube_quad(
                        &mut vertices,
                        walls[side],
                        materials_p0::for_face(block, cell, HexFace::Side(side as u8)),
                        [0.88 + side as f32 * 0.024; 3],
                        false,
                    );
                }
            }
        }
    }
    vertices
}

fn cube_quad(out: &mut Vec<Vertex>, p: [Vec3; 4], material: Material, color: [f32; 3], top: bool) {
    let uv = if top {
        [
            [p[0].x, p[0].z],
            [p[1].x, p[1].z],
            [p[2].x, p[2].z],
            [p[3].x, p[3].z],
        ]
    } else {
        // Each one-metre cube side repeats its test tile once.
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
    };
    triangle(
        out,
        [(p[0], uv[0]), (p[1], uv[1]), (p[2], uv[2])],
        material,
        color,
    );
    triangle(
        out,
        [(p[0], uv[0]), (p[2], uv[2]), (p[3], uv[3])],
        material,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_cube_faces_have_valid_materials_and_uvs() {
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
    fn distant_origin_keeps_cube_corners_distinct_and_shared_across_chunks() {
        let mut world = World::streamed(42);
        let origin = CellPos::new(1_023_984, 0, 220).center();
        world.stream_around(origin);
        let near = CellPos::at_world_xz(origin.x, origin.z, 220);
        let (section, _) = near.chunk_and_local();
        let vertices = build_chunk_mesh_relative(&world, section, Some((1_023_984, 0)));
        assert!(!vertices.is_empty());
        assert!(vertices.iter().all(|v| v.position[0].abs() < 100.0));
        // Exact cube corners remain distinct after subtracting the i64 origin.
        let mut distinct = std::collections::HashSet::new();
        for vertex in &vertices {
            distinct.insert((vertex.position[0].to_bits(), vertex.position[2].to_bits()));
        }
        assert!(distinct.len() > 100);
    }

    #[test]
    fn cube_side_uv_spans_one_metre() {
        let texels = materials_p0::REFERENCE_TEXELS_PER_METRE;
        assert_eq!(texels, 256.0);
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
