# Torus relief + near/far ownership revision

This revision addresses two separate failures reported in the November 4
recording: low-relief terrain and distant heightfield triangles intersecting
nearby full-resolution hexel cliffs.

## Main changes

- Authoritative PeriodicTerrainCandidate now samples continuous periodic
  coordinates instead of snapping all elevation queries to 31.25m raster
  indices. Near chunks, collision, and far mesh use **one** height function.
- Six seeded layers: continental regional drift, regional elevation, rolling
  hills, local slopes and modest detail, modulated by a broad plains/relief
  mask. This is a new experimental generator revision, not save-compatible.
- A separate distant fragment shader discards all LOD terrain inside the
  player-centred full-resolution hexagonal streaming-window footprint.
  This prevents duplicate visible near/far triangles and their cliff pileups
  within the requested window. The renderer updates the centre/radius every
  frame so R2/R5 tests are possible.
- Innermost distant band uses 1m samples. Far bands remain 4/8/16m.
- F2 now logs LOD generation CPU milliseconds, buffer upload CPU-call
  milliseconds, vertex count, MiB and snapped anchor on completed updates.

## Known limits

- Mask is based on **requested** full-resolution radius, not the exact
  temporarily incomplete residency set while chunk streaming is backlogged.
- Near/far seam heightfield vs hex prisms remains an approximation:
  adjacent geometry differs structurally; visible step walls can remain
  at the handoff. Not a finished watertight geometry stitch.
- Surface LOD does not handle caves, overhangs, edits, vegetation, water,
  buildings or persistent caches. GPU execution time is not measured.
- Changing terrain generator elevations is an explicit test-world break:
  existing saved edits and terrain are not currently supported.
- More varied terrain may expose shortcomings in vertical chunk residency
  beyond the former low-relief test world.

## Native test

```bat
git fetch origin
git switch feature/terrain-relief-near-far-mask
cargo test
cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4
```

Keep your existing uncommitted Cargo.lock/rock-gallery files untouched.
Press F12 to display near resident markers; use F2 to inspect LOD reports.
Record one slow 360° sweep and forward walk across several 32m anchor
boundaries. Report clipping, pileups, lost ground, jumps or stalls.

Do not merge until CI and Windows/RTX 2060 visual checks pass.
