# Hexel Survival Game

Native Rust **first-person hexagonal-prism world prototype** and foundation
for the planned survival-game runtime.

**Current playable slice:** exactly one bounded **16 × 16 × 16 hexel chunk**
(4,096 addressable cells). A seeded, stepped grassy landscape is shown in a
native wgpu 3D window with depth testing, face culling and a central crosshair.
It supports walking, mouse-look, sprinting, sneaking, gravity and jumping.

The landscape is deliberately simple **demo terrain**, *not* planet generation.
The player is **1⅔ m tall**, so a 1 m-high hexel is exactly **3/5 of
character height**. The current P0 gallery derives **256px visual previews from our original
high-resolution soil, granite, shale, forest-floor and copper artwork**,
with hex-specific face UVs and grass/ore compositing. See
[P0 material gallery](docs/p0-material-gallery.md) for current controls,
and [first-person scale](docs/first-person-textures.md) for the prototype
dimensions.
No cross-chunk streaming, toroidal movement, digging/building, saves, ecology,
climate, caves, finite water or diffusion inference exists in this slice.

## Visual direction and asset standards

**Canonical reference for all future art and rendering work:**
[Hexel Survival Art Direction & Style Guide](assets/STYLE_GUIDE.md).
It reconciles Hytale-like gameplay readability, the older full-3D foliage
concept, Vintage Story-like geological/ecological depth and the user's
Luis Royo-influenced painterly atmosphere. It specifies hexel face/overlay
rules, ore recognition, grass/flora design, palette and lighting direction,
compression/performance expectations, and in-game approval checks.
**Future renderer features are described as goals, not shipped features.**
Contributors and coding assistants should read the guide before visual changes.

## Launch

