# Grayscale ore-pattern and palette gallery — first art study

**Status:** optional F8/F11 renderer experiment, developed on top of the
existing 200-chunk and F1/F2/F3 GPU diagnostics branches. This
prototype is **not** approved mineral art, a mineral registry rewrite,
mineable ore deposits, per-region geology, production compressed
texture storage or a universal material-family system.

## Reason for the experiment

Native Windows F11 testing verified the grayscale shader path
technically, but its original source-derived mask had **identical
shape and footprint** to the full-color copper candidate. Only the
pigment changed. This study intentionally introduces different
**coverage morphology**, keeping the same real granite/shale hosts
and the original full-color comparison intact.

## Controls and visible options

- **F8**: select the v2 material candidate gallery as before.
- **F11**: toggle original full-color copper (OFF) against
  shader-colored grayscale mineral overlays (ON).
- **F12**: cycle the source grayscale mask while **F8 and F11 are ON**:
  **SOURCE** (previous F11 baseline), **HAIRLINE** (thin curving
  veins), **NODULES** (rounded inclusions), **STOCKWORK** (crossing
  small veins), **RIBBONS** (folded banding), **SCATTER** (clustered
  fine flecks).
- **P**: independently cycle shader pigment ramps while **F8 and
  F11 are ON**: **COPPER**, **HEMATITE** (mostly matte, rusty iron),
  **MOONSILVER** (fictional cool-hued metal).
- **F1**: clickable developer menu including F12 and P, with the
  active pattern/palette names and disabled states when F11 is off.
- **F2/F3/F4/F5/F6/F9/F10**: previous log, GPU benchmark, culling,
  vsync, UV and condition/glint controls are retained. F3 sample
  and cumulative trial labels identify the chosen pattern **and**
  palette for later comparison.

**These are three display-pigment recipes, not three new gameplay
resources.** On either host, each selected grayscale source is
sampled identically. The original authored full-color copper is
always available as F11 OFF; F12/P do not modify that baseline.

## Data and shader scope

- `src/ore_pattern_study.rs` generates five deterministic,
  **tile-periodic** 256×256 grayscale RGBA masks, each with
  independent alpha coverage and tonal RGB. This is **procedural
  placeholder art**, not carefully authored hand-painted texture
  approval. No internet imagery or third-party art is included.
- The existing optional RGBA8-sRGB gallery grows from
  **13 to 18 layers** (12 original P0-compatible slots, source
  grayscale at 12, five new masks at 13–17) only when a compatible
  baked copper source exists. Five extra 256² RGBA8 layers use
  **1.25 MiB** uncompressed GPU storage before mips. This is
  *not* a BC4/BC7 memory win; no texture duplication per chunk.
- `CameraUniform.family` supplies small pattern/palette indices.
  The world shader samples one selected grayscale RGBA mask and
  the correct granite/shale host and uses a material-specific
  linear-light three-stop pigment ramp.
- F10 still controls decorative, light-reactive glints; the
  **hematite** pigment uses a reduced, cooler matte response, and
  **moonsilver** a cool reflective response. F9 wetness/moss/frost
  remain independent gallery effects. There is **no emissive
  moonlight** and no fake mineral inventory/stock from the alpha.
- Material roles `GraniteCopper` and `ShaleCopper` remain
  unchanged gameplay/preview IDs. All alternate pigments are
  **display-only art studies on those exact fixtures**; the
  HUD must never suggest mineable hematite or moonsilver exists
  in the simulated world.

## Visual/measurement acceptance

With baked candidate art and the 200-chunk benchmark:

```cmd
git fetch origin && git switch -C feature/ore-pattern-family-gallery origin/feature/ore-pattern-family-gallery && cargo run --release -- --benchmark-200
```

1. Start with F8 ON, F9 Dry, F10 OFF, F6 OFF. Toggle F11
   OFF/ON. The original full-color material must remain unchanged
   and reappear exactly when F11 is switched off.
2. F11 ON: cycle F12 through all six names. Compare screenshots
   on **granite and shale**, at roughly 1m / 3m / 8m.
   Shapes should be distinctly recognizable, not just tint
   differences. The host must remain legible outside alpha.
3. Hold F12 fixed; use P to change palettes. The mineral footprint
   must **not change** with pigment. Compare with F10 glints
   OFF/ON and F9 weather overlays separately.
4. With F1/F2 closed, run F5 unlocked where supported and F3
   for ≥30s per fixed condition (same camera, F4, F9, F10).
   Copy the F2 running log *after* the measurement. Compare
   submitted geometry and sampled GPU render-pass time,
   not simply uncapped FPS.
5. Verify F12/P menu rows and hotkeys are disabled when F8/F11
   are not both active. Switching F8 OFF still returns P0.
