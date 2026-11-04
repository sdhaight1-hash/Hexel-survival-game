# P2.4 — World-space mineral vein continuity experiment

**Branch:** `feature/p2-4-worldspace-ore-veins` (stacked on
`feature/ore-pattern-family-gallery`, PR #38). **Status: prototype
awaiting native Windows visual and GPU review.** This is a single new
optional diagnostic pattern, not actual generated ore deposits.

## Why

The November 4 native screenshots exposed a common failure: the first
five grayscale candidate masks repeat roughly every metre on separate
hexel faces, producing striped wallpaper and decorative nets. Caps use
world X/Z UV coordinates while side faces use local hex-perimeter/Y
coordinates. Recoloring cannot make independently sampled local-UV
features connect over cap/side boundaries.

## Implementation

- F8 candidate rock + F11 grayscale ON, then cycle **F12** to
  **WORLD VEIN** (after the six original SOURCE/HAIRLINE/NODULES/
  STOCKWORK/RIBBONS/SCATTER positions). The original six choices and
  the F11 OFF full-color composite remain unchanged.
- WORLD VEIN generates **coverage and tonal variation from interpolated
  `v.world_position` in metres**. Two 3D value-noise fields bend and
  interrupt crossing vein sheets several metres apart. No face UV
  resets, per-chunk random seeds, per-cell texture duplication or new
  atlas layer. The current F12 gallery remains 18 RGBA8 layers, not 19.
- Shader uses the **same physical 3D position** on a cap and side:
  the selected mineral formation and its metallic F10 response now
  share one alpha-coverage field. Granite and shale retain their
  existing host-rock texture and per-face UVs, and the independent P
  copper/hematite/moonsilver palette and F9/F10 controls are preserved.
- `src/mesh.rs` vertex geometry, chunk workload, material IDs, and
  world state are **unchanged**. A regression test demonstrates that
  cap/side mesh vertices can have different face UVs at identical
  3D coordinates; the WGSL world-vein path reads those 3D
  coordinates, not the face UVs.
- World-space fields can run through adjacent geometry, **but only
  where the existing preview materials identify an ore-bearing face**
  (`GraniteCopper` or `ShaleCopper`). Material ID boundaries can
  still visibly stop the ore. This does not create underground
  mineable veins, mineral quantity, grade or geological authority.

## Performance and artistic limits

Two trilinear value-noise samples per fragment (plus analytic
sine-based warped fracture sheets) are **potentially more expensive**
than the original single tile mask. This is a deliberately measurable
prototype, not a free, recommended production algorithm. Hardware
benchmarks will determine whether to replace the procedural sampling
with lower-cost **world-aligned volume masks, sparse feature data, or
baked continuous mineral textures**.

This first slice isolates the projection/seam problem. It does **not**
fix all six older masks, reduce competing rock-host detail, change
world ore distribution, implement mining, or prove that the vein is
convincing. Naturalism and density remain visually unapproved.

## P2.5 design extension — spider veining / fracture hierarchy (planned)

**Visual review of the P2.4 screenshots:** World-coordinate continuity is
working: the broad copper sheet spans multiple exposed faces. It is
**not final ore art**, because the result is a uniform, softly edged
peach ribbon with too little fracture structure. The next change should
refine *one* world-space formation before spawning additional
material families. Spider veining is a required variant of that work.

### Three related but distinct mineral morphologies

| Prototype | Structural definition | Visual character |
| --- | --- | --- |
| Main vein | One dominant finite fracture sheet with a mineral core | Localized pinch/swell, chunky intermittent mineral filling |
| **Spider vein** | Main fracture with sparse, tapering **secondary and tertiary fracture sheets** that split, fan and sometimes reconnect | Unequal branch lengths/angles, many thin dead ends, a few richer junctions |
| Stockwork | Several intersecting fracture sets of more comparable importance | Irregular multi-directional network, with nonuniform density |

**No evenly spaced sine stripes or repeating Voronoi/grid web.**
Spider veining should read like connected fracture propagation, not
a spiderweb decal. The term here is an *art-direction label*, not
a newly introduced mineable mineral type.

### Geometry and shader model (future, NOT in this PR)

1. Preserve `v.world_position` as the single input for mineral
   formation. In 3D, use **thin finite fracture sheets/patches**:
   their intersection with a 2D exposed rock face forms visible
   lines. Merely placing 3D curves would often produce spots when
   the curves intersect the surface. Do not restart at local UVs.
2. Construct a small **hierarchical fracture graph**: dominant
   parent, fewer thicker children, more short/thin tertiary branches.
   Allow asymmetric branch directions, occasional skipped segments,
   rejoining and termination. Branch width tapers toward endpoints;
   a few localized junctions thicken or form mineral pockets.
3. Split the material response into distinct fields:
   **sharp irregular mineral core** (strong local copper),
   **low-opacity alteration halo** (host-rock discoloration,
   not metallic), **sparse internal flakes/stringers** (detail
   visible only close up), and retained unmineralized host gaps.
4. Keep glints/specular response **core-weighted**, not applied
   across the halo; Copper / Hematite / Moonsilver still recolor
   the same selected morphology without changing footprint.
5. Compare one parent+spider variant with the P2.4 single
   world-space vein in the **same fixed-camera F12 gallery**.
   Avoid silently changing the SOURCE/F11 original baselines
   or existing masks, worldgen/resource IDs, chunk mesh and F2
   benchmarks. New variant controls must be disabled unless
   the F8+F11 preview is active.

### Cost and evidence gates

- **Do not evaluate an unbounded fracture graph per fragment.**
  First compare a small bounded set of analytic world-space
  sheets against a precomputed world-aligned low-resolution
  mineral field; choose whichever is simpler **and** meets the
  measured GPU budget. Measure the cost of extra noise/sampling.
- Tests must verify deterministic world-coordinate evaluation at
  shared cap/side vertices, no local tile restart, tapering branches,
  and explicit separation of core from halo. Visual review at
  close/medium distance must verify branch endpoints, pockets,
  junctions and natural nonperiodic distribution on both hosts.
- Collect F5 Immediate / F3 with **F1/F2 closed**, same camera,
  culling, palette and glints settings, >=30 seconds per variant.
  Report sampled GPU pass time; old FIFO 60 FPS comparisons are
  not sufficient to prove a shader-performance win.
- Source **geologic analogs**, not literal image templates:
  [USGS fissure/branching/horsetail vein descriptions](https://pubs.usgs.gov/pp/0359/report.pdf),
  [Newfoundland geological guide to stockwork veinlets](https://www.gov.nl.ca/em/files/mines-prospector-matty-mitchell-virtual-guide-to-epithermal-gold-deposits.pdf),
  and [native-copper stockwork specimen](https://commons.wikimedia.org/wiki/File:Native_copper_stockwork_in_skarn_rock_(Madison_Gold_Skarn_Deposit,_Late_Cretaceous,_80_Ma;_west_of_Silver_Star,_Montana,_USA)_4.jpg).
  These support fracture-network variety, not a claim that all
  copper deposits naturally look identical.

**Status: design target only.** No P2.5 spider-vein implementation,
shader/material change, GPU benchmark or native art approval is
claimed by these notes.

## Native acceptance and requested evidence

Run from the existing repository checkout:

```cmd
git fetch origin && git switch -C feature/p2-4-worldspace-ore-veins origin/feature/p2-4-worldspace-ore-veins && cargo run --release -- --benchmark-200
```

1. In F1 select F8 ON, F11 GRAY, F9 Dry, F6 OFF, F10 OFF,
   F12 WORLD VEIN, and P COPPER. View the **same granite/shale
   comparison** from ~1m/3m and include a top cap / side join.
   Close F1 before recording the view. Compare with F12 SOURCE
   and STOCKWORK at the same camera pose.
2. Verify WORLD VEIN does not reset its line at adjacent cap/side
   geometry: orientation should be shared across exposed surfaces.
   Any break exactly where a material switches from ore to bare
   rock is expected with the current fixture, and should be noted
   rather than misdiagnosed as a shader seam.
3. With P fixed, switch F10 ON and check that highlights remain
   confined to the newly computed ore coverage; also compare
   Hematite and Moonsilver without changing the vein shape.
4. For performance, use **F5 UNLOCKED** and **F3 logging**, with
   F1/F2 closed, fixed camera, F4 and other controls unchanged.
   Collect **at least 30 seconds** each for F12 SOURCE and
   WORLD VEIN, ideally three runs each. Afterwards open F2
   and **COPY LOG** to paste results. Compare sampled GPU pass
   time and p95/p99 rather than FPS alone.
5. If the world vein is visually too sparse or too regular,
   please send the screenshots before approving new pattern
   families. We should tune **one shape** rather than multiply
   unapproved texture variants.

**No new assets, dependencies, mesh vertices, material IDs, or
P0/P8 cache invalidation are required.** GitHub CI includes
WGSL Naga validation, texture count checks and a cap/side
world-coordinate test. These verify technical wiring, not GPU
quality or natural mineral appearance.
