//! Seed-first toroidal distant terrain LOD prototype.
//!
//! Four concentric resolution bands are sampled directly from the same
//! periodic elevation function used by near hexel chunks. A worker builds
//! triangle geometry; the render thread uploads completed meshes only.
//! This is surface LOD, not volumetric voxel LOD or persistent edit support.
use crate::{materials_p0::Material, mesh::Vertex};
use glam::Vec3;
use hexel_survival_game::{
    periodic_terrain_candidate::PeriodicTerrainCandidate,
    torus_frame::{local_cell_from_metres, TorusFrame},
};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// Square grids use aligned boundaries: no spatial holes between bands.
/// Outer edge lies beyond the 1000m far plane for an anchor within 32m.
pub const BANDS: [(i32, i32, i32); 4] = [(1, 0, 64), (2, 64, 192), (4, 192, 448), (8, 448, 1152)];
const ANCHOR_STEP: i32 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub centre: (i32, i32),
    pub frame: TorusFrame,
}

impl Key {
    pub fn at(feet: Vec3, frame: TorusFrame) -> Self {
        Self {
            centre: (
                (feet.x / ANCHOR_STEP as f32).floor() as i32 * ANCHOR_STEP,
                (feet.z / ANCHOR_STEP as f32).floor() as i32 * ANCHOR_STEP,
            ),
            frame,
        }
    }
}

pub struct ResultMesh {
    pub key: Key,
    pub bands: [Vec<Vertex>; 4],
    pub cpu_ms: f64,
}

pub struct Worker {
    request: Sender<Key>,
    response: Receiver<ResultMesh>,
    pending: Option<Key>,
}

