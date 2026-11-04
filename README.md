# Hexel Survival Game

Native Rust/wgpu first-person prototype for a planned hexagonal-prism
survival game. **Current scope:** exactly one bounded 16 × 16 × 16
hexel chunk (4,096 addressable cells), stepped demo terrain,
camera/movement, gravity, collision, visible-face meshing and a
small material/overlay **test gallery**. This is **not** world generation.

**World-scale groundwork:** [the streaming-first development plan](docs/world-scale-foundation.md)
and a headless [deterministic terrain sampler](src/generated_terrain.rs)
can now generate arbitrary Q/R/Y chunks on demand for tests, but they
are **not connected to the interactive renderer or world streaming**.
The 1,024 km × 1,024 km torus remains the selected target.

The player is 1⅔ m tall. Each regular hexel is 1 m across its
opposing flats and 1 m high, with six rectangular sides, a top hexagon
and a bottom hexagon. These are prototype dimensions; the final
toroidal grid translations and elevation mapping are still undecided.

**Not implemented:** digging/building, save/load, chunk streaming,
toroidal movement, real caves/mineral deposits, flora spawning,
finite-water physics, climate, ecology and planet-scale generation.

## Run and inspect

For an **opt-in 200-chunk static material benchmark** (the one-chunk
gallery remains the default):

```powershell
cargo run --release --bin material-baker -- bake
cargo run --release -- --benchmark-200
```

Press **F3** for periodic CPU frame/submitted-chunk statistics and
**F4** to toggle nearby-chunk culling against all 200 resident
chunks. Keep the camera fixed when comparing **F8/F11**. These
statistics include vsync and are **not GPU shader timings**.
See [benchmark test notes](docs/benchmark-200-chunks.md).



