# Surface overlays and continuous feature appearance — design notes

**Scope:** proposed implementation boundaries and QA for embedded mineral,
biological/seasonal and reactive surface coverage. This is not a finished
overlay compositor, world-state system or production SCA integration.

**Visual authority:** [canonical style guide](../assets/STYLE_GUIDE.md).
**Roadmap:** [materials and overlays](material-overlay-roadmap.md).
**Implemented P2.2 experiment:** [reactive-surface preview](p2-2-reactive-surfaces.md).
**Art/format authority:** [material authoring](material-authoring-contract.md).
The material renderer must not determine geology, resources, water or ecology.

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
Never make all ores look like tree roots. The experimental implementation
is on [draft research PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30),
not the game's working material/overlay runtime. SCA is **not a
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

## Integration direction

First prove a single deterministic world-anchored feature crossing two
neighboring hexels while preserving separate host, mineral and condition
inputs. Compare a simple fault/spline mask against a bounded optional SCA
graph source only *after* the compositor works. Masks are derived visual
data; actual ore stock and edit/depletion state remain authoritative.

No global runtime texture format, fixed shader-channel allocation, growth
algorithm dependency or per-hexel mask atlas is approved by this document.
