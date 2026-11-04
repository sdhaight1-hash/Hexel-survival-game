# P3.1D — Periodic terrain-source correctness prototype

**Status: headless experimental generator, NOT approved or enabled playable world topology.** This follows [P3.1B exact-raster projection candidate](p3-1b-axial-geography-projection-candidate.md) and [P3.1C metric tradeoff](p3-1c-topology-metric-decision.md). The target remains a 1,024 km × 1,024 km geographic torus. The physical hex metric and playable topology remain unresolved.

## Why wrapping an address is not enough

The live `PrototypeTerrain` uses unbounded world-space noise and **does not wrap**. Merely canonicalizing the q/r address makes equivalent cells identical, but the two *adjacent* cells on either side of a seam can still have unrelated noise values (a vertical cliff). That is not seamless terrain.

This slice adds `PeriodicTerrainCandidate::new(seed)` in `src/periodic_terrain_candidate.rs`, independently of the live source. It obtains periodic 32,768² raster addresses through `CandidateProjection::raster_sample(Axial64)`, and evaluates **smooth, periodic, seed-mixed value noise** with 64² regional and 256² local lattice control points. Integer lattice indices wrap into their first rows/columns, including across corners. The raster is **never allocated**; source values are computed on demand.

`surface_y(Axial64)` is the common near/coarse elevation source. `sample(CandidateCellId)` applies the prototype's grass/soil/rock/air layering without chunk residency; `chunk_surface(CandidateChunkId)` materializes only a 16×16 surface slice; `coarse_surface(origin, spacing, points)` queries the same source at a sparse grid. Values are independent of load order and world state.

## Tests and actual guarantees

- **Exact repeat identity:** signed axial cells and their translated images under both proposed torus period vectors produce exactly the same raster address, elevation and surface-material layers. Y is not periodic.
- **Seam adjacency:** all six hex-neighbor directions at east/west, north/south and four corner test points have local elevation changes (bounded to two layers in the selected test seed). Unlike repeat identity, this test checks **neighboring distinct cells**, not the same cell through translation. It is a sample-based smoothness check, **not** a global mathematical bound on every seed or all possible edge positions.
- **Chunk consistency:** regenerated 16×16 surface slices exactly match direct source heights at their absolute signed cell addresses, independent of chunk generation order, including negative coordinates and near candidate seam boundaries.
- **Coarse parity:** sparse samples match the exact same near source heights at shared cells and repeat at translated origins. This does **not** prove interpolated meshes will match intermediate near-surface cells or avoid visual LOD pops.
- **Reproducibility and bounds:** different seeds produce different fields; signed extreme i64 inputs canonicalize safely; invalid sample counts, step and overflowing coordinates are rejected.

## Deliberate limitations

- Separate **experimental candidate revision 1**, not a replacement for production `PROTOTYPE_TERRAIN_REVISION = 1`. World identity metadata remains `HexTopology::Unresolved`. No persisted terrain/save schema.
- The 32,768² raster maps many neighboring one-metre hex cells to a common sample, so this deliberately simple proof contains **quantization/flat steps**. It is not production-quality natural terrain, geological plate simulation, climate, erosion or diffusion.
- The integer torus projection's N/S affine scale remains an unapproved physical coordinate choice. This code demonstrates *field periodicity under the candidate translations*, not physics/rendering consistency or a world with physically approved seams.
- No GPU mesh, far-distance geometry, occlusion, biome regions, finite water or flora changes. No live `World::stream_around`, `PrototypeTerrain`, collision, presentation or camera code is modified.
- No guarantee of continuous physical gradients at every raster-to-hex boundary; only periodic address identity and tested seam-neighbor samples. Smoothing/quantization and future geographic tiles need their own validation.

## Next gates

Before enabling toroidal gameplay, select and version the physical hex metric and geographic-to-hex transform, define a vertical datum and world manifest, then test seam-aware collision, mesh face adjacency, stable edit ownership and periodic geology/climate. A real near/far LOD mesh prototype must use the **same periodic source** after that decision and validate visual transitions independently.

**Do not merge this candidate into live generation simply because its headless seam tests pass.**
