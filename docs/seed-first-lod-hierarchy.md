# Seed-first multi-resolution terrain LOD — initial integrated architecture

This branch **replaces** the P3.2B-4 rectangular one-resolution underlay. It does
not claim parity with Minecraft Distant Horizons or Voxy SeedGen. The shared
direction is deterministic seed-derived distance geometry with independent
full-detail residency.

## Run and compare

```bat
cargo test
cargo run --release -- --torus-test-ne
cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4
```

Press F12 to show full-resolution resident-perimeter columns and capture a
30–60 second recording while turning, walking forward, and crossing an
anchor boundary. Do **not** combine the first visual check with
`--render-benchmark`; it adds a changing near radius that complicates evidence.

## Implemented

- Same PeriodicTerrainCandidate seed-derived surface elevations as world chunks,
  sampled without generating intermediate 16³ voxel sections.
- Four independent square resolution bands, widths 2m / 4m / 8m / 16m,
  joining at 64m, 192m and 448m. Outer boundary reaches 1152m.
- Explicit fine-to-coarse edge height interpolation to eliminate geometric
  T-junction height discrepancies at LOD-band boundaries (within numerical
  precision). This is **not** full 3D voxel stitching.
- 1000m far clip in opt-in mode; unmodified 200m clip otherwise.
- Generation on a single named CPU worker; non-blocking main-thread polling
  and GPU uploads only when a matching complete mesh is ready.
- One outstanding request per worker; snap-to-32m anchor so small player
  movements reuse buffers; old geometry remains visible while new geometry
  generates, except at torus rebase where it is invalidated.
- World frame identity included in work keys; stale results rejected across
  torus coordinate rebase.
- F12 markers remain available for diagnosing resident-window boundaries.
- Normal non-opt-in render path and collision unchanged.

## Explicitly not yet solved

- **Near/far seam**: the full-resolution near render footprint is an irregular
  hexagonal section window; the first square LOD band still overlaps it, and
  its near-to-coarse seam is not crack-free. No 2m offset is applied now,
  but small clashes remain possible.
- **Persistence/edit invalidation**: LOD only reflects procedural base terrain.
  Local user edits need revision-aware invalidation and far feature baking.
- **Volumetric overhangs, caves, cliffs, vegetation, structures, water**:
  heightfield LOD cannot represent these faithfully. Add material and feature
  representations separately.
- **Memory budget and GPU query timings**: no limit by bytes, disk caching,
  priority queues, temporal hysteresis, screen-space error evaluation, or
  occlusion logic yet. Requests use fixed bands, not a selection quadtree.
- **Benchmark stability**: no verified performance or GPU timing numbers for
  this integration; Windows and GPU validation are mandatory.
- **Worldgen revision**: this deliberately leaves authoritative terrain
  elevation unchanged. A richer Perlin/value-noise terrain composition
  should be versioned and tested independently rather than secretly applied
  only to the distant mesh.

## Pass/fail for first native evaluation

1. Rust tests/build pass on Windows without changing existing lockfile/art assets.
2. F12 perimeter markers visibly stop before distant landscape.
3. No large new holes, cliffs, or stretched foreground geometry while moving.
4. No sustained stalls at 32m anchor changes or torus rebases.
5. Identical unflagged world rendering to the baseline.

This is an **architecture-bearing prototype**, not a finished extreme-distance
LOD renderer. Do not merge to main until native validation and near/far
stitching are satisfactory.