Requires [Rust](https://www.rust-lang.org/tools/install) and a working
graphics adapter supported by wgpu. Launch the native desktop app:

```sh
cargo run --release
```

It opens a 1280 × 800 window. Click the window if needed to capture
the mouse.

| Control | Function |
| --- | --- |
| W / A / S / D, mouse | Movement and looking |
| Space / Left Ctrl / Left Shift | Jump / sprint / sneak |
| Esc / left click | Release / recapture cursor (Esc closes F1 menu first) |
| F1 | Clickable developer diagnostics menu; F1 / Esc closes it |
| F2 | Bottom-running log; Ctrl+C or COPY LOG copies the complete session; F2 / Esc closes |
| F3 / F4 | Frame stats / nearby-chunk culling (F4 available in 200-chunk mode) |
| F5 | Toggle FIFO V-sync vs truly unlocked Immediate when adapter supports it, benchmark scene only |
| F6 | Toggle hex-face UV checker |
| F7 | Cycle checker scale (64 / 128 / 256 texels per metre) |
| F8 | P0 versus three unapproved v2 candidate rock/ore previews |
| F9 | Diagnostic dry/wet/moss/frost/mixed states in F8 gallery |
| F10 | Diagnostic native-copper glints ON/OFF in F8 gallery |
| F11 | Compare authored-color copper vs grayscale source + shader palette, F8 gallery only |
| F12 | Cycle six original texture masks plus P2.4 WORLD VEIN (continuous world-space 3D preview; requires F8 + F11) |
| P | Cycle Copper / Hematite / Moonsilver display palettes (requires F8 + F11) |

The [F1 diagnostics menu](docs/diagnostics-menu.md) displays current
F3–F12 and P values, including F5 presentation; clicking its rows triggers
the **same code path** as the hotkeys. [Benchmark instrumentation](docs/benchmark-instrumentation.md)
adds frame-time percentiles and optional asynchronous GPU pass timings.
[Benchmark history and console](docs/benchmark-history-console.md)
adds rolling 30-second/trial averages, startup CPU/RAM/GPU/scene
metrics, exposed-face/hexel counts and a copyable `target/benchmark-session.log`. The menu is an unpaused debug overlay: close it while
recording benchmarks.

For F8–F10 preview candidates, first validate/bake the new material
catalog if no current cache exists:

```sh
cargo run --bin material-baker -- validate
cargo run --release --bin material-baker -- bake
```

This creates an ignored `target/materials-v2/` cache. The default P0
gallery is built into the executable and retains BC7 GPU compression
with CPU RGBA fallback. The separate F8 preview does **not** prove
general-purpose v2 runtime material loading.

The demo has a raised sample strip for looking at soil/grass cover,
granite, shale and the same ore-only copper artwork over either host.
These are **display fixtures**, not geological deposits or mineable
resources. The three new granite/shale/copper PNGs are candidates,
**not approved production artwork**.

## Tests

```sh
cargo fmt --all -- --check
cargo test --all-targets
python3 scripts/validate_material_catalog.py
python3 scripts/validate_flora_registry.py
```

GitHub Actions validates the code and catalogs; native graphics/art
approval still requires actual first-person review.

## Architecture in brief

- `src/world.rs`: Q/R/Y cells and a bounded deterministic demo chunk.
- `src/player.rs`: movement, collision, camera and bounded demo behavior.
- `src/mesh.rs`: exposed prism faces; meshes are disposable derived data.
- `src/render.rs` and WGSL: native first-person rendering.
- `src/geography.rs`: periodic **rectangular** geographic indexing;
  it does not establish a valid wrapped hexel lattice.
- `src/material_contract.rs`: renderer-independent substrate/cover/ore
  distinctions; not a saved gameplay mineral inventory.
- `src/materials_v2.rs`: startup material-cache validation and a narrow
  F8 preview, not a generalized production GPU material dispatcher.

Selected long-term direction: a finite 1,024 km × 1,024 km
flat-coordinate toroidal geography with deterministic synthetic
tectonics, climate, geology, editable 3D hexels, persistent finite
resource stock, and water conservation across global/regional/local
simulation scales. **Those systems are mostly plans, not present
runtime behavior.**

The adjacent
[Hexel Game Engine](https://github.com/sdhaight1-hash/Hexel-Game-Engine)
and [Voxel Game Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine)
are reference implementations/authoring projects; do not import
their older spherical-world assumption or treat their asset-editor
state as live survival-game state.

## Documentation — start here

- [Documentation map and topic ownership](docs/README.md): navigate
  canonical art, architecture, roadmaps, milestone reports and history.
- [Canonical style guide](assets/STYLE_GUIDE.md): the visual direction
  and art-approval standard only.
- [Core engineering roadmap](docs/roadmap.md) and
  [material/overlay roadmap](docs/material-overlay-roadmap.md).
- [P2.4 world-space vein prototype](docs/p2-4-worldspace-mineral-vein.md): seventh F12 option with 3D position-based coverage across cap/side seams, pending native visual/GPU approval.
- [Ore-pattern family gallery](docs/ore-pattern-family-gallery.md): five original grayscale coverage masks, three pigment ramps, F12/P art controls.
- [F11 grayscale copper comparison](docs/p2-3-grayscale-copper-gallery.md): original A/B art fixture (not a new material format).
- [Material contract](docs/material-authoring-contract.md),
  [P1 catalog and baker](docs/p1-material-pipeline.md), and
  [P2.2 reactive preview](docs/p2-2-reactive-surfaces.md).
- [World generation](docs/world-generation.md),
  [integration boundaries](docs/integration.md),
  [decisions](docs/decisions.md) and
  [flora registry/asset handoff](docs/flora-to-assets.md).

**Current research:** the headless [space-colonization skeleton](src/space_colonization.rs)
and [technical study](docs/space-colonization-study.md) were merged via
[PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30). The module is **not integrated into
gameplay, geology, caves, materials or water**. A continuous world-space
overlay proof remains planned separately, without requiring SCA.

For visual contributions, use [visual development workflow](docs/visual-contribution-guide.md), then review in the real game.
