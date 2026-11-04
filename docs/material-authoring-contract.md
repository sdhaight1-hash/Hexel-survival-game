# Material and vegetation contract — v1

**Applies to all new terrain and resource texture work.** This is a design and
typed-data foundation with validation, **not** a new texture renderer, new
finished texture artwork, live grass growth, or geology generation. The existing
55 atlas-derived assets are legacy/source previews until reviewed and rebuilt.

## The authoritative state must not be a texture

The following are **independent** facts, selected by world/ecology logic, never
inferred from the color of a PNG:

- **Substrate:** the durable solid hexel (soil fertility and composition, or a
  rock species). A grassy soil remains the *same soil* when grass is removed.
- **Ground cover:** a cover family (grass, moss, leaf litter) and its density
  (bare, very sparse, sparse, full). Ground coverage can change with rainfall,
  season, light, disturbance, fire and grazing, without changing soil fertility.
- **Individual vegetation:** a separate species-specific plant/tuft object with
  a height stage (very short, short, medium-short, medium, tall, very tall).
  Removing a tuft must not remove its supporting soil cell.
- **Ore occurrence:** the exact **host rock**, ore identity and grade; deposit
  inventory and depletion must live in persistent world state. Changing the
  host alters the appearance, not the ore species; actual host eligibility is
  decided by geology, never by the renderer.

The new type contracts live in `src/material_contract.rs`. They are not yet
wired into the demo's original `World::demo`, chunk saves or GPU material
array; that is follow-up work. Never persist transient GPU texture indexes.

## Face sets: a hexagonal prism is not a cube

Every solid material defines an explicit **top**, **side** and **bottom**
recipe. The current contract applies the side recipe to each of the **six**
vertical prism faces, for eight total geometrical faces. Future directional
side variants must be an explicit addition; do not reuse cube UV layout.

Each face recipe can contain a **base** texture plus an optional transparent
**overlay**. Textures must be layered over physically appropriate underlying
matter, never used to replace the meaning of the solid block.

- Bare soil: top, side and bottom can use related soil materials.
- Covered soil: top soil + coverage mask; each side must show an
  **upper-edge-anchored grass-to-soil transition**, and bottom remains soil.
  A flat grass texture on the entire 1m-tall side is wrong.
- Rock: top, side and bottom may intentionally share a base material, but must
  still be chosen through explicit face roles.
- Ore: host-rock base + **ore-only alpha overlay**; never blend one opaque
  prepainted brown-rock-and-copper image over an unrelated host.

### Physical texture scale and seam rules

With a **1m-across-flats, 1m-tall regular hexel**, a single side edge is
**0.5773503m wide**. Mapping a full 32-pixel tile equally across that width
and the 1m vertical creates different physical texel densities (the stretched
look in the recent screenshot). UVs or authored side assets must target
**consistent pixels per metre**; do not simply assign U=0..1 across the edge.

A cap is an actual **hexagon**, with a consistent X/Z UV projection and no
square/cube-only art assumptions. Check the visual joins around all six side
corners, top rim, and bottom. Ground-cover transition masks align at the top
rim on all six sides regardless of their orientation. Avoid baked directional
lighting in albedo textures. Use transparent RGBA for overlays; preserve
consistent apparent detail at the chosen physical texel scale. The 32x32
swatches are **legacy-only**: [material format v2](material-format-v2.md)
uses a 128px P0 benchmark and does not require one fixed production size.

## Reusable art rather than combinatorial duplication

Make the substrate face set once, then use parametric, tileable alpha masks
for bare/very sparse/sparse/full ground cover, biome or seasonal tint, and
individually placed vegetation sprites/meshes for each species and height
stage. Do not create one full 3-face soil texture set for every combination of
fertility, climate, season, grass species and coverage.

Similarly, do not create a distinct opaque ore image for every rock × ore ×
grade combination. Reuse host-rock textures and transparent ore overlays,
and allow procedural variation/grade-specific masks later.

## Candidate extension: shared grayscale pattern families and material recipes

**Proposal, not an implemented registry format.** Keep **stable material
identity** (e.g. `ore.native_copper`), **appearance variant** (e.g.
`inclusion.nodular_02`), **mineral palette/material response** and
**world-state occurrence** as distinct IDs/data. An approved family
may reference a curated subset of reusable patterns with weights and
compatibility constraints. A different GPU array layer is an
implementation detail, never an authoritative species, rock or ore ID.

This is a **cross-domain authored-material tool**, not an ore-specific
renderer. The same underlying decomposition can support:

