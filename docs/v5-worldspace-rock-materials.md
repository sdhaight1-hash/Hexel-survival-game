# V5 world-space geological material prototype

**Status:** Experiment only. Not production art, not an ore simulation and not a replacement for V4.

## Run

```cmd
git fetch origin experiment/v5-worldspace-rock-materials
git switch experiment/v5-worldspace-rock-materials
git merge --ff-only FETCH_HEAD
cargo test --lib v5_gallery_is_twelve_isolated_pads_without_materials_on_walkway
cargo run --release -- --rock-v5-gallery
```

No new PNG archive or installer needed. The V5 material layers are procedural,
with deliberately simple grayscale grain and rock-specific color palettes.
This avoids confusing V4's source artwork with the architecture test.

## What to compare

Twelve pads, each 10x10 axial cells and 3 cells tall. A neutral walkway
runs beside them. Face data is continuous 3D position rather than independent
top/side texture artwork. Player spawns near first pad looking along the lineup.

| Pad | Rock | Experiment |
|---:|---|---|
| 01 | Granite | quiet base |
| 02 | Granite | base + broad world-space palette |
| 03 | Granite | plus continuous crystalline structure |
| 04 | Granite | plus sparse accent |
| 05 | Shale | quiet base |
| 06 | Shale | base + broad world-space palette |
| 07 | Shale | plus continuous bedding |
| 08 | Shale | plus sparse accent |
| 09 | Basalt | quiet base |
| 10 | Basalt | base + broad world-space palette |
| 11 | Basalt | plus continuous fractures |
| 12 | Basalt | plus sparse accent |

The structure field uses positions expressed in metres, including vertical Y,
so rock structure can cross cell boundaries on the same surface and around
corners. For comparison, variant positions offset by 12 metres to share the
same local pattern. Per-side face tint is disabled **only in V5** to isolate
texture seams.

## Assessment

1. At ~1m, inspect left and right corners. A line present with F6 checker
   enabled is likely geometry/UV-related, not an authored source-asset seam.
2. At ~3m inspect 10x10 pads. Does the sparse structure read as **rock**,
   rather than shingles, mosaics or repeating wallpaper?
3. At ~8m move sideways slowly, looking for shimmer and bands. This
   first pass deliberately does *not* have a prefiltered procedural noise LOD.
4. Inspect the shale bedding across multiple vertical cells and around the
   top corner; it should be one pattern field rather than 1m restarts.
5. Compare base vs palette. Tint should vary **within** a pad without
   hex-by-hex flat color jumps.
6. Compare stages 3 and 4. Accents should be rare and should not drown
   out the quieter rock.
7. Compare V5 vs V4 at the same distance, and report any frame pacing issues.
   F3 measures CPU metrics, not true GPU execution time.

## Engineering boundaries

- This experiment shows continuous **visual** structure, not simulation-owned
  geological formation or ore inventory.
- It uses renderer-local world coordinates; it has **not** solved stable global
  geology across torus rebases or wrap boundaries.
- The grayscale base and grayscale-to-color mapping are mathematical prototypes,
  not finished palette-mask or BC7 art assets.
- Current procedural noise uses multiple shader evaluations per fragment.
  It is intentionally unoptimized until visual quality is validated.
- It does not implement rock-to-rock contact blending: each pad has one rock.
- The dark-to-light ranges are provisional and may require artist tuning.
- The F6 checker remains available. F8-F10 artistic preview effects are disabled.
- Because V5 is on its own opt-in branch and mode, existing P0/V4 and
  streamed/torus game modes are left unchanged.

**Validation:** Commit includes a world-fixture regression test; native Windows
build, WGSL validation and actual runtime performance need local execution.
