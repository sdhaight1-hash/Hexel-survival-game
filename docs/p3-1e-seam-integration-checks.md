# P3.1E — Headless torus seam integration checks

**Status: candidate-only integration tests.** They do not approve the 1,024 km playable torus, enable wrapping in the live game or select the physical metric. They extend the [P3.1D periodic terrain candidate](p3-1d-periodic-terrain-candidate.md) without changing the terrain source or runtime.

## Purpose

Address equivalence and a periodic heightfield are necessary but insufficient for *playable* seam correctness. A chunk renderer must decide whether the current solid hexel has an exposed side/top/bottom face even when the adjacent section is absent, and player collision must not treat unresident neighbors as air. At the candidate's two seams the neighbor can be represented by a translated axial cell whose **canonical identity** is elsewhere in the address range.

## Test harness

`src/seam_integration_candidate_tests.rs` is registered under `#[cfg(test)]` only. A minimal in-memory cache materializes a 16³ CPU block section from `PeriodicTerrainCandidate::sample` on demand, keyed by `CandidateCellId::candidate_chunk(16)`. The caller's **cell** is canonicalized before both section lookup and within-section indexing. A missing section falls back to the **same periodic source** rather than implicitly becoming air.

The harness checks:

- **Section ownership:** across east, north, and corner period translations, repeated loading of equivalent cells produces only one resident section key; vertical Y does not wrap.
- **Face visibility:** all six horizontal and two vertical neighbor directions use `solid(current) && !solid(neighbor)`. Results must match when sections are absent, resident, loaded independently, and evicted again.
- **Collision support:** sampling the block directly below a tested location must agree across resident/nonresident transitions. This is a block-support query, **not** swept collision, continuous physics, or player walking.
- **Neighbor reciprocity:** a neighbor offset and its inverse return to the same canonical cell, including across translations at all four candidate seam corners.
- **Surface layers:** test locations near calculated surface heights and above/below them to exercise air, grass, soil and rock rather than only solid rock.

These are behavioral contract tests; they do not execute GPU meshing or rendering and do not generate a whole planet, entities, water, structures, ores or edits.

## What remains before activation

1. Approve the horizontal physical metric choice and version the complete world manifest, including vertical datum.
2. Integrate one periodic authoritative source into a future playable world while preserving consistent section keys for edits and saves.
3. Verify actual render mesh face consistency and LOD stitching at both seams, plus walking, swept collision, camera rebasing, and pathfinding. This harness checks only the **visibility predicate** and discrete block support.
4. Test ownership under edits, fluid transfer and eviction. Those states do not yet exist in the candidate.
5. Validate native Windows performance and visual correctness before routing live streaming through canonical topology.

**Do not interpret passing tests as approval of playable torus wrapping.**
