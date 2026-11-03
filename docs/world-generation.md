# Procedural world-generation foundation

**Status:** design specification; not yet a functioning world generator. Based on
the project's supplied "Our Procedural World Generation System — Complete Design"
and the [Voxel Game Engine design-decision register](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/procedural-world-design-decisions.md).
This document distinguishes agreed direction from implementation candidates.

## Goal

A synthetic natural world whose geology, topography, climate, water, soils and
ecology are causally connected, rather than unrelated noise maps. Geological
depth (Vintage Story) and interconnected ecology (Eco) are reference goals,
not imported code or currently implemented features.

## Global topology

- **Selected:** 1,024 km by 1,024 km finite flat-coordinate torus.
- Both geographical axes wrap. Rendering and physics remain locally Cartesian.
- Every global field and relevant spatial query must observe the same periods:
  geography, coastlines, plate structure, climate, river networks, vegetation,
  chunk loading and eventually physics and game-state edits.
- A torus does **not** reproduce Earth's poles or exact atmospheric circulation.
  Use plausible periodic climate forcing rather than claiming Earth realism.

## World-creation pipeline (proposed)

1. **Seed and manifest:** size, generator revisions, world-height settings and
   deterministic inputs.
2. **Abstract periodic tectonics:** plates and tendencies establish continents,
   deep ocean basins, collision mountains, rifts, volcanoes and sediment basins.
3. **Coarse climate and water constraints:** wind patterns, temperature,
   moisture pathways, broad drainage and ocean boundaries.
4. **Terrain Diffusion refinement:** refine detailed regional surface relief
   *subject to* tectonic/coarse environmental constraints.
5. **Hydrology reconciliation and validation:** recompute final drainage where
   refinement changed slopes; validate both seams, coastlines and continuity.
6. **Publish immutable geography:** save independently addressable compressed
   elevation/environment fields. World creation must finish successfully
   before data is marked ready for gameplay.

**Open risk:** a Terrain Diffusion model and conditioning interface that reliably
obey the synthetic plate, climate and periodic-edge constraints have not been
validated. Do not assume the model can simply be dropped into this pipeline.
The previous Tellus/real-Earth underlay is research, not the selected foundation.

## Geographic data versus playable world

The global geography is generated **once per world**. Gameplay reads it and
deterministically creates local editable 3D terrain. A diffusion model need not
be installed or active while playing, and no planet-wide 3D voxel array is stored.

Provisional elevation storage:

| Item | Proposed value |
| --- | --- |
| Raster | 32,768 × 32,768 samples |
| World span | 1,024 km per axis |
| Grid spacing | 31.25 m |
| Single 16-bit elevation layer | 2 GiB raw |
| Tile dimensions | 512 × 512 samples |
| Tile count | 4,096 (64 × 64) |
| Compression candidate | independent Zstandard tiles |

These figures are **not** RAM/VRAM budgets or measured compression sizes.
Elevation must retain its geographic datum. Temperature, rainfall, geology,
water and biome information use separate, appropriately resolved layers. Tile
checksums, schema revisions and generation IDs are necessary before committing
to any permanent format.

## Playable 3D terrain (proposed)

- **Provisional geometry:** hexagonal-prism cells ("hexels"), horizontal axial
  Q/R and vertical Y, with six side neighbours plus above/below.
- Initial section baseline: **16 × 16 × 16**; benchmark 32³, do not assume it wins.
- Global 2D geographic sampling and playable hex-lattice coordinates are
  different representations. Seam translations for the hex lattice are still
  **unresolved**; naïve independent Q/R modulo is not approved.
- Deterministic local generation: exposed rock, soils, faults, strata, ore
  deposits, caves, overhangs and groundwater below/above the saved elevation.
- Maintain persistent mutations (mined ore, placed/removed cells, harvested
  trees and environmental state) separately from reproducible base generation.
  Cache meshes/chunks as disposable derived products, not sources of truth.
- Active simulation uses high detail near players and coarse models far away.

### Vertical mapping (open)

World-height options are **256 / 512 default / 768 / 1,024** levels.
The provisional default sea level is **Y=220** at 512 levels (others:
Y=110 / 331 / 441). The transform from geographic elevation in metres to
playable Y is unresolved: uniform scale, nonlinear summit compression and
terrain-relative options all have consequences. A 1 m tall authored tree cell
does not prove 1 m of real-world relief fits into each Y level.

## Climate, geology and ecology

- Climate is driven by periodic temperature/latitude-like forcing, prevailing
  winds, ocean distance, elevation, orographic rainfall, rain shadows and
  seasons. Stable climate differs from current weather.
- Bedrock and deposits are conditioned by tectonic region: volcanic provinces,
  orogenic belts, sediment basins, fault systems and intrusions. Mineral stocks
  are finite; mining must not replenish them.
- Soil properties, moisture, water supply, climate, elevation and geology
  determine biomes and species eligibility. Ecotones should be gradients
  unless a physical boundary explains the abrupt change.
- Long-term ecology uses global/regional aggregate state plus individual
  simulation around active players; this remains a future requirement.

## Water conservation is mandatory

There are **no infinite source blocks**. Planetary water is accounted for
across oceans, air moisture, clouds/precipitation, snow/ice, rivers, lakes,
soil, aquifers, plants and player structures.

Three planned representations:

1. **Global** conserved reservoirs and transfers.
2. **Regional** rivers, watersheds, lakes, catchments and aquifers.
3. **Local** actively simulated flows, flooding, dams, caves and pumps.

Transfers between scales must be balanced. Unloading a chunk can transfer
its water budget to a coarser reservoir, never duplicate or erase it.
A numerical solver has not been selected.

## First implementation boundary

The initial `src/geography.rs` proves only **periodic rectangular-raster
addressing** and tile boundaries. It does not prove tectonics, diffusion,
geographic seam continuity, hex-lattice wrapping, voxel streaming or
water conservation.
