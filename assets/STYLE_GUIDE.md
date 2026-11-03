# Hexel Survival — Canonical Art Direction & Style Guide

**Status:** project art-direction baseline • **Last reviewed:** 2026-11-03  
**Applies to:** terrain, geology, ores, soils, flora, water, structures, characters,
lighting, atmosphere, UI-related visual language, concept art and asset reviews.  
**Read this before designing, generating, implementing or approving visual assets.**

> **North star:** A **readable, handcrafted, stylized hexel survival world** with
> believable geology and living ecology, given the **weathered beauty, melancholy
> and dramatic warm/cool atmosphere of painterly dark fantasy**.
> Gameplay readability comes first. The world must feel rich, dangerous,
> beautiful and lived-in — **not photorealistic, not neon-cartoon, not a
> textureless world everywhere, and not a literal copy of any reference game
> or artist**.

This document captures decisions and references already discussed by the
project owner. It is a **creative standard for subsequent work**, not a claim
that every effect described below already runs in the engine. Where a specific
look or renderer feature is still unapproved, it is labeled **candidate** or
**future**, not falsely called complete.

## 1. Visual influences and how they fit together

| Reference direction | Keep | Do not copy / overgeneralize |
| --- | --- | --- |
| **Hytale-like stylized 3D** | Clear readable forms, grouped values, hand-designed detail, silhouettes, grass/soil edges, appealing low-complexity shapes | Exact proprietary textures, models, color layouts, character designs or a uniformly toy-like tone |
| **Vintage Story-like geological and ecological depth** | Host-rock-specific ore, strata, separate cover and plants, useful material identity, climate/season context | Its exact texture assets, mod assets, block taxonomy or hard-coded climate numbers |
| **Luis Royo-influenced painterly fantasy mood** | Melancholy, age, hardship and beauty; dramatic chiaroscuro; dust, dusk, firelight, muted neighboring hues, colored shadows, selective accents | Literal recreation of the artist's compositions, characters, illustration style or detailed brushwork on every terrain tile |
| **Earlier Cube World-like / textureless-voxel concept** | Strong **3D foliage geometry**, recognizable silhouettes, colored forms, seasons, weather, beautiful light over simple shapes | A blanket ban on textures: **geology and ground surfaces are textured** in our current direction |
| **Hexel Survival's own design** | Regular hexagonal prisms; simulation-driven geology and ecosystems; distinct authored material faces; mature survival atmosphere | Cube-only UV assumptions, fake worldgen, visual cues that contradict the authoritative world state |

**Synthesis:** geology gives the world its *physical identity*; clear modular
materials and 3D forms give players *readability*; atmosphere gives it
*emotional identity*. The rendering is full, explorable stylized 3D, not a
two-dimensional illustration pasted onto hexel geometry.

**Style guardrail:** avoid the extremes of bright uniform pixel-cartoon,
grainy photoreal scans, or illustration-level clutter. Use *purposefully
simplified* detail within a grounded material palette.

## 2. Non-negotiable visual principles

1. **Readability before realism.** A rock, ore-bearing face, grass patch,
   useful plant, interactable object or hazard must be distinguishable at
   normal play distance, not only when zoomed in or with a tooltip.
2. **Make materials by design, not by random noise.** Start with large,
   readable shapes and material-specific structure; add sparse mid- and
   micro-detail. Break up obvious repetition without making static.
3. **Preserve geological identity.** Granite ≠ shale ≠ basalt ≠ limestone.
   Ore species and host rock are separate facts. Visual simplification must
   not make all materials interchangeable.
4. **Respect the eight faces of a hexel.** The top hexagon, bottom hexagon
   and each of six side rectangles must look correct at physical scale.
   A cube UV template or one image stretched 0–1 across each side is wrong.
5. **Use layers where the world uses layers.** Substrate, ground cover,
   individual vegetation, host rock, ore species, ore grade, wetness, snow
   and weathering have distinct meanings. Do not merge them into a
   single inflexible "biome block" artwork.
6. **Color supports meaning.** Most surroundings use harmonious, relatively
   restrained hues. Mineral signatures, flowers, fire, unusual plants,
   danger and interaction can carry selective contrast.
7. **Atmosphere should elevate, not hide, gameplay.** Dramatic dusk,
   fog and colored shadows must not make important materials impossible to
   identify. Test at neutral daylight as well.