| Consumer | Reusable grayscale/detail information | Must remain specific |
| --- | --- | --- |
| Rock / mineral base | Crystalline speckling, grain, erosion, stratification *within compatible geology* | Rock species, correct bedding direction, chemical color, roughness/normal and true face geometry |
| Embedded ore | Exposed inclusion/vein coverage C, internal detail G | Authorized occurrence/grade/stock, mineral morphology and metal/dielectric response |
| Tree bark | Grooves, ridges, chips, scars, age-related tonal fields | Species trunk/branch form, longitudinal orientation, wood/cut-end face role and bark normal depth |
| Leaf / needle / grass | Internal veins, pigmentation gradients, blemishes and seasonal tonal masks | Species silhouette/alpha cutout, venation topology, two-sided/transmitted-light appearance, plant instance and life state |
| Soil / organic cover | Clods, particles, fine organic detail, patch coverage | Soil identity, climate/moisture authority, root/stone geometry and genuine biome context |
| Environmental overlay | Wetness, frost, moss/lichen, soot or scuff coverage | Real world-state inputs, geometry where thick, physically sensible material response |

**Opaque base vs alpha-cutout vs overlay:** an opaque rock or bark
material normally uses a full-coverage albedo/tonal field. A leaf-card
**silhouette alpha** defines the visible leaf geometry; its internal
grayscale pigment and vein detail are separate. An ore inclusion's
**coverage** is gated by actual geology. These mask types must not be
interchanged just because they are all grayscale.

**Pattern orientation is meaningful:** bark streaks track the tree's
local trunk/branch frame; leaf veins track leaf UV/geometry; shale
bedding follows geologic strata; a regional ore vein follows feature-
local coordinates. A single world-space hash for every material will
make directional structures incoherent.

**Palettes can vary without new bitmaps** (seasonal leaf pigmentation,
rock oxidation, species color morphs), but cutout edges, normal/height
and translucency do not follow automatically from recoloring. No
palette step may change flora species identity or authoritative
geological resource quantity.

An overlay need not store a full-colored RGBA image per ore species.
For suitably compatible minerals, author **linear single-channel
coverage** and optionally a **separate linear grayscale tonal/detail
field**, then supply a mineral-specific small color ramp and reflectance
parameters:

- **Coverage:** `c ∈ [0,1]` means exposed mineral fraction at a face
  sample, after host/occurrence/grade/feature eligibility and occlusion.
  No authorized occurrence means `c = 0` regardless of artwork.
- **Detail:** `g ∈ [0,1]` controls mineral-internal dark/mid/light
  structure. Darkness in `g` must **not** automatically make the mineral
  transparent. This can be omitted for flat-color stylized motifs.
- **Palette:** one color or preferably a few mineral-specific ramp
  stops, interpreted in a correct color space. A single RGB multiply
  is a baseline comparison, not an adequate universal material model.
- **Response:** roughness, metalness, localized highlights, oxidation
  and normal/height appearance are material-specific and separately
  controlled. Normal maps cannot be derived by merely tinting gray.
- **Morphology:** a given ore family may allow some hairline,
  disseminated, nodular, ribbon or stockwork patterns, but never every
  pattern indiscriminately. Real geological eligibility and host
  constraints remain owned by gameplay/geology.
- **Determinism:** select family member from world seed + stable region/
  feature ID + coordinates and fixed weights/context; the selection must
  not change with camera frame, chunk request/rebuild order or cache
  eviction. Persist edits and resource depletion separately.
- **Continuity:** a structural vein uses a *world- or feature-space*
  geometry mask; a tileable grayscale pattern supplies finer detail
  inside it. Randomly picking an independent vein image per hexel is
  not a substitute for continuous geology.

The combination is *derived appearance*:
`surface = blend(host_albedo, palette(g), c)` as a conceptual color
equation. In production, composition and physically appropriate
lighting occur in linear space and cover/condition layers must also
be applied. A colorized ore pixel cannot create or replenish stock.

Do **not** require five unique variants per one of all 55 inventoried
materials: five is a useful *comparison-board target* for a few
representative families. Share pattern sets when visual/geological
similarity is justified, and permit an authored full-color exception
where color variation or patterned chemistry is essential. Do not
pre-bake every rock × ore × pattern × grade × wetness permutation.

The next proof is **one grayscale ore-pattern family, native copper on
both granite and shale**, compared with the existing transparent
full-color copper overlay. Keep its stable ore ID and the v2 P0
fallback. Require visually distinct grade states, copper recognition
when highlights are disabled, host preservation, bounded added GPU
samples and measured texture-memory effects before choosing the
production representation. See [P1 pipeline](p1-material-pipeline.md)
and [overlay visual architecture](overlay-visual-architecture.md).

