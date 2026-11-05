# V5.4 — geological overlays (controlled 12-pad experiment)

The V5.3 shader is the base. Only the V5 shader's grain amplitude and
stage-D disruption treatment have been changed. This does **not** modify
world generation, materials/catalogs, V4 textures, P0 materials, or real
geological state. The five-palette coherent field is preserved verbatim.

## What to test

Each rock retains four ten-by-ten-by-three pads:

| Pads | Rock | A | B | C | D |
|---|---|---|---|---|---|
| 1–4 | Granite | base | +5-region palette | +mineral structure | +joints and quartz traces |
| 5–8 | Shale | base | +palette | +warped bedding | +localized slip-zone traces |
| 9–12 | Basalt | base | +palette | +pores/fractures | +cooling-joint corridors and oxidation |

All base stages use modestly stronger two-scale grayscale grain
(0.36 -> 0.41 typical, 0.24 -> 0.28 basalt; secondary 0.22 -> 0.25).
Stages A-C are otherwise the V5.3 structures. New stage D overlays are
*rock-specific*, sparse, gated by low-frequency three-dimensional noise,
and continuous across exposed hex faces.

Important limits: the thin seam patterns are **visual only**, not cracks
in the solid simulation or genuine geology-derived faults. They use
gallery-local coordinates; production toroidal rebasing will require
canonical geological identity. This is not a commitment to fracture
frequency, and no performance benchmark has been run.

## Windows test

```cmd
git fetch origin experiment/v5-4-geological-overlays
git switch experiment/v5-4-geological-overlays
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

Do not reset local changes, including a modified Cargo.lock, if Git
reports an unrelated checkout conflict.

**Keep F5 on normal materials and F6 OFF** for art inspection.
Use F5 raw-ID and blended palette diagnostics only if a suspicious
color seam appears; the palette field should be unchanged.

Compare C versus D for each rock at ~1 m, 3 m and 8 m. The expected
visible differences are narrow, occasional granite jointing/quartz,
localized shale slip traces, and rare basalt cooling cracks/oxidation.
Inspect adjacent side and top hex faces for discontinuities; inspect
jagged outer silhouettes separately as geometry, not material.
Look for cracks turning into noisy stripes or for significant frame
pacing regressions on the RTX 2060.

If D is indistinguishable from C, adjust feature *coverage*, not base
saturation. If D is too busy, reduce gate density. If fractures alias
or shimmer, investigate filtered/mip-compatible structural masks before
extending this prototype to the full rock catalog.

The shader source was reviewed for balanced braces, reserved `patch`
identifier, and scope; native WGSL validation and runtime execution
are pending the local `cargo test` and `cargo run`.