Requires [Rust](https://www.rust-lang.org/tools/install) and a working graphics
driver with a backend supported by wgpu.

```sh
cargo run --release
```

Or use `cargo run` for a quicker debug build. This is a **native desktop app**,
not a browser game. It opens a 1280 × 800 window. If mouse capture fails on
your OS, click the window to try again.

### Minecraft-style first-person controls

| Input | Action |
| --- | --- |
| W / A / S / D | Walk forward / left / back / right |
| Mouse | Look around |
| Space | Jump (with gravity and landing) |
| Left Ctrl | Sprint while held |
| Left Shift | Sneak / move slowly while held |
| Esc | Release or recapture the mouse |
| Left click (when mouse is released) | Recapture mouse |
| Close window | Exit |

The chunk has **temporary, non-wrapping bounds** so you cannot wander into
unloaded geometry. These boundaries are scaffolding only. They are not the
selected final world's shape and do not imply an infinite or bounded-flat game.

## Why this implementation is structured this way

- `src/world.rs`: authoritative Q/R/Y cell identifiers, 16³ chunk storage,
  one seeded demo chunk, and world-space hex-coordinate conversion.
- `src/player.rs`: testable movement, camera direction, gravity, collision
  with demo topography and world-boundary constraints (no GPU dependency).
- `src/mesh.rs`: exposed hexagonal-prism faces with neighboring-cell checks;
  generated geometry is derived data, not the world state.
- `src/render.rs`, `src/main.rs`, WGSL shaders: window, mouse and keyboard
  input, and wgpu rendering, separate from world simulation.
- `src/geography.rs`: earlier independent 32,768² *rectangular periodic
  geography raster* index. It is **not** the hexel torus coordinate system.

This demo uses **1 metre across opposing flats × 1 metre high** hexagonal-prism
cells to align with the existing Foliage authoring output. These are the
prototype's adopted playable dimensions only; the final global hex-lattice
period translations, world mapping and geographic elevation-to-Y transform
remain unresolved. The first chunk intentionally does **not** fake torus
wrapping by applying an unvalidated Q/R modulo.

## Project boundaries

- [Hexel Game Engine](https://github.com/sdhaight1-hash/Hexel-Game-Engine):
  existing Rust/wgpu playable prototype and geometry reference. Its earlier
  **spherical** planet direction conflicts with the newer **toroidal**
  specification. Do not silently mix those systems.
- [Voxel Game Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine):
  existing Nodes, Presets, Foliage and Assets authoring app, plus the
  [world design decisions](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/procedural-world-design-decisions.md).
  Keep authorship separate from simulation authority.

## Greater world design

The selected direction remains a finite **1,024 × 1,024 km flat-coordinate
torus** with globally periodic fields, synthetic tectonics, climate, one-time
constrained terrain refinement, stored immutable geography and streamed
deterministic local terrain. Save player edits and finite resource depletion
in authoritative overlays; never regenerate harvested material from cached
base chunks. Water must be globally conserved across simulation scales.

None of these planetary systems is claimed to work in this first chunk.

## Compressed texture pipeline

The original high-resolution geology source art now gets processed **at
build time**, not every launch. An embedded **BC7** texture pack uses only
**0.75 MiB** of GPU memory for all 12 current 256px gallery layers on
adapters supporting native BC compression (versus **3 MiB** uncompressed).
The GPU decompresses BC7 automatically during sampling. Unsupported
adapters use a **one-time CPU BC7 decoder** and the ordinary RGBA8 path,
so the game remains playable without BC texture support.

The console reports the selected mode. There is no additional install or
asset-baking command: `cargo run --release` runs the offline bake as
needed through Cargo. See [compression pipeline](docs/texture-compression.md).

## P0 in-game material gallery

The 16³ demo currently includes a **temporary reference strip** with bare,
very sparse, sparse and fully grassy soil, plus granite and shale both with
and without a shared copper overlay. The P0 gallery now builds 256px visual
prototypes from the **original high-resolution soil, granite, shale, forest
floor and copper textures**, with seam reduction and independent grass
and ore overlays. Physical-metre UVs and distinct hexel face roles remain
unchanged. None of the 55 legacy art files has been overwritten or certified
production-ready.

- `F6`: toggle a checker grid for spotting stretching and face seams.
- `F7`: cycle **checker diagnostic** at 64 / 128 / 256 texels per metre.
- `F8`: switch between the default P0 gallery and the opt-in baked v2
  candidate granite, shale and copper preview (after running the v2 baker).
- `F9`: cycle independent wet/moss/frost surface states for **F8 rock faces**.
- `F10`: compare copper's directional metallic glints ON/OFF in F8 mode.
- The middle display row is one metre higher so vertical transitions can be
  inspected in first-person view.

Read [P0 gallery guide](docs/p0-material-gallery.md) and
[material format v2 proposal](docs/material-format-v2.md).
The overarching P0–P7 geology/flora texture program is [issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14).

## P1 material authoring and offline BC7 tooling

The new [P1 material pipeline](docs/p1-material-pipeline.md) inventories **all
55 legacy geology and organic-cover sources**. Three have unapproved candidate
artwork and the other 52 remain pending.
No old atlas crop has been promoted to production art. A versioned
`assets/materials/catalog.v2.json` defines stable IDs, distinct hexel
top/side/bottom material recipes and independent cover/ore overlays.

Validate the catalog with `cargo run --bin material-baker -- validate`.
Once new art has been approved for registration, bake it with
`cargo run --release --bin material-baker -- bake`; files go to
`target/materials-v2`. This tool supports **rectangular and square RGBA**
textures with appropriate physical repeat scales and emits BC7. Three **unapproved original candidate** art assets are now registered:
granite, shale, and native-copper transparent overlay. The default P0
display is unchanged; the separate F8 gallery preview shows candidates
only after a successful v2 bake. It does not yet support arbitrary v2
materials, full per-face role variation or streamed texture resources.

## P2.2 reactive surface preview

The F8 rock/ore gallery now exercises **localized copper metallic response**,
**wetness**, **moss / lichen** and **frost**, without changing saved material
identities or the default P0 scene. These are explicitly controlled preview
states, not climate or ecology simulation. Only copper is metallic; not all
ore or minerals should use its lighting response.

See [P2.2 surface states and visual QA](docs/p2-2-reactive-surfaces.md) for
shading limitations, the F9/F10 controls, layer ownership and next review gates.

## Tests and engineering roadmap

```sh
cargo fmt --all -- --check
cargo test --all-targets
```

GitHub Actions runs those checks on pushes and pull requests. Headless tests
cover data layout, coordinate round trips, determinism, movement, boundaries,
jumping and geometry sanity; they cannot certify in-person GPU/gameplay quality.

All new terrain, ore, soil-cover and grass texture work follows the
[material and vegetation contract](docs/material-authoring-contract.md). The
legacy demo art is not yet a production-ready face set. Run
`python3 scripts/validate_material_catalog.py` to verify registered assets.

Flora species, habitat research and eventual plant appearances are now tracked
separately from terrain materials. See [Flora to assets](docs/flora-to-assets.md)
and the research inventory at `assets/flora/reference_registry.v1.json`.
None of its species currently spawn in the demo. Run
`python3 scripts/validate_flora_registry.py` to check that inventory.

Read [World generation](docs/world-generation.md),
[Integration boundaries](docs/integration.md),
[Decisions](docs/decisions.md) and
[Roadmap](docs/roadmap.md) before expanding the prototype.