8. **Optimization is an art requirement.** Reuse layers, employ compression,
   reduce overdraw and draw calls, scale foliage by distance, and measure
   memory/frametime before adding costly effects. No feature is approved
   solely because it looks good in a still image.
9. **Authorship and simulation stay separate.** A texture or shader must
   not determine ore quantity, fertility, which plant grows, its health or
   the player's saved world changes.
10. **Create original assets.** References are a design vocabulary, not
    source files to trace, crop or reproduce without appropriate rights.

## 3. Color and material language

### Palette direction

- **Base world:** warm umber, sienna, clay and ochre; charcoal and slate;
  weathered gray stone; dusty gold; moss, olive and deep forest greens.
  Saturation is generally moderate or muted; do not globally desaturate
  everything into gray or sepia.
- **Lighting:** maintain warm/cool relationships. Warm sunlight or fire may
  push surfaces toward amber; shade and moonlight may push them toward
  blue-green or violet-gray. Shadows have **hue**, not just less brightness.
- **Highlights:** material-aware, not automatically white. Glossy metal,
  translucent crystal, dry dust and velvet leaves respond differently.
- **Accents:** purposeful reds/oranges (fire, autumn leaves, select flowers),
  mineral greens and blues, precious metals, bioluminescent or magical
  exceptions. A bright accent should be **earned**, not spread everywhere.
- **Biomes/seasons/time:** may adjust lighting, foliage and ground cover
  while retaining each material's identity. Local ecology drives what
  exists; a global color filter must not replace actual vegetation states.

**No rigid global hex-code palette is approved yet.** Create scoped swatch
boards against representative daytime, night, wet and sunset lighting before
locking numeric colors. Prior reference boards show *relationships* more
reliably than exact color values.

### Surface detail hierarchy

- **Macro:** the main diagnostic forms (bedding, fractures, granitic clumps,
  soil aggregates, bark ridges). These must survive a few metres of distance.
- **Mid:** carefully grouped cracks, inclusions, roots, pebbles, clumps,
  natural color variation; deliberately uneven, not repeated grid noise.
- **Micro:** restrained grain for material tactile quality. Avoid
  speckle/static that obliterates ore information or causes shimmer.
- **Pacing:** provide visually calm areas between focal forms; avoid
  every centimetre having equal contrast.
- **Albedo:** author largely without directional shadows/highlights baked in;
  lighting and AO should respond to the world, not a fixed painted light
  direction.

Texture detail **is not proportional to image resolution**. Higher pixel
counts do not excuse noisy designs. Square and rectangular images are both
allowed; author for physical texels per metre and valid hexel UVs.

## 4. Hexel-specific terrain standards

The current demo hexel is **1 metre across flats × 1 metre tall**. Each
of its six vertical edges is approximately **0.5773503 m** wide; the
top/bottom are real hexagons. These are the **playable prototype's**
dimensions, not proof that the full toroidal world is complete.

| Material role | Visual requirement |
| --- | --- |
| **Top cap** | Distinct top-down appearance for soil cover, rock grain, sediment or ground use; proper hex cap UV projection |
| **Six side faces** | Real cross-section: strata, bark, exposed soil, fractures, top-anchored turf; same physical texel density as caps |
| **Bottom cap** | Appropriate underside/exposed interior, not an accidental repeating grass rim |
| **Top–side join** | Related colors/details, no false green paint, no obvious aliasing or bright seam |
| **Side corners / adjacent hexels** | No abrupt stretch, discontinuous texel density or visibly tiled diamond artifacts |
| **Scale** | Inspect at eye height: the 1m hexel is about **3/5** of the prototype character's 1⅔m height |

Keep UVs in metres. Test real **hexagonal** caps and all six rectangular
sides; do not require hex-shaped PNG files. A single side texture recipe
can be shared across six sides initially; any directional variants require
explicit design.

The legacy 32px textures and current P0 256px generated BC7 swatches are
**references/test assets**, not a mandatory resolution, production art,
or artistic finish threshold.

## 5. Geological and ore art rules

### Host-rock bases

- **Igneous:** grouped crystalline masses or volcanic structure; granite
  may show clustered light/dark/feldspar grains, but not uniform noise.
- **Sedimentary:** bedding, grain size, lamination, fractures and weathering
  readable on the side. Shale is flatter and more layered than granite.
- **Metamorphic:** purposeful foliation, banding or reworked crystalline
  direction; avoid interchangeable gray rock.
