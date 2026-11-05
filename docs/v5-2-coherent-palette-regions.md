# V5.2 — Coherent palette regions (art experiment)

This experiment replaces V5.1's independently hashed **per-hexel**
palette selection and CPU vertex averaging with a single continuous
**world-space** material color field. V4 remains unchanged, and the
same `--rock-v5-gallery` scene is used for comparisons.

## Run on Windows

From the repo, with your modified Cargo.lock left alone:

```cmd
git fetch origin experiment/v5-coherent-palette-regions
git switch experiment/v5-coherent-palette-regions
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

If Git refuses to switch because of local modifications, do not reset.
Stop and inspect the conflicting files.

## What changed

- Each of granite, shale and basalt has **five palette endpoints**,
  with deliberate warm, cool, dark and weathered variations.
- A deterministic integer-coordinate hash chooses a palette endpoint
  at each point on a **5-metre 2D anchor grid**.
- Cubic smoothstep bilinear blending mixes nearby palette endpoint
  multipliers. Every fragment of a hexel surface samples the same
  coordinate field. A hexel boundary is not a palette boundary.
- This eliminates the previous mesh-vertex tint averaging, including
  the triangular interpolation artifacts we suspected in V5.1.
- The quieter grayscale grain and continuous 3D structure pass are
  retained. No simulation or host-rock data is changed.
- There is no blending across unrelated rocks, air or water in this
  experiment. Rocks are still isolated gallery pads.

**Important:** the palette field is 2D in X/Z so it stays consistent across
neighboring cells and vertical walls; rock bedding/fractures remain 3D.
A future production implementation must use stable geological coordinates
and canonical torus identities, not these local gallery positions.

## F5 test modes

Press F5 to cycle:

1. **Normal material**: quiet rock plus palette field, structure, accents.
2. **Raw palette-region IDs**: five intentionally vivid colors show which
   region endpoints are selected. This diagnostic is NOT final artwork.
3. **Blended palette weights**: visualizes the *smooth field* between the
   selected endpoints. Again, this is diagnostic, not final artwork.

F6 UV checker still overrides the material display. Keep F6 **off**
when evaluating F5.

The first pad for each rock (01/05/09) remains the base-only neutral
control in the F5 diagnostic. Others display palette regions.

## Pass / fail

- **PASS continuity:** the color is smooth within/across hex cells, with no
  triangle-shaped tint borders, hard one-hex patches, or bright rim halos.
- **PASS identity:** granite is warmer than shale, basalt remains darker;
  palette variation stays subtle enough not to look painted.
- **PASS diagnostic:** F5 mode 2 shows distinct region colors; mode 3
  shows smooth transitions; mode 1 returns to rock colors.
- **FAIL:** unexpected color seams at chunk boundaries or around
  different side orientations, abrupt flashes on moving camera, or
  a severe framerate regression.

## Limitations

- Only three material families are implemented. Expand to 23 only once
  the color scale and region method work in-game.
- This is a visual color system, not geological host blending or ore
  simulation. Palette blending does not create rock species or change ore.
- The prototype does not yet provide a physically correct 3D region field
  through a vertical stack, and 5 metres is a tunable starting scale.
- No filtered procedural noise MIP or measured GPU shader cost yet.
- Repo changes were inspected and committed, but local native execution
  and GPU validation require a running test.
