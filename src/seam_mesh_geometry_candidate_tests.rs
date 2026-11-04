//! Test-only physical hex-face seam geometry contract (P3.1F).
//!
//! Models the side-corner ring currently used in src/mesh.rs, without
//! changing the renderer. An affine geographic metric must transform BOTH
//! centers and corners or adjacent face edges will not coincide.

use glam::DVec2;

use crate::{
    axial_geography_candidate::{CandidateCellId, CandidateProjection},
    hex_torus_candidate::{Axial64, HEX_ROW_METRES, PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET},
};

const NEIGHBORS_BY_MESH_SIDE: [(i64, i64); 6] = [
    (0, 1),
    (-1, 1),
    (-1, 0),
    (0, -1),
    (1, -1),
    (1, 0),
];
const HEX_CORNER_RADIUS: f64 = 1.0 / 3.0_f64.sqrt();

#[derive(Clone, Copy)]
enum PhysicalMetric {
    RegularHex,
    GeographicAffine,
    // Intentionally wrong: projected centers and unprojected corners.
    HybridCentersOnly,
}

fn center_delta(cell: Axial64, origin: Axial64, metric: PhysicalMetric) -> DVec2 {
    // Avoid large float world origins: differences are integer until the
    // final conversion to local physical metres.
    let dq = (i128::from(cell.q) - i128::from(origin.q)) as f64;
    let dr = (i128::from(cell.r) - i128::from(origin.r)) as f64;
    let row_pitch = match metric {
        PhysicalMetric::RegularHex => HEX_ROW_METRES,
        PhysicalMetric::GeographicAffine | PhysicalMetric::HybridCentersOnly => {
            CandidateProjection::z_metres_per_r()
        }
    };
    DVec2::new(dq + 0.5 * dr, row_pitch * dr)
}

fn corner_delta(side: usize, metric: PhysicalMetric) -> DVec2 {
    let angle = (30.0 + side as f64 * 60.0).to_radians();
    let regular = DVec2::new(HEX_CORNER_RADIUS * angle.cos(), HEX_CORNER_RADIUS * angle.sin());
    match metric {
        PhysicalMetric::GeographicAffine => {
            DVec2::new(regular.x, regular.y * CandidateProjection::north_scale_ratio())
        }
        PhysicalMetric::RegularHex | PhysicalMetric::HybridCentersOnly => regular,
    }
}

fn edge(cell: Axial64, origin: Axial64, side: usize, metric: PhysicalMetric) -> [DVec2; 2] {
    let center = center_delta(cell, origin, metric);
    [
        center + corner_delta(side, metric),
        center + corner_delta((side + 1) % 6, metric),
    ]
}

fn face_gap_at_seam(cell: Axial64, side: usize, metric: PhysicalMetric) -> f64 {
    let (dq, dr) = NEIGHBORS_BY_MESH_SIDE[side];
    let neighbor = Axial64::new(cell.q + dq, cell.r + dr);
    let local = cell.candidate_wrap();
    let remote = CandidateProjection::nearest_periodic_image(local, neighbor.candidate_wrap());
    assert_eq!(remote.candidate_wrap(), neighbor.candidate_wrap());
    let opposite_side = (side + 3) % 6;
    let near_edge = edge(local, local, side, metric);
    let far_edge = edge(remote, local, opposite_side, metric);
    (near_edge[0] - far_edge[1])
        .length()
        .max((near_edge[1] - far_edge[0]).length())
}

fn seam_points() -> [Axial64; 8] {
    [
        Axial64::new(0, 0),
        Axial64::new(-1, -1),
        Axial64::new(PERIOD_Q - 1, 0),
        Axial64::new(PERIOD_Q, 0),
        Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1),
        Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R),
        Axial64::new(R_PERIOD_Q_OFFSET + PERIOD_Q - 1, PERIOD_R - 1),
        Axial64::new(R_PERIOD_Q_OFFSET + PERIOD_Q, PERIOD_R),
    ]
}

#[test]
fn six_neighbor_mesh_edges_align_across_both_seams_in_either_consistent_metric() {
    for cell in seam_points() {
        for side in 0..6 {
            for metric in [PhysicalMetric::RegularHex, PhysicalMetric::GeographicAffine] {
                let gap = face_gap_at_seam(cell, side, metric);
                assert!(
                    gap < 1e-9,
                    "physical neighbor-face gap {gap} at {cell:?} side {side}"
                );
            }
        }
    }
}

#[test]
fn stretching_only_centers_breaks_shared_face_geometry() {
    let origin = Axial64::new(PERIOD_Q - 1, 0);
    let gap = face_gap_at_seam(origin, 0, PhysicalMetric::HybridCentersOnly);
    assert!(gap > 1e-6, "hybrid metric must exhibit a gap, got {gap}");
    assert!(gap < 0.0001, "this must remain a tiny metric discrepancy");
}

#[test]
fn vertical_hex_face_alignment_and_nearby_camera_coordinates_remain_precise() {
    let p = CandidateCellId::new(1_000_000_000_000, -1_000_000_000_000, 220);
    let above = CandidateCellId::new(p.q, p.r, p.y + 1);
    let below = CandidateCellId::new(p.q, p.r, p.y - 1);
    let local_above = CandidateProjection::relative_metres(above, p);
    let local_below = CandidateProjection::relative_metres(below, p);
    assert_eq!(local_above.to_array(), [0.0, 1.0, 0.0]);
    assert_eq!(local_below.to_array(), [0.0, -1.0, 0.0]);

    let neighbor = CandidateCellId::new(p.q + 1, p.r, p.y);
    let local_neighbor = CandidateProjection::relative_metres(neighbor, p);
    assert_eq!(local_neighbor.to_array(), [1.0, 0.0, 0.0]);
    assert_eq!(
        p.equivalent().candidate_chunk(16),
        CandidateCellId::new(
            p.q + PERIOD_Q + R_PERIOD_Q_OFFSET,
            p.r + PERIOD_R,
            p.y,
        )
        .equivalent()
        .candidate_chunk(16)
    );
}
