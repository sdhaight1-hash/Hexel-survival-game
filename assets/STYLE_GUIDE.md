# Hexel Survival — Canonical Art Direction & Style Guide

**Status:** project art-direction baseline • **Last reviewed:** 2026-11-03
**Revision note:** vegetation rendering, plant-structure generation,
LOD/persistence and Voxel-Game-Engine reference incorporated.  
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

**Illustrated companion infographics:**
[1 — Visual identity](style-infographics/01-visual-style.png) ·
[2 — Geology and terrain materials](style-infographics/02-terrain-materials.png) ·
[3 — Flora and vegetation architecture](style-infographics/03-flora-architecture.png).

**Important:** These are **AI-generated concept/reference diagrams**, not production
textures, implementation screenshots, or precise engineering specifications.
Some miniature panels simplify hexagonal prisms as blocky/cube-like shapes
or contain approximate/inaccurate shorthand text. **The written rules
in this guide and the source contracts take precedence**: a hexel has
a hexagonal top and bottom plus six rectangular sides; the project
has **no mandatory 4 or 32 texels/metre standard**; ore is composited
*on* its host rock; and vegetation cutout cards need **not** be
camera-facing billboards. Illustration does not imply a feature is implemented.

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

### Illustrated reference — the visual pillars

[![Hexel Survival visual style infographic: artistic influences, palette,
hexel readability and asset approval criteria](style-infographics/01-visual-style.png)](style-infographics/01-visual-style.png)

*Companion board 1/3: a visual mood and style reference, **not** a definitive
face-geometry or texel-density diagram. See §4 for exact hexel dimensions.*

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

### Illustrated reference — terrain and material hierarchy

[![Hexel Survival material infographic: geology, soil cover, ore overlays,
UVs, texture production and readability](style-infographics/02-terrain-materials.png)](style-infographics/02-terrain-materials.png)

*Companion board 2/3: conceptual material layering and ore-readability
examples. Rock, soil and ore are **not** always physically stacked as shown;
an ore overlay belongs on its host material. The prose, catalog and UV
contracts below are authoritative.*

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

### Illustrated reference — hybrid vegetation and procedural growth

[![Hexel Survival flora infographic: cutout cards, 3D meshes, hybrids,
plant growth methods, LOD and persistence](style-infographics/03-flora-architecture.png)](style-infographics/03-flora-architecture.png)

*Companion board 3/3: visual design goals and prototype concepts, **not**
proof of live ecosystem features. Many foliage cards are fixed-angle
crossed planes, not necessarily billboards; tree and plant mechanics
remain subject to the source/runtime distinctions below.*

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
  **Hybrid foliage is the default direction:** choose fixed-orientation
  textured cutout planes, actual 3D geometry, or both according to each
  species' shape and gameplay role. A few efficient cards are desirable
  for dense grasses and leaves; distinctive stems, caps, seedheads and
  branches may deserve real depth. Avoid needless layers of tiny alpha cards.
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

### Plant geometry: 2D cards, 3D meshes and hybrids

**Established design direction:** use a **hybrid vegetation asset system**.
Vintage Story uses crossed-plane vegetation for some grass and custom
plant geometry for other forms; Hytale's stylized model vocabulary
includes flat quads as well as volumetric geometry. Those are useful
design patterns, **not sources of assets to copy**. Do not decide
a representation from the words "flower" or "mushroom" alone.

A **card** is a flat textured polygon in 3D space with cutout alpha; it
need not billboard toward the camera. Crossed X-cards, staggered sprays,
radial cards and deliberately angled fronds all give different silhouettes.
A **mesh** supplies actual volume, thickness, shading and occlusion.
A **hybrid** uses both in one coherent plant asset.

