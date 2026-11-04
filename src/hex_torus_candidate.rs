//! Integer axial coordinates and a test-only rectangular hex-torus candidate.
//!
//! Physical square geography and regular-hex lattice periods differ slightly.
//! This module is *not* enabled in gameplay until the geographic mapping,
//! saved-coordinate policy and global vertical datum are approved.

use glam::DVec2;

pub const HEX_ROW_METRES: f64 = 0.866_025_403_784_438_6;

/// Each translation is an exact integer vector of axial hex cells,
/// and is aligned to the existing 16-cell chunk side.
pub const PERIOD_Q: i64 = 1_024_000;
pub const PERIOD_R: i64 = 1_182_400;
pub const R_PERIOD_Q_OFFSET: i64 = -591_200;
pub const TARGET_WORLD_METRES: f64 = 1_024_000.0;

/// Exact integer/fixed canonical identity; do not derive IDs from camera f32.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Axial64 {
    pub q: i64,
    pub r: i64,
}

impl Axial64 {
    pub const fn new(q: i64, r: i64) -> Self {
        Self { q, r }
    }

    pub fn world_xz_m(self) -> DVec2 {
        DVec2::new(
            self.q as f64 + self.r as f64 * 0.5,
            self.r as f64 * HEX_ROW_METRES,
        )
    }

    /// Canonicalize an exact lattice position against the candidate's two
    /// periodic translations (Q=1_024_000, R=1_182_400 and Q=-591_200).
    ///
    /// Canonical physical x is [0, 1_024_000), z is [0, period_z).
    /// i128 avoids overflow when reducing large signed i64 coordinates.
    pub fn candidate_wrap(self) -> Self {
        let r = self.r.rem_euclid(PERIOD_R);
        let wraps_r = self.r.div_euclid(PERIOD_R);
        let unwrapped_q = i128::from(self.q) - i128::from(wraps_r) * i128::from(R_PERIOD_Q_OFFSET);
        // Twice the horizontal position is an exact integer and keeps
        // half-metre parity for odd axial rows.
        let twice_x = (2 * unwrapped_q + i128::from(r)).rem_euclid(2 * i128::from(PERIOD_Q));
        let q = (twice_x - i128::from(r)) / 2;
        Self {
            q: i64::try_from(q).expect("canonical q is bounded"),
            r,
        }
    }

    pub fn candidate_translated(self, periods_x: i64, periods_z: i64) -> Self {
        Self {
            q: self.q + periods_x * PERIOD_Q + periods_z * R_PERIOD_Q_OFFSET,
            r: self.r + periods_z * PERIOD_R,
        }
    }

    /// Exact-world-space nearest hex centre using f64 cube rounding.
    /// These are prototype-world metres, not the final geographic raster map.
    pub fn from_world_xz_m(pos: DVec2) -> Self {
        let zf = pos.y / HEX_ROW_METRES;
        let xf = pos.x - 0.5 * zf;
        let yf = -xf - zf;
        let mut x = xf.round();
        let mut y = yf.round();
        let mut z = zf.round();
        let dx = (x - xf).abs();
        let dy = (y - yf).abs();
        let dz = (z - zf).abs();
        if dx > dy && dx > dz {
            x = -y - z;
        } else if dy > dz {
            y = -x - z;
        } else {
            z = -x - y;
        }
        let _ = y;
        Self::new(x as i64, z as i64)
    }
}

