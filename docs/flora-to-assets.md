# Flora registry → materials and rendered assets

**Status:** foundation contract and research inventory only. No living plants,
new plant textures, flora world generation, or vegetation rendering is
implemented by this change.

## Responsibility boundary

A terrain hexel describes its **substrate** (soil type and nutrients, or
geological rock). The existing [material contract](material-authoring-contract.md)
describes top, six sides, bottom, ground-cover overlays and independent plants.

**Flora is a distinct layer:**

1. **Geography, climate, geology, soils and water** provide long-term habitat
   fields and local conditions.
2. A **versioned flora registry** decides which species *might* exist; it
   distinguishes global initial placement from established-plant survival,
   local establishment, seasonal growth, and harvest. Unknown data is not
   interpreted as globally suitable habitat.
3. A **plant instance** records the selected species, growth stage, age,
   health, orientation and gameplay-relevant state. It is not a solid soil
   hexel; harvesting it must not rewrite the supporting soil's fertility.
4. A **separate appearance binding** maps a stable species ID and growth stage
   to authored meshes/sprites and texture materials. The renderer never
   selects species by reading pixels or climate directly. Temporary GPU
   texture-array indices are not persistent identifiers.
5. **Chunk rendering** uses those assets and the independent surface-cover
   state. Meshes are disposable; authoritative plant/soil state is not.

This makes the research useful to texture production **without having to
invent or generate 208 separate asset sets immediately**.

## Asset contract for future species artwork

Only when an asset exists and passes validation, bind an approved flora ID to
a **stable asset ID**. That asset is free to reference reusable:

| Family | Authored surfaces or sprites/models |
| --- | --- |
| Tree | bark around trunk, end-grain on exposed log top/bottom, branch bark, leaf clusters, sapling/young/adult geometry, seasonal leaf states |
| Shrub | stems, canopy leaves, flowers/berries, juvenile/adult geometry |
| Grass | ground-cover alpha masks, grass-to-soil side transitions, separate height-stage tufts, dry/dead variants |
| Crop | seedling, vegetative, mature, fruiting and harvested/withering appearances |
| Flower | foliage, stem, bud, petals, flower/seedhead stage |
| Fungus | cap, stem, gills/pores, shelf or ground-host attachment, growth/decay stage |
| Aquatic | water-aware anchors and floating/emergent/submerged geometry |

For grass, do **not** bake a particular species into the soil block. The
soil top and side are composited with coverage masks, independently from
individual grass tufts. For trees, exposed cut-log end grain is a different
face role than circumferential bark. Materials need to respect a **hexagonal
prism** (not cube UVs) and keep physical pixels-per-metre consistent.

For now every inventory record sets `visuals: null`. That means **no bound
art exists in this registry**, not "use the closest tree's texture". We will
add asset IDs only after approving and publishing actual assets from the
existing authoring tool. The current soil/rock/ore PNGs in
`assets/textures` are background terrain materials, not substitute leaves
or branches.

A proposed future visual-binding record (illustrative only; **not** a live
registry or an asset that exists yet):

```json
{
  "species_id": "flora.vanilla.tree.birch_silver_1",
  "visual_asset_id": "flora.tree.birch_silver_1",
  "life_stage": "adult",
  "seasonal_variant": "summer",
  "trunk_material_id": "wood.birch.bark",
  "leaf_material_id": "foliage.birch.summer"
}
```

The asset IDs above are *examples*, not registered files or save-compatible
values. A later approved asset catalog will validate references and require
fallback behavior for missing variants.

## Candidate grayscale-pattern reuse in published flora assets

A future approved flora visual asset may select curated grayscale
bark-detail, leaf-pigment or grass-fiber patterns and apply
species/season/age palettes. This could avoid keeping independent
full-color textures for every palette variant. The proposal is
documented in [material authoring](material-authoring-contract.md)
and [vegetation visual architecture](vegetation-visual-architecture.md).

**Do not collapse separate roles:** a leaf's alpha-cutout silhouette
is distinct from grayscale inner vein/pigment detail; longitudinal
bark ridges require trunk/branch-aligned coordinates; actual shape,
wood ownership, age, growth stages, harvested/dead state and species
identity still belong to the saved plant/asset data, not a tint.