| Plant family / role | Default candidate | Shape and texture considerations |
| --- | --- | --- |
| Dense short grass and meadow fillers | Small crossed/radial cutout-card clusters | Good density at low geometry cost; multiple varieties and heights; avoid flat grass "walls" |
| Tall grass, reeds, cattails, papyrus | Hybrid blades/cards and volumetric stems/seedheads | Readable stem height, seedhead and waterline identity |
| Small wildflowers | Cards or hybrid | Discernible flowerhead, petals, leaves, bud/seed stages; useful species must be findable |
| Ferns, vines and many foliage sprays | Angled/double-sided cutout cards on simple support geometry | Frond shape and direction, avoid card stacks with huge empty alpha areas |
| Mushrooms and fungi | Usually simple 3D caps/stalks; cards for appropriately thin forms | Cap, gills/pores, shelf attachment, distinctive species silhouettes |
| Bushes, berries and shrubs | Hybrid | Woody structure, layered leaves, visible berries/flowers and useful harvest parts |
| Crops | Growth-stage-specific hybrid/3D | Seeds, stalks, harvestable heads/fruits, depleted/ripe state all readable |
| Trees | Volumetric trunk/branch structure + grouped leaf/needle cards | Species-specific crown, bark and exposed end-grain; canopy silhouette at a distance |
| Cacti and succulents | Mostly 3D | Important mass, arms, spines/ridges simplified for actual player scale |
| Water plants | Species-dependent | Flat water-lily surfaces, emergent hybrid reeds, underwater plant cards/meshes |

These are **starting choices, not universal biological rules or final asset
approvals**. A landmark mushroom could justify more geometry; a distant
shrub might use only a few cutout planes. Do not confuse the greenery
**painted on a terrain cap** (ground-cover layer) with actual clumps,
flowers and mushrooms growing above it (independent plant instances).

**Style standard:** simplified but botanically recognizable forms,
grouped blades and foliage masses, graceful non-cubic curves where useful,
restrained texture noise and colors compatible with the weathered,
painterly world palette. We borrow Hytale's clarity, not an absolute
cuboid-only modeling restriction. The older textureless-voxel exploration
supports geometry-driven silhouettes, **not** rejection of bark, leaf
or terrain textures.

### Plant structure generation is separate from appearance and ecology

**Core architectural rule:** *where a plant can grow*, *which structure
its recipe creates*, *how its life state changes*, and *how its mesh/cards
are rendered* are **four distinct responsibilities**.

```text
Habitat/placement + approved species rules
    -> deterministic plant instance + growth state
    -> species-authored structural recipe
    -> versioned structural blueprint (wood, twigs, leaf sites, bounds)
    -> rendered near/far mesh, leaf cards, textures and wind
```

The blueprint is a **design target**, not an already shipped type in
Hexel Survival. It should permit several growth stages and different
render meshes/LODs without letting a material texture become the authority
for plant identity, fertility or resources. A 3D plant can have a 2D-card
leaf crown; an L-system tree need not be rendered with individual voxels.

**Algorithm choices and intended role:**

| Approach | Intended use | Status / constraint |
| --- | --- | --- |
| **Species-authored templates and parametric branching grammars** | Default for ordinary grass clusters, flowers, mushrooms, crops, common trees and shrubs | **Preferred first implementation.** Predictable and artist-directed; vary seeds within species bounds |
| **Constrained/parametric L-systems** | Selected branching trees, vines and woody plants with distinctive branching rhythms | **Candidate advanced authoring tool**, not mandatory, not currently a general L-system engine in either runtime |
| **Space-colonization / attractor-guided growth** | Special spreading/ancient trees, unusual canopies and obstacle-responsive growth | **Candidate specialized precompute/cached authoring**; the old editor's *CanopyGuided* mode is only inspired by this idea, **not** a full iterative space-colonization solver |
| **Coherent noise / weighted scatter** | Habitat patch placement, crown irregularity, cover density, bounded structural variation | **Foundational candidate**; noise placement is not itself plant biology, and noise-driven 3D shapes do not replace authored species forms |

Do **not** call L-systems an automatic realism guarantee or implement
multiple full generators merely for completeness. Even mathematically
complex branches require authored crown envelopes, taper, phyllotaxy,
leaf arrangement, gravity response and silhouette constraints.
Space-colonization attractors are **geometric targets**, not
automatically a simulation of actual sunlight or nutrients.
Use these algorithms **offline or on controlled regeneration events**,
not as repeated work every render frame.

**Species examples:** oak may use heavy, irregular limbs and broad
crown lobes; pine regular branching tiers and needle sprays; willow
drooping curtains and riparian habit; larch spur needles; palms crown
fronds; mushrooms usually need no branching solver at all.
All shape parameters are **artistic/biological approximations** until
validated, not measured universal botanical constants.

### Plant size, hexel ownership and persistent life state

- The demo's **1m-across-flats, 1m-tall hexels** describe the terrain
  substrate and can describe *harvestable wood occupancy* for big trees.
  **Do not snap small plant render geometry to a 1m lattice.**
  Flowers, fungi, roots, stems, grass blades and thin branches may be
  centimetre-scale 3D geometry rooted at locations on a hexel surface.
