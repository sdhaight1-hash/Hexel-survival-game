# Hexel Survival — Canonical Art Direction & Style Guide

**Status:** project art-direction baseline • **Last reviewed:** 2026-11-03
**Revision note:** visual requirements for coherent cross-hexel patterns,
material-specific sheen and vegetation silhouette clarified.  
**Applies to:** terrain, geology, ores, soils, flora, water, structures, characters,
lighting, atmosphere, UI-related visual language, concept art and asset reviews.  
**Read this before designing or approving visual assets.**

> **North star:** A **readable, handcrafted, stylized hexel survival world** with
> believable geology and living ecology, given the **weathered beauty, melancholy
> and dramatic warm/cool atmosphere of painterly dark fantasy**.
> Gameplay readability comes first. The world must feel rich, dangerous,
> beautiful and lived-in — **not photorealistic, not neon-cartoon, not a
> textureless world everywhere, and not a literal copy of any reference game
> or artist**.

This document defines **appearance and visual acceptance**, not the
runtime architecture, engineering roadmap, implementation history or
authoring toolchain. Those subjects are organized in the
[documentation map](../docs/README.md). An illustrated reference or
candidate effect is not a shipping feature.

## Visual companion infographics

Two original concept posters remain relevant to **visual direction**
and are retained in this guide. They are illustrations, **not** actual
engine screenshots, technical UV layouts or proof of a shipping feature.
The style rules take priority over schematic block shapes, arbitrary
texel counts, glowing mineral motifs or artistically exaggerated forms.

- [Visual identity, palette and artistic pillars](style-infographics/visual-identity.webp)
- [Terrain and geology: texture and ore readability](style-infographics/terrain-geology.webp)

The third original poster—plant-generation techniques and LOD—is still
relevant but now belongs with [vegetation system design](../docs/vegetation-visual-architecture.md), not the visual art standard.
New **editable technical diagrams** live with their owning design
documents under [assets/diagrams](diagrams/README.md).

The files are stored under [`assets/style-infographics/`](style-infographics/).
They are not production-game material atlases and should not be sampled,
reused as textures, or interpreted as accurate previews of runtime graphics.

## 1. Visual influences and how they fit together

![Hexel Survival art direction infographic: readability, geological identity, painterly mood, palette, hexel face principles, and approval checks](style-infographics/visual-identity.webp)



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

![Terrain and geology infographic: layered soil and rock roles, hexel face mapping, ore overlays, stylized texture production, and performance](style-infographics/terrain-geology.webp)



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

**Curated material variation, not random recoloring (candidate
direction):** families of stone, ore, soil, bark, foliage and organic
cover may share selected grayscale tonal/detail patterns, with
material- or species-specific color ramps and surface responses.
**Color alone must not define geological or biological identity.**
Granite must retain crystalline structure; shale must retain bedding;
copper and galena need distinct mineral motifs; tree species need
recognizable bark grooves and leaf silhouettes. Seasonal leaf color
should not turn a birch into an unrelated tree. Repeated bark grooves
must follow branch/trunk direction rather than wrap randomly.

Reuse only *compatible* patterns. Some surfaces still need authored
full-color art, unique normal/roughness information or genuinely
different geometry. Nearby variants should form coherent clusters, not
a random checkerboard. Ore grade, host recognition, species silhouette
and neutral-light gameplay readability remain critical.
Detailed data, selection and palette contracts belong in
[material authoring](../docs/material-authoring-contract.md),
[flora assets](../docs/flora-to-assets.md) and
[overlay architecture](../docs/overlay-visual-architecture.md), not
in this style guide.

**Fantasy materials are first-class, not mistakes to recolor away.**
Blue-leaf weeping trees, silver foliage, unusual autumn pigments,
iridescent minerals and fictional metals can belong in Hexal if
their silhouettes, tonal groupings, visual hierarchy and physical
response remain coherent with the established painterly world.
A blue willow should still read as a **willow through its hanging
branch and leaf clusters**, not merely through a bright color choice.
Rare fantasy materials can carry stronger accents, but color changes
alone do not automatically justify emissive bloom. Frost and snow
must read as accumulated surface cover; a white palette is not
equivalent to snow mass or icy geometry.