impl Worker {
    pub fn spawn(source: PeriodicTerrainCandidate) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Key>();
        let (result_tx, result_rx) = mpsc::channel::<ResultMesh>();
        std::thread::Builder::new()
            .name("seed-lod-surface".to_owned())
            .spawn(move || {
                while let Ok(mut key) = request_rx.recv() {
                    // If an anchor has moved, do not spend time on queued
                    // superseded requests.
                    while let Ok(newer) = request_rx.try_recv() {
                        key = newer;
                    }
                    let started = std::time::Instant::now();
                    let bands = std::array::from_fn(|i| {
                        let (spacing, inner, outer) = BANDS[i];
                        let next_spacing = BANDS.get(i + 1).map(|band| band.0);
                        build_band(source, key, spacing, inner, outer, next_spacing)
                    });
                    if result_tx
                        .send(ResultMesh {
                            key,
                            bands,
                            cpu_ms: started.elapsed().as_secs_f64() * 1000.0,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("failed to start distant terrain LOD worker");
        Self {
            request: request_tx,
            response: result_rx,
            pending: None,
        }
    }

    /// Non-blocking: at most one active request and no main-thread meshing.
    /// Return only geometry matching the current torus frame and anchor.
    pub fn poll(&mut self, desired: Key) -> Option<ResultMesh> {
        let mut newest = None;
        loop {
            match self.response.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    if result.key == desired {
                        newest = Some(result);
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if newest.is_none() && self.pending.is_none() {
            if self.request.send(desired).is_ok() {
                self.pending = Some(desired);
            }
        }
        newest
    }

    pub fn invalidate(&mut self) {
        self.pending = None;
    }
}

fn height(source: PeriodicTerrainCandidate, frame: TorusFrame, x: f32, z: f32) -> f32 {
    let local = local_cell_from_metres(x, z);
    let global = frame.cell(local).canonical;
    (source.surface_y(global) + 1) as f32
}

/// Fine-band outer edges must lie on the neighboring coarse-band edge.
/// Otherwise an intermediate fine vertex creates a visible T-junction crack.
fn edge_height(
    source: PeriodicTerrainCandidate,
    frame: TorusFrame,
    x: i32,
    z: i32,
    centre: (i32, i32),
    outer: i32,
    next_spacing: Option<i32>,
) -> f32 {
    let Some(coarse) = next_spacing else {
        return height(source, frame, x as f32, z as f32);
    };
    let on_x = (x - centre.0).abs() == outer;
    let on_z = (z - centre.1).abs() == outer;
    if !on_x && !on_z {
        return height(source, frame, x as f32, z as f32);
    }
    let along = if on_x { z } else { x };
    let low = along.div_euclid(coarse) * coarse;
    let high = low + coarse;
    let t = (along - low) as f32 / coarse as f32;
    let (a, b) = if on_x {
        (
            height(source, frame, x as f32, low as f32),
            height(source, frame, x as f32, high as f32),
        )
    } else {
        (
            height(source, frame, low as f32, z as f32),
            height(source, frame, high as f32, z as f32),
        )
    };
    a + (b - a) * t
}

fn vertex(position: Vec3, material: Material, tint: [f32; 3]) -> Vertex {
    Vertex {
        position: position.to_array(),
        color: tint,
        uv: [position.x, position.z],
        material: material as u32,
    }
}

fn emit_triangle(out: &mut Vec<Vertex>, a: Vec3, b: Vec3, c: Vec3, material: Material) {
    let tint = match material {
        Material::Granite | Material::Shale => [0.92, 0.91, 0.90],
        Material::SoilTopVerySparse => [0.98, 0.94, 0.83],
        _ => [0.93, 0.98, 0.89],
    };
    out.extend([
        vertex(a, material, tint),
        vertex(b, material, tint),
        vertex(c, material, tint),
    ]);
}

/// All visible distant materials must be derived from the same global cell
/// block identity as nearby hexels; no independent LOD slope thresholds.
fn surface_material(
    source: PeriodicTerrainCandidate,
    frame: TorusFrame,
    x: f32,
    z: f32,
) -> Material {
    use hexel_survival_game::world::Block;
    let cell = frame.cell(local_cell_from_metres(x, z)).canonical;
    match source.surface_cover(cell) {
        Block::Grass => Material::SoilTopFull,
        Block::Gravel => Material::SoilTopVerySparse,
        Block::Shale => Material::Shale,
        Block::Rock => Material::Granite,
        Block::Air | Block::Soil => Material::SoilBare,
    }
}

pub fn build_band(
    source: PeriodicTerrainCandidate,
    key: Key,
    spacing: i32,
    inner: i32,
    outer: i32,
    next_spacing: Option<i32>,
) -> Vec<Vertex> {
    assert!(spacing > 0 && inner % spacing == 0 && outer % spacing == 0);
    let side = (2 * outer / spacing) as usize;
    let width = side + 1;
    let mut points = Vec::with_capacity(width * width);
    for r in 0..width {
        for q in 0..width {
            let x = (key.centre.0 - outer + q as i32 * spacing) as f32;
            let z = (key.centre.1 - outer + r as i32 * spacing) as f32;
            points.push(Vec3::new(
                x,
                edge_height(
                    source,
                    key.frame,
                    x as i32,
                    z as i32,
                    key.centre,
                    outer,
                    next_spacing,
                ),
                z,
            ));
        }
    }

    let active = side * side - (2 * inner / spacing).pow(2) as usize;
    let mut out = Vec::with_capacity(active * 6);
    for r in 0..side {
        for q in 0..side {
            let mid_x = -outer + q as i32 * spacing + spacing / 2;
            let mid_z = -outer + r as i32 * spacing + spacing / 2;
            if mid_x >= -inner && mid_x < inner && mid_z >= -inner && mid_z < inner {
                continue;
            }
            let a = points[r * width + q];
            let b = points[r * width + q + 1];
            let c = points[(r + 1) * width + q];
            let d = points[(r + 1) * width + q + 1];
            let material =
                surface_material(source, key.frame, (a.x + d.x) * 0.5, (a.z + d.z) * 0.5);
            // Test the centre and four interior quarter points. The largest
            // deviation from bilinear corner interpolation owns refinement,
            // preserving narrow peaks/valleys a centre-only sample misses.
            // Interior fan geometry never introduces shared-edge T-junctions.
            let mut best_error = 1.5_f32;
            let mut apex = None;
            if spacing >= 2 {
                for (u, v) in [
                    (0.5_f32, 0.5_f32),
                    (0.25, 0.25),
                    (0.75, 0.25),
                    (0.25, 0.75),
                    (0.75, 0.75),
                ] {
                    let x = a.x + (d.x - a.x) * u;
                    let z = a.z + (d.z - a.z) * v;
                    let exact = height(source, key.frame, x, z);
                    let bilinear = a.y * (1.0 - u) * (1.0 - v)
                        + b.y * u * (1.0 - v)
                        + c.y * (1.0 - u) * v
                        + d.y * u * v;
                    let error = (exact - bilinear).abs();
                    if error > best_error {
                        best_error = error;
                        apex = Some(Vec3::new(x, exact, z));
                    }
                }
            }
            if let Some(m) = apex {
                emit_triangle(&mut out, a, c, m, material);
                emit_triangle(&mut out, c, d, m, material);
                emit_triangle(&mut out, d, b, m, material);
                emit_triangle(&mut out, b, a, m, material);
            } else {
                emit_triangle(&mut out, a, c, b, material);
                emit_triangle(&mut out, b, c, d, material);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexel_survival_game::hex_torus_candidate::Axial64;

    #[test]
    fn bands_are_aligned_without_planar_gaps() {
        for pair in BANDS.windows(2) {
            assert_eq!(pair[0].2, pair[1].1);
            assert_eq!(pair[0].2 % pair[1].0, 0);
        }
    }

    #[test]
    fn band_resolution_never_coarser_than_eight_metres() {
        assert!(BANDS.iter().all(|band| band.0 <= 8));
    }

    #[test]
    fn bands_have_expected_triangle_counts_and_are_finite() {
        let source = PeriodicTerrainCandidate::new(123);
        let key = Key {
            centre: (0, 0),
            frame: TorusFrame::new(Axial64::new(432_776, 1_182_375)),
        };
        for (i, &(spacing, inner, outer)) in BANDS.iter().enumerate() {
            let next_spacing = BANDS.get(i + 1).map(|band| band.0);
            let vertices = build_band(source, key, spacing, inner, outer, next_spacing);
            let expected =
                ((2 * outer / spacing).pow(2) - (2 * inner / spacing).pow(2)) as usize * 6;
            assert!(vertices.len() >= expected);
            assert!(vertices.len() <= expected * 2);
            assert_eq!(vertices.len() % 3, 0);
            assert!(vertices
                .iter()
                .all(|v| v.position.iter().all(|c| c.is_finite())));
        }
    }

    #[test]
    fn fine_outer_boundary_matches_coarse_height_interpolation() {
        let source = PeriodicTerrainCandidate::new(123);
        let frame = TorusFrame::new(Axial64::new(432_776, 1_182_375));
        let centre = (0, 0);
        let actual = edge_height(source, frame, 64, 3, centre, 64, Some(4));
        let a = height(source, frame, 64.0, 0.0);
        let b = height(source, frame, 64.0, 4.0);
        assert!((actual - (a * 0.25 + b * 0.75)).abs() < 0.0001);
    }

    #[test]
    fn snapped_key_is_stable_for_small_moves() {
        let frame = TorusFrame::new(Axial64::new(0, 0));
        assert_eq!(
            Key::at(Vec3::new(1.0, 220.0, 31.9), frame),
            Key::at(Vec3::new(30.0, 220.0, 1.0), frame)
        );
    }
}