- Large woody plants may use **logical hexel-owned cells** for collision,
  cutting and persistence while their *rendered* branch tubes and foliage
  are finer and curved. Keep mesh ownership and interaction semantics
  explicit; avoid visible phantom branches after removing a cell.
- A reused source plant definition is not a world instance. Plant state
  must independently record growth/age, health, season, harvesting,
  damage and player actions, according to approved gameplay systems.
  Cutting a tree or picking a mushroom must **not** be undone by a
  chunk being unloaded and procedurally regenerated.
- World seed + **canonical periodic Q/R position** can produce an
  unmodified initial individual deterministically. Later changes need
  authoritative saved edits/state. Version the generator/asset recipe so
  changing branch rules does not silently replace trees in old saves.
  Cross-chunk crowns, roots and branches need ownership/culling rules.
- Older authoring tools may describe local **Q/R/Z (Z-up)**; the current
  Hexel Survival scene uses **Q/R/Y (Y-up)**. Any import must convert
  coordinates and units explicitly and check periodic boundary rules.

### Rendering, alpha and vegetation performance

- **Cards:** use double-sided, fixed-orientation alpha-cutout where
  appropriate. Prefer cutout over expensive sorted alpha blending for
  ordinary leaves/grass, but measure **alpha overdraw**, not just
  triangles. A tiny low-fill-rate quad can be worse than compact geometry.
- **Silhouette/edge quality:** test card cutout thresholds, filtering,
  alpha-aware mipmaps and thin-leaf coverage; prevent distant grass from
  disappearing or shimmering. Ordered coverage, MSAA alpha-to-coverage
  and thin-leaf transmission are **comparative options**, not automatic
  required features of the playable renderer.
- **Assets/texture memory:** share atlas/BC7-ready leaf/bark materials
  among instances; cache baked recipe outputs. Seasonal tint changes
  should not require rebaking foliage or regenerating tree geometry.
  Existing P0 BC7 success **does not imply** leaf-alpha mips and
  production foliage compression are solved.
- **Instances/LOD:** batch repeated geometry and plant instances by
  material/species where practical; use chunk/region visibility,
  distance-appropriate silhouettes, wind cutoff and bounded buffers.
  Close plants may be hybrids, middle-distance plants simplified card/
  mesh clusters, far trees simple 3D crowns or multi-view impostors, and
  very distant vegetation ecological canopy coverage. A camera-facing
  billboard is **optional**, not a universal distance rule; hilltop
  viewpoints must not reveal paper-thin forests.
- **Wind:** cosmetic motion belongs in render geometry, preferably
  vertex/palette parameters, and should be rooted, species-specific and
  cheaper at distance. It must not alter tree wood occupancy or harvest
  state. Avoid per-frame CPU regeneration of plants and material maps.
- **Measurement:** profile generated geometry count, upload count,
  draw calls, alpha-tested pixels/overdraw where measurable, GPU texture
  residency, generation/bake time, frame time and visible LOD transitions
  on representative hardware. No invented FPS or memory saving claims.

### Existing Voxel Game Engine: substantial reference, not a dependency

**Primary technical reference:**
[sdhaight1-hash/Voxel-Game-Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine).
It is a working **Rust desktop asset-authoring editor** with separate
Nodes, Presets, Foliage and Assets workspaces, **not** the playable
survival world's renderer, ecology or persistence layer.

**Verified existing authoring capabilities in that repository:**

- **43 foliage presets** (42 tree/bamboo types plus a separate Shrub
  preset) with seeded, species-bounded growth rules: branch pitch,
  taper/base flare, plant architecture, phyllotaxy and leaf placement.
  See [tree growth rules](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/tree-growth-rules-v4.md).
- **Classic** and limited **CanopyGuided** branch generation. Neither
  implies a full L-system implementation or general obstacle-responsive
  space-colonization ecosystem.
- **Hexel-owned, editable tree wood** with organic tapered mesh surfaces
  clipped to their owning wood hexels. Leaf cutout cards, bark UVs and
  branch structures can coexist with destructible logical cells.
  See [organic hexel tree meshing](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/organic-hexel-tree-meshing.md).
- **Versioned `FloraPrefab` authoring definitions** that combine a
  `FoliageProject`, stable asset ID and approximate climate preference;
  `FloraPlacement` is a serialized reference, **not live world
  generation or saved life state**.
  See [flora prefab contract](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/flora-prefab.md).
