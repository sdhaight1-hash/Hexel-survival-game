# V5.8 — Geological structure-linked internal palette colors

Built directly on V5.7. The successful **five families per rock**, deterministic
per-hexel assignment, **14 cm edge-only blending** and grayscale geology
are unchanged.

## Why

The V5.6 micro-palette used nearly imperceptible independent color noise,
which got overwhelmed by the clearer V5.7 grayscale granite/shale/basalt
patterns. The goal now is not simply to raise random color contrast:
**the geological structures that already generate rock brightness also
select related colors inside each hexel's assigned palette family.**

## What actually changed

The grayscale functions now return their existing intensity PLUS two
already-computed geological masks. No second scan of the 27-cell granite
mineral distribution or basalt pores, and no new 3D-noise evaluations.

- **Granite:** mineral-cell ID controls the broad variation among cooler
  dark minerals, warmer feldspar-like grains, paler quartz-like grains,
  and slightly warmer light grains. Fine within-grain variation uses
  the existing micrograin signal.
- **Shale:** the existing warped bedding mask controls dark vs light
  sediment layers; its finer lamination mask adds a second related
  tonal mixture. The color follows strata instead of unrelated flecks.
- **Basalt:** the existing pore-occupancy mask controls dark vesicle
  coloring relative to fine groundmass. The groundmass's existing
  small-scale noise influences quieter secondary tonal breakup.

Each rock has **four related RGB multipliers** around its parent palette.
The assigned one-of-five family still controls the main color through
the original mesh attribute. A tiny deterministic per-family bias keeps
the related tones coherent with the family. No texture displacement,
specular shininess, bump shading, or new geometry has been introduced.

**A stages (1/5/9) are grayscale-only controls**; B/C/D receive the
internal colors. The internal pattern attenuates to neutral at real
same-rock contacts, while V5.5's CPU-generated edge tint continues
to interpolate the parent palette there. Air boundaries do not fade.

V5.7's structural overlays remain world-space and unmodified. These are
prototype pigment multipliers, not photogrammetry, finished art assets
or a physically based subsurface materials system.

## Test on Windows

```cmd
git fetch origin experiment/v5-8-structure-linked-pigment
git switch experiment/v5-8-structure-linked-pigment
cargo test --bin hexel-survival-game v58_pigment_tracks_geology_without_new_noise_or_bump
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

Do not reset modified Cargo.lock or any other uncommitted user changes.

Use F5 **normal materials** and F6 **OFF**. Compare in this order:

1. **A** pads (01, 05, 09): grayscale geology should match V5.7.
2. **B** pads (02, 06, 10): color should now vary WITH the existing
   mineral, bedding and pore pattern *inside* individual hexels.
3. **C/D** pads: check whether V5.4 geological overlays still remain
   subtle and continuous, not washed out or visually doubled.
4. Inspect seams where two solid cells touch and exposed-to-air borders.
   The original V5.5 narrow palette edge blending must remain intact.
5. Inspect from 1 m, 3 m and 8 m, including slow motion for shimmer.
   Compare perceived frame pacing against V5.7.

The shader was code-reviewed for structure and variable use, but there
is no Rust compiler or GPU shader validator in the authoring container.
The two local tests and runtime are the acceptance gates.
