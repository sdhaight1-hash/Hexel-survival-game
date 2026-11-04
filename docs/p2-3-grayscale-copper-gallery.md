# P2.3 precursor — grayscale copper shader comparison (F11)

**Implemented scope:** a small **art / GPU shader A/B experiment** in the
existing 16³ first-person material gallery. No material-family registry,
season renderer, world-space vein continuity, new mineral deposits, ore
inventory, save/load, worldgen, BC4 path, runtime v2 material dispatcher or
approved production art is created by this slice.

## Purpose

Before generalizing grayscale patterns into rock, ore, bark, leaf and
seasonal palette families, compare **one actual native copper mask**
colored in the shader against the existing full-color v2 copper
candidate. Both remain on **granite and shale**, using exactly the same
ore-only source footprint and no fabricated mineral occurrences.

The existing F8 preview was already CPU-compositing a transparent
colored copper source over two different host rocks. The new
experiment preserves that path and adds one shared grayscale source
layer, which is sampled **once for either host** when F11 is on.

## What runs

- `src/materials_v2.rs` extracts grayscale tonal detail from the
  copper-only RGBA source using linear-light luminance weights,
  re-encodes its RGB as sRGB gray for the existing gallery texture,
  and **preserves alpha independently** as mineral coverage.
- The optional F8 gallery is its original 12 RGBA layers plus a
  **thirteenth shared copper C/G source** when the candidate is valid.
  No v2 schema revision, CPU texture-pack format change, new GPU
  bind group or production asset IDs were introduced.
- `src/world_shader.wgsl` uses the original copper composite layers
  when F11 is off. When F11 is on, it samples the grayscale copper
  and the host rock (granite or shale) separately and applies a
  small **three-stop copper-only linear-light palette ramp**.
- The shader continues to apply diagnostic F9 conditions and F10
  glints to the resulting appearance. Its source alpha remains
  the copper coverage used to suppress glints under frost/moss.
- F8 off returns the original embedded P0 gallery and resets F11.
  F6 UV checker continues to override the appearance.

**This palette is a hardcoded comparison fixture, not the general
palette-ramp schema or shader system described by draft research
PR #32.** If the new mode looks worse than authored color, do not
generalize it prematurely.

## Run

In the project root, bake the existing v2 candidate cache once, then
launch the first-person demo:

```powershell
cargo run --release --bin material-baker -- bake
cargo run --release
```

Controls:

- **F8:** original embedded P0 vs v2 candidate gallery.
- **F11 (only with F8 on and native copper available):** toggle
  **full-color authored copper** vs **shared grayscale copper
  + shader palette** over both granite and shale.
- **F10:** copper metallic glints on/off in either F11 mode.
- **F9:** Dry / Wet / Moss / Frost / Mixed; compare cover occlusion.
- **F6/F7:** original physical UV-checker diagnostics.

If no valid v2 source/bake exists, F8 and F11 remain unavailable
and P0 continues working. The extra grayscale layer is **RGBA8
sRGB** because that is the actual format supported by the current
narrow preview; it is **not BC4**, a compressed packing claim,
a streaming texture array or new production format.

## Acceptance review

Compare the same copper on both rock hosts at about **1m / 3m /
8m**, in neutral light, shadow and moving first-person views.
Toggle F11 repeatedly, test F10 OFF (pigment still identifies
copper), ON (localized highlights only) and all F9 covers.
Look for loss of warm mineral variation, muddy intermediate values,
host bleaching, false glints, block-edge repeats or obvious
luminance bias caused by converting from a previously colored source.

Record measurements before approving future packing:
- 1 extra uncompressed 256² RGBA8 gallery layer = **256 KiB**
  before mips; it is allocated only when compatible copper exists.
- One **conditional** extra host sample on F11 copper faces
  (with explicit gradients), not on unrelated P0 rock or foliage.
- Compare GPU frame time/samples and resident texture bytes on the
  actual device. No performance win is claimed yet.
- Rust tests check tone/coverage independence and deterministic
  grayscale conversion, while the existing Naga test checks WGSL
  parsing and semantic validation. CI is not visual acceptance.

## Future work deliberately excluded

Keep the broader family/schema/seasonal architecture in
[draft PR #32](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/32),
the [material/overlay roadmap](material-overlay-roadmap.md)
and [overlay architecture](overlay-visual-architecture.md).

A later slice can:
1. Author **neutral grayscale source art** rather than extracting
   luminance from an already-colored copper image.
2. Evaluate a few curated grayscale pattern variants with
   deterministic region/feature selection.
3. Add a second mineral with distinct morphology and its own
   palette/finish, **if** visual quality justifies shared patterns.
4. Prove truly connected world-space ore geometry across caps/sides
   before attaching optional SCA skeletons.
5. Independently create a foliage phase/keyframe slider with one
   approved plant asset. A tree/season renderer does not exist in
   this gallery and should not be implied by F11.
