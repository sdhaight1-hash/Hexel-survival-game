# Surface overlays and continuous feature appearance — design notes

**Scope:** proposed implementation boundaries and QA for embedded mineral,
biological/seasonal and reactive surface coverage. This is not a finished
overlay compositor, world-state system or production SCA integration.

**Visual authority:** [canonical style guide](../assets/STYLE_GUIDE.md).
**Roadmap:** [materials and overlays](material-overlay-roadmap.md).
**Implemented P2.2 experiment:** [reactive-surface preview](p2-2-reactive-surfaces.md).
**Art/format authority:** [material authoring](material-authoring-contract.md).
The material renderer must not determine geology, resources, water or ecology.

![Technical design diagram showing authored substrate, geological feature geometry, independent coverage conditions and material response with one continuous copper vein crossing several regular hexagon caps](../assets/diagrams/material-overlay-stack.svg)

*Proposed pipeline. The continuous feature-mask interface is not yet implemented; P2.2 covers only narrow gallery previews.*


## Reactive surface conditions and coverage

Surface appearance must not collapse the world's distinct facts into a
single pre-baked texture. The visual stack conceptually distinguishes
**durable substrate**, **embedded ore/minerals**, **biological or seasonal
cover** (moss, lichen, grass, leaf litter, snow) and **surface conditions**
(wetness, frost, weathering, soot, localized damage or tracks). These are
not necessarily separate draw calls, nor must every category have a PNG.

- Metal-bearing mineral regions may receive *material-specific*, localized
  and view/light-dependent sheen or sparse glints. Do not assign metallic
  reflections to every mineral or use full-bright glow to indicate value.
- Wetness changes darkness and reflected highlight response without
  making rock metallic. Frost/snow coverage should favor exposed surfaces;
  accumulated snow or thick ice may require geometry.
- Moss and lichen are restrained patterned covers, not a flat global green
  tint. Thin shader coverage is acceptable for a preview; substantial
  clumps and plants will need geometry.
- A later decal/mark system may represent footprints, scuffs, mining cuts,
  ash/soot and structure wear without inventing new substrate IDs.
- When several conditions overlap, more superficial covers can conceal
  ore glints. Substrate and resource identity must remain intact.
- Current shader conditions are **diagnostic F8 presets**, not simulated
  weather/ecology or persistent world records. Their presence does not
  constitute artistic approval. See [P2.2 QA](p2-2-reactive-surfaces.md).

## Continuous patterns across neighboring hexels (planned)

**New art/pipeline rule:** the **surface material** and a **spatial feature
that crosses it** are different things. Large rock bedding, exposed ore
veins, extended fractures, roots and branching fungal structures must not
visibly restart at each 1m cell boundary simply because one local overlay
texture was repeated. The same underlying feature should be sampled or
projected consistently across adjacent exposed hex-prism caps, side
faces, corners and section boundaries.

A world-space / feature-local **coverage field** is one candidate for
this purpose: a bounded 3D rock/ore/fracture structure can generate
coverage where it intersects actual exposed faces; the host's authored
texture supplies its material character underneath. Thin patterns may be
shader masks or decals; substantial cracks, stems, crevices and snow
banks can require owned geometry. A continuous-looking *overlay* must
not reveal buried ore through non-ore rock, or create resource stock.
Precision must remain stable under chunk-origin shifts and eventually
the periodic world seam. The full toroidal hex-lattice translation is
still unapproved, so cross-world-seam functionality remains a future
acceptance gate.

**Space colonization belongs here only as an optional geometric authoring
tool** for selected connected deposits, fractures, roots, vines and other
branching motifs. It can output candidate connected skeleton edges; it
does not determine rock identity, mineral grade, metallic response,
weathering, actual ore occurrence, water flow, or visual quality.
Geologically planar veins, sediment layers, stockworks and replacement
bodies may require fault geometry, bands, splines or other techniques.
Never make all ores look like tree roots. The bounded experimental implementation is available in
[src/space_colonization.rs](../src/space_colonization.rs) after
[merged research PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30), but remains
unconnected to the game's material/overlay runtime. SCA is **not a
prerequisite** for completing the broader material pipeline.

**Appearance-composition order is semantic, not necessarily a stack of
separate draw passes:** durable substrate; embedded minerals in eligible
host; biological/seasonal cover; transient conditions; localized
marks and exposed highlights. Snow, frost or moss can **hide** exposed
metallic glints while leaving substrate, ore quantity and mineable state
unchanged. Wetness usually needs no branching network; it should follow
actual exposure and water/environment state when implemented.

An authored tileable alpha mask remains an appropriate fallback for
small/local ore clusters and the present F8 gallery. For *connected*
patterns, compare the tiled baseline against a single world-space feature
before committing to a universal implementation. Shared details,
feature-local masks and selective decals/geometry are preferred over
baking every rock × ore × grade × wetness × season combination.

**New visual review gate (not yet passed):** compare tiled and
feature-continuous copper on granite and shale at approximately 1m, 3m
and 8m, including two adjacent cells, the true hexagonal cap, all six
side orientations, a corner, a section edge, neutral light, shadow and
camera motion. Host recognition, poor-grade readability, grounded
material-specific sheen and pattern continuity matter more than how
complicated the generator is. Keep the simpler method if it looks
better or is cheaper at equivalent clarity. See the
[revised material and overlay roadmap](material-overlay-roadmap.md) and
[P2.2 preview record](p2-2-reactive-surfaces.md).

### Shared growth geometry as one optional input

![Space colonization as a reusable structural generator with flora, geology, cave, conduit and surface-pattern consumers, each maintaining its own authoritative rules](../assets/diagrams/spatial-growth-consumers.svg)

