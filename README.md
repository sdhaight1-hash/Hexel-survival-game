# Hexel Survival Game

Native Rust/wgpu first-person prototype for a planned hexagonal-prism
survival game. **Current scope:** exactly one bounded 16 × 16 × 16
hexel chunk (4,096 addressable cells), stepped demo terrain,
camera/movement, gravity, collision, visible-face meshing and a
small material/overlay **test gallery**. This is **not** world generation.

The player is 1⅔ m tall. Each regular hexel is 1 m across its
opposing flats and 1 m high, with six rectangular sides, a top hexagon
and a bottom hexagon. These are prototype dimensions; the final
toroidal grid translations and elevation mapping are still undecided.

**Not implemented:** digging/building, save/load, chunk streaming,
toroidal movement, real caves/mineral deposits, flora spawning,
finite-water physics, climate, ecology and planet-scale generation.

## Run and inspect

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
| Esc / left click | Release / recapture cursor |
| F6 | Toggle hex-face UV checker |
| F7 | Cycle checker scale (64 / 128 / 256 texels per metre) |
| F8 | P0 versus three unapproved v2 candidate rock/ore previews |
| F9 | Diagnostic dry/wet/moss/frost/mixed states in F8 gallery |
| F10 | Diagnostic native-copper glints ON/OFF in F8 gallery |

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
- [Material contract](docs/material-authoring-contract.md),
  [P1 catalog and baker](docs/p1-material-pipeline.md), and
  [P2.2 reactive preview](docs/p2-2-reactive-surfaces.md).
- [World generation](docs/world-generation.md),
  [integration boundaries](docs/integration.md),
  [decisions](docs/decisions.md) and
  [flora registry/asset handoff](docs/flora-to-assets.md).

**Current research:** the reusable space-colonization skeleton and
optional uses in geology/caves/overlays/water remain in
[draft PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30);
they are **not merged into the game**. A continuous world-space overlay
proof is planned separately, without requiring SCA.

For visual contributions, use [visual development workflow](docs/visual-contribution-guide.md), then review in the real game.
