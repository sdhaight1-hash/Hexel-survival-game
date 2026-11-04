# Space colonization across Hexel Survival: research and implementation study

**Status (2026-11-03):** research + small, headless Rust skeleton prototype.
Not an approved flora asset, geological simulator, cave system, ore generator,
surface-overlay runtime, or production-ready world-generation integration.

**User inspiration:** [space-colonization video](https://www.youtube.com/watch?v=Y6QLhjvytXk).
The linked video's transcript was not independently retrieved; technical
claims below are grounded in primary publications and the existing codebase.

## Executive decision

Use **one small deterministic branching-skeleton algorithm** as an optional
*geometry authoring primitive*, with separately designed growth controls and
output interpreters for vegetation, selected fractures/mineral deposits,
selected cave networks, and certain surface patterns.

Do **not** make space colonization the universal terrain, cave, ore, plant,
or overlay generator. Its intrinsic output is a *rooted, mostly tree-like graph*
of line segments, not a hydrogeological simulation, connected cave game level,
rock classification, nutrient simulation, or mineral inventory.

The existing style guide prefers authored grammars/templates for common
flora and reserves colonization for special trees or obstacle-responsive
structures. This study **extends possible research uses**, not that priority.

## 1. Evidence and limits

1. Runions, Lane and Prusinkiewicz (2007), [Modeling Trees with a Space
   Colonization Algorithm](https://algorithmicbotany.org/papers/colonization.egwnp2007.large.pdf).
   Defines spatial attractors, nearest-node influence, growth by normalized
   vector averaging, kill distance, tropism, and subsequent pipe-model
   thickness; examples include obstacle-responsive trees and shrubs.
2. Runions et al. (2005), [Modeling and Visualization of Leaf Venation
   Patterns](https://doi.org/10.1145/1073204.1073251).
   Important precedent for branching and, with extensions, *looped*
   vein networks: ordinary tree-style SCA alone does not create loops.
3. Franke and Müller (2022), [Procedural generation of 3D karst caves
   with speleothems](https://doi.org/10.1016/j.cag.2021.10.002).
   Geological cave plausibility requires global passage networks, local
   shape, erosion and cave-specific structure; SCA is a possible source
   for some paths, not a replacement for those models.
4. Codeço, Weis and Andersen (2022),
   [Numerical Modeling of Structurally Controlled Ore Formation
   in Magmatic-Hydrothermal Systems](https://doi.org/10.1029/2021GC010302).
   Permeable faults, fracture connectivity and fluid conditions affect
   ore location and grade. Pure isotropic SCA is not a geological model.
5. Johnson, Yannakakis and Togelius (2010),
   [Cellular automata for real-time generation of infinite cave levels](
   https://www.um.edu.mt/library/oar/handle/123456789/22895).
   A useful low-cost cave morphology alternative, especially for local
   passage shape and game-space readability.
6. Current repository boundaries: `src/world.rs` owns 16³ demo hexels;
   `src/mesh.rs` only builds visible geometry; `src/surface_layers.rs`
   holds diagnostic wet/moss/frost preview values;
   `assets/STYLE_GUIDE.md` defines species, ore and art conventions;
   `docs/world-generation.md` contains future planet-local worldgen plans.
   None contains a live cave/ore/vegetation feature generator.

### Core mechanism

Given roots R, an ordered set of attractors A, growth step D, influence radius
`d_i` and kill radius `d_k`, each remaining attractor chooses the nearest
existing skeleton node within `d_i`. For node `v`, accumulate its assigned
unit directions and grow a child along the normalized sum:

```text
direction(v) = normalize(sum(normalize(a - v) for a assigned to v))
child(v)     = v + D * normalize(direction(v) + tropism)
```

An attractor disappears after a node gets within `d_k`. The result is
connected segments with parent links and often branching. Competition,
envelopes, point distributions and tropisms create different shapes.
Segment widths, material identities, collision, grade, rendering and
persistence are **downstream choices**, not outputs of this equation.

No user-supplied video or original paper demonstrates that this algorithm
automatically creates plausible caves or ore deposits; those are **our
proposed adaptations**, requiring independent validation.

## 2. Where to use it

| Feature | Fit | How | Main caveat |
| --- | --- | --- | --- |
| Ancient/irregular trees | **High** | Constrained crown attractors, species tropism, authored trunk and leaf-site rules | Authoring/precompute, not frame-by-frame growth; common trees still use cheaper recipes |
| Tree roots | **High** | Soil/water/nutrient attractors, surface/rock exclusions, tapered root graph | Soil penetration and anchoring constraints required |
| Vines and climbing plants | **High** | Attractors on climbable surfaces, endpoint/segment collision | Need explicit attachment and ownership |
| Fungal mycelium | **Medium/high** | Organic-resource attractors, growth/cost field, sparse subsurface networks | Connected networks and ecology demand more than a tree |
| Branching mineral veins/stockworks | **Medium/high, conditional** | Fracture-biased targets, host constraints, deposit radii, grade field | Many deposit types are sheets, layers or replacement bodies; do not SCA them |
| Cave tributaries / lava tubes / rootlike passages | **Medium, conditional** | Path skeleton, radius profile, erosion/morphology pass, graph validation | SCA has no native loops, chambers, flood safety or traversability |
| Surface mineral markings | **Medium/high** | Project underground deposit segment field onto exposed faces | Render-only mask must agree with actual ore occupancy |
| Rock fractures, branching cracks | **Medium/high** | 2D face or 3D structure constrained to host | Art direction and cross-face continuity matter |
| Creeping lichen, corrosion, frost filaments | **Medium** | Surface-constrained growth paths for accent masks | Wet/moss/frost already have simpler diagnostic shaders |
| Rivers and watershed tributaries | **Low/conditional** | Could inspire small tributary layout under elevation constraints | Drainage follows elevation; use hydrology routing as authority |
| Layered sediment, placer deposits, uniform clouds | **Poor** | Do not use SCA | Deposition processes are not branching growth |

### A. Flora: the clearest application

Keep species-authored structure **primary**. The runtime or authoring tool may
select a branching recipe for exceptional canopies, old oaks, root systems,
sprawling shrubs and vines. First author the trunk/root starting geometry and
a crown/soil volume. Then sample attractors from species-specific envelopes,
light access, growth competition and obstruction. Additional modules compute
taper, branch radii (the pipe model), leaf or fruit attachment sites and
mesh/card LODs. A branch skeleton is *not* a rendered tree by itself.

SCA and L-systems complement rather than replace each other:
- A grammar controls species identity, bud spacing and structural rhythm.
- Colonization steers selected growing tips to available volume/attractors.
- An explicit structural blueprint and renderer produce wood occupancy and
  efficient leaf cards or meshes, preserving the 1m hexel vs finer geometry
  distinction in the style guide.

### B. Ore deposits: **two independent uses**

**Geological generation.** In a preselected eligible tectonic region, define
a mineralizing event and a *fault/fracture field*. Sample attractors along
fault planes, contacts, permeable rock, intrusion margins or constrained
upward paths. Grow a graph only where the host allows mineralization.
Assign each edge a width, compositional suite, grade, alteration halo and
deposit ID. Rasterize this *3D region feature* into each overlapping local
chunk; it does not originate from per-cell independent noise.

**Visual continuity.** For each exposed hex-prism face, evaluate its
world-space positions against the same deposit skeleton. Distance-to-segment
and a stable, seam-independent warp/mask can determine where a vein cuts the
face. This creates a **single multi-hexel line** crossing top and side
surfaces rather than a copy of the same ore speckle in each cell.
Draw a high-contrast material-appropriate vein/cluster pattern only where
actual ore-bearing cells or deposit exposures authorize it.

A rough scalar field from a graph might be:

```text
d(x) = min over deposit edges e of distance(x, e)
mineralized(x) = eligible_host(x) && d(x) < edge_radius(x)
ore_grade(x) = mineralized(x) ? event_grade * alteration(x) : 0
```

In practice, blend edge radius with geological variability and validate
volume/grade distributions. The **saved resource amount** belongs to
authoritative ore/depletion records, never to shader brightness or a mask.
Mining uncovers new faces of the same volume while depleted ore stays gone.

**Critical failure mode:** growing through arbitrary rock without fracture
orientation creates botanical-looking metal roots. To avoid that, sample
targets from anisotropic fault planes, limit curvature, correlate branching
with stress and host permeability, and mix SCA with planar lodes and
stratabound bodies. Distinct ores must retain the style guide's morphology
and metallic/roughness rules; SCA cannot make all ores shiny or glow.

### C. Cave systems: a promising **secondary** structure tool

Not all caves are branching; solution caves often contain networks/loops,
fracture-controlled passages, chambers, sumps and multilevel connections.
SCA alone creates a rooted tree and **does not guarantee navigability**.

Proposed hybrid pipeline:

```text
tectonics + rock solubility + groundwater head + drainage targets
          |
  select cave-bearing region and main hydrological/structural paths
          |
  SCA for subsidiary feeder passages (constrained attractors)
          |
  optional validated cross-links / loops + special chambers
          |
  passage radius and local erosion/CA/noise shaping
          |
  carve intersecting local hexels with halo and seam ownership
          |
  flood/air/water inventory, access and path connectivity checks
          |
  rock surfaces, mineralization, speleothems, lighting, ecology
```

For karst, rock solubility and water movement govern the principal network.
For lava tubes, eruption and flow channels govern it; for mine workings,
human design governs it. Space colonization is only a candidate for side
branch topology, rootlike fissures or deliberately fantastic formations.

**Gameplay validation:** traversable minimum throat dimensions, source/exit
accessibility, network connectivity, no isolated required objectives, bounded
drops, supported bridge/sump behavior, no destructive conflict with protected
geography, and finite water accounting. Put chambers and cross-links into a
graph explicitly; never assume a graph of parent edges contains cycles.

### D. Overlays: patterned appearance, not content authority

Use space-colonized 2D or 3D paths as **pattern generators** for ore streaks,
exposed roots, fractures, fungal filaments or localized reactive growth.
The existing wet/moss/frost gallery remains a separate condition system.
An overlay should be parameterized by stable world-space coordinates,
feature ID, surface orientation and grade/condition, not an independently
random image assigned per hexel.

Pipeline: authoritative structure -> exposed-face mask -> authored
ore/rock/biological material -> material-specific lighting response.
Thin cracks or small patterns can use shader masks; prominent stems, holes
or thick veins can warrant mesh/occupancy changes. The final art must pass
the guide's first-person 1m/3m/8m readability tests and preserve host-rock
recognition. Appearance cannot add or restore mineral inventory.

## 3. Proposed engineering contracts and ownership

```text
stored global geology + geography + seed/revisions
    -> deterministic feature candidate and enclosing REGION id
    -> sampled attractors and forbidden/allowed volumes
    -> space_colonization::grow (headless 3D connected graph)
    -> typed adapter:
         FloraStructure  |  MineralDeposit  |  CaveGraph  |  SurfacePattern
    -> persistent feature state where gameplay requires it
    -> chunk-local occupancy/face masks (derived, regenerate safely)
    -> scene meshes/materials/LODs (disposable GPU data)
```

**First prototype delivered in `src/space_colonization.rs`:**
- `grow(root, attractors, params, segment_allowed)`: deterministic
  input-order-based, bounded 3D graph with parent indices.
- `GrowthParams`: segment length, influence/kill radius, directional
  bias, node and iteration budgets.
- `GrowthResult`: nodes, remaining target count and explicit stop reason.
- `point_segment_distance_squared`: basic occupancy/overlay distance query.
- Headless tests for connected growth, obstacles, reproducibility,
  finite inputs, budgets and segment distance.

No new dependencies: use the existing `glam::Vec3`. The segment predicate
is intentionally caller-owned, so the same core does not invent geology,
habitat, fluid equations or voxel conversion. The caller must test the
**full segment**, including wall/obstacle collision, not merely endpoints.

**Reference limitations:**
- One seed/root; no explicit grafts/loops or parallel multiple-root growth.
- Naive nearest-node scans cost approximately
  `O(iterations * attractors * nodes)`; candidate separation also scans
  nodes. **Not suited to large chunk workloads** without indexing.
- No spatial hash/BVH, tree-branch pipe-model thickness, radius profiles,
  controlled splines, topology validator, voxel rasterizer, erosion,
  geology, resource grade, material binding or rendering.
- Ordered `Vec` accumulation gives repeatability in one build/platform,
  not guaranteed bit-identical floating-point output across all CPU
  architectures, optimization targets or future algorithm versions.
- Symmetric attraction can stall, especially when target directions cancel.
  Too-small kill distances relative to step length can also oscillate.
- The closure does not magically ensure geologic plausibility; constraints
  are the caller's responsibility.
- No default demo changes and no F-key toggles: feature generation remains
  experimental until its outputs and persistence are tested.

## 4. Cross-chunk and periodic-world rules

Do not seed a mineral vein or cave independently in every 16³ chunk: that
would chop networks at boundaries and change results with load order.
Instead, generate features over a *larger canonical region*, including a
halo for maximum feature reach, and clip/rasterize their geometry into
every intersected chunk. For each region/feature, use a stable hash of
world seed, generator revision, region coordinate and feature index.

World-space geometry matters: `CellPos::center()` currently maps axial
(Q,R,Y) to local Cartesian metres using the 1m across-flats hex prism.
Do not treat the three indices as equal-distance Cartesian axes.
The older authoring app may use local Z-up: convert and version that
boundary separately.

The 1,024 km torus is a **selected world direction**, but canonical playable
hex-lattice seam translations remain unresolved. Do not introduce Q/R
modulo as a shortcut. Complete those tests before implementing wrapping
deposit, root or cave features across the actual world seam.

Feature identity and resource depletion must be persistent. Cached chunk
surfaces cannot be a source of truth; mining or harvesting is a saved
mutation layered on top of immutable seed-derived base features.
Generated cave geometry must also be reconciled with finite water volumes.

## 5. Staged plan and test gates

**Stage SC-0: bounded geometry proof (this branch).** Headless graph code,
reference paper, no default runtime integration. Verify `cargo fmt --all
-- --check` and `cargo test --all-targets` in CI before merge.
The code is exploratory, not an approved gameplay feature.

**Stage SC-1: ore-and-face continuity proof (recommended next).**
Create a small offline 3D synthetic deposit with a fixed seed, host-mask
fixture and explicit grade. Sample continuous geometry on adjacent hex-prism
top and side faces; export a debug capture with vein centerlines/IDs.
Tests: a deposit crossing a chunk boundary is identical regardless of
which chunk is evaluated first; no ore on non-host rocks; no same-face
texture resets; volume/grade unchanged by viewing; removal remains
removed after save/reload. Compare vein/cluster/hybrid 1m, 3m and 8m
gallery readability against P2.2 style rules.

**Stage SC-2: constrained cave topology proof.** Build one region-scale
limestone/synthetic-rock fixture with one principal passage and one
SCA-derived tributary network; add optional deliberate cross-links and
chambers. Check graph reachability *and* hexel traversal clearance,
intersections, segment barriers, chunk-boundary seams and water budget.
Compare with non-SCA cave routes so the algorithm proves an advantage.

**Stage SC-3: specialized vegetation proof.** Compare a fixed authored
oak/willow-style recipe against constrained SCA using the same approved
species identity and crown envelope. Generate a structural blueprint and
separate render LODs; no species registry authorizations are implied.
Benchmark node count, generated mesh/card count, bake time and near/far
visual quality. Only adopt where it beats the simpler template.

**Stage SC-4: optimization and production criteria.** After representative
fixtures exist, introduce a deterministic uniform spatial hash / nearest
neighbor acceleration, node/segment budgets by feature type, fixed-region
generation caching, parallel region work with stable seeds, and debug
instrumentation. Capture actual timings before setting numerical budgets.
Version the generator and persist or migrate existing generated feature
identity. Do not regenerate player edits.

## 6. Decision gates

1. **Approve only the core graph as research infrastructure**, not an
   all-purpose replacement for authored flora or geological generation.
2. Run SC-1 first: adjacent-hexel connected mineral patterns have immediate
   relevance to the current ore-art quality problem.
3. Run SC-2 only as a hybrid with topology, host-rock and clearance tests.
4. Keep the first-person gallery unchanged until a reviewable side-by-side
   feature proof exists.
5. If the specialized solver is not visibly or measurably better than a
   simpler fault spline, L-system, graph route or coherent-noise field,
   **do not use it for that feature**.

### Performance and acceptance measurements (to collect, not fabricated)

- Generation time vs attraction count, iterations and graph node count
- Peak CPU memory and number of nearest-neighbor comparisons
- Same-seed hash reproducibility, deterministic cross-chunk clipping
- Exposed-face seam-continuity failures and invalid host occupancy count
- Cave required-path reachable/clearance success and water conservation
- Ore visibility and false-positive host/grade identifications at 1/3/8m
- Resulting mesh triangles, draw calls, LOD pop and first-person readability
