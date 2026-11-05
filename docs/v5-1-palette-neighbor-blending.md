# V5.1 — Five-palette neighbor-blended rock coloration

**Status:** Opt-in gallery experiment. This does not modify worldgen, ore content,
geological formation identity, or any gameplay simulation state.

## Why this version exists

V5 made rocks quieter but color variation was barely visible. V5.1 keeps
the existing procedural grayscale base and continuous 3D structures, while
providing **five hand-scoped palette variants per tested rock**, assigned
deterministically per solid hexel.

Each palette is a three-channel tint of the rock-specific base dark/light
gradient; the grayscale grain remains quiet. For this first comparison, the
base-only pad deliberately retains the untinted control.

## Gallery

Run:
```cmd
git fetch origin experiment/v5-palette-neighbor-blending
git switch experiment/v5-palette-neighbor-blending
cargo test --bin hexel-survival-game rock_palette
cargo run --release -- --rock-v5-gallery
```

No new PNGs needed. Pads remain:
- 01-04 granite: control, palette, structure, accent
- 05-08 shale: control, palette, structure, accent
- 09-12 basalt: control, palette, structure, accent

The palette is enabled for stages B, C and D. The normal view now uses 1.8x palette deviation from neutral (an artistic diagnostic tuning, not production approval).

## F5: verify the implementation visually

Press **F5** in the running V5 gallery to display exaggerated palette
interpolation instead of the rock albedo. Neutral gray is a tint of 1.0;
stronger RGB patches show the palette signal. F5 toggles it back to the
normal material appearance. This is a diagnostic, **not a proposed art style**.

- Pads **01, 05, 09** (base-only) should be neutral gray.
- Pads **02–04, 06–08, 10–12** should clearly show color patches that blend.
- Adjacent solid cells of the same pad must not show a hard color seam.
- Empty gaps and walkway should not inherit the palette.
- F6 enables the UV checker independently; turn it off to view the F5 color test.

If the F5 diagnostic produces no visible changes, inspect the branch, the
vertex-color path and the shader uniform before tuning colors again.

## Neighbor rules

At the **center** of a hex cap, use the own-cell palette. At a **hex corner**
average up to three touching cells, and at each vertical Y boundary average
up to two touching Y layers. The CPU mesh only includes neighbors when:

1. The neighbor actually contains a solid block.
2. The neighbor has the exact same gallery material ID.
3. Both cells are from the same gallery pad.

Air and empty spaces never contribute. Water is not modeled in this gallery:
it is not a candidate for same-rock blending. Real contact-driven
rock-to-rock blending is deliberately not implemented yet.

The interpolated vertex colors produce smooth color transitions **within**
surfaces without adding texture reads or extra shader noise.
This only changes the V5 preview. V4 and P0 are not tinted by this module.

## What to inspect

1. Compare pad 01 vs 02 (and 05/06, 09/10). Are color changes visible
   without producing checkerboard-looking individual cells?
2. Inspect **corners on exposed top faces** and **horizontal joins between
   stacked side faces**. Look for triangular interpolation artifacts.
3. Inspect an exterior corner and the two-hex air gap. No pale halo should
   leak out of the rock edge, and adjacent pads must not mix colors.
4. Compare stages B and D; if D is still too busy, reduce structural
   and accent strength instead of weakening the palette.
5. F6 enables procedural checker. Any line persisting with checker on is
   probably unrelated to the palette colors.
6. Check the same surfaces at 1m, 3m and 8m; note shimmer and pattern aliasing.

## Limits / next steps

- This is **vertex-interpolated color**, not a physically modeled mineral
  alteration simulation, and not arbitrary per-pixel neighbor lookup.
- It handles same-rock transitions. Inter-rock contacts need geological
  contact data to decide sharp versus gradational boundaries.
- On the future toroidal world, palette hashes must use canonical global
  identity, not the fixture's local Q/R coordinates.
- First-pixel shading still uses the existing simple procedural noise;
  it has no filtered procedural mip chain yet.
- Added Rust unit tests cover deterministic palette selection, air-edge
  isolation, horizontal shared corners, vertical boundaries and controls.
- GPU execution / Windows native build and perceptual quality are unverified
  until the local test runs.
