//! Five deterministic per-rock palette families with hexel-neighbor edge averaging.
//! Visual-only: rock identity and ore state stay authoritative in the world.
//! The gallery uses axial cell coordinates; production must use canonical torus IDs.
use hexel_survival_game::world::{CellPos, World};

const NEIGHBORS: [(i32, i32); 6] =
    [(0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];

// RGB multipliers applied to the rock's dark/light grayscale palette.
// Index is deterministic, not RNG or transient graphics state.
const GRANITE: [[f32; 3]; 5] = [
    [1.02, 1.00, 0.99], // neutral warm gray
    [1.09, 0.95, 0.90], // pale rose
    [0.95, 1.00, 1.07], // cool quartz gray
    [1.13, 1.05, 0.92], // warm feldspar
    [0.88, 0.91, 0.97], // darker mica-rich region
];
const SHALE: [[f32; 3]; 5] = [
    [1.00, 1.00, 1.00], // neutral blue-gray
    [1.09, 1.02, 0.94], // warmer silt
    [0.93, 1.01, 1.09], // cool slate
    [0.89, 0.94, 0.91], // deeper carbon-rich
    [1.09, 1.08, 1.00], // pale silty
];
const BASALT: [[f32; 3]; 5] = [
    [1.00, 1.00, 1.00], // charcoal
    [1.09, 1.04, 1.00], // warm charcoal
    [0.93, 0.98, 1.08], // blue-black
    [1.10, 1.11, 1.06], // weathered gray
    [0.88, 0.93, 0.94], // dark dense basalt
];

fn hash(q: i32, r: i32, y: i32, rock: u32) -> usize {
    let mut h = (q as u32).wrapping_mul(0x9E37_79B9)
        ^ (r as u32).wrapping_mul(0x85EB_CA6B)
        ^ (y as u32).wrapping_mul(0xC2B2_AE35)
        ^ rock.wrapping_mul(0x27D4_EB2D);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    (h % 5) as usize
}

fn cell_palette(cell: CellPos, pad: u32) -> [f32; 3] {
    let rock = pad / 4;
    // All four stages of a rock have identical local palette-seed patterns.
    let local_q = cell.q - (pad % 4) as i32 * 12;
    let index = hash(local_q, cell.r, cell.y, rock);
    match rock {
        0 => GRANITE[index],
        1 => SHALE[index],
        _ => BASALT[index],
    }
}

/// At a hex corner, up to three horizontal cells meet. At an integer Y
/// boundary, both vertically adjacent cells can contribute. A neighbor
/// contributes only if it is an actual solid pad cell of the same material.
/// Air/gaps and unrelated pad materials are *never* blended.
pub fn vertex_tint(
    world: &World,
    cell: CellPos,
    corner: Option<usize>,
    boundary_y: i32,
) -> [f32; 3] {
    if !world.is_rock_v5() {
        return [1.0; 3];
    }
    let Some(pad) = world.gallery_rock_index(cell) else {
        return [1.0; 3];
    };
    // Stage A is deliberately untouched for a meaningful control.
    if pad % 4 == 0 {
        return [1.0; 3];
    }
    let mut sum = [0.0_f32; 3];
    let mut count = 0;
    let mut candidates = [(cell.q, cell.r); 3];
    let lateral_count = if let Some(i) = corner {
        let before = NEIGHBORS[(i + 5) % 6];
        let after = NEIGHBORS[i % 6];
        candidates[1] = (cell.q + before.0, cell.r + before.1);
        candidates[2] = (cell.q + after.0, cell.r + after.1);
        3
    } else {
        1
    };
    for &(q, r) in &candidates[..lateral_count] {
        for y in [boundary_y - 1, boundary_y] {
            let neighbor = CellPos::new(q, r, y);
            if world.gallery_rock_index(neighbor) != Some(pad)
                || !world.get(neighbor).is_solid()
            {
                continue;
            }
            let rgb = cell_palette(neighbor, pad);
            for channel in 0..3 {
                sum[channel] += rgb[channel];
            }
            count += 1;
        }
    }
    // This is possible only if the caller passes an unrelated boundary.
    if count == 0 {
        return cell_palette(cell, pad);
    }
    for channel in &mut sum {
        *channel /= count as f32;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_selection_is_deterministic_and_five_way() {
        let mut seen = std::collections::HashSet::new();
        for q in 0..60 {
            seen.insert(hash(q, 2, 1, 0));
        }
        assert_eq!(seen.len(), 5);
        assert_eq!(hash(7, 3, 2, 1), hash(7, 3, 2, 1));
    }

    #[test]
    fn empty_neighbors_do_not_contribute_at_outer_boundary() {
        let world = World::rock_v5_gallery();
        let cell = CellPos::new(12, 0, 2); // granite, palette-enabled pad B
        let corner = vertex_tint(&world, cell, Some(3), 3);
        let center = vertex_tint(&world, cell, None, 3);
        // An outside corner has no contributors beyond this rock pad.
        assert_eq!(corner, center);
    }

    #[test]
    fn base_control_and_walkway_are_neutral() {
        let world = World::rock_v5_gallery();
        assert_eq!(vertex_tint(&world, CellPos::new(0, 0, 2), None, 3), [1.0; 3]);
        assert_eq!(vertex_tint(&world, CellPos::new(5, 13, 0), None, 1), [1.0; 3]);
    }
}
