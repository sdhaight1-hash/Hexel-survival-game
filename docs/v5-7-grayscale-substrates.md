# V5.7 — Grayscale rock substrates, not more palette tuning

This branch starts from V5.6, preserving the five deterministic palette families
per rock, 14 cm solid-only edge color blending, and the subtle internal
four-related-tone micro pigments. It does **not** change the gallery geometry,
P0 material pack, world generation or the existing geological overlay system.

## Controlled change

**Removed:** V5.6 height signal, perturbation derivatives, the bump-normal
shading path, renderer toggle state and V5-only F4 bump shortcut. The geometric
normal and the same moderate directional sun lighting remain.

**Replaced:** the generic two-scale cloudy grayscale material with THREE
rock-specific 3D substrates:

- **Granite:** a coarse irregular 3D mineral-cell partition. Different cell
  identities represent light feldspar/quartz versus dark mica-rich domains;
  each has additional fine grain. Thin grain contacts are understated.
- **Shale:** warped but directionally coherent sediment lamination at two
  scales, broken up by localized cleavage/disruption. It is intentionally
  anisotropic, unlike granite.
- **Basalt:** dense fine groundmass interrupted by rare, rounded darker
  vesicles. No broad granite mineral patches or shale bands.

The grayscale substrate is sampled in rock-family world coordinates, not
hashed separately per exposed face. A/B/C/D pads share identical local
substrate coordinates. Stages C/D retain their existing structural details,
jointing, quartz traces, slip zones and cooling cracks. These later features
are still provisional grayscale/color overlay effects.

The per-hexel palette and tiny per-hexel pigment now sit on top of visibly
different grayscale designs. These are **procedural grayscale prototype
materials**, not finished baked art or imported photographic textures. If
these shapes work, bake or author filtered textures rather than shipping
expensive per-fragment procedural cellular scans.

## How to test

```cmd
git fetch origin experiment/v5-7-rock-grayscale-substrates
git switch experiment/v5-7-rock-grayscale-substrates
cargo test --bin hexel-survival-game rock_palette_edges
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

Keep F5 in normal-material mode and F6 UV checker OFF. F4 is no longer
a bump toggle; it resumes its existing non-V5 debug behavior.

Inspect grayscale **A** pads (1 granite, 5 shale, 9 basalt) first.
They should be distinguishable without any palette tint. Then inspect
B pads (2, 6, 10) with the saved per-hexel palettes; C and D add the
unchanged structural overlays. View at 1m and 3–8m.

Look especially for:
- Mineral domains reading as jagged artificial pixels instead of stone,
- Shale layers turning into repetitive clean stripes,
- Basalt pores turning into widely repeated dots,
- Moire/shimmer or heavy GPU time when close to the material,
- Any color-edge regression relative to V5.6.

## Performance warning

The cellular granite and sparse-vesicle basalt currently scan up to
27 candidate cells per rock fragment. That is deliberately an exploratory
quality baseline, **not** a production-performance implementation.
Compare F3 performance / subjective frame pacing to V5.6 before approving
the art direction. The next optimization should precompute these details
into filtered grayscale tiles or a compact volume atlas if they look right.
No performance measurements or local shader execution are available yet.