## Fantasy material variants and phenotype identity (proposal)

The shared pattern/palette system must support **fictional materials**
and **fictional flora** with the same explicit contracts as natural
ones. A reusable grayscale pattern does not need a real-world
material counterpart; it needs stable identity, a vetted compatible
morphology, physically or artistically deliberate surface response,
and correct world-state ownership.

- A **palette variant** changes approved appearance within the same
  geological material or biological species (for example, a seasonal
  blue/purple pigment ramp on a particular willow asset). Keep
  material/species IDs unchanged if gameplay properties and underlying
  identity are unchanged.
- A **distinct fantasy material/species** gets a new authoritative ID
  when its properties, habitat, gathering products, damage response,
  chemical behavior, light emission or growth/life rules truly differ.
  Do not silently create a gameplay species merely by tinting a leaf.
- A **structural variant** changes a silhouette, crystal form, wood
  grain, leaf mesh/cutout or ore deposit geometry. This requires
  appropriate shape assets or structural generation; grayscale
  recoloring alone cannot supply it.
- An **environmental state** changes cover, surface moisture, snow,
  frost, health or seasonal pigment. These remain independent from
  an asset's base recipe and from the finite-water/climate authority.
- A **light-emitting fantasy material** needs an explicit emissive
  property and corresponding gameplay/lighting contract if it should
  illuminate the world. Unusual color never implies emission.

The user's five illustrated tree examples—autumn broadleaf,
blue-green broadleaf, drooping blue willow, winter-bare/frosted
branches and a snowy grove—motivate this separation. They do **not**
prove that the screenshots share one geometry or texture source.
Nothing in this proposal turns those reference images into
production art, registers new fantasy species or adds texture assets.

**Versioning rule:** persist stable material/species identity and
authoritative world changes, while palette/pattern IDs are
versioned derived-art recipes. Preserve visual selection across
rebuilds and world reloads. A missing optional pattern may fall
back visually, but cannot silently change resource identity,
species, harvested state or emission.

## Asset locations and gates

- Existing categorized source art and 32px previews are in `assets/textures`.
- `assets/textures/manifest.json` inventories **55 legacy textures**. Its opaque
  ore crops are *references*, **not** ore-only alpha overlays.
- `assets/materials/catalog.v1.json` registers only three **legacy-preview**
  face sets and one opaque forest-floor swatch. Their existence is **not
  production approval**. New approved materials must carry status and explicit
  top/side/bottom paths; use stable type IDs, not a temporary GPU array index.
- `scripts/validate_material_catalog.py` validates legacy manifest and catalog
  paths and checks each inventoried 32x32 asset is an RGBA8 PNG. GitHub Actions runs
  it on PRs. It cannot detect art seams, inappropriate alpha content, color
  grading, or pixel density; an in-game six-face review is also required.
- Original atlases stay for visual reference; do **not** edit existing 55
  swatches in-place merely to satisfy the new model. Make new face assets and
  overlays deliberately, then register them.

## Next acceptance slices

1. Re-author **one** soil's top/side/bottom and grass-to-dirt transition.
   Display the four cover densities side by side while soil ID stays stable.
2. Render six height stages for **one** grass species as independent above-
   ground vegetation. Remove one tuft while preserving soil and coverage.
3. Render native copper as **one transparent overlay** over both granite and
   shale test fixtures. Verify composite appearance, valid host/ore identifiers
   and subsequent persistent resource depletion.
4. Validate seams and matching texel scale around all six sides and the cap
   at close first-person camera distances before adding more of the library.
5. Make the GPU renderer a consumer of the material contract (with a versioned
   asset registry) rather than making the renderer decide soil or ore identity.

## Reference principles (not copied assets)

Vintage Story demonstrates distinct block-face textures, base/overlay
composition, a ground-cover transition render stage and host-dependent ore
eligibility. We use these **principles** only; no Vintage Story textures or
engine code are copied:

- https://wiki.vintagestory.at/Modding:Textures/en
- https://wiki.vintagestory.at/Modding:CompositeTexture
- https://wiki.vintagestory.at/Modding:Render_Stages
- https://wiki.vintagestory.at/Ore_Deposits

All of this must eventually operate over our own streamed Q/R/Y hexel runtime,
finite materials and synthetic toroidal-world geology.

## Flora asset integration

New tree, shrub, grass, crop and fungus artwork must be tied to independently
versioned flora species IDs and growth states, never inferred from a soil or
ore texture filename. See [Flora → rendered assets](flora-to-assets.md).
This is a binding contract only; the 208 research inventory entries do not
have published textures or authoring assets yet.
