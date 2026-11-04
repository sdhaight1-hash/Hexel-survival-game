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

## P3.1C geometric compatibility finding (2026-11-04)

The existing candidate east/north periods are exactly orthogonal in the
regular 1-m-across-flats hex lattice and aligned to the current 16-cell
section boundaries. If an exactly northward period must keep both axial
translation components divisible by 16, its R component must be a
multiple of 32. An exhaustive local scan around the existing candidate
verifies that **R = 1,182,400** has the smallest mismatch to a
1,024,000-metre north period under those restrictions:
**11.562565 m short**. The next possible R = 1,182,432 is
**16.150248 m long**. This is a test of that restricted period family,
not a proof about all alternative hex dimensions, orientations or shears.

**Preferred option for further headless validation, not yet approved:**
retain the exact 1,024 km × 1,024 km geographic torus and the
chunk-aligned integer seam periods, with the P3.1B affine geographic
projection (north metric scale approximately 1.00001129 relative to
regular hexes). The alternative is to reduce the physical north period
by 11.56 m and accept that geographic coordinates are not exact physical
metres. Both require an explicit world-coordinate/geometry decision.

No change to `HexTopology::Unresolved`, active collision, near-hex
geometry, world generation, chunk ownership, or save schema follows
from this recommendation alone. See the
[P3.1C metric comparison](p3-1c-topology-metric-decision.md).

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
