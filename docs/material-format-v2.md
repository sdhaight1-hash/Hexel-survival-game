# Hexel material format v2 — P0 prototype and proposed production contract

**Status:** proposed production format; no v2 asset compiler or JSON loader exists yet.
The playable P0 scene uses **source-art-derived visual prototypes** in
`src/materials_p0.rs` with the renderer in `src/render.rs`. It decodes
five of our original high-resolution PNG crops (soil, granite, shale,
forest floor and native copper), seam-blends the source surfaces into
256px tiles, then composes grass and ore masks **at build time**.
The resulting reference colors are compressed into an embedded BC7 pack,
with a CPU-decoded RGBA fallback where GPU BC7 isn't supported.

All 55 legacy atlas crops remain unchanged. These derived P0 color tiles
are now generated at **build time**, not startup, and remain neither a new
permanent material format nor approved final artwork. See
[texture compression](texture-compression.md) for the BC7 pack and fallback.

## Why not hex-shaped texture files?

A hexel has two horizontal hexagon caps and six vertical rectangular faces.
Standard 2D square/rectangular images are fine: **UV mapping maps image coordinates
to each actual polygon**. Hex-shaped files are not a requirement. Avoid
unconditionally stretching UV 0–1 over every hexel face.

The current hexel is 1 m across flats, 1 m tall, so a regular hexagon edge
is about **0.5773503 m**. A side image intended for 128 texels/m must not
be mapped at 128 px across that edge: the visible width is about 74 px.
The P0 engine now maps cap UVs in X/Z metres and side U around its six-edge
perimeter in metres. Side V spans one metre. A half-texel V inset prevents
the *top-edge-only* grass rim from wrapping onto the bottom edge when using
the one-metre repeat texture.

This mapping is a first correct physical-scale test, **not** an approved
universal solution for layered cliffs, anisotropic sediment, or massive
world-scale coordinates. Cap/world UV origin precision must eventually be
chunk-local to avoid floating-point precision loss on the 1,024 km torus.

## What the current in-game P0 lab actually draws

The game launches the existing, bounded one-chunk first-person scene. At
axial `r=9..12`, the area in front of the player contains an explicit
**visual reference strip**; it does not represent real geological generation.
The row `r=10` is raised by 1m to expose correctly mapped vertical sides.

| Q cells | Visible fixture | Substrate, mask and appearance |
| --- | --- | --- |
| 2–4 | Bare soil | bare low-fertility-style soil base |
| 5–6 | Very sparse grass cover | soil + independently generated sparse mask |
| 7–8 | Sparse grass cover | same soil + intermediate mask |
| 9–10 | Full grass cover | same soil + dense mask |
| 11 | Granite | original procedural rock base |
| 12 | Copper in granite | granite base + copper-only overlay |
| 13 | Shale | original procedural rock base |
| 14 | Copper in shale | shale base + **same** copper-only overlay |

The top uses a soil-and-grass cap composite; the side uses soil with an
**upper rim** grass-to-dirt transition; the bottom remains soil.
Underlying gallery cells stay soil, not ore-bearing rock throughout the
column. This is a gallery, **not a mineable resource deposit or registered
ecology**. Visuals are baked from source artwork and independent ground-cover/ore masks
into 12 fixed-resolution layers **only for P0** so the GPU shader stays minimal.

### Inspecting at runtime

```text
cargo run --release
F6: toggle 16px checker-grid UV overlay on every terrain face
F7: cycle checker-grid density: 64, 128, 256 texels per metre
WASD / mouse / Space / Ctrl / Shift: existing movement controls
Esc: release or recapture cursor
```

The F7 density comparison applies to the **checker diagnostic**, not to
the baked grass-rim texture, because blindly repeating the whole grass-rim
image vertically would add a second false grass layer halfway down the side.
P0 source art is now retained at its original higher resolution and sampled
into 256×256 RGBA gallery tiles at a 256-texel/m working scale. This is a
preview density, **not** a global production standard. The checker still
cycles 64 / 128 / 256 texels per metre.

**Acceptance tests:** valid source PNG decoding and 256² RGBA layers,
approximate base-tile seam continuity, correct material selection by face,
denser masks in order, top-only grass rim with plain bottom rows,
copper composite over two distinguishable hosts with most underlying
host-rock pixels intact, and side U density of approximately 147.8 pixels
per 0.577m side. GPU appearance must be checked
on the user's native Windows device; automated tests cannot prove aesthetics.

## Proposed v2 production material recipe

This is an **illustrative JSON contract, not a file that is currently read**:

```json
{
  "schema_version": 2,
  "id": "soil.low_fertility",
  "status": "candidate",
  "physical": { "reference_metres": 1.0, "texels_per_metre": 128 },
  "faces": {
    "top": {
      "base": "soil.low_fertility.albedo",
      "overlay": "ground_cover.grass.top_mask"
    },
    "side": {
      "base": "soil.low_fertility.albedo",
      "overlay": "ground_cover.grass.side_top_rim"
    },
    "bottom": { "base": "soil.low_fertility.albedo", "overlay": null }
  },
  "variations": { "stable_seeded_selection": true },
  "authored_revision": "to-be-assigned"
}
```

Production layers must reference **real approved assets**, not the P0
procedural demo layers. A future compiler must ensure:
- Stable IDs never depend on transient GPU array indices or save-file offsets.
- Base albedo and alpha overlays can have **different source resolutions**;
  bake/pack by compatible format/dimensions or use atlases. There is **no
  universal 32px or 128px restriction**.
- Correct color spaces: albedo is sRGB, geometric/data/mask/roughness maps
  are linear. Transparent masks require appropriate alpha handling.
- Mipmaps, atlas padding and sampler wraps are authored per material role;
  use side-edge continuity and test cap joins across all six sides.
- Optional normal/roughness/height channels, physically sensible layer scale,
  host rock identity, grade visual masks and seasonal variant references.
- Packed production runtime resources are derived, versioned outputs. Source
  art and authoring tools own the edit-time assets, game data owns geology,
  ore quantity, cover density and living plants.
- Runtime should resolve `material ID + face role + overlay state` without
  the mesh guessing geology or climate.
- Build and GPU memory limits must be benchmarked before deciding final
  texture-array, atlas or compressed asset format.

## Roadmap and connected work

Full texture and vegetation milestone plan: GitHub **issue #14**.
Related foundation work: issues #9 (faces/grass), #11 (host-rock ores),
and #13 (first approved flora art).
This P0 scene does **not** implement real grass plants, species assets,
persistent crops, automated material loading, chunk streaming, fixed hex
torus periods, or production geology. Those have their own acceptance gates
and should not be marked complete by the demonstration.

## Build-time color compression (implemented)

P0 color layers are baked into GPU BC7 blocks with a one-time CPU RGBA
fallback. The GPU memory budget is 0.75 MiB on BC-capable adapters and
3 MiB otherwise for all 12 layers. Quality validation, supported-device
selection and remaining limitations are documented in
[Texture Compression Pipeline](texture-compression.md). BC7 does not
satisfy our later requirements for mobile ETC2/ASTC formats, mips,
alpha-mask data formats or cross-platform texture streaming by itself.
