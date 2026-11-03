# World decision register

**As of 2026-11-03.** Status tags are intentionally explicit. See the attached
world-generation design and the [existing Voxel Game Engine register](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/procedural-world-design-decisions.md).
Nothing listed as a design choice is a claim of runtime implementation.

| Topic | Direction | Status |
| --- | --- | --- |
| World topology | Flat-coordinate, finite 2D torus, wrap X/Z | **Decided** |
| Footprint | 1,024 × 1,024 km | **Decided** |
| Global terrain origin | Synthetic tectonics, not Tellus/Earth data | **Decided** |
| Global detail | Constrained Terrain Diffusion at world creation, not in gameplay | **Decided direction; unvalidated interface** |
| Global geography | Store immutable base geography; no full planet 3D voxel bake | **Decided** |
| Water | Finite inventory, conservation across scales | **Decided invariant** |
| Height presets | 256 / 512 default / 768 / 1,024 | **Decided** |
| Elevation raster | 32,768², 31.25 m/sample | **Provisional** |
| Tiles/compression | 512² samples; Zstandard candidate | **Provisional** |
| Default sea level | Y=220 at 512-high | **Provisional** |
| Playable geometry | Hexagonal prisms, axial Q/R/Y | **Provisional preference** |
| Initial chunk sections | 16³, benchmark 32³ | **Provisional** |
| Climate | Periodic, geography-driven, season-aware | **Design goal; solver open** |
| Ecology | Global/regional aggregates + active local simulation | **Design goal; solver open** |
| Runtime state storage | Persistent edits + finite stock overlays | **Required; technology open** |

## Decisions required before broad implementation

1. **Toroidal hex-lattice:** choose exact period translations, orientation,
   dimensions and physical scale. Test seam neighbours, collisions, movement,
   planting and chunk adjacency. Independent Q/R modulo is *not* assumed valid.
2. **Geographic elevation → playable Y:** choose and version a height
   transform with defined sea level and clipping constraints.
3. **Terrain Diffusion conditioning:** test that the selected model respects
   synthetic geological constraints and both periodic edges; define fallback.
4. **Published asset format:** choose minimum identifiers, baked materials,
   geometry/occupancy and revisions, including authoring Z → world Y mapping.
5. **Conservative hydrology solver:** define conserved quantities, numerical
   error budgets and transitions between local, regional and global stores.

## Engineering rules

- Benchmark choices that affect memory, chunk dimensions and performance;
  do not present estimates as measurements.
- Prefer standalone deterministic tests and minimal interfaces before GPU
  integration or new infrastructure.
- Never mutate canonical geography when editing gameplay terrain.
- Do not silently change or rewrite the existing authoring/engine projects
  while introducing this game's integration layer.