pub fn candidate_vertical_span_m() -> f64 {
    PERIOD_R as f64 * HEX_ROW_METRES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CHUNK_SIDE;

    #[test]
    fn period_vectors_are_chunk_aligned_and_physically_orthogonal() {
        for axis in [PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET] {
            assert_eq!(axis % i64::from(CHUNK_SIDE), 0);
        }
        let start = Axial64::new(1234, -4567).world_xz_m();
        let east = Axial64::new(1234 + PERIOD_Q, -4567).world_xz_m() - start;
        let north = Axial64::new(1234 + R_PERIOD_Q_OFFSET, -4567 + PERIOD_R).world_xz_m() - start;
        assert!((east.x - TARGET_WORLD_METRES).abs() < 1e-6);
        assert!(east.y.abs() < 1e-6);
        assert!(north.x.abs() < 1e-6);
        assert!((north.y - 1_023_988.437_434_720_2).abs() < 1e-6);
        assert!((north.y - TARGET_WORLD_METRES).abs() < 12.0);
    }

    #[test]
    fn current_north_translation_is_closest_chunk_aligned_orthogonal_choice() {
        // For a perfectly northward regular-hex period, q = -r/2.
        // If BOTH q and r must be multiples of CHUNK_SIDE=16,
        // r must be a multiple of 32. Search a generous neighborhood
        // to confirm the candidate is the best permitted integer period.
        let alignment = 2 * i64::from(CHUNK_SIDE);
        assert_eq!(PERIOD_R.rem_euclid(alignment), 0);
        assert_eq!(R_PERIOD_Q_OFFSET, -PERIOD_R / 2);
        let chosen_error = (candidate_vertical_span_m() - TARGET_WORLD_METRES).abs();
        let mut best_r = None;
        let mut best_error = f64::INFINITY;
        for r in (PERIOD_R - 3_200)..=(PERIOD_R + 3_200) {
            if r.rem_euclid(alignment) != 0 {
                continue;
            }
            let q = -r / 2;
            assert_eq!(q.rem_euclid(i64::from(CHUNK_SIDE)), 0);
            let physical_north_metres = r as f64 * HEX_ROW_METRES;
            let error = (physical_north_metres - TARGET_WORLD_METRES).abs();
            if error < best_error {
                best_error = error;
                best_r = Some(r);
            }
        }
        assert_eq!(best_r, Some(PERIOD_R));
        assert!((best_error - 11.562_565_279_775_3).abs() < 1e-6);
        assert!((chosen_error - best_error).abs() < 1e-6);
        // The next permitted period overshoots by about 16.15m.
        let next_error = ((PERIOD_R + alignment) as f64 * HEX_ROW_METRES
            - TARGET_WORLD_METRES)
            .abs();
        assert!((next_error - 16.150_247_641_257).abs() < 1e-6);
        // This does NOT prove no alternative, nonorthogonal period or
        // differently sized hex geometry could satisfy other requirements.
    }

    #[test]
    fn wrapping_preserves_identity_across_both_edges_and_corners() {
        for original in [
            Axial64::new(0, 0),
            Axial64::new(-1, -1),
            Axial64::new(PERIOD_Q - 1, PERIOD_R - 1),
            Axial64::new(31, 16),
            Axial64::new(-8_000_000, 11_100_000),
        ] {
            let canonical = original.candidate_wrap();
            let x = canonical.world_xz_m().x;
            assert!((0.0..TARGET_WORLD_METRES).contains(&x));
            assert!((0..PERIOD_R).contains(&canonical.r));
            for dx in -2..=2 {
                for dz in -2..=2 {
                    assert_eq!(
                        original.candidate_translated(dx, dz).candidate_wrap(),
                        canonical
                    );
                }
            }
        }
    }

    #[test]
    fn nearest_hex_round_trip_handles_far_positions_with_f64() {
        for origin in [
            Axial64::new(-1, 0),
            Axial64::new(15, 16),
            Axial64::new(1_024_000, -1_182_400),
            Axial64::new(-2_048_000, 2_364_800),
        ] {
            assert_eq!(Axial64::from_world_xz_m(origin.world_xz_m()), origin);
        }
    }

    #[test]
    fn period_identical_neighbors_are_still_reciprocal_at_seams() {
        let neighbors = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
        for origin in [
            Axial64::new(0, 0),
            Axial64::new(PERIOD_Q - 1, 0),
            Axial64::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1),
        ] {
            for (dq, dr) in neighbors {
                let other = Axial64::new(origin.q + dq, origin.r + dr);
                let back = Axial64::new(other.q - dq, other.r - dr);
                assert_eq!(origin.candidate_wrap(), back.candidate_wrap());
                assert_eq!(
                    other.candidate_wrap(),
                    other.candidate_translated(1, -1).candidate_wrap()
                );
            }
        }
    }
}
