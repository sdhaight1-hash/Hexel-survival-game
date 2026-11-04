# World generation, climate palettes, voxel rendering and asset references — synthesis

**Status:** Research / proposed integration, 2026-11-04. **No implementation or new production decision is implied by this review.** The authoritative project decisions remain [decisions](decisions.md), [world generation](world-generation.md), [world-scale foundation](world-scale-foundation.md), [integration](integration.md), [material contract](material-authoring-contract.md), and the [canonical art guide](../assets/STYLE_GUIDE.md). See also the [earlier focused worldgen/LOD review](worldgen-streaming-reference-review.md).

## Executive recommendation

Preserve one source of truth for a **finite, two-axis periodic 1,024 × 1,024 km geographic world**. At world creation, generate and persist globally coherent elevation, geology, hydrology and climate/environmental fields; at runtime, derive nearby detailed hexel terrain and vegetation deterministically, apply independently persistent edits/resource/organism/water state, and render near full-detail plus distant non-authoritative summaries. This is **not** a proposal to preallocate every 3D voxel or to run learned terrain inference in gameplay.

The additional references below suggest three useful *separations*:
1. **Physical/environmental truth vs visual appearance.** Climate data and real world state drive vegetation, cover, weather and colors; a palette is a derived renderer input, not an ecological simulation.
2. **Feature definitions vs material/geometry realization.** Density, geological descriptors, terrain materials, props and tint rules consume coherent world fields and are deterministic under pinned revisions.
3. **Authoritative local hexels vs disposable rendering caches.** Meshes, GPU buffers, distant LOD and optional generated-chunk caches are replaceable. Mining, construction, fluids, growth and depletion are not.

## Confirmed baseline in the Hexel repo

- P3.2A opt-in `--streamed-world` runs a **95-section moving Q/R/Y working set** of 16³ sections and incremental GPU mesh sync. The pure `PrototypeTerrain` consumes world seed and absolute coordinates. It is a temporary unbounded heightfield without approved toroidal wrap, persistence, caves, global climate, actual geology, or far LOD.
- In the user's native P3.2 run, the window remained at **95 resident sections** while exchanges such as +25/−25 and +150/−150 occurred. The largest reported three-second accumulation was **26.40 ms generation**, **77.89 ms CPU mesh/buffer calls** and **5.21 MiB uploads**. These aggregates **do not reveal worst per-frame stalls**. The player reported a possible slight loading delay.
- `src/geography.rs` currently proves periodic *rectangular raster indexing*, not periodic playable hex-grid adjacency. `src/hex_torus_candidate.rs` is a test-only candidate. No final wrapped lattice mapping has been approved.
- BC7 and P2 surface-preview work are functional prototypes; F8/F9/F10 are not generalized environment-responsive material rendering. The world-space ore shader is not authoritative mineral inventory.
- Existing [P3.3 in world-scale foundation](world-scale-foundation.md) is **persistence and correctness**, not a committed streaming-performance milestone. Any new perf-focused stage must be explicitly scheduled without silently redefining that label.

## Whole-world pre-generation: what to store

The adjacent [Voxel-Game-Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine) documentation already proposes **periodic synthetic tectonics → globally reconciled geography / constrained Terrain Diffusion at world creation → compressed canonical tiles → deterministic local hexels plus separately saved edits**. Its desktop asset/material editor exists; its proposed planetary game runtime, diffusion world and conservation solver are **not implemented** there. Reuse ideas and tested algorithms selectively, not that repo as a gameplay engine.

| Representation | Source of truth? | Store / compute strategy | Why |
| --- | --- | --- | --- |
| World manifest, seed, topology, coordinate/algorithm versions | Yes | Persist, immutable or explicit migrations | Identity and reproducibility |
| Continental tectonics, elevation, geology, climate normals, watersheds, base reservoir metadata | Yes, once generated | Compressed addressable geographic tiles and versioned feature descriptors | Cross-region coherence and replay |
| Cave network, ore province/vein descriptors, ecological placement seeds | Canonical definitions | Persist compact descriptors where globally/regionally coordinated; generate bounded fine geometry on request | Consistent borders without a planet-sized dense voxel store |
| Nearby base hexel occupancy | Derivable | Recompute from canonical data or optional verified cache | Avoid global 3D allocation |
| Edited terrain, mined ore, placed assets, plant state, water transfers | Yes, mutable | Persist separately with stable identities and durable/atomic transitions | Never resurrect depleted stock or lose player work |
| Chunk meshes, light data, physics/nav views, GPU buffers, distant terrain LOD | No | Build/cache/invalidate within budgets | Derived data |

Provisional storage arithmetic from the older world plan: a 32,768² `int16` elevation raster is **2 GiB raw** (31.25 m sample spacing across 1,024 km); 512² tiles yield **4,096 tiles**. This is *only elevation*, not all climate/geology/fluid state, not a compression benchmark and not a 1 m voxel representation. The geographic raster-to-hex-lattice mapping and final sea-level/height transform are unresolved.

