# P3.2B-4: opt-in distant terrain underlay (experimental)

This test does **not** implement production terrain LOD or replace the authoritative
streaming system. It is a minimal visual comparison for the abrupt resident-window
horizon observed in the November 4 gameplay recordings.

## Run

```bat
cargo test
cargo run --release -- --torus-test-ne --distant-terrain
```

Control / baseline run (without distant geometry):

```bat
cargo run --release -- --torus-test-ne
```

F12 still toggles magenta resident-boundary markers. Record a short camera pan and
forward walk at the same viewpoint with the new flag, ideally compared against
a baseline screenshot or recording.

## Design and constraints

- Draws a visual-only coarse 4-meter grid extending 176 meters from a player-centred
  snapped anchor; the camera's pre-existing far clipping plane remains 200 m.
- Samples the existing World::support_top procedural surface. The torus source
  remains authoritative. No second terrain seed, persistent tiles, or replacement
  world generation.
- Bakes a single vertex buffer only when the player enters another 16-meter
  local grid cell; invalidates it on torus rebasing.
- Draws the distant surface first through the existing depth-tested terrain
  pipeline; nearby full-resolution meshes remain unchanged. The distant surface
  is offset downward by 2 m to reduce z-fighting with near terrain.
- Does not affect collision, streaming budgets, residency counts, torus
  identity, or the default non-opt-in rendering path.

## Known issues / release gate

- Coarse rectangular tiles do not yet have a crack-free stitch to true hex-prism
  terrain; small gaps, protrusions, poor overlap, and visible tile slope facets
  are possible, especially on steep terrain.
- This is not a full distance hierarchy, occlusion-culling system, or horizon
  solution. It may introduce noticeable work at anchor crossings and more GPU
  fragments than desirable.
- The existing benchmark's tick FPS is not reliable GPU presentation FPS.
- No native Windows compile, visual test, or GPU profiling was available when
  preparing this branch.

**Accept only as a useful proof of concept if** (1) it compiles and runs, (2) the
horizon extends beyond the magenta resident markers during motion, (3) the
distant surface follows the same geology without large visible discontinuities,
and (4) it does not introduce unacceptable hitches. Otherwise keep this branch
experimental and refine the representation before attempting production integration.
