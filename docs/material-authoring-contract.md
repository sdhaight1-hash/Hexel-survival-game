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
nearest-neighbor clarity at the 32x32 baseline.

## Reusable art rather than combinatorial duplication

Make the substrate face set once, then use parametric, tileable alpha masks
for bare/very sparse/sparse/full ground cover, biome or seasonal tint, and
individually placed vegetation sprites/meshes for each species and height
stage. Do not create one full 3-face soil texture set for every combination of
fertility, climate, season, grass species and coverage.

Similarly, do not create a distinct opaque ore image for every rock × ore ×
grade combination. Reuse host-rock textures and transparent ore overlays,
and allow procedural variation/grade-specific masks later.

## Asset locations and gates

- Existing categorized source art and 32px previews are in `assets/textures`.
- `assets/textures/manifest.json` inventories **55 legacy textures**. Its opaque
  ore crops are *references*, **not** ore-only alpha overlays.
- `assets/materials/catalog.v1.json` registers only three **legacy-preview**
  face sets and one opaque forest-floor swatch. Their existence is **not
  production approval**. New approved materials must carry status and explicit
  top/side/bottom paths; use stable type IDs, not a temporary GPU array index.
- `scripts/validate_material_catalog.py` validates manifest and catalog paths
  and checks each inventoried 32x32 asset is an RGBA8 PNG. GitHub Actions runs
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