- **Minerals:** distinctive crystal habit, fracture, color, sheen and
  geological context as appropriate; not every mineral must be glowing.
- Consistent art language across families matters more than imitating
  individual photographic specimens.

### Ore-bearing faces: **host + ore-only overlay + grade**

**This is a binding gameplay-readability rule.** The previous P0 copper
extraction was too subtle: orange flecks could be mistaken for ordinary
granite grains. Do not use it as the final ore visual standard.

- The **host rock remains plainly recognizable**; the ore overlay only
  depicts mineral-bearing formations. No rectangular baked-in rock backdrop
  around a mineral inclusion.
- **Ore species need distinct shape signatures** in addition to color:
  copper can have branching seams/nodules, galena angular metallic clusters,
  iron ores strong dark/red/brown masses, gold small clear metal-rich veins,
  etc. These are **candidate art motifs**, not claims that every geological
  occurrence has one morphology.
- Favor **readable groups**: a few medium/large mineral shapes plus
  subordinate fine detail, rather than tiny scattered colored grains.
- Four existing grade classes: **poor, medium, rich, bountiful**. Increasing
  grade should usually increase visible presence/connectedness while
  preserving host character. **Actual resource quantity comes from game
  state**; the pixels are a visual cue, not a numerical inventory.
- Even **poor** exposed ore must be identifiable at practical mining
  distance under neutral light, without a tooltip. Avoid arbitrary
  full-bright glow; genuine luminous minerals are an exception.
- Keep alpha masks reusable across host types rather than authoring every
  rock × ore × grade permutation as a separate opaque texture.
- Ore hidden behind opaque rock **must not appear through the rock** merely
  because an occurrence is nearby.

**Initial approval test:** copper on **granite and shale**, three candidate
visual families (**veins / clusters / hybrid**) and poor–medium–rich
prominence. Favor a **hybrid** only after in-game side-by-side comparison;
the direction is proposed, not yet approved. Verify readability at
approximately 1m, 3m and 8m in neutral light and a dark environment.

## 6. Soil, grass and vegetation

### Soil + living ground cover

- Soil fertility/substrate is independent of grass coverage; dry or sparse
  does not necessarily mean poor soil.
- **Top:** a designed grassy surface with clumps and small variation,
  not flat green paint or noisy photographic leaves.
- **Sides:** an irregular **shallow upper turf rim** with some organic
  transition into soil. Keep the lower soil face legible, with restrained
  root/aggregate detail; never extend long repeated green stripes down
  the whole 1m face.
- **Bottom:** soil or the appropriate substrate, not grass.
- Separate **bare, very sparse, sparse and full** cover, plus cover
  families (grass, moss, leaf litter) when implemented. Dry/wet/seasonal
  variants should not require duplicating the whole soil asset.
- **Modeled grass tufts are separate plants**, not part of the soil
  material or a guarantee of the underlying soil fertility.
- Species, height stage, cover density, wind, growth state and season
  remain separate from ground substrate.

### Trees, shrubs, crops and fungi

Use the earlier *textureless full-voxel foliage* reference for one
important idea: **recognizable forms at a distance**. That does **not**
mean forcing all leaves into cubes or banning leaf/bark textures.

- Prefer clear, intentionally grouped **3D geometry/silhouettes** for
  major plants, trunks, branches, grass clumps and distinctive foliage.
  Use textured/cutout foliage only where it measurably improves the look
  within performance budgets. Avoid needless layers of tiny alpha cards.
- Trees need meaningful bark and **end-grain** face roles; foliage must not
  share a generic grass material. Species should remain identifiable
  through shape, leaf form and seasonal behavior.
- Shrubs, wildflowers, crops, mushrooms and reeds require recognizable
  growth forms and enough visual contrast to find useful species.
  Rare colors can accent the scene but cannot replace correct habitat data.
- Wind motion should reflect species and growth form; use restrained
  animation and sensible distance/quality cutoffs.
- The flora research registry currently inventories **208 historical
  research rows**, not 208 approved assets or necessarily unique species.
  Do not make up production assets, growth states or spawn parameters from
  this number.

## 7. Landscape, architecture and human-made objects

- **Landscape:** large readable silhouettes (cliffs, river valleys,
  forests, mountain systems) should emerge from the geological/climate
  design, not aesthetic hard-coded biome borders.