**Initial approval test:** copper on **granite and shale**, three candidate
visual families (**veins / clusters / hybrid**) and poor–medium–rich
prominence. Favor a **hybrid** only after in-game side-by-side comparison;
the direction is proposed, not yet approved. Verify readability at
approximately 1m, 3m and 8m in neutral light and a dark environment.

### Surface layering and connected detail

Treat **host rock**, **embedded minerals**, **living/seasonal cover**
and **temporary surface conditions** as distinct visual roles. The rock
must stay recognizable under its ore; significant moss, snow or frost can
conceal a mineral's exposed highlights without changing whether the
resource exists. Copper may use localized metal response; quartz,
limestone and ordinary wet rock must not inherit copper's glints.

**Connected features should look connected.** A vein, rock fracture,
root or fungal filament that extends across adjacent 1m hexels should
retain its course across visible tops, sides and corners. Do not
restart an identical high-contrast squiggle on each face. Fine tiling
and localized alpha masks remain perfectly valid for granular texture
and small, naturally discontinuous clusters.

No single procedural method is required. Space colonization is one
candidate for certain *branching* shapes, not for general textures,
sedimentary layers, wetness or all ores. Geological forms must not
uniformly resemble tree roots. Thin patterns may use masks/decals;
substantial formations can need 3D shape.

**Art gate:** compare the same granite and shale base with tiled versus
continuous copper patterns at ~1m, ~3m and ~8m, in motion and neutral
light. If the simpler representation is clearer, use it. Never use
emission as a substitute for legible ore.

For system interfaces, condition ownership, region/cell boundaries and
the experimental SCA proposal, see
[surface overlay architecture](../docs/overlay-visual-architecture.md),
the [material/overlay roadmap](../docs/material-overlay-roadmap.md) and
the [P2.2 gallery record](../docs/p2-2-reactive-surfaces.md).

## 6. Soil, grass and vegetation

For the original illustrated vegetation-structure poster, see
[vegetation visual architecture](../docs/vegetation-visual-architecture.md).
Species shape and texture standards below remain binding.



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
- The research flora inventory is **not** an approved asset roster.
  Do not invent production assets, life states or spawning rules
  from historical species references.

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

**Seasonal color transitions must feel grown, not switched.**
A tree may blend between approved spring/summer/autumn pigment
palettes across the game year, including imaginative blue or silver
foliage phenotypes. Preserve coherent canopy shading, recognizable
species form and regional mood throughout intermediate colors;
avoid abrupt global hue swaps and visually muddy transitions.
Evergreen and fantasy species need not share the same calendar.
**Leaf loss, snow/frost and bare branches are separate visual states**
and must not be faked by making all foliage white or invisible
through a color ramp. Shader keyframes, world-time evaluation and
asset schema belong in the [vegetation architecture](../docs/vegetation-visual-architecture.md), not in this art guide.


### Structural appearance and movement

Species **silhouette**, branch rhythm, crown shape, leaf arrangement,
bark/end-grain roles and changing life stage are artistic obligations
regardless of how a structure was created. Use predictable authored
forms as the default; reserve space-colonization, L-systems or other
specialized algorithms for shapes that actually benefit.

A small plant can have fine geometry much smaller than a hexel; a
large tree can use logical wood ownership while retaining smooth,
visually coherent branches. Harvesting and distance LOD should never
leave orphaned-looking trunks, impossible floating foliage or a
visibly different species. In wind, the base must remain anchored
while flexible parts move subtly.

Use material-aware cutout cards, compact meshes and hybrids as needed;
avoid paper-thin forests, harsh alpha-edge noise and foliage that
disappears at a few metres. The appearance is judged in the playable
scene, not merely the authoring tool.

