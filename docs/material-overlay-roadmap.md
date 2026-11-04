# Texture, material and overlay roadmap — revised after P2.2

**Status:** planning revision, 2026-11-03. This document supersedes the
*sequencing and completion claims* in [master art epic #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14); existing
technical specifications and the [canonical style guide](../assets/STYLE_GUIDE.md) still govern implementation and visual approval.
Do not read an unchecked future feature as shipping functionality.

## Decision

Keep **three distinct layers** of responsibility:

1. **Authored material identity:** stable IDs, face-appropriate tileable
   base/color/normal/roughness inputs as supported, physical repeat in
   metres, BC7 source baking and suitable GPU bindings. A granite surface
   remains granite independently of overlays.
2. **Feature/condition coverage:** a continuous spatial or surface mask,
   derived either from deterministic region-scale geometry (ore vein,
   fracture, root), authored decal/patch placement, or authoritative
   environment state (wetness, frost, moss and disturbance). These
   mechanisms are complementary, not all required for every material.
3. **Material response/compositing:** host remains visible; ore species,
   coverage/grade, seasonal cover and surface conditions modulate albedo,
   roughness/metallic response or geometry as appropriate. Render output
   **never establishes** mineable quantity, water, ecology, ore location
   or persistent edits.

**Space colonization is an optional source of connected geometry, not a
texture format, material editor, overlay compositor, or prerequisite for
the general texture system.** Its experimental Rust graph lives in
[draft PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30),
not `main`. The initial code creates parent-linked segments only; it
does not generate an actual mineral deposit, shader mask or cave system.

The new architecture supports a *single* region-owned ore/fracture
structure appearing across adjacent faces without per-hexel randomly
restarting its pattern. A tileable rock texture supplies fine material
identity underneath. Pattern geometry should be **feature-local or
world-anchored**, converted to chunk-local coordinates for renderer
precision; the proposed toroidal hex-lattice seam mapping is still
unresolved.

## Verified status from current `main`

| Track | Code / assets delivered | Remaining acceptance and production work |
| --- | --- | --- |
| P0 — test board | Bounded 16³ first-person demo; eight hex-prism faces with physically scaled UV; F6/F7 debug checks; original reference gallery; build-time 12-layer BC7 with RGBA fallback | Visual taste/QA and native GPU checks; P0 is not approved final art |
| P1 — format/baking | `catalog.v2.json` inventories 55 IDs; independent PNG RGBA8/BC7 baker, validator, index/cache validation; narrow optional F8 test gallery | General runtime v2 GPU material binding, grouped variable-dimension/mip strategy, broader face recipes and approved sources |
| P2 reference materials | Three registered candidates: granite, shale, native copper overlay; all 256² art revised in P2.1 | Source candidates need player-scale approval; soil/grass authored reference, first living flora visuals, grade fixtures and full P2 slice absent |
| P2.2 reactive surfaces | F8+F9 gallery supports wet, moss/lichen, frost, mixed presets; F10 controls localized copper sheen/glints; overlap hides glints beneath coverage | Simulated/persisted condition inputs, cross-face structural masks, true decal/marks, other materials and artist approval absent |
| P3 — 55 geology identities | All 55 **inventoried** (5 soils, 1 forest-floor cover, 23 rocks, 16 ore, 10 minerals) | 52 `needs_art`; 3 `candidate`; **0 approved**; not a finished replacement library |
| P4/P5 vegetation | Typed flora/cover concepts, research registry and authoring references | No production flora asset binding, accepted species assets, growth stages, streaming/harvest integration |
| P6 — ecology/weather overlays | Gallery-only F9 diagnostic values | No world-driven state, snow/ice accumulation, weather, growth, persisted surface damage or decals |
| P7 — optimization | Measured P0 BC7 memory reduction from 3.00 MiB RGBA to 0.75 MiB BC7 for 12 × 256² layers; BC7 fallback and tests | Mipmaps, general texture packing, streaming, scalable overlays, foliage alpha performance and target-machine measurement |

The P0 gallery is real, while its art is still provisional. The P1
authoring/baker work is functional, **not fully integrated into the
production renderer**. The P2.1/P2.2 art and effect demos need native
visual acceptance. Do not reimplement P0 or P1; extend and validate.

## Compositing and ownership contract

```text
AUTHORING / CACHED ASSETS                  AUTHORITATIVE GAME DATA
  source rock/soil textures                  host rock / soil identity
  ore-only material masks                    mineral occurrence + grade + stock
  species materials                          cover species and life state
  shader material recipes                    weather, wetness, damage, snow
              |                                          |
              |                          optional deterministic region feature
              |                            └─ SCA / fault spline / other path
              |                                   -> 3D deposit/fracture field
              |                                          |
              +----------------+-------------------------+
                               |
                   exposed face coverage request
            world/feature position + normal + stable IDs
              (not one separately random mask per hexel)
                               |
       substrate + mineral/feature + biological cover + conditions
                               |
               lighting/material response + LOD
                               |
                 shader / decal / owned geometry
```

- **Base materials:** high-quality tileable or appropriately projected
  authored rock/soil/wood textures. They do not need SCA.
- **Embedded geological patterns:** geology chooses ore-bearing volume,
  host compatibility, grade and resource identity. A vein/stockwork may
  use SCA *or* a simpler fault, spline, sheet or noise structure.
  Project or sample the **same** bounded 3D feature onto exposed caps
  and all six side orientations; suppress mineral when the authoritative
  host/occurrence is absent.
- **Cover and reactive conditions:** wetness, frost and thin moss can
  start as deterministic bounded masks/material parameters; thick snow,
  vegetation, long roots and actual holes require owned geometry/state.
  Don't grow thousands of procedural SCA networks just to paint wetness.
- **Localized marks:** erosion patterns, cracks, soot, footprints and
  mining scuffs may use small decals or sparse projected masks. The
  game saves edits and their semantic origin; caches remain disposable.
- **Rendering:** only selected metallic minerals get view/light-dependent
  metal response; preserve host, readable grade cues and low-specular
  alternatives. Covers can hide exposed ore highlights without
  changing subsurface ore quantity.

A seam-free feature need not be an extra full-resolution overlay PNG
per hexel. Prefer small shared mask/detail assets and bounded world-space
data when equivalent, measure shader samples/feature lookup and texture
residency, and fall back to baked region masks or mesh where appropriate.
Do not prescribe a universal shader, new attachment format or texture
channel reservation before a tested vertical slice exists. The F8
candidate copper alpha currently doubles as a preview mask, **not** a
universal production packing contract.

## Revised dependencies and phase gates

The historic **P0–P7 labels remain as milestones** for issue/PR continuity,
but tasks now have explicit prerequisites rather than pretending the
whole art library must ship before any overlay work.

### Gate A — close visual reference review (P0 / P2.1 / P2.2)

**Already delivered:** P0 board, BC7, F8 source revisions, F9/F10
reactive demonstration.

**Next:** capture native same-camera 1m/3m/8m neutral-light and
shadow comparisons of bare granite, shale and copper on both; inspect
cap/side joins, six sides, shimmer, glint locality, moss/frost occlusion.
Get user art-direction approval **or revise the three candidates**.
Document evidence; tests cannot approve visual style.

### Gate B — finish reusable runtime material plumbing (remaining P1)

Make the real v2 material index and stable face/overlay IDs selectable
through a narrow runtime path with a P0 fallback. Test independently
sized source materials, physical repeat, invalid/missing packs,
BC7 support/fallback, correct hex-face roles, and unchanged authoritative
terrain IDs. Avoid making a complete platform texture streamer before
the small reference slice works.

**Exit:** an authored candidate/approved material selected by stable
ID reaches a real face in the gallery without hardcoded special-case
rock IDs. P0 remains available as fallback.

### Gate C — independent overlay compositor and world-space continuity (new P2.3)

Implement **one minimal feature-mask interface/fixture** for a known
world-space line or small synthetic region deposit across neighboring
hexel caps/sides, plus a separate scalar wetness or thin cover mask.
The host/mask/condition identities remain independent. Sample stable
world/feature coordinates, not per-cell random UV restarts. Test
tile/chunk request order, side-to-cap joins, outside-host suppression,
mask ordering/occlusion and P0 fallback.

**Exit:** moving the camera or rebuilding neighboring meshes does not
shift/reseed a connected visible pattern; no extra ore is fabricated.
Use one host/ore/condition fixture, not all 55 materials.

### Gate D — optional SCA-to-overlay adapter proof (new P2.4 / SC-1)

On a bounded preselected geological fixture, run the existing
research SCA graph **only if suitable**, assign feature ID and
geological eligibility, derive radius/width and cross-face mask from
its edges. Compare a non-SCA spline/fault baseline with the same
constraints at near/medium/distance views. Require no botanical-looking
veins, correct poor/medium/rich readability and localized metallic
response, deterministic region/chunk clipping and no hidden ore exposure.
Depletion is not inferred from alpha.

**Exit:** ship the specialized feature source **only if it outperforms**
the simpler baseline visually or structurally within measured CPU/GPU
budgets. The entire material/overlay system must work without this
adapter; PR #30 remains a separate draft until approved.

### Gate E — game-state handoff and edit persistence (P2 production / M1–M3)

Replace diagnostic F9 condition values with **actual** persisted or
derived wetness/cover/damage inputs incrementally. Link ore visual
coverage to actual saved ore occurrences and depleted state; test mining,
newly exposed faces, unload/reload, and deterministic feature ownership.
Actual finite water, climate, moss ecology and geological worldgen may
be absent initially: use explicit saved fixtures, never imply a
simulated planet. Scope edits to state contracts, not massive engine
rewrites.

### Gate F — complete material and flora art (P3 / P4 / P5)

After the reference family passes, build remaining **52 unapproved**
source identities deliberately, with host-specific patterns and
nonmetallic mineral responses. Build soil/cover variants and first
grass/tree/fungus authoring-to-runtime fixtures. Grow the approved flora
roster from the 208 **research** rows only after selection; don't
manufacture 208 production assets. Preserve reusable textures/geometry
and staged growth or harvest state.

### Gate G — real environmental/impact overlay coverage (P6)

Author separate snow/frost accumulation, wetness, lichen, ash/soot,
wear/tracks/damage and seasonal effects, with priority/masking,
decals or geometry where justified. Let weather, ecology, finite water
and player actions provide the *authoritative* values as those systems
arrive. No dependency on SCA for ordinary reactive coverage.

### Gate H — optimization and quality closure (P7)

Benchmark mipmaps/alpha coverage, BC7 packs, format fallback,
asset grouping, shader sample cost, world-space feature lookups,
decals/geometry batching, streaming stability and shader LOD.
Use representative busy scenes and actual hardware frametimes, not
texture-compression percentages as FPS claims. Maintain visually
equivalent ore and hazard readability at all quality tiers.

## First next implementation slice

**Prefer Gate A and Gate C planning together, then Gate B/C as narrow
implementation milestones.** The immediate visual risk is unapproved
rock/copper art; the immediate architectural risk is building a
per-hexel fixed-mask pipeline that cannot represent connected features.
Don't expand to 55 source artworks until we have at least one
approved-quality reference and one stable cross-face mask path.

An inexpensive initial comparison should show **the same** granite
with (i) existing tiled copper alpha, (ii) a world-space continuous
copper vein driven by a fixed graph/spline, and (iii) no ore, across two
or more neighboring hexels and different face angles. Evaluate at
1m/3m/8m and in motion. Add the SCA generator as a separate measured
candidate *after* this plumbing proves useful.

**Not this revision:** no engine/shader code, new material statuses,
water solver, global geology, flora production, image assets, or
merging of draft PR #30.
