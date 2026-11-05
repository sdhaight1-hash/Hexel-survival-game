//! P3.2B-2: integer global torus identity with a bounded f32 player frame.
//!
//! This is the coordinate foundation, NOT yet wired into the live executable.
//! In particular, section residency, GPU meshes and player collision must all
//! use the same frame when runtime rebasing is enabled. No saved-world schema.

use crate::hex_torus_candidate::{Axial64, HEX_ROW_METRES, PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET};
use crate::world::{CellPos, CHUNK_SIDE};
use glam::DVec2;

pub const REBASE_DISTANCE_METRES: f32 = 512.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeriodIndex {
    pub east: i64,
    pub north: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalCell {
    pub raw: Axial64,
    pub canonical: Axial64,
    pub period: PeriodIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TorusFrame {
    /// Global axial cell whose center is the local coordinate origin.
    pub origin: Axial64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameShift {
    pub axial_q: i64,
    pub axial_r: i64,
    pub metres_x: f64,
    pub metres_z: f64,
}

impl TorusFrame {
    pub const fn new(origin: Axial64) -> Self {
        Self { origin }
    }

    /// Choose an aligned origin close to an arbitrary global start cell.
    /// Keeps the player's physics and rendering near local zero even at 1Mm.
    pub fn at_spawn(spawn: Axial64) -> (Self, DVec2) {
        let side = i64::from(CHUNK_SIDE);
        let anchor = Axial64::new(
            spawn.q.div_euclid(side) * side,
            spawn.r.div_euclid(side) * side,
        );
        let local = Axial64::new(spawn.q - anchor.q, spawn.r - anchor.r);
        (
            Self::new(anchor),
            DVec2::new(local.q as f64, local.r as f64),
        )
    }

    pub fn cell(&self, local: CellPos) -> GlobalCell {
        self.axial(local.q, local.r)
    }

    pub fn axial(&self, local_q: i32, local_r: i32) -> GlobalCell {
        let raw = Axial64::new(
            self.origin.q + i64::from(local_q),
            self.origin.r + i64::from(local_r),
        );
        global_identity(raw)
    }

    /// The frame is rebased only when necessary, by *whole chunk* steps.
    /// Both coordinates are local hex metres. Caller must invalidate/rebuild
    /// its local CPU/GPU section keys after a returned shift.
    pub fn rebase_if_needed(&mut self, x: &mut f32, z: &mut f32) -> Option<FrameShift> {
        if x.abs().max(z.abs()) < REBASE_DISTANCE_METRES {
            return None;
        }
        let local = Axial64::new(x.round() as i64, z.round() as i64);
        let side = i64::from(CHUNK_SIDE);
        let dq = local.q.div_euclid(side) * side;
        let dr = local.r.div_euclid(side) * side;
        if dq == 0 && dr == 0 {
            return None;
        }
        let shift = DVec2::new(dq as f64, dr as f64);
        let new_q = self.origin.q.checked_add(dq)?;
        let new_r = self.origin.r.checked_add(dr)?;
        let new_x = (f64::from(*x) - shift.x) as f32;
        let new_z = (f64::from(*z) - shift.y) as f32;
        self.origin = Axial64::new(new_q, new_r);
        *x = new_x;
        *z = new_z;
        Some(FrameShift {
            axial_q: dq,
            axial_r: dr,
            metres_x: shift.x,
            metres_z: shift.y,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrossingCounts {
    pub east: u64,
    pub west: u64,
    pub north: u64,
    pub south: u64,
}

impl CrossingCounts {
    pub fn total(self) -> u64 {
        self.east + self.west + self.north + self.south
    }
}

/// Emit transitions of canonical identity, never visual guide-plane crossings.
pub fn crossing_report(
    before: GlobalCell,
    after: GlobalCell,
    counts: &mut CrossingCounts,
) -> Option<String> {
    if before.period == after.period {
        return None;
    }
    let de = after.period.east - before.period.east;
    let dn = after.period.north - before.period.north;
    let mut directions = Vec::new();
    if de > 0 {
        counts.east += de as u64;
        directions.push("EAST");
    }
    if de < 0 {
        counts.west += (-de) as u64;
        directions.push("WEST");
    }
    if dn > 0 {
        counts.north += dn as u64;
        directions.push("NORTH");
    }
    if dn < 0 {
        counts.south += (-dn) as u64;
        directions.push("SOUTH");
    }
    Some(format!(
        "TORUS WRAP {} | RAW ({},{}) -> ({},{}) | CANONICAL ({},{}) -> ({},{}) | PERIOD ({},{}) -> ({},{}) | COUNTS E={} W={} N={} S={} TOTAL={} | periodic terrain, NO teleport",
        directions.join("+"),
        before.raw.q, before.raw.r, after.raw.q, after.raw.r,
        before.canonical.q, before.canonical.r, after.canonical.q, after.canonical.r,
        before.period.east, before.period.north, after.period.east, after.period.north,
        counts.east, counts.west, counts.north, counts.south, counts.total()
    ))
}

/// Local metric locations of the nearest east/west and north/south seam planes.
/// Designed for the debug renderer, not for game collision.
pub fn nearest_seams(frame: TorusFrame) -> (f32, f32) {
    // Debug markers for the square lattice; periodic topology still has
    // the original north-direction Q offset from the toroidal generator.
    let east =
        (frame.origin.q as f64 / PERIOD_Q as f64).round() * PERIOD_Q as f64 - frame.origin.q as f64;
    let north =
        (frame.origin.r as f64 / PERIOD_R as f64).round() * PERIOD_R as f64 - frame.origin.r as f64;
    (east as f32, north as f32)
}

/// Stable canonical terrain identity independent of camera frame.
pub fn global_identity(raw: Axial64) -> GlobalCell {
    let north = raw.r.div_euclid(PERIOD_R);
    let shifted_q = i128::from(raw.q) - i128::from(north) * i128::from(R_PERIOD_Q_OFFSET);
    let r = raw.r.rem_euclid(PERIOD_R);
    let east = (2 * shifted_q + i128::from(r)).div_euclid(2 * i128::from(PERIOD_Q));
    GlobalCell {
        raw,
        canonical: raw.candidate_wrap(),
        period: PeriodIndex {
            east: i64::try_from(east).expect("period exceeds i64"),
            north,
        },
    }
}

/// Authoritative local metres of an i64-global axial cell relative to the frame.
pub fn local_metres(frame: TorusFrame, cell: Axial64) -> DVec2 {
    DVec2::new(
        (cell.q - frame.origin.q) as f64,
        (cell.r - frame.origin.r) as f64,
    )
}

pub fn local_cell_from_metres(x: f32, z: f32) -> CellPos {
    let cell = Axial64::new(x.round() as i64, z.round() as i64);
    CellPos::new(
        i32::try_from(cell.q).expect("local q exceeded i32"),
        i32::try_from(cell.r).expect("local r exceeded i32"),
        0,
    )
}

pub fn north_period_metres() -> f64 {
    PERIOD_R as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_log_counts_both_axes_and_reverses() {
        let frame = TorusFrame::new(Axial64::new(
            PERIOD_Q + R_PERIOD_Q_OFFSET - 32,
            PERIOD_R - 32,
        ));
        let a = frame.axial(31, 31);
        let b = frame.axial(32, 32);
        let mut counts = CrossingCounts::default();
        assert!(crossing_report(a, b, &mut counts)
            .unwrap()
            .contains("EAST+NORTH"));
        assert!(crossing_report(b, a, &mut counts)
            .unwrap()
            .contains("WEST+SOUTH"));
        assert_eq!(counts.total(), 4);
        assert_eq!(nearest_seams(frame), (48.0, 32.0));
    }

    #[test]
    fn arbitrary_negative_and_far_spawns_start_near_local_zero() {
        for spawn in [
            Axial64::new(0, 0),
            Axial64::new(-1, -1),
            Axial64::new(PERIOD_Q - 1, PERIOD_R - 1),
            Axial64::new(-15_000_000, 18_000_000),
        ] {
            let (frame, local) = TorusFrame::at_spawn(spawn);
            assert!(local.x.abs() < 24.0 && local.y.abs() < 14.0);
            let cell = local_cell_from_metres(local.x as f32, local.y as f32);
            assert_eq!(frame.cell(cell).raw, spawn);
            assert_eq!(frame.cell(cell).canonical, spawn.candidate_wrap());
        }
    }

    #[test]
    fn rebasing_preserves_raw_and_canonical_global_position() {
        let (mut frame, _) = TorusFrame::at_spawn(Axial64::new(PERIOD_Q - 17, PERIOD_R - 17));
        let target = Axial64::new(frame.origin.q + 1104, frame.origin.r - 1056);
        let before = local_metres(frame, target);
        let mut x = before.x as f32;
        let mut z = before.y as f32;
        let initial = frame.cell(local_cell_from_metres(x, z));
        let shift = frame
            .rebase_if_needed(&mut x, &mut z)
            .expect("large move must rebase");
        assert_eq!(shift.axial_q % i64::from(CHUNK_SIDE), 0);
        assert_eq!(shift.axial_r % i64::from(CHUNK_SIDE), 0);
        assert!(x.abs() < 32.0 && z.abs() < 32.0);
        let after = frame.cell(local_cell_from_metres(x, z));
        assert!(((before.x - shift.metres_x) - f64::from(x)).abs() < 0.001);
        assert_eq!(initial.raw, after.raw);
        assert_eq!(initial.canonical, after.canonical);
    }

    #[test]
    fn each_direction_and_corner_have_consistent_period_changes() {
        let start = Axial64::new(PERIOD_Q + R_PERIOD_Q_OFFSET - 32, PERIOD_R - 32);
        let frame = TorusFrame::new(start);
        let centre = frame.axial(0, 0);
        assert_eq!(centre.period, PeriodIndex { east: 0, north: 0 });
        let east = frame.axial(48, 0);
        let north = frame.axial(0, 32);
        let corner = frame.axial(32, 32);
        assert_eq!(east.period, PeriodIndex { east: 1, north: 0 });
        assert_eq!(north.period, PeriodIndex { east: 0, north: 1 });
        assert_eq!(corner.period, PeriodIndex { east: 1, north: 1 });
        assert_eq!(frame.axial(0, 0).period, centre.period);
    }

    #[test]
    fn periodic_identity_does_not_depend_on_frame_or_loading() {
        let p = Axial64::new(31, 80);
        let a = TorusFrame::new(Axial64::new(16, 64));
        let b = TorusFrame::new(Axial64::new(-1600, 1504));
        let same_a = a.axial(15, 16);
        let same_b = b.axial(1631, -1424);
        assert_eq!(same_a, same_b);
        assert_eq!(same_a.canonical, p.candidate_wrap());
    }
}