**Risk controls:** resumable world creation and checksums; tile- and model-version pinning; seam tests on both geographical periods and all corners; avoid order-dependent generation; conservative state handling when unloading/chunk borders cross; never let a merely missing/unloaded chunk mean authoritative air.

## Vintage Story: environmental data and palettes

**What to learn:** climate/region maps can hold temperature, rainfall, forest density, landform/geology signals, while climate and seasonal *colormaps* turn those signals into context-dependent vegetation/surface tints. This separation helps avoid baking an entirely different grass/leaf texture for every climate/season.

- Candidate Hexel **A: baseline fields:** temperature normals, precipitation seasonality, soil properties, geology, elevation, habitat/growing-season potential, region hydrology, perhaps a forest-density/succession prior.
- Candidate Hexel **B: dynamic physical state:** current temperature, soil moisture, snow/ice, drought, rainfall, actual vegetation health and ecosystem change. Some are evaluated from time + normals; changes of state that must survive reload are persisted. Water inventory is conserved by simulation, **not** by the tint system.
- Candidate Hexel **C: renderer palettes:** continuous per-material/species climate/season lookup, with interpolation; optional environmental overlays respond to moisture, slope, exposure and substrate. Keep albedo tint apart from metalness, normal and roughness. Copper must not become a different ore because a tint changed.
- Avoid treating discrete biome IDs or Worley partitions as complete climate physics. Biomes/assemblages can be *derived* from combined fields, with physically plausible transitions. Palette color cannot substitute for forest density, canopy shape, soil, rainfall or water balance.
- **Minimum proof:** one grass + one foliage material using neutral bases, continuous temperature/rainfall tint, seasonal transition and saved deterministic samples. Verify season changes do not regenerate chunk geometry; review the look at normal player distances against the art guide.

