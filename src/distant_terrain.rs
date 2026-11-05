//! Experimental distant surface underlay. Visual only: never authoritative
//! for collision, block identity, or a replacement for near hexel meshes.
use crate::{materials_p0::Material, mesh::Vertex};
use glam::Vec3;
use hexel_survival_game::world::World;

pub const SPACING_M: i32 = 4;
pub const EXTENT_M: i32 = 176;
const UNDERLAY_OFFSET_M: f32 = 2.0;

/// Rebuild only when the player enters a different 16m local anchor square.
pub fn anchor(feet: Vec3) -> (i32, i32) {
    (
        (feet.x / 16.0).floor() as i32 * 16,
        (feet.z / 16.0).floor() as i32 * 16,
    )
}

/// A single coarse underlay beneath resident terrain. The procedural source
/// provides all heights, including outside the currently resident window.
/// Overlap is intentional: depth-tested exact near geometry hides the underlay.
/// This does not yet guarantee crack-free transitions at steep slopes.
pub fn build(world: &World, centre: (i32, i32)) -> Vec<Vertex> {
    let cells = (2 * EXTENT_M / SPACING_M) as usize;
    let mut vertices = Vec::with_capacity(cells * cells * 6);
    let mut heights = Vec::with_capacity((cells + 1) * (cells + 1));
    for r in 0..=cells {
        for q in 0..=cells {
            let x = (centre.0 - EXTENT_M + q as i32 * SPACING_M) as f32;
            let z = (centre.1 - EXTENT_M + r as i32 * SPACING_M) as f32;
            let y = world.support_top(x, z).unwrap_or(0.0) - UNDERLAY_OFFSET_M;
            heights.push(Vec3::new(x, y, z));
        }
    }
    let width = cells + 1;
    for r in 0..cells {
        for q in 0..cells {
            let a = heights[r * width + q];
            let b = heights[r * width + q + 1];
            let c = heights[(r + 1) * width + q];
            let d = heights[(r + 1) * width + q + 1];
            for p in [a, c, b, b, c, d] {
                vertices.push(Vertex {
                    position: p.to_array(),
                    color: [0.85, 0.9, 0.82],
                    uv: [p.x, p.z],
                    material: Material::SoilTopFull as u32,
                });
            }
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexel_survival_game::hex_torus_candidate::Axial64;

    #[test]
    fn stable_anchor_until_next_local_grid() {
        assert_eq!(anchor(Vec3::new(0.1, 230.0, 15.9)), (0, 0));
        assert_eq!(anchor(Vec3::new(16.0, 230.0, -0.1)), (16, -16));
    }

    #[test]
    fn generated_underlay_is_bounded_and_finite() {
        let (world, _) = World::torus(42, Axial64::new(432_776, 1_182_375));
        let mesh = build(&world, (0, 0));
        let n = (2 * EXTENT_M / SPACING_M) as usize;
        assert_eq!(mesh.len(), n * n * 6);
        assert!(mesh.iter().all(|v| v.position.iter().all(|n| n.is_finite())));
    }
}