Space colonization can provide a **candidate structural skeleton**, while
ordinary authored masks, geological faults and other deterministic sources
remain equally valid. The merged headless SCA prototype is research code, not a dependency
of the overlay renderer or an implemented feature-mask generator.

## Shared authored patterns versus specialized material consumers

The grayscale-family proposal extends **beyond mineral overlays**:
rock/soil bases, bark, leaf surfaces, and seasonal/biological covers
can also reuse certain tonal patterns and palette ramps. But this
document is specifically the **surface-overlay consumer** of that
larger model. The [material-authoring contract](material-authoring-contract.md)
owns shared IDs, mask semantics and palette recipes; the
[vegetation architecture](vegetation-visual-architecture.md) owns bark
orientation, species silhouettes and foliage transparency.

An opaque granite surface does not need ore-like alpha coverage;
a leaf-card alpha silhouette is not an ore inclusion mask; and bark
striation should be aligned with local wood geometry rather than
world-space geology. Reuse compatible *data representation*, not
one universal mask projection or shader for every material.

## Candidate: grayscale mineral patterns and palette response (overlay consumer)

**Status:** proposed representation and art/benchmark comparison only.
Today F8 composites a **colored** native-copper RGBA overlay on the CPU;
its alpha is reused as an F10 preview mask. No general family lookup,
grayscale source format, shader palette ramp, BC4 encoder, or multi-ore
surface recipe exists in the shipped renderer.

A useful split is:

```text
approved host material (granite / shale) ──────────────────────────┐
authoritative geology: occurrence + grade + remaining stock       │
  └─ eligible feature geometry / region-owned shape                │
       └─ structural coverage / host clipping                      │
curated grayscale pattern family                                  │
  ├─ coverage alpha C (where ore is visible)                       │
  └─ tonal detail G (dark/light *inside* the ore, optional)         │
mineral species recipe                                              │
  ├─ compatible morphology subset + deterministic variant weights │
  ├─ color ramp / metallicness / roughness / oxidation             │
  └─ permitted hosts and response limits                           │
        ↓                                                          ↓
     final visible coverage       +       mineral color response over host
                                 ↓
                      cover/wetness/frost occlusion
                                 ↓
                           shaded surface
```

**Two different grayscale roles must not be conflated:** coverage
`C=0` is no visible mineral, whereas `G=0` can mean a *dark* pixel
inside a fully present ore inclusion. For one-channel quick prototypes
a designed tonal threshold may encode both, but this is an explicit
lossy simplification—not a generic material contract.

**Palette choice is mineral-specific.** Two or three tone-ramp stops
may help oxidized copper, silver-gray galena, yellow gold or red/brown
hematite retain believable midtones. Species-dependent geometry,
crystal structure, patina and dielectric/metallic reflection must
survive recoloring. Do not assign gold the identical tree-root-shaped
vein distribution as native copper just by changing RGB constants.

**Geological geometry and detail remain independent.** A shared
world-space deposit/fracture field decides whether the ore exists
and provides cross-hexel continuity. Curated grayscale tile patterns
can fill that field with micro/nodule detail without individually
restarting the deposit at each hexel. This is fully compatible with
optional SCA edges, simpler fault splines and hand-authored masks.

**Storage and performance must be measured, not assumed.**
An RGBA BC7 tile uses 16 bytes per 4×4 texels. BC4 can store one
linear scalar in 8 bytes per 4×4 texels, but **the current v2 baker
supports BC7, not BC4**, and additional texture lookups, palette
ramp sampling, branching or extra draw calls can cancel memory
benefits. A family of five IDs is not necessarily a GPU
`texture_2d_array`: array layers require compatible dimensions,
format and mip layout. Prefer shared assets and grouped compatible
arrays or other indexing *only after* the actual fixtures are
measured; maintain the existing BC7/RGBA fallback path. Avoid
packing linear coverage data into an sRGB color channel without an
explicit decoding/packing contract.

**Deterministic art variation:** select approved members from stable
world/feature coordinates and semantic context, using regional
weights/clustering to prevent checkerboard-style random variation.
Variations are an artistic family, not a new ore type or an editor
that changes the saved occurrence whenever a chunk loads.

### Minimal acceptance comparison

On the existing granite/shale test fixture, compare:

1. Current full-color transparent copper candidate (baseline).
2. One grayscale `C` + `G` inclusion pattern and one copper ramp.
3. Three to five compatible grayscale morphologies, weighted
   deterministically, using **the same** copper response in both hosts.
4. The same grayscale morphologies with a **second, visually different
   mineral recipe** (e.g. silver-gray galena), but only where the
   morphology actually suits that mineral.
5. A world-space continuous vein using the shared detail mask, with
   fixed occurrence and grade independent of appearance.

Require 1m/3m/8m native moving-camera screenshots, shadow checks,
F10 highlights off/on, host readability, no source/stock fabrication,
no pattern resets across caps/sides, correct linear color/coverage,
and GPU time + memory comparisons against the existing BC7 path.
Reject excessive sameness or shader cost and keep full-color
source-art exceptions. See [the material roadmap](material-overlay-roadmap.md).

## Integration direction

First prove a single deterministic world-anchored feature crossing two
neighboring hexels while preserving separate host, mineral and condition
inputs. Compare a simple fault/spline mask against a bounded optional SCA
graph source only *after* the compositor works. Masks are derived visual
data; actual ore stock and edit/depletion state remain authoritative.

No global runtime texture format, fixed shader-channel allocation, growth
algorithm dependency or per-hexel mask atlas is approved by this document.
