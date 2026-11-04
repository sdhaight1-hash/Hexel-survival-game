# Reference review: Hexal world generation, streaming, LOD and hex meshing

**Review date:** 2026-11-04. **Status:** evidence-based research and proposed
design gates, **not implemented engine features**. This supplements the
[world-scale foundation](world-scale-foundation.md) and
[world-generation design](world-generation.md). The geographic target
remains **1,024 km × 1,024 km** as documented; the playable lattice
period and vertical datum are still undecided. The ten user-provided
references are treated as engineering evidence/inspiration, not code
or art that should be imported.

## Direct references — what each source demonstrates

| User reference | Verified scope / useful observation | Hexal-specific decision |
| --- | --- | --- |
| [Voxy Seed Gen Is Changing Minecraft For Good (flowstate)](https://youtu.be/msTtI4-F69g) | Showcase of Voxy SeedGen seed-derived distant LOD, not proof that all blocks are generated | Derive cheap distant representations directly from authoritative seed/geographic source |
| [Incredible voxel mesh optimisations! (Daniel P H Fox)](https://www.youtube.com/watch?v=LxVLqCiDqd8) | Describes greedy and binary meshing, empty chunk/mesh rejection, chunk-level visibility/occlusion checks | Adopt empty rejection, border-aware face culling and cache reuse; benchmark hex-specific mesh rules before applying cube-oriented binary meshing |
| [Voxy SeedGen file listing](https://www.curseforge.com/minecraft/mc-mods/voxy-seedgen/files/all?page=1&pageSize=20&showAlphaFiles=hide) | Mod describes seed-math LOD generation, and redirects from archived CurseForge listing to [Modrinth](https://modrinth.com/mod/voxy-seedgen) for newer releases | Separate distant seed-only surface LOD from real nearby section generation. Do not copy mod code (All Rights Reserved) or assume its reported performance applies to our renderer |
| [Distant Horizons](https://www.curseforge.com/minecraft/mc-mods/distant-horizons) | Progressive, distance-dependent terrain detail far beyond normal detailed view | Separate fine interactive meshes from distant visual summaries, with transitions and invalidation |
| [My Game Has the Longest Voxel Render Distance Ever (latticeGlade)](https://www.youtube.com/watch?v=WYnIkOXiSh8&t=6s) | Author's custom raycast voxel-renderer showcase, with claimed extremely long views and acknowledged object/memory difficulties | Treat as an alternative rendering research path, **not** a mandate to replace our working wgpu triangle pipeline or proof of comparable GPU cost |
| [How I Achieved The Ultimate Voxel Biomes (latticeGlade)](https://www.youtube.com/watch?v=jlKO17lcbkQ) | Biome/forest feel depends on terrain elevation, canopy thickness and undergrowth/"ruffage", not merely color | Make forest visual acceptance about density, light penetration, silhouettes and atmosphere; preserve causal climate/soil/ecology inputs |
| [How to Implement Worley Biomes in Voxel Games with RUST (Tantan)](https://www.youtube.com/watch?v=m0BCrB_iRiA) | Worley regions, biome classification, blending and Rust integration | Use region partitions as a candidate *structural prior*, not a replacement for climate, hydrology and geological fields; evaluate gradual boundary mixing |
| [Hexagonal Grids (Red Blob Games)](https://www.redblobgames.com/grids/hexagons/) | Axial/cube coordinates, neighbor/distance/rounding, storage and wraparound are treated explicitly | Preserve axial Q/R absolute identity; prototype and test mathematically exact seam translations and shortest wrapped paths before world saves |
| [These Optimizations Doubled My Voxel Engine FPS (Dev In A Box)](https://www.youtube.com/watch?v=5mayFhgIpQY) | Author reports gains from redundant math removal, repeated calculation avoidance and memory-layout work in Rust/WebGPU engine | Profile generation, meshing, upload, shader and memory separately; avoid vague "optimized" claims or unmeasured global rewrites |
| [Greedy meshing, visually (Technical Fluff)](https://fluff.blog/2023/04/24/greedy-meshing-visually.html) | Visual explanation of merging adjacent rectangular voxel regions | Adapt *principle*, not rectangular geometry: six side normals and two hex caps require shape-aware merging and identical material/UV/light attributes |

**Verification note:** YouTube titles, descriptions and, where available,
transcript passages were retrieved for context. This is a **focused
architectural review, not a complete frame-by-frame critique**.
Creator benchmark/performance claims are not independently replicated.
Current Voxy SeedGen project description reports **13.2 million LOD
chunk positions in ~40 seconds on a 28-thread test system** with
distance-dependent output, *not* full chunk/ore/cave generation;
its own documentation says remote underground content may omit
caves and ores. The more recent release adds an *optional* real-near
chunk generation mode. We should learn from the data split, not
inherit these omissions in authoritative mining simulation.

## Three separate world representations (proposal)

**A — Global canonical geography.** Seed + versioned tectonic,
lithology, elevation, climate, drainage and regional ecological
fields, accessed via independent geographic tiles. The current
`src/geography.rs` has **32,768²** raster indexing on the
1,024,000-m geographic domain at **31.25 m** sample spacing;
this is a provisional elevation storage plan, not 1-m terrain,
biomes, physical water or hex-periodicity.

**B — Nearby, editable 3D hexels.** Only sections in a bounded
resident window receive 1-m hex-prism block data, collision,
entity/fluids and mesh. Canonical absolute integer/fixed
coordinates + seed/version determine original cells; player
changes/mined ore override this via persisted edit state.
Unloaded sections are **unknown**, not empty rock or air.
Stream across Q/R/Y; prioritize collisions and forward-visible
pieces, not an entire vertical column.

**C — Distant, non-authoritative render LOD.** Derive distant
surface shape, water boundaries and biome/vegetation summaries
directly from the **same canonical geography**, or derive
simplified records from closer already-generated data. Never
generate all 1-m cells only to throw them away at distance.
Distant data may omit underground caves/ore **for display**
but never becomes the source for mining, collision or saved
terrain edits. Mined mountains, structures and ecosystem
changes eventually need coarse-LOD invalidation/reconciliation.

**One world source, different costs.** If distant seed-derived
geometry disagrees with locally streamed full resolution, the
LOD transition will pop or contradict collision. This is a
correctness bug, not a necessary "optimization".

## Proposed order of implementation

| Gate | Minimum deliverable | Validation |
| --- | --- | --- |
| **G1 Coordinate/topology (P3.1)** | Canonical world manifest, hex torus translation candidates, local-relative render precision | ±Q/R/Y, both torus edges and corners, shortest wrapped paths, inverse mapping, chunk alignment, exact global IDs |
| **G2 Streamed playable detail (P3.2)** | Moving bounded section set with incremental generation, mesh/upload/eviction, collision when neighbors are absent | Walk over dozens of chunks, come back, terrain/edits are stable, loaded memory bounded, no edge walls |
| **G3 One far-LOD prototype** | One distant surface-only terrain representation from the same seed and elevation source, with near/far transition | Fixed viewpoint 0.1–5 km visual parity, seam/crack/popping tests, CPU/GPU/VRAM timings |
| **G4 Save and invalidate** | Saved modifications; generator/LOD caches invalidated on edits; reload independent of order | Modify/restart/evict/revisit; original geology and removed ore not duplicated |
| **G5 Authoritative global geology and biome fields** | Causal province/plate/elevation/climate/soil/mineral descriptors and regional vegetation | Seam- and version-consistent across tiles, fixed-seed visual tests, hydrology continuity, rich clusters constrained by host |

These gates are **engineering order**, not promises that a given
render radius, meshing technique or terrain algorithm is already
fast enough for our target PC.

### Specific hex grid issue to resolve in G1

The current world uses a regular one-metre-across-flats
hex-centre basis: `x = q + r/2`,
`z = (sqrt(3)/2) r`. The current global raster is axis-aligned
square; exact geographic periodicity and an exact **regular-hex
metric square** are not simultaneously trivial.

One **test-only** candidate for a nearly 1,024-km × 1,024-km
rectangle uses axial period translations

```text
T1 = (1,024,000,        0)
T2 = ( -591,200, 1,182,400)
```

Both translations preserve the regular hex lattice and are
multiples of the current 16-cell section side. Under the
current basis their geographic displacements are exactly
`(1,024,000 m, 0)` and approximately
`(0, 1,023,988.44 m)`, respectively. The
difference in the second dimension is ~**11.56 metres**
(~0.00113%). This is **only an illustrative candidate**:
before accepting it, choose whether the agreed geography
metric can tolerate a small discrepancy, or how a consistent
geographic-to-hex coordinate transform will reconcile
geographic raster periods with playable physics. A naïve
independent `q % N; r % N` does not produce these translations.
Validate wrap-aware nearest-image distances, cell neighbors,
chunk keys, terrain, ores, stream residency, fluids and edits at
**both seams and all four corners**.

A coordinate mapping must also explicitly distinguish stable
integer/fixed global ownership from camera-relative `f32`
rendering; at a megametre of absolute magnitude, globally
stored single-precision metre geometry is not adequate for
1-m hexel rendering.

### Meshing priorities for hex prisms

1. First count empty/solid/nontrivial sections, reject invisible
   interior faces and empty meshes, retain an occupancy/neighbor
   query that works even when the neighboring section is not
   resident. Incrementally remesh only affected section(s).
2. Measure indexed vertices or compact per-face streams before
   rewriting the mesher; our benchmark currently uses a
   non-indexed 6-triangle cap and 2-triangle rectangle side.
3. Consider merging *vertical runs of coplanar side rectangles*
   **only when** material, normal, visibility, lighting and
   world-space UV rules agree. A hex cap cannot turn into an
   arbitrary Minecraft greedy quad without changing actual
   polygon geometry and silhouettes.
4. Introduce hierarchical visibility and occlusion only after
   profiling: the current F4 rejects distant resident chunks
   by radius, **not** frustum- or occlusion-visible faces. A
   200-chunk static GPU benchmark does not simulate streaming
   or dense foliage.
5. Instrument section generation CPU, pending queue, mesh CPU,
   upload bytes/ms, resident CPU/GPU memory, submitted chunks,
   triangles, GPU render pass and frame p95/p99. Keep fixed
   camera F5 unlocked tests separate from streaming movement
   tests and from the unapproved ore material gallery.

## Visual world-gen acceptance derived from the biome references

Worley regions can serve as proposed *spatial domains* for
climate or ecological classes, with blended boundaries.
But actual biomes require consistent elevation, rain/temperature,
drainage, soil, vegetation succession and geology. Do not add
biome IDs as noise-colored tiles divorced from these causes.
Forest quality is not established by pretty color maps:
canopy density, views through the treeline, tree silhouette,
understory, shadow/atmospheric depth and climbable terrain
must read correctly at local and distant LOD. Our art guide
still determines approval.

## Rejected shortcuts and licensing

- Do not make full-res 16³ blocks for every far-Lod position;
  the Voxy SeedGen performance claim explicitly depends
  on producing only the needed LOD representation.
- Do not treat seed-LODs as gameplay geology, use client
  display approximations to generate mined stocks, or allow
  detailed saved edits to disappear from visible far scenes.
- Do not switch to raycasting because a showcase video says
  it renders "trillions"; that is not a validated comparison
  against our specific hex terrain, rendering requirements
  or hardware.
- Do not import mod code or assets from Voxy SeedGen
  (All Rights Reserved). Distant Horizons lists LGPLv3;
  do not incorporate its code without deliberate license
  review. These links are **architecture references**.
- Do not over-generalize naive quad greedy meshing to hex
  caps or multi-layer materials; verify exact triangle
  equivalence and visible seams first.

**Immediate next engineering action:** G1 topology/precision
proof followed by G2 dynamic chunk residency, while retaining
P3.0 pure terrain as a temporary generator. **Do not pause
the playable-world work for a giant biome simulator or swap
the renderer without data.**
