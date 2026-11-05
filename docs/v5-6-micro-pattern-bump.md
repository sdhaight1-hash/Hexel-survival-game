# V5.6 — Per-hexel related-tone patterns and height-derived normals

V5.6 builds on V5.5's per-hexel five-palette assignment and **14cm,
solid-only edge blend**. The V5.4 geological structure and disruption
overlays remain world-space and unchanged.

## The change

Each granite, shale and basalt hexel retains ONE of its rock's five
palette families. Inside each hexel, that family now carries a small
four-tone internal pigment pattern, rather than reading as a solid tint.

- Each rock family has four tightly related RGB multipliers around neutral.
  Granite is mildly warmer/mineral-speckled, shale is directionally
  laminated, basalt is denser and darker.
- A stable per-hexel pattern seed (derived from axial q/r/y and rock type)
  selects a unique 3D internal pattern. All A/B/C/D pads of a rock use
  the same seed at the same local test coordinates.
- Two shared noise samples provide both pigment variation and a shallow
  scalar height signal. The fragment shader derives a lighting normal
  from the height gradients; there is no separate unrelated normal noise.
- Micro-pattern pigment and height fade near **actual same-rock solid
  contacts**. The existing V5.5 mesher still blends the base palette
  across those contacts. Air and gaps do not contribute to the blend.
- This is deliberately a low-contrast surface-relief illusion, not
  actual displaced geometry, mesh cracks or a new rock geology model.

## Lighting A/B test

**F4** toggles normal perturbation ONLY in this opt-in V5 gallery:
- **F4 ON (default):** directional diffuse lighting uses the
  height-derived shading normal.
- **F4 OFF:** *identical materials and directional lighting*, but the
  unperturbed geometric face normal is used. This isolates the bump
  contribution without changing the base palette, light or scene.

**F5** remains the V5.5 normal / raw palette ID / palette blend cycle.
For the artistic test, keep **F5 normal** and **F6 checker OFF**.
F4 still controls chunk culling in all other modes.

## Run on Windows

```cmd
git fetch origin experiment/v5-6-micro-pattern-bump
git switch experiment/v5-6-micro-pattern-bump
cargo test --bin hexel-survival-game rock_palette_edges
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

Stop before resetting anything if local changes conflict with the
branch switch. A modified Cargo.lock should not be discarded.

## Evaluation

1. Compare B pads (granite 02, shale 06, basalt 10) at 1m and 3m.
   The palette remains per hexel, but each cell should have very subtle
   internal tonal variation instead of a flat painted patch.
2. With F5 normal, toggle F4 while holding a camera angle near a side
   surface. The hue and macro color should NOT change; only directional
   relief cues should change. Repeat for all three rock types.
3. Inspect edges between solid cells and exterior air borders. There
   should be no abrupt fine-grain seams at shared solid contacts.
4. Compare stages C/D to ensure the existing bedding, joints, quartz,
   cooling cracks and slip traces remain world-continuous.
5. Turn camera slowly and walk: look for specular-like shimmer, jagged
   aliasing or frame drops. This adds two procedural 3D noise samples
   per V5 rock fragment and screen derivatives, on top of the V5.5
   high-vertex gallery geometry. Profile before using in production.

## Architectural limits

- This experiment does NOT turn the rock patterns into authored texture
  arrays. It uses four near-tone procedural endpoints and two seeded noise
  signals; these can later be baked to a filtered texture/height atlas
  if art direction is approved.
- At shared contact strips, unrelated micro-patterns fade toward the
  jointly blended parent palette rather than directly sampling both
  neighboring procedural patterns. This is a practical first approximation
  to 'blend the final pattern at edges', not full neighbor-pattern mixing.
- The 12-pad gallery still lacks canonical torus-coordinates for seed
  ownership. No P0/V4 or worldgen path has acquired simulation-driven
  weather, faults or displacement.
- Rust/shader source has been inspected; native shader validation and
  rendering on the target GPU are pending local test execution.
