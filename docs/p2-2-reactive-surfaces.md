# P2.2 — Reactive surface and metallic-ore preview

**Status:** First *gallery-only* implementation. This is not a new survival
simulation, terrain resource generator, weather system, ore-grade model, or
general-purpose physically based renderer. The current v2 art remains
`candidate` and the default P0 material array is unchanged.

## Purpose

A hexel is not a single baked appearance. Four different responsibilities
may affect its visible face while its durable substrate identity remains
unchanged:

1. **Substrate:** granite, shale, soil, wood and other durable material IDs.
2. **Embedded matter:** an ore or mineral occurrence in its actual host,
   driven by geological state rather than inferred from a texture.
3. **Biological/seasonal cover:** moss, lichen, grass, leaf litter, snow.
4. **Surface condition and localized marks:** wetness, frost, oxidation,
   ash, soot, tracks, cracks and damage.

These are conceptual roles, not four GPU passes. More masks are not
automatically better. The existing game material contract remains
authoritative for substrate/cover/ore ownership.

## What this milestone actually renders

P2.2 extends **only the existing F8 candidate gallery** on granite and
shale; the same rock materials remain usable with or without copper.

| Property | First-pass implementation | Deliberate limitation |
| --- | --- | --- |
| Native copper | Ore-only alpha preserved within F8 composite-layer alpha; warm directional highlight and rare world-anchored view/light glints | Only copper; stylized specular, not full metal PBR; no actual deposit quantity |
| Wetness | Independent scalar; shaded rock darkening and wider directional specular response | Fixed gallery key light; not rain, puddles or actual roughness maps |
| Moss / lichen | Separate patch coverage controlled by a scalar; muted olive coloring | 2D pigment, not organisms, cavity-aware placement or geometry |
| Frost | Separate patch coverage, stronger on upward-facing surfaces | Thin visual dusting, not snow height, ice physics or seasonal ecology |
| Combined | All three environmental conditions can be nonzero concurrently | Diagnostic only, not a plausible automatically simulated climate |

For copper the shader reads **mask coverage**, not just copper color.
The mask follows the ore-only overlay and is reusable on both rock hosts.
Moss and frost diminish exposed metal and suppress glints where the ore is
covered. The shader does not glow in the dark, reveal hidden ore or assign
metallic behavior to every mineral. A nonmetallic mineral such as quartz
will need a dielectric response, not copper's metal treatment.

The **F8 preview array** repurposes its ore composite **alpha** as a
metallic-coverage mask. The color output remains opaque and the P0 array
is untouched. This is a compact proof of concept, **not** a final material
format specification or a generic persisted alpha-channel rule.

## Manual test on Windows

From the project root, after updating `main`:

```powershell
cargo run --release --bin material-baker -- bake
cargo run --release
```

Keep **F6 OFF**, then use:

- **F8:** switch between original P0 textures and the candidate v2 view.
- **F9:** cycle **Dry → Wet → Moss / lichen → Frost → Combined → Dry**.
  These are deterministic display presets, not actual weather/ecology.
- **F10:** toggle native-copper metallic highlights/glints independently.
  The copper remains visible through its albedo when glints are off.

F9 and F10 require a valid baked F8 candidate cache. P0 and the F6 UV
checker intentionally ignore these effects. No new gameplay keys or
save-state fields are created.

### Acceptance checks

1. With F8 on and F9 dry, compare copper **with F10 on/off** from at
   least two viewing angles. Glints must vary with light/view, be localized
   to the copper (not adjacent granite/shale), and never become full-bright
   emissive outlines or constant sparkle noise.
2. At ~1 m, ~3 m and ~8 m, compare granite, shale and both copper hosts.
   Host-rock structure must remain recognizable; native copper needs a
   clear mineral signature even with F10 off.
3. Cycle all F9 modes: wet rock looks subtly darker/smoother, moss stays
   patchy and earthy, and frost is thin and upward-biased. In Combined,
   cover must hide part of the copper highlight rather than layering
   metallic glare over moss or frost.
4. Toggle F6 and F8. The checker and original P0 gallery must look
   unchanged. Reload without a v2 bake and verify P0 remains available.
5. Get neutral-light native Windows screenshots before approving either
   the style or the effect intensities. Shader tests are **not** a GPU
   screenshot or proof of visual quality.

## Code boundaries and future work

- `src/surface_layers.rs` is a small **render-independent** state/preset
  contract. It does not store a biome, harvestable resource, simulation tick
  or transient texture-layer index. Real climate and ecology systems will
  eventually supply state through dedicated simulation boundaries.
- `src/materials_v2.rs` preserves copper alpha as **coverage only** for
  F8 preview; the host and ore material IDs remain separate. The default
  P0 compressed material pack and all original legacy assets are unchanged.
- `src/world_shader.wgsl` is a narrow shading experiment. No new
  transparent render pass, extra vegetation card, texture atlas or
  global frame animation is required.

**Next overlay candidates, not implemented here:** weathering/oxidation
and salt deposits; soot/ash/fire damage; scuffs, mining marks and tracks
(which may warrant decals); structure wear/patina; flowing water or
substantial snow/ice (which may need geometry). Grass tufts and thick
moss remain separate vegetation geometry. Extend the library only after
the first four effects pass the in-game art-review standard in
`assets/STYLE_GUIDE.md`.

**Current approval decision:** prototype capability delivered; **art and
lighting quality still require first-person visual review**.
