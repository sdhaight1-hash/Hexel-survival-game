# P3.1 foundation slice: one seed/height source, multiple representations

**Status:** headless-only implementation, not a playable world-generation overhaul. The existing 95-section streamer, terrain revision 1, renderer, materials, player physics and original near-field heightfield remain unchanged.

## Basis and design decisions

The existing `PrototypeTerrain` produces arbitrary signed-axial-cell heights and full 16³ sections independent of load order. `geography.rs` represents a separate proposed periodic 1,024-km square raster (32,768 samples/axis, 512 samples/tile). `hex_torus_candidate.rs` implements only a **test-only** axial seam candidate: its vertical period differs from the geographic metric target by about 11.56 m. That candidate is **not approved** and isn't active during gameplay.

In `src/worldgen_foundation.rs`:

- `WorldGenerationIdentity::prototype(seed)` centralizes current generator revision, geographic design constants and seed as a **lightweight in-memory descriptor**. `HexTopology::Unresolved` intentionally prevents callers from believing the geographic raster wrap implies a valid playable-hex torus.
- `CoarseSurfacePatch::sample(&terrain, origin_q, origin_r, spacing_cells, points_per_axis)` samples a rectangular grid in absolute signed **axial cell space**. It uses `PrototypeTerrain::surface_y` directly, not a second noise algorithm, independent biome blend, or cached near chunks.
- Each patch stores integer heights, spacing and origin. Adjacent patches with shared sample coordinates agree at their edges by construction. Point count is bounded to 2..=129 and coordinates are checked against existing i32 ownership bounds before allocation.
- The result is a **sample data source for future far-terrain LOD**, not a mesh, visible radius, anti-popping transition, GPU optimization, geographic-tile reader, or approved torus transform. Between sparse sample points, a future interpolation/mesh scheme must be validated against detailed geometry. Coarse samples matching the near source at common points does not prove complete visual parity.

## Explicit architectural boundaries

- A geographic tile (raster coordinates and periodic wrap) is **not** interchangeable with a playable-hex coarse patch (signed axial Q/R). No geography-to-axial mapping exists yet.
- The identity is **not** serialized and does not create a save format or publish an irreversible coordinate contract.
- Terrain `revision = 1` is unchanged. No new biome, geology, water, strata, flora, ore, erosion or Terrain Diffusion models are claimed.
- Existing global `f32` player coordinates remain a precision limit. `Axial64` stays a separate candidate; upgrading ownership/rendering needs a dedicated validated milestone.
- No distant cell inventory is instantiated. A patch samples at most 129² source elevations without generating underground block sections.

## Acceptance checks

Rust unit tests cover metadata/topology honesty, exact parity with the detailed terrain surface and grass/air at shared coordinates, adjacent patch boundaries across negative coordinates, seed-specific deterministic regeneration and invalid/overflow request rejection. The existing generator tests continue to cover full 16³ cell/chunk parity and order independence.

**Next dependency gate:** decide and document the exact geographic-to-playable-hex period translation, global coordinate ownership and metric discrepancy before enabling periodic seams or persisted world IDs. After that, independently prototype and measure an actual coarse mesh, seam stitching and near/far transitions. Do not interpret this data sampler as a finished LOD renderer.
