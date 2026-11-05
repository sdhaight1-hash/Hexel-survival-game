//! Opt-in V5.5 gallery: deterministic per-hexel palette with narrow
//! solid-only edge blending. Not a gameplay/geology ownership system.
use glam::Vec3;
use hexel_survival_game::world::{CellPos, World};

const DIRECTIONS: [(i32, i32); 6] =
    [(0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];
// Five RGB multipliers per rock; these are pigment, not lighting.
const GRANITE: [[f32; 3]; 5] = [
    [1.02, 1.00, 0.98], [1.19, 0.96, 0.88], [0.88, 1.00, 1.17],
    [1.17, 1.09, 0.91], [0.84, 0.90, 1.03],
];
const SHALE: [[f32; 3]; 5] = [
    [1.0, 1.0, 1.0], [1.17, 1.03, 0.9], [0.87, 1.01, 1.18],
    [0.82, 0.91, 0.91], [1.15, 1.13, 0.96],
];
const BASALT: [[f32; 3]; 5] = [
    [1.0, 1.0, 1.0], [1.18, 1.06, 0.97], [0.88, 0.97, 1.17],
    [1.18, 1.17, 1.09], [0.82, 0.90, 0.94],
];
const BLEND_WIDTH: f32 = 0.14; // metres, measured *inside* the exposed face

fn palette_hash(cell: CellPos, pad: u32) -> u32 {
    let rock = pad / 4;
    // All gallery stages for the same rock show the same palette pattern.
    let q = cell.q - (pad % 4) as i32 * 12;
    let mut h = (q as u32).wrapping_mul(0x9e3779b9)
        ^ (cell.r as u32).wrapping_mul(0x85ebca6b)
        ^ (cell.y as u32).wrapping_mul(0xc2b2ae35)
        ^ rock.wrapping_mul(0x27d4eb2d);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h
}

fn palette_index(cell: CellPos, pad: u32) -> u32 {
    palette_hash(cell, pad) % 5
}

/// Stable local-cell pattern seed shared across gallery A/B/C/D stages.
pub fn pattern_seed(world: &World, cell: CellPos) -> u32 {
    world.gallery_rock_index(cell).map_or(0, |pad| palette_hash(cell, pad))
}

fn palette(cell: CellPos, pad: u32) -> [f32; 3] {
    let idx = palette_index(cell, pad) as usize;
    match pad / 4 {
        0 => GRANITE[idx],
        1 => SHALE[idx],
        _ => BASALT[idx],
    }
}

pub fn raw_id(world: &World, cell: CellPos) -> u32 {
    world.gallery_rock_index(cell)
        .map_or(0, |pad| palette_index(cell, pad))
}

fn same_solid_rock(world: &World, pos: CellPos, pad: u32) -> bool {
    world.get(pos).is_solid() && world.gallery_rock_index(pos) == Some(pad)
}

fn weight(distance: f32) -> f32 {
    let t = (1.0 - distance / BLEND_WIDTH).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The cell owns the center of its visible faces. Only a narrow region close
/// to a shared border uses a neighbor's palette. Air, water, empty gaps and
/// other pads are not palette contributors. Evaluating both sides of a join
/// at distance zero yields the same weighted color.
pub fn tint_and_pattern_weight_at(world: &World, cell: CellPos, position: Vec3) -> ([f32; 3], f32) {
    let Some(pad) = world.gallery_rock_index(cell) else {
        return ([1.0; 3], 1.0);
    };
    let neutral_control = pad % 4 == 0;
    let mut total = if neutral_control { [1.0; 3] } else { palette(cell, pad) };
    let mut weight_sum = 1.0_f32;
    let mut strongest_contact = 0.0_f32;
    let center = cell.center();

    for (index, &(dq, dr)) in DIRECTIONS.iter().enumerate() {
        let neighbor = CellPos::new(cell.q + dq, cell.r + dr, cell.y);
        if !same_solid_rock(world, neighbor, pad) { continue; }
        let theta = (60.0 + index as f32 * 60.0).to_radians();
        let along = (position.x - center.x) * theta.cos()
            + (position.z - center.z) * theta.sin();
        let w = weight((0.5 - along).max(0.0));
        strongest_contact = strongest_contact.max(w);
        if w > 0.0 && !neutral_control {
            let tint = palette(neighbor, pad);
            for channel in 0..3 { total[channel] += tint[channel] * w; }
            weight_sum += w;
        }
    }
    for (y, distance) in [
        (cell.y - 1, position.y - cell.y as f32),
        (cell.y + 1, (cell.y + 1) as f32 - position.y),
    ] {
        let neighbor = CellPos::new(cell.q, cell.r, y);
        if !same_solid_rock(world, neighbor, pad) { continue; }
        let w = weight(distance.max(0.0));
        strongest_contact = strongest_contact.max(w);
        if w > 0.0 && !neutral_control {
            let tint = palette(neighbor, pad);
            for channel in 0..3 { total[channel] += tint[channel] * w; }
            weight_sum += w;
        }
    }
    for channel in &mut total { *channel /= weight_sum; }
    (total, 1.0 - strongest_contact)
}

pub fn tint_at(world: &World, cell: CellPos, position: Vec3) -> [f32; 3] {
    tint_and_pattern_weight_at(world, cell, position).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn micro_pattern_seed_is_stable_across_gallery_stages() {
        let world = World::rock_v5_gallery();
        let a = CellPos::new(12, 2, 1);
        let b = CellPos::new(24, 2, 1);
        let c = CellPos::new(36, 2, 1);
        assert_eq!(pattern_seed(&world, a), pattern_seed(&world, b));
        assert_eq!(pattern_seed(&world, b), pattern_seed(&world, c));
    }

    #[test]
    fn micro_pattern_fades_only_at_a_solid_contact() {
        let world = World::rock_v5_gallery();
        let a = CellPos::new(14, 2, 2);
        let b = CellPos::new(14, 3, 2);
        let joined = (a.center() + b.center()) * 0.5;
        assert!(tint_and_pattern_weight_at(&world, a, joined).1 < 0.01);
        assert!(tint_and_pattern_weight_at(&world, b, joined).1 < 0.01);
        assert_eq!(tint_and_pattern_weight_at(&world, a, a.center()).1, 1.0);
        let exterior = CellPos::new(12, 0, 2);
        let edge = exterior.center() + Vec3::new(0.0, 0.0, -0.5);
        assert_eq!(tint_and_pattern_weight_at(&world, exterior, edge).1, 1.0);
    }

    #[test]
    fn five_palettes_are_deterministic() {
        let mut ids = std::collections::HashSet::new();
        for q in 12..22 {
            for r in 0..10 {
                let cell = CellPos::new(q, r, 1);
                let pad = 1;
                assert_eq!(palette_index(cell, pad), palette_index(cell, pad));
                ids.insert(palette_index(cell, pad));
            }
        }
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn interior_keeps_own_palette_and_open_edge_never_blends() {
        let world = World::rock_v5_gallery();
        let c = CellPos::new(12, 0, 2);
        assert_eq!(tint_at(&world, c, c.center()), palette(c, 1));
        // Exterior edge, with no solid neighbors on that side.
        let p = c.center() + Vec3::new(0.0, 0.0, -0.5);
        assert_eq!(tint_at(&world, c, p), palette(c, 1));
    }

    #[test]
    fn shared_vertical_boundary_is_equal_from_both_cells() {
        let world = World::rock_v5_gallery();
        let a = CellPos::new(14, 2, 1);
        let b = CellPos::new(14, 2, 2);
        let pos = Vec3::new(a.center().x, 2.0, a.center().z);
        let a_tint = tint_at(&world, a, pos);
        let b_tint = tint_at(&world, b, pos);
        for c in 0..3 { assert!((a_tint[c] - b_tint[c]).abs() < 1e-6); }
    }

    #[test]
    fn shared_horizontal_edge_is_equal_from_both_cells() {
        let world = World::rock_v5_gallery();
        let a = CellPos::new(14, 2, 2);
        let b = CellPos::new(14, 3, 2);
        let pos = (a.center() + b.center()) * 0.5;
        let ca = tint_at(&world, a, pos);
        let cb = tint_at(&world, b, pos);
        for c in 0..3 { assert!((ca[c] - cb[c]).abs() < 1e-5); }
    }

    #[test]
    fn stage_a_is_untinted() {
        let world = World::rock_v5_gallery();
        let c = CellPos::new(0, 0, 1);
        assert_eq!(tint_at(&world, c, c.center()), [1.0; 3]);
    }
}
