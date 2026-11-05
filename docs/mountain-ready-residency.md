# Mountain-ready surface streaming and geological LOD

## Purpose

November 5 native video showed missing near terrain faces and a flat-looking
distant heightfield. Existing near streaming used 5 vertical sections fixed
around player Y and distant LOD masks the entire near hex footprint, leaving
holes when those sections did not contain the true surface.

## New behavior (only `--torus-test-ne --distant-terrain`)

- Each resident 16×16 horizontal hex chunk column samples the shared
  authoritative periodic height function at all 256 cells.
- It requests sections spanning the **lowest surface Y section minus one**
  through the **highest surface Y section** in that column. This fills
  cliffs where low and high surface cells coexist, without allocating
  an entire mountain down to bedrock.
- Cached section bounds are pruned to the current R2 window and invalidated
  on torus coordinate rebase.
- Near surface uses the actual Grass/Rock block ID based on elevation
  and neighboring-slope steepness; deeper layers remain Soil/Rock.
- Distant triangles select rock or grass texture from altitude and
  triangle gradient, using the existing material atlas.
- A new torus-periodic ridge/uplift field creates defined highland belts
  with local maxima of hundreds of metres above surrounding plains.
- Headless checks include surface vertical section coverage and a
  seed-42 mountain-region relief/peak test.

## Limits and risks

- Near/far horizon still has an approximate hex-to-triangle connection,
  not a perfect watertight seam.
- At the start of a large heightfield move, streaming may temporarily
  lack some requested vertical sections. The current near mask still
  clips distant geometry based on the requested horizontal footprint,
  not actual section completion. If this produces brief gaps, a
  residency-aware mask is the next direct fix.
- With more materialized sections at rugged slopes, chunk/mesh generation
  and GPU upload budgets can spike; measure before optimizing.
- Distant geometry does not contain volumetric caves or overhangs;
  nearby materials have only current grass/soil/granite textures.
- High-elevation rock is not snow and no biome climate system exists yet.
- Changing deterministic worldgen is deliberate; do not assume existing
  world saves remain compatible.

## Native check

```bat
git fetch origin
git switch feature/mountain-surface-residency-materials
cargo test
cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4
```

Inspect rocky hills, tall surface cliffs, holes before/after walking, and F12
resident markers. Use F2 and Ctrl+C to capture counts, generation and uploads.
For the first view, turn slowly and walk for 30–60 seconds.
