# Integrated terrain revision V4 — seed-owned geography, hex LOD and lighting

## Scope

Opt-in executable: `cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4`.

This is a coordinated **experimental** replacement of several prototypes. It is
not a production save/schema revision and is not merged into main.

### World-generation ownership

A deterministic world seed and canonical global axial coordinate are the
single source for terrain elevation, exposed material, and underground host.
The region model uses periodic lowland/basin and plateau masks, uplift belts,
domain-warped ridge shapes, foothill variation and analytical valley
depressions. **This is not an erosion simulation** and does not promise real
drainage watersheds. Domain masks wrap on both torus translations.

A `ColumnProfile` owns the top elevation, top block and underlying host.
The section streamer computes this profile **once per horizontal hex** and
reuses it for all 16 vertical blocks. Nonresident block queries use the same
rules and skip expensive surface geology checks for air/deep cells.

Experimentally exposed blocks: Grass, Soil, Rock (granite), Shale and Gravel.
Material/atlas mappings use existing 12 P0 layers; no production artwork is
replaced. The LOD material is selected from the exact same global surface
classification, not a separate triangle slope cutoff.

### Distant rendering

- Band 0 (roughly 0–64m square): **exact hex-prism surface topology**
  constructed from global surface heights, with 6-sided caps and exposed
  soil/rock cliff-wall quads. This is seed-derived without allocating all
  nearby voxel chunks.
- Bands 1–3 (64–1152m square rings): 2/4/8m sampled heightfield with five
  interior error probes per cell; missing peaks/depressions trigger four
  triangles instead of two. Coarse/fine outer edges interpolate to aligned
  boundaries.
- LOD fragments are hidden only for full-resolution columns whose complete
  surface-bearing vertical sections are actually resident. Incomplete
  columns keep distant geometry visible instead of making empty holes.
- No disk LOD cache, persistent-edit delta, volumetric distant caves,
  overhangs or arbitrary deformable structures. **Hex/triangle seam
  watertightness is not mathematically guaranteed**. Far error cannot be
  bounded by testing only five interior probes.

### Materials and atmosphere

Original temporary high-contrast diagnostic pixel rendering for grass,
brown soil, beige gravel, gray granite and slate shale, with small pixel
grain plus coherent surface patches. This is **not Minecraft artwork** and
is intentionally not the final art direction.

A moving sun shares its phase with sky/day/night color, directional diffuse
lighting and one 1024x1024 **real shadow-map pass**, centered on the camera,
covering approximately 320m horizontally. That one near cascade is not a
Cascaded Shadow Map system; extremely distant features receive ordinary
diffuse lighting.

### Verification

Rust CI tests cover periodic identities and seam neighbors, mountain relief
and spawn relief, profile-vs-unresident block identity, sampled regional
materials, moving hex LOD anchors, finite band vertices, and WGSL Naga
validation for world, sky and shadow shaders.

CI is **not** a replacement for WGPU pipeline creation and gameplay on the
target Windows Vulkan RTX 2060.

Run separately:

```bat
git fetch origin
git switch feature/terrain-system-grand-sweep-v1
cargo test
cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4
```

Use F12 boundary markers and F2 logs. Record a slow panorama, walking toward
a distinctive ridge, crossing multiple snapped LOD anchors, and a complete
180-second sun cycle. Capture the TERRAIN V4 region survey, LOD worker time,
uploaded MiB, backlog and frame timings. Check for missing faces and seams
from both sides of the near/far boundary.

## Outstanding risks

1. One band of seed-derived hex caps/cliffs can use materially more vertices
   and CPU sampling than the prior inner square heightfield.
2. Near/far geometry remains two distinct representations; where a hex
   footprint meets a square band, intersection gaps/overdraw can still occur.
3. One bounded 1024 shadow cascade may exhibit aliasing or false self-shadow.
4. The new generation algorithm changes deterministic heights: existing
   saved terrain/edits are not compatible.
5. Regional drainage and climate remain heuristic, not physical simulation.
6. The production material atlas still lacks gravel-specific art, sediment,
   snow and full biome textures.

Keep PR draft until native geometry, GPU startup and performance acceptance.
