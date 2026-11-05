# V5.3 — restrained rock character on coherent palettes

This is an opt-in art experiment on top of V5.2. It retains the five
deterministic palette-region endpoints and world-space blending, while
revising rock detail so granite, shale, and basalt are recognizable at
normal gameplay distances without returning to V4's noisy/shingle look.

## Changed

- **All three:** stronger but restrained two-scale grayscale grain.
- **Granite:** clustered feldspar tint and sparse dark mica. No polygon outlines.
- **Shale:** warped, interrupted 3D bedding instead of uninterrupted sinusoidal
  stripes; continuity survives across exposed hexel faces.
- **Basalt:** predominantly dark, quiet groundmass, localized small pores, and
  irregular short fracture traces rather than a regular grid.
- **Stage D:** sparse highlights/accent features, not an always-on dense overlay.
- **Not changed:** V5.2 five-palette selection, palette blending, material
  identity, ore data, torus/worldgen, gallery footprint, P0/V4 modes.

## Install and run (Windows Command Prompt)

```cmd
git fetch origin experiment/v5-3-rock-character
git switch experiment/v5-3-rock-character
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

If switching branches conflicts with your modified Cargo.lock, do not reset.
No PNG ZIP, Python, or new runtime dependency is needed.

## Exact comparison

The 12 pads are unchanged:

| Pads | Rock | Stage |
|---|---|---|
| 01–04 | Granite | A quiet base / B palette / C structure / D accents |
| 05–08 | Shale | A / B / C / D |
| 09–12 | Basalt | A / B / C / D |

1. Keep **F6 off** and **F5 in normal view**. Compare each B/C/D trio at
   roughly 1, 3, and 8 metres. C should be recognizably different from B,
   and D should not overpower C.
2. On shale (pads 07–08), inspect whether warped bands cross individual
   hexel boundaries and turn around corners without a 1-metre reset.
3. Look for large, unintended dark blotches in granite and for fake
   repeating grid fractures in basalt. Report the pad and face.
4. Slowly move sideways. Look for texture crawling/flickering.
5. Use F5 to inspect raw five-color region IDs, then blended palette
   weights; both should look identical to V5.2 because this experiment
   deliberately did not modify the palette system.
6. F6 checker helps separate face/UV seams from material structures.

## Boundaries

This is not production texture art or an integrated 23-rock system.
No 3D world-owned geological data, rock-contact blending, or guaranteed
seamless torus rebasing has been added. No filtered noise mip levels.
The shader performs several 3D value-noise evaluations per pixel;
check smoothness and performance, especially on the RTX 2060.

Code was committed to GitHub and source-inspected; a Windows runtime
build and the shader validation test still need to be run locally.
