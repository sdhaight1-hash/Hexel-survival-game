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
- [Industry comparison and design implications](material-variation-industry-research.md)
- [200-chunk visual/performance benchmark](benchmark-200-chunks.md)
- [F2 running log and sampling history](benchmark-history-console.md)
