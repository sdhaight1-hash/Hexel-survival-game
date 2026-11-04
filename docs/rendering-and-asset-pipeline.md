# Rendering, material production and performance — technical direction

**Scope:** renderer capabilities, future atmosphere/lighting options,
offline material production, compression, foliage budgets and performance.
These implementation details are separated from the
[canonical visual style guide](../assets/STYLE_GUIDE.md); it remains
authoritative for palette, readability and artistic approval.
This page summarizes intended architecture and current limits; it does
not claim the unimplemented features exist.

For actual BC7 data and fallback tests, see
[texture compression](texture-compression.md). For v2 authoring and
baking, see [P1 pipeline](p1-material-pipeline.md). For phase gates see
[material/overlay roadmap](material-overlay-roadmap.md).

## Renderer and atmosphere: direction vs implementation

The **style/reference board proposes these goals**. They are **not claims
that the current 16³ wgpu demo has these features**.

| Technique | Intended treatment | Status |
| --- | --- | --- |
| Dynamic time-of-day lighting | Broad readable light / shadow shape changes, warm/cool shifts | **Future** |
| Colored shadows | Hue-sensitive light response and material-appropriate shadow tone | **Future** |
| Material-aware stylized lighting / limited tonal bands | Moderated cel shading, not posterized every surface | **Candidate** |
| Ambient occlusion | Ground contact and geometry depth; no dirty halos around every hexel | **Future** |
| Outlines | Prefer characters, focal silhouettes/interactables; **not every hexel edge** | **Candidate, configurable** |
| Distance haze / fog | Scale, depth and mood without losing important nearby resources | **Future** |
| Color grading | Time/weather/biome-aware subtle palette bias; controlled saturation, exposure and contrast | **Future** |
| Selective bloom / highlights | Fire, reflective ore and rare luminous elements, not universal glows | **Candidate** |
| Wind + foliage response | Species/group-specific motion with distance and quality cutoffs | **Future** |
| Weather and seasons | Rain, snow, storm, changing leaf/cover states; world data remains authoritative | **Future** |
| Low / medium / high visual presets | Measured performance fallbacks, same gameplay signals across tiers | **Future** |
| Neutral / full-style A/B view | **Developer comparison tool** using identical scene/time/camera | **Future** |

**Current reality:** one bounded 16×16×16 first-person test chunk,
meter-scaled hex-face UVs, a P0 material gallery, **F6** UV checker
and **F7** checker-density diagnostic. BC7 for 12 P0 256px material
layers is working on supported GPU hardware with an RGBA fallback.
A limited **F8** gallery previews revised granite, shale and transparent
copper candidates; **F9** cycles diagnostic wet/moss/frost coverage;
**F10** toggles localized copper glints. These are **not** simulated,
approved art or generalized production overlays. There is **no**
style on/off mode, full P1 runtime GPU material binding, living flora
renderer, planetary weather, chunk streaming or complete day/night
renderer yet. Do not imply otherwise.

Lighting and post-process settings must not make critical resource
identification depend solely on cinematic light, bloom or outlines.
Enable asset inspection in neutral/base light.

## Asset pipeline, compression and performance budgets

- **Authoritative inventory:** `assets/materials/catalog.v2.json` tracks
  55 legacy geology/material identities: **3 `candidate` (granite, shale,
  native copper), 52 `needs_art`, and 0 `approved`**. The P2.1 image
  revisions and P2.2 shader preview do not change approval statuses.
- **Face/overlay contracts:** see
  [Material authoring contract](material-authoring-contract.md)
  and [P1 authoring pipeline](p1-material-pipeline.md).
  Stable IDs, explicit face roles and physical `repeat_metres` belong in
  the catalog; gameplay never persists GPU layer indices.
- **Source art:** new files go to `assets/production/` after authoring,
  not over the 55 original source textures. Record authorship and review
  status; keep originals for comparison.
- **Validation:** `cargo run --bin material-baker -- validate`.
  **Offline BC7 bake:** `cargo run --release --bin material-baker -- bake`.
  The current tool supports rectangular and square 4px-aligned RGBA8
  sources; it produces separate BC7 textures and an index under
  `target/materials-v2/`. The runtime **checks and validates** a present
  index at startup but does **not yet** bind its recipes as
  general-purpose GPU materials.
- **Compression:** P0 uses build-time BC7, native GPU BC7 where supported,
  one-time CPU decode RGBA fallback otherwise. It currently uses one mip
  level. The **75% texture memory saving** applies to that 12-layer test,
  not total game GPU memory or FPS.
- **Future optimization:** quality-tested mipmaps, texture grouping/
  atlasing, streaming/LOD, alpha overdraw, geometry instancing and
  foliage distance tiers, and platform-specific compression (ASTC/ETC2)
  are evaluated when needed rather than assumed implemented.
- **Foliage source vs instance:** author shared bark/leaf cutout textures,
  structural blueprints and appearance variants **once per recipe**,
  not per world plant. Track provenance and versions; climate and
  harvested state remain outside the visual material system.
- **Foliage GPU cost:** budget alpha-cutout overdraw, leaf-mip coverage,
  transparent-surface artifacts, geometry and upload churn separately.
  A lower-poly card layout is not automatically cheaper or clearer than
  compact hybrid geometry. Preserve crucial plant recognition on all tiers.
- **No fixed 32px rule.** Physical material appearance and measured
  budget determine source resolutions. Compare 64/128/256+ texels/metre
  only where each actually delivers visible benefit.
- **Gameplay parity:** low-quality settings may reduce detail/effects,
  but must not hide ores, useful flora or hazards that higher quality
  settings reveal.
