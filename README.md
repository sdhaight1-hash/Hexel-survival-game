# Hexel Survival Game

Native Rust **first-person hexagonal-prism world prototype** and foundation
for the planned survival-game runtime.

**Current playable slice:** exactly one bounded **16 × 16 × 16 hexel chunk**
(4,096 addressable cells). A seeded, stepped grassy landscape is shown in a
native wgpu 3D window with depth testing, face culling and a central crosshair.
It supports walking, mouse-look, sprinting, sneaking, gravity and jumping.

The landscape is deliberately simple **demo terrain**, *not* planet generation.
No cross-chunk streaming, toroidal movement, digging/building, saves, ecology,
climate, caves, finite water or diffusion inference exists in this slice.

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

## Tests and engineering roadmap

```sh
cargo fmt --all -- --check
cargo test --all-targets
```

GitHub Actions runs those checks on pushes and pull requests. Headless tests
cover data layout, coordinate round trips, determinism, movement, boundaries,
jumping and geometry sanity; they cannot certify in-person GPU/gameplay quality.

Read [World generation](docs/world-generation.md),
[Integration boundaries](docs/integration.md),
[Decisions](docs/decisions.md) and
[Roadmap](docs/roadmap.md) before expanding the prototype.
