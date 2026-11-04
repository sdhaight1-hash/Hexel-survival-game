# Geological ore-deposit reference and chunk-independent generation contract

**Status (2026-11-04):** research-backed **P2.5 design proposal**, **not implemented**. The working P2.4 WORLD VEIN shader is an optional cosmetic comparison, not world-owned ore. Use this document to constrain the next implementation slice. Do not claim a geological process simulation.

## Sources: what geology actually supports

1. **Hydrothermal fluids and structural pathways.** Metal-bearing fluids move through faults, joints and permeable lithologies, vertically and laterally. Cooling, mixing, boiling, chemical reaction and changing pressure/temperature may precipitate minerals. Veins and their alteration halos reflect where the fluid travelled and reacted, not a world-space sine wave. [USGS: Dilles & John (2021)](https://www.usgs.gov/publications/porphyry-and-epithermal-mineral-deposits); [USGS: physical hydrogeology](https://www.usgs.gov/publications/physical-hydrogeology-ore-deposits).
2. **Porphyry Cu systems are clustered.** Mineralized fracture density, stockwork veinlets, disseminated grains and spatial alteration zoning make broad volumes of rock variably mineralized. Grade is not constant through a vein or uniformly sprinkled among nearby rock cells. [USGS porphyry Cu](https://pubs.usgs.gov/bul/b1693/html/bull9ik5.htm); [USGS porphyry Cu-Mo](https://pubs.usgs.gov/bul/b1693/html/bull5mlr.htm); [USGS porphyry Cu-Au](https://pubs.usgs.gov/bul/b1693/html/bull5m0l.htm).
3. **Ore shoots are localized.** In certain epithermal vein systems, intersections and changes in strike/dip can localize rich shoots; repeated fluid flow and fracture sealing can leave banding and breccia. Do **not** assume every intersection is automatically high grade. [USGS Creede epithermal vein model](https://pubs.usgs.gov/bul/b1693/html/bull7a0j.htm).
4. **Not every deposit is a vein.** Sediment-hosted copper can be stratabound/stratiform and disseminated as cement/replacement along bedding with mineral zoning. Other deposit families are magmatic, metamorphic or surficial (including placers). A one-size-fits-all spider network would be geologically wrong. [USGS sediment-hosted Cu](https://pubs.usgs.gov/bul/b1693/html/bull27xx.htm); [USGS sediment-hosted Cu deposit model](https://pubs.usgs.gov/sir/2010/5070/m/); [USGS mineral deposit classifications](https://pubs.usgs.gov/of/2003/of03-220/MINMOD/deposit_models.pdf).
5. **Mineral appearance depends on chemistry/weathering.** Common ore minerals in porphyry copper deposits include copper–iron sulfides (e.g., chalcopyrite and bornite), not necessarily bright native metallic copper. Near-surface weathering can generate green/blue copper minerals; exposed iron oxides may be reddish. Game art must distinguish host alteration, gangue, primary ore and weathered coatings rather than recoloring a whole vein as shiny metal. [USGS porphyry Cu description](https://pubs.usgs.gov/bul/b1693/html/bull9ik5.htm).

**Interpretation:** geological examples constrain plausible shapes and relationships. The proposed generator below is a deliberately **simplified causal analogue**, not an empirical predictor of deposits, a fluid solver or an ore reserve model.

## The correct world and chunk relationship

**World generation owns ore bodies; chunks merely sample them.**

- Each seed/version/tectonic province defines potential intrusive sources, host lithologies, sediment layers, faults, fractures and fluid/mineralization domains. A candidate **deposit descriptor** is a finite 3D volume with canonical stable ID, type, mineral assemblage, geometry, alteration zone, variability seeds and overall stock.
- Chunk (q,r,y) requests query these **same global/canonical descriptors**. They do **not** roll for a fresh vein. Deposits and branches can cross an arbitrary number of horizontal **Q/R** chunk boundaries and vertical **Y** chunk boundaries. Neither the section origin nor any face UV determines the vein.
- The first target is 16×16×16 hexel sections in absolute axial Q/R/Y with world-space geometry expressed in the existing physical metre scale. The vertical mapping to the long-term terrain model is still an open issue (see `docs/world-generation.md`); don't claim validated geological depths or sea-level layers yet.
- The selected long-term world is a **1,024 km × 1,024 km torus** (geographic axes). The map between global geographic periods and hex-lattice seam translations is explicitly **unresolved**. No naïve independent `q % period`/`r % period` patch: that would risk geometry and geology disagreeing at the world seam. This design requires the canonical hex seam transform to be settled and tested before toroidal deposit continuity is declared supported.
- The GPU may use camera-relative positions for numerical precision over large distances; **ore identity and stock must remain based on canonical integer/fixed world coordinates**, not camera origin or rounded fragment-space floats.

### Proposed descriptor and query contracts (not Rust API yet)

```text
WorldManifest = { world_seed, geology_version, coordinate_mapping_version }
DepositDescriptor = {
    id, owner_region, formation_kind, mineral_assemblage,
    stable_world_anchor, 3D orientation / extent,
    parent_fracture + bounded child fracture segments,
    host_eligibility, grade_distribution_seed, finite_stock_budget
}
OreSampleAtCell = {
    deposit_id(s), ore_kind, concentration_or_grade,
    ore_bearing_fraction, host_alteration, gangue_fraction
}
```

Generation order for **base** geology: geological province/host -> candidate source and permeability -> bounded deposit descriptors -> deterministic fracture/bedding/cluster evaluation -> per-cell ore-bearing fraction/grade -> derived material/mesh and mining inventory. **Persist player edits separately**; reloading/unloading chunks cannot restore depleted ore.

For generation across spatial partitions: assign each descriptor a **canonical owner region** determined from its anchor and world seed/version. A chunk queries all neighboring owner regions whose bounded descriptors might intersect it (via a fixed maximum reach or deterministic spatial index), deduplicates by descriptor ID and resolves overlaps in stable order. Its content is invariant under chunk generation order. Do **not** seed deposits from the chunk being loaded; do **not** let an unloaded neighbor masquerade as known air.

Existing `World::get` returns `Air` when a section is absent, appropriate only for the **static current fixture**. In a streaming world, missing neighbor data must be resolved from backing deterministic/persisted terrain state or treated as unknown and remeshed later; it is **not** evidence of an exposed rock face or a missing vein.

## Geologically motivated formation archetypes

| Family | 3D structural origin | Expected horizontal / vertical expression | How clusters and grade arise |
| --- | --- | --- | --- |
| Main hydrothermal vein / lode | Finite bent fracture sheet, dip and strike, pinch/swell | Steep, shallow, or oblique; traces on caps and sides change with sheet angle | Main filled sections with gaps; a few richer shoots at favorable bends/intersections |
| Spider veins / horsetail splays | Sparse, hierarchical child fracture sheets seeded off a parent | Child veins fan obliquely and taper in all 3 axes, crossing chunk borders | Richer near selected parent-child junctions, thinner/leaner at many ends; no uniform web |
| Stockwork | Several distinct fracture sets and branches near an intrusion/structural center | 3D clustered box/ellipsoid with many unequal orientations | Veinlet density and disseminated grains increase toward favorable fracture/permeability zones |
| Disseminated/porphyry halo | Altered host volume surrounding source and vein network | Lumpy, broader 3D volume not confined to lines | Low-to-moderate pervasive mineralization with correlated high-grade pockets |
| Stratabound / bedding replacement | Eligible sedimentary layer/reactive contact with permeable pathways | Mostly subhorizontal or dipping sheets **following layers**, not fixed screen-up direction | Lens-shaped enriched beds, sparse replacement and local high-grade concentrations |
| Contact/igneous-related lens (later) | Reactive intrusive boundary / host contact | Irregular pockets/lenses tracking contact surfaces | Host-specific reaction pockets; no assumption of vein-like shape |
| Placer/weathering (later) | Surface transport, accumulation and supergene processes | Shallow terrain/water-linked zones, not underground generic vein lines | Hydrological/slope/oxidation controls; distinct conservation and surface rules |

**Orientation rule:** Each fracture owns world-space tangent/normal/extent. A low-angle sheet is not a "vertical vein rotated in the shader"; it is a 3D formation whose intersection with any arbitrary exposed face defines the visible cross-section. Structural priors can vary by province and host; allow steeply dipping, near-horizontal and intersecting networks without any chunk-axis bias.

**Deposit clustering rule:** Sample **a few correlated parent systems**, rather than independent per-cell ore rolls. Concentration combines (a) position in finite deposit volume, (b) host compatibility, (c) fracture/sheet density or bedding suitability, (d) selected rich shoots at offsets/changes of orientation, (e) low-frequency correlated patches and fine-scale bounded variation. Unmineralized host gaps must remain. Keep total finite ore stock accounting tied to the cells/edits, not the current visible overlay.

### Core, halo, veins and micro detail

Separate **fracture mineral fill/core**, weaker **alteration halo**, optional **disseminated specks**, and *host rock*. More mineral fill raises ore concentration and may contribute glints; the broad alteration halo changes host appearance but is **not necessarily mineable ore or shiny metal**. Reflectivity/color depend on mineral species and weathering, not concentration alone.

P2.4 already demonstrates fragment-level world-space continuity, but every fragment recomputes 3D noise. P2.5 should explore a **small CPU-generated descriptor plus deterministic per-cell ore fraction and/or precomputed world-aligned mask**, then render inexpensive local detail. Do not assume a large per-fragment fractal network is fast. Preserve the fixed F8/F11/F12 source comparison and compare GPU pass times at F5 Immediate, F1/F2 closed.

## Minimal first code milestone (P2.5A)

Before constructing several ore families, implement one `CopperFractureSystem` study that exercises the architectural contract:

1. One finite **oblique/steep parent fracture** spanning horizontal **q=15/16**, **r=15/16** and vertical **y=15/16** section boundaries; 2–4 unequal tapering spider branches, and 1–2 selected richer pockets. Use seed-driven stable world anchors and 3D geometry; these counts are **test-fixture values**, not geological frequency claims.
2. One **shallow/bedding-following** variant using the same underlying 3D descriptor/query pattern, to demonstrate that mineral continuity is not only vertical.
3. Generate a deterministic `OreSampleAtCell` at absolute cell coordinates for both variants. Per-cell grade and mineralized fraction must match exactly whether a cell is queried through chunk A, chunk B, or directly from the world descriptor. Seed and geology version changes intentionally change output.
4. A chunk generator or optional fixture visualizes sampled ore with the existing host and F10 color/glint rules; do **not** silently rewrite authoritative `World::demo` or the 200-chunk benchmark. Keep each original F11/F12 comparison available.
5. Persist mine/removed-ore edits only when an actual gameplay-mining interface exists; until then, prove that ore totals are finite, no section reload generates new inventory, and that overlapping deposits have deterministic ownership.
6. **Performance:** measure descriptor query/build CPU ms, extra per-cell memory and scene GPU pass ms. The derived renderer cannot allocate a unique texture for each chunk or evaluate a growing fracture graph for each fragment.

### Tests and acceptance gates

- **Seams:** sample adjacent sections across positive/negative Q and R boundaries and Y at 15/16, -1/0; compare exact descriptor IDs, grade and ore fractions in boundary neighborhoods. Check cap/side world projection.
- **Order and streaming:** generate A then B vs B then A; evict/reload A; vary thread scheduling; results and initial stock must be identical. Absent neighbor sections cannot be interpreted as air/ore absence.
- **Conservation:** stock summed from owned ore-bearing cells is counted once; applying mined edits subtracts stock once, without duplication on section reload, even if a vein crosses 5+ sections.
- **Shape:** graph is bounded/finite; no tile restart; both steep and shallow/oblique types; child widths taper and branch density is spatially concentrated, not an equal-width lattice. Host eligibility is explicit.
- **Torus:** only after finalizing the hex-lattice periodic seam mapping, repeat above tests at both selected world-wrap boundaries and corners. The current raster-periodicity tests do **not** satisfy this contract.
- **Visual review:** fixed-camera granite and shale comparisons on caps, vertical faces, horizontal faces, across multiple sections; core versus halo and pinch/swell/spider branches must remain legible at close and medium viewing distances.
- **Performance:** record F5 unlocked F3 p50/p95/p99 and sampled GPU pass for at least 30 s with menus closed. Compare P2.4, P2.5A and source at identical camera/F4/F9/F10 conditions; do not attribute FIFO-limited 60 FPS to comparable cost.

### Avoid these shortcuts

- A per-chunk random ore-roll with matching edge pixels; it does not make a coherent, conserved deposit.
- Shader-only random veins that look continuous but have no finite world-owned mineral mass.
- One absolute sine wave extending endlessly around the world; finite deposits and correlated clustering matter.
- Applying spider-branch shapes to stratabound ore, residual placer or igneous mineral bodies.
- Deposits driven by arbitrary fragment-space coordinates, camera rebasing, or independent torus q/r modulo.
- Premature physically detailed fluid/thermal/chemical simulation. Use geology-constrained fields and validate the results first.

## Status / next step

Research and implementation contract only. **No production geology engine, physical fluid transport, ore mining or cross-chunk persistence is implemented by this document.** The next code slice should be **P2.5A finite, deterministic world-owned copper fracture descriptors and seam tests**, followed by visualization as a separate incremental PR.

Related: [planned world topology and persistence](world-generation.md), [P2.4 preview and spider-vein art direction](p2-4-worldspace-mineral-vein.md), [ore-pattern gallery](ore-pattern-family-gallery.md).
