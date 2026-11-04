# Industry research — material families, palette variation, seasonality and spatial features

**Research date:** 2026-11-03  
**Status:** evidence review and proposed tests only. This is **not** an adopted renderer API, new v2 schema, completed shader, source-art approval or assertion that another game uses our proposed implementation.

**Research question:** How do Hytale, Vintage Story, Eco and graphics-engine / academic references inform Hexal's proposed reusable grayscale pattern families, world-space materials, fantasy palettes, seasonal interpolation, flora variation, and environmental overlays?

**Read with:** [canonical visual style](../assets/STYLE_GUIDE.md), [material authoring](material-authoring-contract.md), [overlay architecture](overlay-visual-architecture.md), [flora architecture](vegetation-visual-architecture.md), [material roadmap](material-overlay-roadmap.md), and [merged spatial-growth research](space-colonization-study.md).

## Executive assessment

Three games supply *different* kinds of evidence:

| Product | Verified design mechanisms | What it does **not** establish |
| --- | --- | --- |
| **Hytale** | Explicit block/worldgen tinting; separate biome tint providers, weighted material selection, mixed tint transitions; unique leaf placement; controlled illustrated texture style; graph-shaped cached worldgen in **pre-release** docs | Universal grayscale material families, shipped annual season-palette blending, or space-colonization implementation |
| **Vintage Story** | Separate `ClimateColorMap` and `SeasonColorMap` identifiers, position/environment-sensitive color mapping; changing seasonal foliage; independent temperature, rain and physical snow; biome/climate variation | That every base art asset is grayscale or that every environmental state is a GPU tint |
| **Eco (Global Survival)** | Climate-linked biome/ecology, plant species/state and pollution; client foliage prefab variants and growth animation exposed in modding guides; actual weather/lighting updates | Verified equivalent to Vintage Story's annual foliage color-map shader; its ecology is not proof of grayscale texturing or a global finite-water engine |

**Recommendation:** use a shared **authored pattern + material/species recipe + contextual palette** model, while world-state, shape, and simulation remain owned by the proper systems. Keep original full-color artwork as an allowed option; never require all 55 geology identities or all flora species to become recolors.

## 1. Hytale — variable materials, biome tints and restrained art

### Confirmed

