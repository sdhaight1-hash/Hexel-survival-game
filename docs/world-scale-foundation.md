# World-scale foundation: from static material gallery to a streamed planet

**Status:** independent main-based foundation roadmap plus first headless source of deterministic terrain. This work does not depend on pending material/ore-art PRs.
The **target remains a 1,024 km × 1,024 km geographic torus**, as
recorded in [world-generation.md](world-generation.md). The playable
hex-lattice wrapping transform is not yet chosen. A different rectangular
extent (e.g. 120 km × 1,024 km) is **not** approved by this branch.

## Architectural correction

**Do not instantiate the entire playable world.** At one-metre-scale
hexels, global geographic dimensions imply vastly too many vertical
cells to allocate as a voxel array. Maintain a finite, queryable
geographic domain; lazily generate and retain **only nearby 16³**
hexel sections; and persist world edits separately from deterministic
base generation.

The old one-chunk mode is an art/physics unit fixture. The
200-chunk mode is an intentionally static rendering benchmark; neither
is the playable-world architecture. Keep both as regression tests.
Do not scale those fixture loops to millions of chunks.

## Already present

- `src/geography.rs`: periodic **32,768 × 32,768 global raster**
  addressing, split into 512² independent tiles, over a
  **1,024,000 m** geographic edge. It proves raster index
  wrapping only; it does not define the playable hex seam.
- `src/world.rs`: absolute signed integer axial Q/R/Y cells and
  Euclidean chunk/local addressing. `CellPos::center()` currently
  converts world coordinates into `f32`; distant rendering/physics
  will need careful precision management.
- P2.4: a world-position cosmetic vein shader. It does not create
  canonical world-owned mineral deposit inventory.

## New code in this branch — P3.0, pure local generator

`src/generated_terrain.rs` adds `PrototypeTerrain::new(seed)`,
`sample(CellPos)`, `surface_y(q,r)` and
`generate_chunk(ChunkPos)`. Its inputs are **absolute signed
axial cell coordinates** and a seed, with no RNG state tied to a
chunk's generation order or load history.

It creates an intentionally simplistic heightfield near provisional
Y=220: surface grass, three soil layers, rock below and air above.
It is a deterministic source for a future streaming system, **not
continent-scale geology, terrain diffusion, real rivers, climates,
caves, biomes, weather, mining, persistence or production art**.
It is not assumed to wrap on the eventual geographic torus.
No existing demo, benchmark, renderer or player behavior changed.

Tests exercise arbitrary distant positive/negative Q/R values,
vertical Y sections, 15/16 and -1/0 boundaries, absolute
cell/chunk parity, regeneration after eviction, order-independent
generation, and changes of world seed. They do **not** establish
GPU or world topology behavior.

## Next production milestones — dependency ordered

### P3.1 — Select canonical playable world coordinates
- Explicit world manifest: seed; generation/coordinate revisions;
  geographic period; chosen vertical bounds/datum; topology.
- Resolve the mapping from the square geographic torus to the
  hex axial lattice, including **exact Q/R seam translations**,
  both directions and corners. Do **not** silently use separate
  Q and R modulo on an oblique lattice or claim the existing raster
  wrap proves hex connectivity.
- Use a stable integer or fixed-point global position for
  ownership, save IDs and ore descriptor queries. Derive local
  camera-relative floats for rendering. Test positions near
  ±1,024 km and across selected seam translations.
- Make spatial/temporal sampling reproducible under seed and
  schema version. No irreversible save format until seam rules
  are validated.

### P3.2 — Resident chunk streamer (first playable world)
- Camera/player chunk key drives an active working set in
  **all three axes**. Generate nearby sections on demand,
  prioritize the visible/forward area, and evict distant
  sections under a bounded memory budget. Nothing is loaded
  at global scale by default.
- A missing resident section is **unknown/unloaded**, not
  authoritative air. Define a `sample_base_cell()` path for
  deterministic neighbors; remesh both sides when a section
  becomes available or changes.
- Remove the static boundary barrier only once collision and
  support queries can be resolved for streamed neighbors.
- Renderer must upload/release only changed chunk buffers and
  draw from a dynamic resident set. Avoid rebuilding all loaded
  terrain every frame. Preserve F3 statistics for loaded,
  meshed, culled and submitted chunks plus build/upload timings.
- Begin with the simple P3.0 heightfield, then replace its
  geometry source when real geographic tiles are available.

### P3.3 — Persistence and correctness
- Immutable base terrain is derived from seed/version; edits,
  mined/depleted ore, placed blocks and active entities are
  persisted independently with stable IDs.
- Replay edits after reloading a chunk. Loading order,
  thread scheduling, process restart and evictions must
  not change original generation or regenerate removed ore.
- Provide a headless stress suite that walks through hundreds
  or thousands of distant chunk addresses and reloads them in
  random orders; verify memory remains bounded and seams agree.

### P3.4 — Authoritative geographic terrain / geology
- Lazy-read geographic height, bedrock, strata, faults,
  climate and hydrology tile data from the global world
  manifest. Generate/edit playable hexels only where needed.
- Deposit descriptors are **finite world-owned 3D volumes**
  with branching, orientation and clustered mineral grade,
  independent of chunk sampling. See
  [ore-geology research on the separate P2.4 branch](https://github.com/sdhaight1-hash/Hexel-survival-game/blob/feature/p2-4-worldspace-ore-veins/docs/ore-deposit-geology-and-continuity.md).
- After validation, use those data to drive material
  core/halo/micro-detail instead of letting a shader invent
  ore quantity.

### P3.5 — Actual scale acceptance
- Start with a cold load near a world edge and distant point,
  then stream, travel, return, unload, reload and revisit
  persisted changes. Verify streaming memory and CPU/GPU
  behavior rather than preloading the planet.
- Repeat across both geographical wrap boundaries **only after
  the canonical hex-lattice mapping is selected**. Verify
  path/collision, texture, elevation, geology and edits agree
  after wrapping.
- Keep a **small controlled gallery** for material-art A/B and
  microbenchmarks, but make the moving streamed world the
  primary game-development scene.

## Non-goals and performance honesty

- This branch **does not make a million-kilometre-squared world
  traversable yet**. It only establishes on-demand deterministic
  chunk source data, independent of resident working sets.
- No world preview with texture/water/vegetation at planet scale
  exists yet. Rendering all distant terrain as 1m hexels would
  defeat streaming; later LOD/terrain summaries are required.
- `f32` global coordinates at megametre distances lose fine
  precision. Long-distance movement/rendering requires local
  origin/rebasing and absolute integer or high-precision
  global coordinates; do not patch this by raising F4's distance.
- The benchmark's F4 112m distance filter **is not streaming**.
  It only skips some draw calls for already-loaded meshes.

**Recommended first playable-world acceptance:** walking over
multiple dynamically loaded Q/R/Y sections, seeing resident
chunk counts change, returning to a previously visited section
with stable terrain, and no hard demo boundary. Then expand
distance and add persistence/torus before another full ore pass.