An asset-binding catalog should reference **stable approved
material-family and pattern IDs**, not provisional GPU array slots.
Color variants can be derived from one authored pattern, but the
real species must remain identifiable without relying exclusively
on hue. This proposal changes no records in the existing
research-only flora registry and introduces no unapproved visuals.

### Optional fantasy phenotype without cloning species records

For an approved willow visual, one variant might specify
`leaf_palette_id: foliage.willow.blue` while retaining its
`species_id`, geometry, trunk material, persistence and biological
rules. **This is illustrative only**, not an active flora-catalog
field. If blue foliage signifies a distinct fantasy species with
different habitat, growth, harvested resources or interactions,
approve and register that as a **new species identity** instead.

Seasonal autumn recoloring is another candidate palette recipe.
Winter-bare branches and snow/frost coverage additionally need
leaf-retention, structure and environmental state, not just a white
palette. A fantastical luminous canopy requires an intentional
emissive-material/lighting designation; unusual hue alone must not
make vegetation glow.

The user-supplied tree images are concept references and remain in
the conversation; they have **not** been installed as repo source
art, given a provenance license for distribution or approved as
production assets.

### Seasonal palette arrays are appearance bindings, not species IDs

A future approved flora asset may reference a stable
`seasonal_palette_set_id` (illustrative field **only**) containing
several pigment ramps keyed by normalized world-year phase. The
renderer can sample the relevant neighboring ramps and blend the
leaf's grayscale inner-detail colors continuously. A blue fantasy
willow may have a different seasonal palette set from an ordinary
willow even when they share geometry and life rules.

**Persist real state, derive color:** species identity, approved
phenotype, maturity, leaf retention, harvest/removal and actual
climate/health effects belong to gameplay or approved flora asset
state. Current world time plus a deterministic region/species
mapping determines color phase when implemented. Never store
per-frame interpolated RGB in the flora registry or bake a new
PNG each day. Coordinate any leaf-loss geometry and snow cover
through their own contracts; a palette does not define either.

The current reference registry remains **research-only**. No
`seasonal_palette_set_id` field, species palette lookup or
world-time-driven foliage renderer is implemented in this PR.


## Registry data / source cautions

`assets/flora/reference_registry.v1.json` holds **208 historical research
rows**, not necessarily 208 unique accepted species. Some rows are variants
of one biological species; mod names can overlap with vanilla. Sources span
Vintage Story game/wiki versions and independently versioned mods.

- Prefixes `flora.vanilla`, `flora.wildcraft` and
  `flora.florasphere_historical` keep sources distinct. This does **not**
  approve importing proprietary meshes/textures or copying mod asset files.
- Each row has its original source URL, evidence status and date.
- Null climate inputs are unknown. Normalized rain/forestation values are
  explicitly 0..1; historical 0..255 values are converted by dividing by 255.
- Reference-world elevation fractions do **not** automatically map to the
  gameplay Y coordinates or toroidal synthetic planet's altitude. Re-map and
  revalidate them with our own geography.
- A source's world-generation temperature range is not a plant's cold-death
  temperature or fruiting requirement. Keep those rule sets separate.
- Unverified claims such as "willow only one block from freshwater" are not
  represented as facts. Actual riparian suitability must be measured or
  designed explicitly.
- Current rows are `research_only`. **None may drive live world generation.**

`scripts/validate_flora_registry.py` checks IDs, source/provenance,
normalized interval sanity, required fields and that every record is
research-only. `src/flora.rs` adds headless eligibility types and tests,
where `Unknown` or unapproved rules fail closed. This intentionally
does not mean that the historical reference ranges have been fact-checked.

## First production vertical slice

1. Choose **one grass** and **one tree** from the reference inventory.
2. Author their original bark/leaf/tuft visuals and register them with
   explicit face/material roles, independent of terrain soil.
3. Implement and test `species_id → approved asset ID → reusable materials`.
4. Test bare/sparse/full coverage and changing plant growth/season without
   changing the underlying soil type or reproducing deleted vegetation.
5. Only then implement deterministic worldgen eligibility, persistence, and
   ecological transitions with verified *Hexel Survival* climate rules.

Reference pages: https://wiki.vintagestory.at/Tree ,
https://wiki.vintagestory.at/Flower ,
https://wiki.vintagestory.at/Mushroom ,
https://mods.vintagestory.at/wildcrafttree ,
https://mods.vintagestory.at/florasphere .