Details about the earlier Voxel Game Engine's **43 foliage presets**,
branch grammars, organic meshing, LOD, performance, wood ownership,
coordinate transforms and first asset fixture live in
[vegetation visual architecture](../docs/vegetation-visual-architecture.md).
Flora registry state and species binding live in
[flora to assets](../docs/flora-to-assets.md).

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

## 8. Lighting and atmosphere: visual direction

- **Readable light:** warm natural sunlight, cool colored shadows and
  firelight emphasize material form while preserving essential ore
  and interactable recognition. Evaluate assets under neutral light
  before art-directed grading.
- **Depth and mood:** atmospheric haze, soft contact shadow and
  restrained contrast create age and scale without hiding nearby
  hazards or drowning the world in darkness.
- **Selective highlights:** reflective copper, appropriate crystals,
  water and rare luminous elements behave differently. Don't apply
  universal bloom, shine, glow or black outlines to all terrain.
- **Stylization:** limited tonal grouping or subtle outlines may help
  focal objects, but the world remains full 3D geometry, not a
  flat posterized illustration.
- **Wind and seasonal visuals:** plants should remain species-specific
  and recognizable. Weather and climate can change the appearance,
  but not arbitrary substrate or resource identity.
- **Quality tiers:** reduced-detail settings must preserve the
  same gameplay cues and silhouettes; performance is part of art
  acceptance, not license to hide important ore or plants.

**These are style targets, not claims that the current demo implements
dynamic weather, cinematic lighting, flora wind or general post-processing.**
See [rendering and asset-pipeline design](../docs/rendering-and-asset-pipeline.md) for planned features,
current limitations, runtime compression, material baking and metrics.

## 9. Asset production: artistic requirements

Author original high-quality surfaces and plant silhouettes before
scaling to whole libraries. Use stable source identity and reviewed
families, consistent physical detail and carefully tested top/side/bottom
transitions. Avoid repeating a successful gallery swatch dozens of
times merely to satisfy a catalog count.

A material inventory is **not** proof of approved final artwork;
a flora research registry is **not** an approved asset roster.
Experimental gallery effects are **not** artistic approval.
For actual implementation counts and current approval states, use the
[material/overlay roadmap](../docs/material-overlay-roadmap.md).

Catalog and baker contracts belong in
[material authoring](../docs/material-authoring-contract.md) and
[P1 material pipeline](../docs/p1-material-pipeline.md);
runtime packing, alpha overdraw and quality budgets belong in
[rendering and asset pipeline](../docs/rendering-and-asset-pipeline.md).
All staged work and completion criteria belong in the
[material/overlay roadmap](../docs/material-overlay-roadmap.md).

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
   low-light. Perform an honest comparison with available lighting,
   and record which intended contexts cannot yet be tested.
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
7. **For continuous spatial overlays (future):** review connected vein,
   fracture or root masks across two adjacent hexels, cap/side/corners
   and section boundaries. Verify a stable pattern as the camera moves,
   identical output after chunk rebuilds, no invalid host exposure,
   and readable fallback with glints disabled. Do **not** treat an
   unimplemented world-space mask as current functionality.
8. **In motion:** look for texture shimmer, visible tiling, blur, obvious
   alpha noise and loss of visual cues from normal camera movement.
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

## 11. Collaboration and approvals

All artists, developers and agents should use this guide as the default
visual design reference and preserve its intent. Technical behavior,
project status, tooling, provenance validation and asset-review forms
live in the [visual contribution workflow](../docs/visual-contribution-guide.md) and
[documentation map](../docs/README.md).

**Only the project owner approves a final visual direction or individual
production asset.** A benchmark, shader, generated image or passing
unit test cannot certify style cohesion. Preserve the established
direction until a deliberate review changes it.

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
- Engineering references, algorithms and runtime contracts are
  indexed in the [documentation map](../docs/README.md) and
  [vegetation visual architecture](../docs/vegetation-visual-architecture.md).
  They inform implementation but do not supersede these visual standards.

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