- **Structures:** weathered stone, worn plaster, aged timber, cloth,
  masonry and useful medieval materials; lived-in wear rather than
  uniform "grimdark dirt" on every surface.
- **Narrative tone:** loneliness, hardship, warmth of refuge, ruins,
  historical depth, windswept grass, changing seasons. Avoid turning
  every scene permanently bleak; bright mornings and flourishing
  habitats make dusk and danger meaningful.
- **Characters/props:** sturdy readable silhouettes, believable tool
  materials and costume construction; an intimate painterly storytelling
  mood is suitable for concept/key art, but runtime models should obey
  camera distance, animation and gameplay legibility.
- **UI and icons:** a related muted palette, clear contrast, readable
  item silhouettes, consistent metal/wood/cloth visual treatment. Avoid
  ornate decoration that obscures interaction targets.

## 8. Renderer and atmosphere: direction vs implementation

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
meter-scaled hex-face UVs, a small P0 material gallery, **F6** UV checker
and **F7** checker-density diagnostic. BC7 for 12 P0 256px material
layers is working on supported GPU hardware with an RGBA fallback.
There is **no** style on/off mode, full P1 runtime material loader,
living flora renderer, planetary weather, chunk streaming or complete
day/night renderer yet. Do not imply otherwise.

Lighting and post-process settings must not make critical resource
identification depend solely on cinematic light, bloom or outlines.
Enable asset inspection in neutral/base light.

## 9. Production asset pipeline, compression and budgets

- **Authoritative inventory:** `assets/materials/catalog.v2.json` tracks
  55 legacy geology/material identities, currently all `needs_art`.
  This guide changes **no approval states**.
- **Face/overlay contracts:** see
  [Material authoring contract](../docs/material-authoring-contract.md)
  and [P1 authoring pipeline](../docs/p1-material-pipeline.md).
  Stable IDs, explicit face roles and physical `repeat_metres` belong in
  the catalog; gameplay never persists GPU layer indices.
- **Source art:** new files go to `assets/production/` after authoring,
  not over the 55 original source textures. Record authorship and review
  status; keep originals for comparison.
- **Validation:** `cargo run --bin material-baker -- validate`.
  **Offline BC7 bake:** `cargo run --release --bin material-baker -- bake`.
  The current tool supports rectangular and square 4px-aligned RGBA8
  sources; it produces separate BC7 textures and an index under
  `target/materials-v2/`. The runtime does **not yet** load this index.
- **Compression:** P0 uses build-time BC7, native GPU BC7 where supported,
  one-time CPU decode RGBA fallback otherwise. It currently uses one mip
  level. The **75% texture memory saving** applies to that 12-layer test,
  not total game GPU memory or FPS.
- **Future optimization:** quality-tested mipmaps, texture grouping/
  atlasing, streaming/LOD, alpha overdraw, geometry instancing and
  foliage distance tiers, and platform-specific compression (ASTC/ETC2)
  are evaluated when needed rather than assumed implemented.
- **No fixed 32px rule.** Physical material appearance and measured
  budget determine source resolutions. Compare 64/128/256+ texels/metre
  only where each actually delivers visible benefit.
- **Gameplay parity:** low-quality settings may reduce detail/effects,
  but must not hide ores, useful flora or hazards that higher quality
  settings reveal.

## 10. In-game review and acceptance gates

**Every new material passes all of these, not just a pretty image review:**

1. **In isolation:** recognizable material family, correct macro pattern,
   coherent restrained palette, no photograph-like static or misleading
   baked lighting.
2. **On hexels:** top, bottom and all six sides, six corners, cap/side
   seams, adjacent repeats, mixed soil/rock neighbors, UV scale in metres,
   plausible vertical stratification/turf edge.
3. **At player scale:** near inspection (~1m), resource recognition (~3m)
   and distance silhouette (~8m); check 1⅔m prototype eye/body scale
   where relevant. These are **provisional visual QA distances**, not
   engine LOD thresholds.
4. **Across contexts:** neutral daylight, overcast/wet, sunset and
   low-light. **Only neutral/basic lighting is available today**;
   other comparisons are future acceptance gates, not blockers to
   writing candidate assets now.
5. **For ore:** identify the ore-bearing cell without tooltip at
   practical distance, distinguish ore from naturally colored host
   minerals, and retain the host's recognizable identity. Grade changes
   prominence without inventing resource amounts.