References: [Vintage Story API: IMapRegion](https://apidocs.vintagestory.at/api/Vintagestory.API.Common.IMapRegion.html), [client world color-map API](https://apidocs.vintagestory.at/api/Vintagestory.API.Client.IClientWorldAccessor.html), [survival mod grass overlay implementation](https://github.com/anegostudios/vssurvivalmod/blob/master/Block/BlockWithGrassOverlay.cs). API and mod behavior can change; these links are research references, **not** a promise of direct engine parity.

## Hytale WorldGen V2: authorable rules with distinct responsibilities

Useful conceptual divisions from published Hytale WorldGen material:

- **Density/shape rules:** regional 3D solid/empty field and local modifiers for cliffs, overhangs, caves and arches. They enrich, rather than replace, canonical elevation and geology.
- **Material providers:** choose rock, strata, soil and surface cover from geology/depth/slope/exposure/moisture. Separate *what material is present* from how a material shader colors it.
- **Prop rules:** seed and place vegetation, objects and structures based on habitat and region, using stable IDs and independently mutable gameplay instances.
- **Environment and tint rules:** derive ambient behavior and blended appearance from climate and conditions; boundary-aware tinting is a useful art technique but not the source of biome or water authority.
- **Authoring:** human-curated rules and debug views are attractive; **do not build a general-purpose terrain node editor first**. Implement pure, versioned functions with fixtures, then consider an editor as a frontend to the same rule graph.

Sources: [Hytale: The Future of World Generation (2026)](https://hytale.com/news/2026/1/the-future-of-world-generation), [Hytale worldgen authoring docs](https://docs.hytale.com/creating-content/world-generation/how-to-edit-and-create-biomes/), [Hytale Update 6 notes](https://hytale.com/news/2026/8/update-6-patch-notes). Distinguish advertised capabilities from benchmarked Hexel feasibility.

## TanTanDev and Rust voxel implementations

- [Binary greedy mesher demo](https://github.com/TanTanDev/binary_greedy_mesher_demo): concrete Rust benchmark comparison of culled versus bit-mask/greedy meshing. **Do not transplant cubic merging wholesale:** Hexel has six lateral rectangular faces and hexagonal caps, plus UV/material/light/overlay constraints. First measure mask culling, adjacent section reads, mesh CPU cost, submitted geometry and cache locality; merge *only geometrically and materially compatible* faces. A fast mesh algorithm may still produce undesirable visuals or uploads.
- [Shrubbery](https://github.com/TanTanDev/shrubbery): serializable growth steps, deterministic generation, space-colonization attractors, voxelization and golden-hash reproducibility tests. Compare its *design patterns* with current `src/space_colonization.rs`, foliage registry and the older Voxel-Game-Engine flora authoring; don't add a second competing growth authority. Seed/species/version/placement and mutations must be separated.
- [3D cellular automata](https://github.com/TanTanDev/3d_celluar_automata): local cave morphology research; not a substitute for tectonics, cave connectivity across chunk boundaries or finite groundwater.
- [First voxel engine](https://github.com/TanTanDev/first_voxel_engine): earlier Rust/wgpu endless-terrain demonstration. It explicitly describes itself as unoptimized; treat as learning material, not a performant architectural baseline.
- [TanTanDev repositories](https://github.com/TanTanDev) and [Rust programming playlist](https://www.youtube.com/playlist?list=PLEMJrEzR-PkvK7_XgcEPqA-G6RcTIv_JS) / [voxel devlog playlist](https://www.youtube.com/playlist?list=PLEMJrEzR-Pkuu0btt7VCUhXoXOkP4ExBA) for follow-up experiments. **Full episode-by-episode inspection was not established** in this research pass.

Additional user-provided videos / references (retain exact URLs without inventing contents):
- https://youtu.be/oztbyI-1NII
- https://youtu.be/m0BCrB_iRiA — discussed in [earlier reference review](worldgen-streaming-reference-review.md) as a Worley-biome Rust tutorial.
- https://youtu.be/qnGoGq7DWMc — referenced in discussion of voxel meshing; exact full video content not independently audited.
- https://youtu.be/msTtI4-F69g
- https://youtu.be/LxVLqCiDqd8
- https://www.youtube.com/watch?v=WYnIkOXiSh8

Also see [Voxy SeedGen](https://www.curseforge.com/minecraft/mc-mods/voxy-seedgen) and [Distant Horizons](https://www.curseforge.com/minecraft/mc-mods/distant-horizons) for ideas about distant simplified data **without** conflating display LOD with authoritative full 3D simulation. Check licenses before reusing code, weights, textures or other assets.

## Integration pipeline (proposed)

```text
World config / versions / finite lattice topology
    -> periodic tectonics + broad geography + climate/hydrology constraints
    -> optional offline Terrain Diffusion surface refinement (requires seam proof)
    -> final drainage/watershed + geology/deposit/cave feature descriptors
    -> validated immutable, compressed canonical tiles
             |                        |
      local detail provider      distant geographic LOD
             |
      deterministic density + strata + material + props
             |
      authoritative persistent edits / water / ecology / resource stock
             |
      bounded streaming scheduler -> CPU meshing -> GPU uploads -> renderer
             |
      visual climate/season palette + reactive cover (non-authoritative)
```

Operationally, the chunk streamer should be **source-agnostic**: resident request → load tile / derive local chunk / optionally read validated cache → apply modifications → mesh/upload. The renderer cannot decide the existence, amount or identity of water/ore from shader overlays. Distant LOD must be invalidated when gameplay changes the surface.

## Recommended next experiments and gates — not a renumbered roadmap

1. **Frame-pacing measurement on existing P3.2:** p50/p95/p99 frame time, per-frame generation/meshing/upload, queue depth, visible holes, memory and resident count, including sprint/teleport/turnaround/revisit. Current three-second reports are insufficient.
2. **Bounded scheduling proof:** priority by collision/support, proximity and view direction; cap per-frame main-thread work and process off-thread work only after safe ownership/revision checks. Ensure unloaded data doesn't silently become air.
3. **Tiny saved geographic fixture:** one manifest, a handful of periodically coherent compressed elevation/climate/geology tiles, checksums, version tests and restart parity. Prove geographic raster wrap **and separately** the approved hex-lattice translation; do not promote the test-only torus candidate without review.
4. **One environment-tint proof:** climate palette + season driving grass/foliage tint without geometry rebuild or changing physical water/resource state.
5. **Mesher A/B benchmark:** current hex face culling vs bit-mask approach on identical representative chunks (solid, hollow, layered, mixed surface, ore boundary and foliage); include generated triangles, ms, upload bytes, memory and visible correctness.
6. **Small coherent ecology/terrain fixture:** a geological contact/ore descriptor crossing chunks, one deterministic tree whose saved state survives harvest/reload, and one cave crossing a boundary.
7. **One distant-heightmap LOD proof:** from *the same* saved geography; show parity with nearby terrain, handle surface edit invalidation, and measure transition artifacts and VRAM/frame time.

### Acceptance invariants

- **Determinism:** seed + manifest + revisions + canonical tile versions reconstruct identical untouched cells, independent of order/thread count.
- **Persistence:** deleted ore/harvested tree/placed block remains changed after eviction, restart and wrap traversal.
- **Continuity:** geology, climate, water and terrain agree across neighboring tiles/chunks and both world seams.
- **Conservation:** water mass changes location/store, never appears from visual palettes or cache reload.
- **Performance:** resident storage is bounded; no unbounded work is performed in a single frame; measured comparisons, not claims from other engines.
- **Visual identity:** climate tints, overlays and merged surfaces obey the [art guide](../assets/STYLE_GUIDE.md); no arbitrary recoloring of metallic properties or material identity.

## Explicit non-goals / unresolved decisions

No full 3D planet pre-bake; no live diffusion inference requirement; no third-party runtime dependency or copied code adopted from these references; no immediate generic graph editor or renderer rewrite. Still unresolved: playable torus period mapping, elevation-to-Y transform, specific climate/season equations, geographic field resolutions, compression benchmarks, hydrology solver, chunk cache policy, distant LOD transition and priority-frame budget. Resolve these with narrow tests before committing schema or scale promises.