- **Reusable bark/leaf procedural materials**, cutout leaf cards,
  visual wind, seasonal palettes, a bounded bake cache and GPU preview
  experiments in mipmaps, edge coverage, material-aware foliage shading
  and thin-leaf backlighting. These are **editor/preview features**.
  See [foliage bake cache](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/shared-foliage-bake-cache.md).

**Integration policy:** study or reuse the **ideas and data contracts**
surgically; do **not** copy the full editor, replace our game renderer,
merge incompatible crate stacks or silently transfer all presets into
the new flora registry. The old 43 editor presets and current Hexel
Survival **208 research rows** are different inventories with overlapping
but non-equivalent taxonomies and approval states. Map stable identities
and licenses explicitly. Verify local Z-up -> world Y-up and save-version
compatibility before any import.

The preferred eventual boundary is:

```text
Voxel editor (optional authoring reference / future publishing tool)
    -> validated, versioned flora asset + material + structure/LOD
    -> Hexel Survival runtime chooses placement, simulates life/state,
       stores edits, and renders using its own optimized wgpu pipeline
```

**Not implemented in Hexel Survival:** a published tree-asset loader,
vegetation world spawning, instancing, LOD, persistent tree harvesting,
living seasons, full space colonization or general L-systems.
There is **no commitment to port** the older foliage code; first
verify the smallest useful output/asset interface.

### First vegetation art and architecture proof

Before producing a large flora library, put a **small comparison board**
in the existing first-person prototype: several grass densities/cards,
a mushroom with a simple volumetric cap/stalk, a wildflower or crop
hybrid, and two tree shapes (e.g. an oak with a bounded grammar and a
willow with weeping leaf clusters). Start with available templates and
authoring references; **do not build an L-system and space-colonization
engine just to populate this board**.

At normal eye height and from a hill/overhead angle, compare **cards
vs. compact meshes vs. hybrids** for silhouette, texture/alpha detail,
species recognition, wind, animation/growth-stage readability,
performance and LOD pop. Check a useful plant's discoverability at
the same distances and lighting used for ore reviews. Record actual
GPU/CPU results. Retain current P0 soil/rock/ore visuals and BC7
compression unless a measured reason requires changes.

**Do not treat this guide's example plants or older editor prefabs as
approved production assets.** This proof must happen in-engine before
deciding mass-production standards for grasses, flowers, crops and trees.

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
   fragile unreadable silhouettes. Compare fixed cutout cards, small
   meshes and hybrids from every camera angle; inspect thin leaves,
   visible alpha edges, recognizability and visual coherence with
   adjacent soil/rock materials. A generated tree must still look
   like its named species after seed variation and LOD simplification.
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

**Recommended proof boards:** (1) low-fertility soil and grass cover states,
granite, shale and clear copper overlays in both hosts; (2) mixed card,
mesh and hybrid grass, mushroom, flower/crop and two different tree
architectures. Test both boards at first-person and overhead viewpoints.
Test **neutral** versus full-style rendering once that toggle exists.
Retain P0 screenshots as the baseline, not as mandatory final aesthetic
quality. Species authoring output is not approved until verified in
the playable runtime.

## 11. Instructions for future AI assistants and collaborators

When starting **any** new visual, vegetation, texture, shader, UI or lighting
task in this repository:

1. **Read this file first**, then the relevant existing engine/material/
   flora docs. For tree/vegetation work, also inspect the **current**
   Voxel-Game-Engine authoring source and documentation before proposing
   a new tree generator, material baker or clipping algorithm. Check
   both repositories' actual status, not just earlier chat descriptions.
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
- For flora: card/3D/hybrid choice, structural algorithm, authored species
  bounds, growth stage, cutout/LOD/wind plan and persistence implications
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
- Prior Hexel Survival discussions on **Vintage Story/Hytale vegetation
  card-versus-mesh choices; L-systems, space colonization, noise and
  template grammars; deterministic world growth and performance**.
  These are architecture discussion outcomes, not implemented features.
- Existing technical reference:
  [Voxel Game Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine)
  and its [Foliage/Assets authoring boundary](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/asset-creation-subsystems.md),
  [species growth profiles](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/tree-growth-rules-v4.md),
  [organic hexel wood](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/organic-hexel-tree-meshing.md),
  [flora prefabs](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/flora-prefab.md)
  and [foliage cache](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/shared-foliage-bake-cache.md).
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