6. **For grass/flora:** species/cover variation and growth states remain
   distinct; avoid repeating green vertical streaks, flat coatings and
   fragile unreadable silhouettes.
7. **In motion:** look for texture shimmer, visible tiling, blur, obvious
   alpha noise and loss of visual cues from normal camera movement.
8. **Technical:** run registry/catalog validation and tests; review
   BC7 quality on supported GPU and RGBA fallback as available; record
   CPU/GPU memory, frame timing and draw-cost regressions for
   substantial feature changes.
9. **Approval:** store candidate/reference/approved status **honestly**.
   The project owner evaluates screenshots from the live game and approves
   material identity, gameplay readability and style cohesion.
   No automatic "approved" merely because CI is green.

**Recommended proof board:** low-fertility soil and grass cover states;
granite, shale and clear copper overlays in both hosts; a small stand of
distinct grasses/foliage models later. Test **neutral** versus full-style
rendering once that toggle exists. Retain P0 screenshots as the baseline,
not as mandatory final aesthetic quality.

## 11. Instructions for future AI assistants and collaborators

When starting **any** new visual, vegetation, texture, shader, UI or lighting
task in this repository:

1. **Read this file first**, then the relevant existing engine/material/
   flora docs. Check repo state before recommending code changes.
2. State whether a detail is **established**, a **candidate**, or a
   **future goal**. Do not mislabel a concept sheet as an implemented effect.
3. Translate the task into **testable visual and technical outcomes**,
   including hexel geometry, gameplay readability and performance.
4. Preserve the established world/material separation. **Do not**
   refactor geology, worldgen or the renderer just to produce an art asset.
5. For new images, produce **original** work; don't reproduce Hytale,
   Vintage Story, Luis Royo, or another creator's protected art.
6. Prefer a small **in-game vertical slice** (soil / two rock hosts /
   one readable ore / flora sample) to mass-generating unvalidated files.
7. Keep authored source and baked runtime derivatives separate. Reuse BC7
   and P1 validation rather than inventing an incompatible asset pipeline.
8. For new engine effects, measure a neutral baseline, provide graceful
   low-tier fallback, and never trade away gameplay visual cues.
9. After testing, update the relevant docs and record what **actually**
   changed; do not quietly rewrite this guide or claim production approval.
10. When a new request conflicts with this guide, **surface the conflict
    and ask the owner to decide** before treating the new look as canonical.

### Asset review mini-brief

For each new sample, record:

- Stable material/species ID, source/creator and approval state
- Family + geological/ecological role, and why it is recognizable
- Top/side/bottom or plant-geometry requirements; base/overlay split
- Color/value family, macro/mid/micro shape design, physical repeat
- Host-rock/grade/cover variants if applicable
- Screenshots with F6 checker where relevant, normal view and comparison
- Artifact resolution, compression/alpha behavior and performance notes
- Observed defects, open questions and the owner's decision

## 12. Reference sources and authority

**User-provided sources that informed this guide:**

- ChatGPT Library: **"Luis Royo Inspired Fantasy Style Guide.png"** —
  reference board for muted palettes, colored shadows, selective accents,
  outline/cel-shading candidates, foliage wind, grading and quality tiers.
- ChatGPT Library: **"Voxel World: No Textures.png"** — earlier reference
  board for strong 3D foliage forms, color-based voxel material appearance,
  weather/seasons and atmospheric scale; **not** a current no-textures mandate.
- User-shared Hytale-style screenshots, grass hexel/UV diagrams,
  Vintage Story-style geology and flora references, painterly narrative
  scenes and ore-readability discussions in this project's conversation.
- Project contracts: [material rules](../docs/material-authoring-contract.md),
  [P1 catalog/baker](../docs/p1-material-pipeline.md),
  [flora mappings](../docs/flora-to-assets.md),
  [material format](../docs/material-format-v2.md),
  [compression](../docs/texture-compression.md),
  [world vision](../docs/world-generation.md).

The reference imagery is **not copied into this repository**; this text
records the intent so it remains actionable even without Library access.
Inspirations do not grant a license to extract, trace or redistribute
third-party art or game textures.

**Precedence:** explicit new direction from the project owner takes
priority; preserve existing engine/data contracts until a deliberate
migration; otherwise this document is the default for visual work.
Unresolved candidates remain open until the owner approves them.

---

*This document establishes a shared visual decision framework. It does
not create new art, change the world generator, turn on rendering effects,
or approve the pending 55 material replacements.*