6. Art quality must be approved from native Windows screenshots.
   **Passing CI, deterministic mask tests, and shader validation
   do not prove painterly style, good natural geology or acceptable
   morphology at scale.**

## Native screenshot review — 2026-11-04

The project owner provided native Windows screenshots of the v2
gallery with F8 ON, F11 grayscale ON, and multiple F12/P states,
including an unobstructed close-up of the two rock hosts. This
constitutes **visual evidence of functional morphology and palette
switching**, but **not artistic approval**.

| Pattern | Visual review | Next decision |
| --- | --- | --- |
| SOURCE | Sparse, irregular source-derived copper inclusions; still clearly tiled | Retain as immutable A/B baseline |
| HAIRLINE | Uniform near-parallel wavy strings, strongly repeated | Replace with discontinuous branching fracture traces |
| NODULES | Large round-ended blobs, especially conspicuous in cool-blue pigment; same shapes visibly tiled | Make clustered, angular/rough-edged mineral pockets with varied size |
| STOCKWORK | Dense, highly regular wire/grid-like curves, not believable vein intersections | Replace repeated periodic lattices with sparse crossing fracture networks |
| RIBBONS | Closely spaced synchronized sinusoidal bands resembling decoration | Reject this waveform; trial irregular host-specific laminae or shear zones |
| SCATTER | Least distracting, small copper flecks, but diffuse/repetitive with limited clustering | Keep as a starting point; grow sparse clusters with clear primary/secondary scale |

**Color/pigment observations:** Copper, Hematite and Moonsilver
change the same chosen mineral footprint, as intended.
However, neither a saturated orange fill nor broad pastel-blue
nodule swatches automatically read as metal or ore. F10 specular
response cannot be judged reliably from still screenshots;
the static images also do not prove animated shader highlights.

**Host/geometry observations:** the high-frequency granite host
competes with ore coverage; the shale cross-section is much
quieter and visually distinct. The unobstructed close-up further
shows the texture projection/size change around cap/side
boundaries and how sharply the tiles repeat. These are not
evidence of a corrupted mesh or wrong gameplay ore ID.

**Direct implementation cause verified in code:** `src/mesh.rs`
writes side UVs as a per-hexel 1m perimeter/height mapping while
caps use world X/Z; `src/world_shader.wgsl` samples the
grayscale mask at `v.uv`. `src/ore_pattern_study.rs` uses
periodic, mostly sinusoidal/repeated fields. Tiling is therefore
expected, not a GPU defect. Cycling a pigment cannot fix
these shape and projection artifacts.

**Result:** do **not** approve any of the five generated masks as
production art, and do **not** promote the tile-based pattern
generator to general geology or resource-ownership logic.

### Next controlled engineering slice — P2.4 proposal

1. Preserve F11 full-color and F12 SOURCE baselines.
2. Test one mineral morphology at a time with **stable
   world-space 3D coverage**, so a mineral feature crosses
   adjacent caps and sides at the same 3D position, rather
   than restarting on every hexel's local UVs.
3. Replace the periodic line generators with intentional
   macro/meso/micro hierarchy: a few dominant irregular veins
   or pockets, smaller branches, and restrained flecks. Host
   rock remains legible; not a uniform net or repeated stripes.
4. Measure the new shader with F5 Immediate, F1/F2 closed and
   F3 ≥30 seconds at identical camera settings. Extra triplanar
   samples or procedural field noise are **not free** and must
   be measured before generalizing them.
5. Seek native fixed-camera screenshots on granite and shale
   at close and several-metres distance, including cap/side
   joins and F10 off/on. No more screenshot requests for this
   first review—the uploaded set is sufficient to set targets.

## Subsequent experiment: P2.4 world-space vein

After this visual review, a **seventh F12 option, WORLD VEIN**, is available in the stacked [P2.4 implementation](p2-4-worldspace-mineral-vein.md). It uses a 3D world-position field rather than adding a seventh texture mask. The six original tile-based patterns, this screenshot review, and their findings remain valid historical baselines. The new method has not yet passed native art/performance review.

## What not to generalize yet

The five masks are simple noise/curve-generated test patterns,
not authored sedimentology and not connected across neighboring
hex-prism caps/side UVs in real 3D world space. On distant faces,
repeating 1m textures may still look synthetic. These are deliberate
gates before choosing actual artist-approved reusable masks,
bark/leaf/soil families, BC4 packing, feature-space projection,
world-space mineral veins, region/biome controls or seasonal
foliage ramps.

References:
- [F11 original comparison](p2-3-grayscale-copper-gallery.md)
- [Industry comparison and design implications (draft PR #32)](https://github.com/sdhaight1-hash/Hexel-survival-game/blob/docs/grayscale-material-families-study/docs/material-variation-industry-research.md)
- [200-chunk visual/performance benchmark](benchmark-200-chunks.md)
- [F2 running log and sampling history](benchmark-history-console.md)
