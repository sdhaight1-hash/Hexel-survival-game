//! V5.9 gallery-only five-family ownership and contact interpolation.
//! Each family is a *set of colors*, authored in the fragment shader.
//! This module supplies only family identities and neighbor weights.
use glam::Vec3;
use hexel_survival_game::world::{CellPos, World};

const DIRECTIONS: [(i32, i32); 6] =
    [(0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (1, 0)];
const BLEND_WIDTH: f32 = 0.14;

fn palette_index(cell: CellPos, pad: u32) -> usize {
    let rock = pad / 4;
    let q = cell.q - (pad % 4) as i32 * 12;
    let mut h = (q as u32).wrapping_mul(0x9e3779b9)
        ^ (cell.r as u32).wrapping_mul(0x85ebca6b)
        ^ (cell.y as u32).wrapping_mul(0xc2b2ae35)
        ^ rock.wrapping_mul(0x27d4eb2d);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    (h % 5) as usize
}

pub fn raw_id(world: &World, cell: CellPos) -> u32 {
    world.gallery_rock_index(cell).map_or(0, |pad| palette_index(cell, pad) as u32)
}

fn blend_weight(distance: f32) -> f32 {
    let t = (1.0 - distance / BLEND_WIDTH).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn same_pad(world: &World, pos: CellPos, pad: u32) -> bool {
    world.get(pos).is_solid() && world.gallery_rock_index(pos) == Some(pad)
}

/// Barycentric/interpolated GPU attributes encode five *family weights*.
/// No primary RGB tint is generated or multiplied by the shader.
/// Solid contacts mix complete families; exposed air edges do not.
pub fn family_weights_at(world: &World, cell: CellPos, position: Vec3) -> [f32; 5] {
    let Some(pad) = world.gallery_rock_index(cell) else {
        return [0.0; 5];
    };
    let mut weights = [0.0_f32; 5];
    weights[palette_index(cell, pad)] = 1.0;
    let mut total = 1.0_f32;
    let center = cell.center();

    for (index, &(dq, dr)) in DIRECTIONS.iter().enumerate() {
        let neighbor = CellPos::new(cell.q + dq, cell.r + dr, cell.y);
        if !same_pad(world, neighbor, pad) { continue; }
        let theta = (60.0 + index as f32 * 60.0).to_radians();
        let along = (position.x - center.x) * theta.cos()
            + (position.z - center.z) * theta.sin();
        let w = blend_weight((0.5 - along).max(0.0));
        if w > 0.0 {
            weights[palette_index(neighbor, pad)] += w;
            total += w;
        }
    }

    for (y, distance) in [
        (cell.y - 1, position.y - cell.y as f32),
        (cell.y + 1, (cell.y + 1) as f32 - position.y),
    ] {
        let neighbor = CellPos::new(cell.q, cell.r, y);
        if !same_pad(world, neighbor, pad) { continue; }
        let w = blend_weight(distance.max(0.0));
        if w > 0.0 {
            weights[palette_index(neighbor, pad)] += w;
            total += w;
        }
    }

    for v in &mut weights { *v /= total; }
    weights
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: &[f32; 5], b: &[f32; 5]) {
        for i in 0..5 { assert!((a[i] - b[i]).abs() < 1e-5, "{a:?} != {b:?}"); }
    }

    #[test]
    fn five_families_are_deterministic() {
        let mut ids = std::collections::HashSet::new();
        for q in 12..22 {
            for r in 0..10 {
                let c = CellPos::new(q, r, 1);
                ids.insert(raw_id(&World::rock_v5_gallery(), c));
            }
        }
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn same_local_hexel_keeps_family_across_stages() {
        let w = World::rock_v5_gallery();
        for q in 0..10 {
            let ids: Vec<_> = (0..4)
                .map(|stage| raw_id(&w, CellPos::new(12 * stage + q, 2, 1)))
                .collect();
            assert!(ids.iter().all(|id| *id == ids[0]));
        }
    }

    #[test]
    fn interiors_and_air_borders_do_not_mix_families() {
        let w = World::rock_v5_gallery();
        let c = CellPos::new(12, 0, 2);
        let center = family_weights_at(&w, c, c.center());
        let exterior = family_weights_at(&w, c, c.center() + Vec3::new(0.0, 0.0, -0.5));
        assert_approx(&center, &exterior);
        assert_eq!(center.iter().filter(|v| **v > 0.0).count(), 1);
    }

    #[test]
    fn both_sides_of_horizontal_contact_agree() {
        let w = World::rock_v5_gallery();
        let a = CellPos::new(14, 2, 2);
        let b = CellPos::new(14, 3, 2);
        let join = (a.center() + b.center()) * 0.5;
        assert_approx(&family_weights_at(&w, a, join),
                      &family_weights_at(&w, b, join));
    }

    #[test]
    fn both_sides_of_vertical_contact_agree() {
        let w = World::rock_v5_gallery();
        let a = CellPos::new(14, 2, 1);
        let b = CellPos::new(14, 2, 2);
        let join = Vec3::new(a.center().x, 2.0, a.center().z);
        assert_approx(&family_weights_at(&w, a, join),
                      &family_weights_at(&w, b, join));
    }

    #[test]
    fn every_weight_vector_normalizes() {
        let w = World::rock_v5_gallery();
        for q in 12..22 {
            for r in 0..10 {
                let c = CellPos::new(q, r, 2);
                let sample = family_weights_at(&w, c, c.center());
                assert!((sample.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            }
        }
    }
}
