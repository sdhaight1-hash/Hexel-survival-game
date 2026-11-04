//! P3.1E integration contract tests for the *candidate* torus only.
//!
//! These tests intentionally use a tiny in-memory CPU section cache and never
//! run in gameplay. They compare resident and absent-neighbor behavior against
//! PeriodicTerrainCandidate, including cross-period cell ownership.

use std::collections::HashMap;

use crate::{
    axial_geography_candidate::{CandidateCellId, CandidateChunkId},
    hex_torus_candidate::{PERIOD_Q, PERIOD_R, R_PERIOD_Q_OFFSET},
    periodic_terrain_candidate::PeriodicTerrainCandidate,
    world::{Block, CHUNK_SIDE},
};

const NEIGHBORS: [(i64, i64, i64); 8] = [
    (1, 0, 0),
    (1, -1, 0),
    (0, -1, 0),
    (-1, 0, 0),
    (-1, 1, 0),
    (0, 1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

fn offset(p: CandidateCellId, dq: i64, dr: i64, dy: i64) -> CandidateCellId {
    CandidateCellId::new(p.q + dq, p.r + dr, p.y + dy)
}

fn translated(p: CandidateCellId, east: i64, north: i64) -> CandidateCellId {
    CandidateCellId::new(
        p.q + east * PERIOD_Q + north * R_PERIOD_Q_OFFSET,
        p.r + north * PERIOD_R,
        p.y,
    )
}

struct ResidentSections {
    source: PeriodicTerrainCandidate,
    sections: HashMap<CandidateChunkId, Vec<Block>>,
}

impl ResidentSections {
    fn new(seed: u64) -> Self {
        Self {
            source: PeriodicTerrainCandidate::new(seed),
            sections: HashMap::new(),
        }
    }

    fn load(&mut self, pos: CandidateCellId) {
        let canonical = pos.equivalent();
        let key = canonical.candidate_chunk(i64::from(CHUNK_SIDE));
        self.sections.entry(key).or_insert_with(|| {
            let side = i64::from(CHUNK_SIDE);
            let mut cells = Vec::with_capacity((CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE) as usize);
            for y in 0..side {
                for r in 0..side {
                    for q in 0..side {
                        cells.push(self.source.sample(CandidateCellId::new(
                            key.q * side + q,
                            key.r * side + r,
                            key.y * side + y,
                        )));
                    }
                }
            }
            cells
        });
    }

    fn unload(&mut self, pos: CandidateCellId) {
        self.sections
            .remove(&pos.candidate_chunk(i64::from(CHUNK_SIDE)));
    }

    fn sample(&self, pos: CandidateCellId) -> Block {
        let canonical = pos.equivalent();
        let key = canonical.candidate_chunk(i64::from(CHUNK_SIDE));
        if let Some(section) = self.sections.get(&key) {
            let side = i64::from(CHUNK_SIDE);
            let q = canonical.q.rem_euclid(side) as usize;
            let r = canonical.r.rem_euclid(side) as usize;
            let y = canonical.y.rem_euclid(side) as usize;
            section[(y * CHUNK_SIDE as usize + r) * CHUNK_SIDE as usize + q]
        } else {
            // A missing resident section is NOT air; the same periodic source
            // still supplies collision support and meshing neighbor blocks.
            self.source.sample(pos)
        }
    }

    fn face_exposed(&self, pos: CandidateCellId, direction: usize) -> bool {
        let (dq, dr, dy) = NEIGHBORS[direction];
        self.sample(pos).is_solid() && !self.sample(offset(pos, dq, dr, dy)).is_solid()
    }

    fn has_support(&self, pos: CandidateCellId) -> bool {
        self.sample(offset(pos, 0, 0, -1)).is_solid()
    }
}

#[test]
fn canonical_section_ownership_survives_both_seams_and_corner_translations() {
    let mut cache = ResidentSections::new(42);
    let points = [
        CandidateCellId::new(-1, -1, 220),
        CandidateCellId::new(PERIOD_Q - 1, 0, 220),
        CandidateCellId::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1, 220),
        CandidateCellId::new(R_PERIOD_Q_OFFSET + PERIOD_Q - 1, PERIOD_R - 1, 220),
    ];
    for p in points {
        cache.load(p);
        let canonical_key = p.candidate_chunk(i64::from(CHUNK_SIDE));
        let count = cache.sections.len();
        for east in -1..=1 {
            for north in -1..=1 {
                let image = translated(p, east, north);
                assert_eq!(image.candidate_chunk(i64::from(CHUNK_SIDE)), canonical_key);
                cache.load(image);
                assert_eq!(cache.sections.len(), count);
                assert_eq!(cache.sample(image), cache.sample(p));
            }
        }
    }
}

#[test]
fn neighbor_face_visibility_and_collision_are_identical_with_unloaded_sections() {
    let mut cache = ResidentSections::new(97);
    let locations = [
        CandidateCellId::new(0, 0, 220),
        CandidateCellId::new(PERIOD_Q - 1, 0, 220),
        CandidateCellId::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1, 220),
        CandidateCellId::new(R_PERIOD_Q_OFFSET + PERIOD_Q - 1, PERIOD_R - 1, 220),
        CandidateCellId::new(-16, -16, 220),
    ];
    for location in locations {
        let height = i64::from(
            cache
                .source
                .surface_y(crate::hex_torus_candidate::Axial64::new(
                    location.q, location.r,
                )),
        );
        for y in [height - 4, height - 1, height, height + 1] {
            let p = CandidateCellId::new(location.q, location.r, y);
            // Source-only: both the current and neighbor sections are absent.
            let baseline: Vec<_> = (0..8).map(|i| cache.face_exposed(p, i)).collect();
            let baseline_support = cache.has_support(p);
            let baseline_block = cache.sample(p);

            cache.load(p);
            assert_eq!(cache.sample(p), baseline_block);
            assert_eq!(cache.has_support(p), baseline_support);
            for (i, &(dq, dr, dy)) in NEIGHBORS.iter().enumerate() {
                assert_eq!(cache.face_exposed(p, i), baseline[i]);
                let neighbor = offset(p, dq, dr, dy);
                // The neighboring section may be across either world seam.
                // Load it independently and require face/collision parity.
                cache.load(neighbor);
                assert_eq!(cache.face_exposed(p, i), baseline[i]);
                assert_eq!(cache.sample(neighbor), cache.source.sample(neighbor));
                cache.unload(neighbor);
                assert_eq!(cache.face_exposed(p, i), baseline[i]);
            }
            assert_eq!(cache.has_support(p), baseline_support);
            cache.unload(p);
            assert_eq!(cache.sample(p), baseline_block);
            assert_eq!(cache.has_support(p), baseline_support);
        }
    }
}

#[test]
fn seam_neighbor_directions_are_reciprocal_with_canonical_ownership() {
    let cache = ResidentSections::new(7);
    let corners = [
        CandidateCellId::new(0, 0, 219),
        CandidateCellId::new(PERIOD_Q - 1, 0, 219),
        CandidateCellId::new(R_PERIOD_Q_OFFSET, PERIOD_R - 1, 219),
        CandidateCellId::new(R_PERIOD_Q_OFFSET + PERIOD_Q - 1, PERIOD_R - 1, 219),
    ];
    for p in corners {
        for (i, &(dq, dr, dy)) in NEIGHBORS.iter().enumerate() {
            let adjacent = offset(p, dq, dr, dy);
            let reverse = offset(adjacent, -dq, -dr, -dy);
            assert_eq!(reverse.equivalent(), p.equivalent());
            for east in -1..=1 {
                for north in -1..=1 {
                    let image = translated(adjacent, east, north);
                    assert_eq!(image.equivalent(), adjacent.equivalent());
                    assert_eq!(cache.sample(image), cache.sample(adjacent));
                    assert_eq!(
                        cache.face_exposed(image, i),
                        cache.face_exposed(adjacent, i)
                    );
                }
            }
        }
    }
}