1. [Official block technology note (2019)](https://hytale.com/news/2019/2/exploring-hytale-s-block-tech) describes adjustable RGB tint, foliage crossed-plane/block shapes, rotating foliage placement and shader-level plant movement. **Historical source with an explicit outdated notice**; use the newer docs below for current claims.
2. [Hytale's current biome-editing docs](https://docs.hytale.com/creating-content/world-generation/how-to-edit-and-create-biomes/) explicitly separate terrain geometry, Material Providers, Props, Environment Providers and **Tint Providers** (usually vegetation/soil).
3. [Material Provider reference](https://docs.hytale.com/hytale-generator/material-provider/) includes **Weighted** material selection with a seed and **FieldFunction** assignment through spatial ranges. [Worldgen modifier docs](https://docs.hytale.com/assets/worldgen/modifier/) describe weighted tint entries, noise selectors and masks.
4. [Official Update 6, August 2026](https://hytale.com/news/2026/8/update-6-patch-notes): smoother terrain and foliage tinting across biome borders; `Mix TintProvider` blends tints inside biomes and `DistanceToBiomeEdge` supports spatial context.
5. [Official Hytale art-direction guide, December 2025](https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale) explicitly prioritizes illustrated art, colored/painterly light and shadow and **does not use a conventional full PBR texture workflow**. It emphasizes readability, distinctive silhouettes, appropriate detail density and avoiding automatic procedural noise.
6. [Hytale pre-release graph generator](https://pre-release.docs.hytale.com/hytale-generator/graph-generator/graph-generator/) describes a **separate connected graph of spatial content**, applied in graph passes and cached by cells, with graph generation separate from content generation. **Pre-release documentation; do not claim the release branch supports every API shown.** It does *not* identify its graph algorithm as space colonization.

### Hexal interpretation

- Separate **material identity**, **pattern selection**, **regional tint** and **environment** in our data boundaries. Let regional signals blend; don't choose a completely independent palette per hexel.
- Reuse Hytale's *authoring principle* of expressive artwork. Full grayscale recoloring can erase the intentional warm/cool pigment and painted shadow relationships; a material family must retain art direction.
- A world-space/cached graph can provide candidate feature geometry while renderers and content systems interpret the graph independently. This strengthens the reusable spatial-growth abstraction; it is **not proof that Hytale uses SCA**.
- Do **not** copy Hytale's texture-density numbers as Hexal's standard: our 1m across-flats hexel and current 256px test gallery have different physical mapping and validation needs.

**Confidence:** High for official tint/biome/art mechanisms and Update 6; pre-release graph documentation is provisional.

## 2. Vintage Story — concrete seasonal and climate-color contracts

### Confirmed

1. [Official 1.13 Seasons Update](https://www.vintagestory.at/blog.html/news/the-seasons-update-v113-pre1-r251/) announced seasonal foliage including pine shades, red blueberries, yellow maples; snow accumulating on a wide range of blocks; latitude and seasonal day/night temperatures; flipped seasonal behavior across hemispheres.
2. [Open public VS API block definition](https://github.com/anegostudios/vsapi/blob/master/Common/Collectible/Block/Block.cs) exposes `ClimateColorMap` and `SeasonColorMap` independently, plus relevant extra color bits. [Client world API](https://github.com/anegostudios/vsapi/blob/master/Client/API/IClientWorldAccessor.cs) documents `ApplyColorMapOnRgba`, using named climate/season maps and a position or measured climate inputs.
3. [ShapeElement API](https://github.com/anegostudios/vsapi/blob/master/Common/Model/Shape/ShapeElement.cs) allows per-shape-element climate and season colormap references — not just one planet-wide foliage material.
4. The [official season wiki](https://wiki.vintagestory.at/Seasons/en) describes foliage shifting before the official season boundary, different climates/latitudes and separately meaningful snow/temperatures. Its default year uses 108 in-game days but these are configurable game-design numbers, **not** a requirement for Hexal.
5. [Vintage Story temperature wiki](https://wiki.vintagestory.at/Temperature) documents geography, rainfall, elevation, annual temperature and climate-mediated biomes; it does not imply that the shader owns ecological climate.
6. [Example community foliage mod](https://mods.vintagestory.at/naturalfoliage) changes foliage season tint maps to alter color while preserving seasonal behavior. Useful indirect evidence that season palette data is artist-adjustable, not proof of how all geometry is rendered.
7. [Official seasons engineering changes](https://www.vintagestory.at/forums/topic/2291-the-seasons-update-v113-pre1/) also mention season/climate map identifiers in mesh data and tessellation — the color mapping is part of asset/mesh/render handoff.

### Hexal interpretation

- Strong precedent for **multiple contextual color dimensions**: biome/climate and temporal season can affect the *same surface* without new base textures.
- The renderer needs a stable way to obtain contextual color for an exposed surface; no reason to save a new PNG or a per-frame color into every world cell.
- Keep true **leaf density/retention**, **snow** and **ground cover** distinct from pigment color. A winter tree and a white-tinted summer tree are not equivalent.
- Adopt the *principle* of gradual transitions, not a copy of VS's calendar, texture atlas encoding or specific shader API.

**Confidence:** High — official release notes, modder-facing public APIs and community wiki align.

## 3. Eco — ecology, independently rendered growth and public modding clues

### Confirmed

1. The [Eco Wiki biome overview](https://wiki.play.eco/en/Biomes) describes temperature/precipitation-defined habitats, ecological species dependencies, changing biome boundaries and displacement as environmental conditions change. [Plants overview](https://wiki.play.eco/en/Plants) ties species presence to temperature, water, pollution and population conditions.
2. [Eco ModKit documentation](https://docs.play.eco/) confirms separately exposed client Unity asset tools and server-side game rules. A [community Wiki modding guide](https://wiki.play.eco/en/Modding_Eco_beyond_items_and_statues) describes `WeightedPrefabBuilder` for variation, foliage `LODGroup`, `PlantGrowth` and age/scale/alpha curves; shader controls including pollution. **Community guide descriptions of modding are not a guaranteed specification of all vanilla tree internals.**
3. [Update 12, October 2025](https://store.steampowered.com/news/posts/?appids=382310&enddate=1760208171&feed=steam_community_announcements) replaced the time-of-day, lighting and weather presentation system; therefore pre-2025 statements that Eco has *no weather* are obsolete.
4. [Official Eco 2026 Update 14 and follow-up announcements](https://steamcommunity.com/app/382310/allnews/) describe four differentiated ground pollution classes, spread via water, dilution, ecological and economic damage and live plant/tree rendering bug fixes. This supports the importance of **consequential world state versus presentation**.

### Not established

I found **no authoritative current evidence** of an Eco feature equivalent to Vintage Story's specific continuously interpolated **annual foliage season colormaps**, or of a reusable grayscale-to-mineral palette format. Don't claim either. An older research chapter's pre-2025 statement that Eco lacked seasons/weather is **not safe as a current weather claim** because Update 12 changed weather.

### Hexal interpretation

- **Simulation authority drives appearance**: illness, pollution, soil disturbance, plant growth and habitat changes determine state; palette/coverage/geometry render those facts.
- Use optional *weighted prefab or structure variants* independently from palette variation. Leaves appearing with age are not merely a new color.
- Avoid deriving finite water, climate or plant survival from shader outputs.

**Confidence:** High for official biome/gameplay intent and announced updates; medium for client ModKit details from community-maintained guide; no asserted annual foliage renderer.

## 4. Wider game-engine and graphics research

### Minecraft Bedrock — industrialized biome-tint categories

[Mojang's material-instance schema](https://github.com/Mojang/bedrock-schemas/blob/main/types/bp/blocks/components/MinecraftMaterialInstances.d.ts) names foliage/grass/water tint methods, including birch, evergreen and dry foliage, with tint logic commonly tied to biome temperature/rain. It also distinguishes alpha-test and blended render modes. This supports Hexal's idea of **material role + palette provider**, with different foliage behavior based on class, not per-species full texture duplication. It is not evidence of four-season behavior.

### Unreal Engine — instance parameters, world controls and gradient lookup

- [Material Instances](https://dev.epicgames.com/documentation/unreal-engine/instanced-materials-in-unreal-engine?lang=en-US) parameterize a common parent material, enabling many looks without compiling an entirely separate shader for each.
- [Material Parameter Collections](https://dev.epicgames.com/documentation/unreal-engine/using-material-parameter-collections-in-unreal-engine?lang=en-US) push shared scalar/vector data like weather and wetness into many materials.
- [Curve Atlases](https://dev.epicgames.com/documentation/unreal-engine/curve-atlases-in-unreal-engine-materials?lang=en-US) store and sample interpolated color curves. **This is a particularly close precedent** for small GPU palette ramps and seasonal/material response keyframes.

**Hexal implication:** separate global world-year/environment parameters from instance/region phenology and material-specific palettes. Use a compact curve table or texture-based LUT *if benchmarked*, not thousands of CPU-rebaked color maps.

### Unity + SpeedTree — per-tree variation from world position

[Unity's SpeedTree importer](https://docs.unity.com/en-us/engine/6000.5/manual/creating-environments/script-terrain/terrain-trees/speed-tree/class-speed-tree-importer/speed-tree-importer-model) includes main color and **position-dependent color variation**. It separately addresses normal maps, subsurface scattering and wind. This is the most direct industrial proof of **world-space coherent color variation within one tree species**. The exact system doesn't prove seasonal arrays, but strongly supports our deterministic instance and region variation concept.

### Godot — lookup ramps and huge instancing tradeoffs

- [GradientTexture1D](https://docs.godotengine.org/en/latest/classes/class_gradienttexture1d.html) generates a sampled texture from a color gradient, a straightforward model for grayscale-value → color-ramp lookup.
- [MultiMesh batching](https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html) supports large numbers of instances and shader `INSTANCE_CUSTOM` data, but explains its culling tradeoff. Hexal should group foliage by region/chunk rather than simply drawing the entire planet in one large foliage instance buffer.

### Academic real-time texturing — break visible repetition without extra libraries

- [Heitz and Neyret, *High-Performance By-Example Noise* (HPG 2018)](https://eheitzresearch.wordpress.com/722-2/) describe infinite, nonrepeating-looking natural texture synthesis from small examples, including bark, sand, moss and granite.
- [Mikkelsen, *Practical Real-Time Hex-Tiling* (JCGT 2022)](https://jcgt.org/published/0011/03/05/) adapts random texture tiling and blending, particularly for surface-detail normal maps. **Important:** the method uses a triangular/hexagonal stochastic sampling arrangement in texture space; it is **not automatically a mapping solution for Hexal's 3D hexagonal prisms** and is more expensive than a single texture lookup.
- [Grenier et al., *Real-time Terrain Enhancement with Controlled Procedural Patterns* (2023/2024)](https://onlinelibrary.wiley.com/doi/full/10.1111/cgf.14992) shows context-aligned procedural erosion detail over terrain, a useful reminder that geological directions and slopes matter more than arbitrary random texture rotation.

**Hexal implication:** if five curated variants are still repetitive, evaluate **noise/tiling modulation** before authoring 50 more textures. Start with simple stable variants; do not adopt multi-tap stochastic blending before the renderer and first art family are approved.

### Texture memory: a real, limited benefit

[Microsoft block compression documentation](https://learn.microsoft.com/en-us/windows/uwp/graphics-concepts/texture-block-compression) specifies **BC4 = 8 bytes per 4×4 block** and **BC7 = 16 bytes per 4×4 block**. Thus an unmipped 256² BC4 mask uses 32 KiB and a BC7 tile uses 64 KiB. **Two BC4 inputs (coverage + tonal detail) equal one BC7 input in compressed storage**, before mips, palette LUTs and GPU samples. More shared masks across many species may still significantly reduce the number of assets. Current Hexal v2 supports BC7, **not BC4**.

## 5. Biological reference: why time alone is not local season

[USGS multi-species phenology research](https://www.usgs.gov/publications/time-branch-out-application-hierarchical-survival-models-plant-phenology) links spring leaf-out to day length, heat, winter chilling and frost, and autumn leaf color to shorter days/cold. Different species respond differently. [USA National Phenology Network's May 2026 summary](https://www.usanpn.org/news/article/weather-impacts-autumn-phenology-differently-eastern-and-western-us) finds regional differences in autumn color timing and the role of weather.

**Design rule:** one planetary **world-year phase** can be shared, but the palette phase visible on a given plant is a **species/phenotype/region-specific transformation** of that time and eventually of validated climate. Initial art-only sliders can use time alone; gameplay ecology must not silently pretend it knows leaf physiology. This applies equally to fantastical blue-leaf species: their palette timeline is authored fiction, but still explicit and deterministic.

## 6. Hexal architecture recommended by the combined research

```text
AUTHORING (stable, approved data)
  Material / Flora Identity
    ├─ Compatible Pattern Family: fine grayscale detail, direction + weights
    ├─ Palette Set: color ramp(s), optional cyclical temporal keyframes
    ├─ Surface Recipe: finish, roughness, metal response, optional emission
    └─ Silhouette / normal / geometry and UV or feature-local orientation

WORLD AUTHORITY
  Immutable geology & habitat context + stored edited / mined / harvested state
    ├─ biome / region / climate inputs
    ├─ annual world-time phase -> local species phenology
    ├─ actual mineral occurrence, grade and depletion
    └─ weather, snow, wetness, pollution, foliage retention and health

OPTIONAL STRUCTURAL FEATURE INPUT
  Simple fault/spline; hand-authored geometry; bounded SCA skeleton
    -> region-owned world/feature-space coverage
    -> never determines ore inventory, water mass, organism identity

RENDER-DERIVED APPEARANCE
  Stable pattern/variant selector (not frame-random)
    + material family coordinates (wood-local, leaf UV, stratum/feature-local)
    + palette lookup (grayscale tone + species/local seasonal phase)
    + optional ore coverage and separately applied environmental state
    + material-specific lighting/geometry
    -> frame output; discardable caches
```

**Specific caution:** a palette LUT has at least two logical dimensions for seasonal plants: grayscale tonal detail and locally mapped seasonal phase. An implementation could store small linear color-ramp keyframes and interpolate two ramps or precompute a compact **tone × season** lookup texture. Which is faster depends on sampling, color quality and memory measurements; do not prescribe it as a universal shader format now.

**Data-role differences:**
- Opaque rock base: color/pattern across its entire eligible face; no fake ore.
- Ore: independent geological coverage C + internal grayscale G; correct host/grade and stock.
- Leaf: silhouette alpha cutout + vein/pigment detail; age/leaf retention independent.
- Bark: pattern aligned to trunk/branches; endgrain separate.
- Snow/wetness: condition source from environment; thick accumulations may need geometry.

**Avoid** a global `palette_id` selected by RGB-only material identity, a universal 3D projection for leaves/bark, per-block dice-roll appearance, or a production rewrite of `catalog.v2.json` before a small side-by-side proof.

## 7. Revised experiment order and decision gates

| Experiment | Baseline | Experimental variant | Accept only if |
| --- | --- | --- | --- |
| **A. Copper host** | Existing full-color F8 native copper over granite + shale | One C/G gray mask + copper palette, then 3–5 curated morphologies | Copper reads with F10 highlights OFF; host remains visible; no false ore |
| **B. Rock family** | Granite/shale authored 256px candidates | Compatible detail masks and regional palette variations | Stone identity and bedding orientation survive; fewer repeating faces |
| **C. Tree pigment** | One approved tree asset (future) | Same leaf silhouette/detail, several ramps; one fantasy-blue phenotype | Species silhouette recognized; no new gameplay species from tint alone |
| **D. Seasons** | Static season samples | 0→1 local phenology transition, with year wrap and regional offsets | No abrupt/muddy transitions; correct linear color; deterministic reload |
| **E. World feature** | Simple fixed fault/spline | Shared detail mask in continuous region-owned vein; optional SCA later | Cap/side/neighbor continuity, bounded cost, no resource stock inferred from pixels |
| **F. Performance** | P0 BC7/RGBA fallback and v2 candidate texture cache | Palette shader and pattern family in same fixed camera/view | GPU texture residency, shader samples, CPU time and frame time are measured, with acceptable fallback |

**Minimum comparison scenes:** 1 m / 3 m / 8 m, neutral/shadow light, player motion, multiple neighboring true hex-prism faces, mid/far foliage, grade/season slider and rebuild/save/load. Always document **artist approval** separately from passing tests.

### Decision matrix

- **Adopt now as design principles:** stable semantic identity, split material/condition/structure authority, deterministic variant selection, artist-vetted families, optional fantasy palette and seasonal keyframes.
- **Prototype next:** copper C/G and ramp vs current full-color copper; small rock/foliage comparisons with a diagnostic season control; continuous world-space feature mask.
- **Defer:** BC4 baking and packing, universal palette LUT schema, automatic full flora asset approval, mass conversion of 55 identities, GPU stochastic multi-sampling, production SCA-to-overlay integration, planetary season and snow simulation.
- **Reject as a default:** every material from one grayscale texture, universal base RGB tint, a new GPU draw call per individual material variation, new resource species on mere pigment changes, or atmospheric/weather state inferred from visible pixel color.

## Evidence quality and boundaries

- **High:** Hytale's official docs/patches; Vintage Story's official release and public API; Epic, Unity, Godot, Microsoft, Mojang documentation.
- **Medium:** community-maintained Eco Wiki ModKit plant descriptions; older Hytale blog explicitly labeled outdated; academic methods may not fit Hexal's specific renderer without benchmarks.
- **Not verified:** exact shader formula or GPU packing for any of the three games; Hytale annual-season palette interpolation; Eco annual-leaf colormaps; Hytale use of SCA; any claim that the user's five tree screenshots share one texture or geometry.
- **Date sensitivity:** as of November 2026, Eco has newer Weather/Update 14 changes; do not repeat pre-2025 claims that it lacks weather. Hytale's graph generator was consulted in **pre-release documentation**, and must not be treated as a stable API commitment.

**No source textures, external game assets, or third-party imagery are copied into Hexal by this report.**
