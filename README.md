# Hexel Survival Game

Native Rust/wgpu first-person prototype for a planned hexagonal-prism
survival game. **Default scope:** one bounded 16 × 16 × 16 hexel
material/gallery fixture. **Opt-in scope:** a moving section streamer
with deterministic heightfield terrain, player traversal and incremental
GPU chunk meshes. Neither mode is a finished global world generator.

**World-scale work:** [The streaming-first plan](docs/world-scale-foundation.md)
now includes a headless [seeded terrain generator](src/generated_terrain.rs)
for arbitrary Q/R/Y sections, including far-away positions. The
[opt-in moving streamer](docs/p3-2-streamed-world.md) now uses that source
for live local terrain; distant LOD, persistence, approved torus mapping
and high-precision global movement are **not implemented**. The selected
geographic target remains 1,024 km × 1,024 km.

The player is 1⅔ m tall. Each regular hexel is 1 m across its
opposing flats and 1 m high, with six rectangular sides, a top hexagon
and a bottom hexagon. These are prototype dimensions; the final
toroidal grid translations and elevation mapping are still undecided.

**Not implemented:** digging/building, save/load, production-grade async
streaming/LOD, toroidal movement, real caves/mineral deposits, flora spawning,
finite-water physics, climate, ecology and planet-scale generation.

## Run and inspect

Requires [Rust](https://www.rust-lang.org/tools/install) and a working
graphics adapter supported by wgpu. Launch the native desktop app:

```sh
cargo run --release
```

To explore the first live moving world instead of the bounded gallery:

```sh
cargo run --release -- --streamed-world
```

The streamer keeps 95 nearby Q/R/Y sections resident and prints CPU
chunk-generation/mesh-upload counters every 3 seconds. This is an
**unbounded heightfield test**, not wrapped full-world geography;
see [native traversal checklist](docs/p3-2-streamed-world.md). With F3,
the same session log reports `Is_Moving` and `Is_Looking`, streamed
event-loop intervals, renderer CPU stage timing, and four-way mouse/look
comparisons. These are **CPU measurements, not GPU frame timestamps**;
see [render timing](docs/p3-render-cpu-stage-timing.md).
The full F2 session is written to `target/benchmark-session.log`.

It opens a 1280 × 800 window. Click the window if needed to capture
the mouse.

| Control | Function |
| --- | --- |
| W / A / S / D, mouse | Movement and looking |
| Space / Left Ctrl / Left Shift | Jump / sprint / sneak |
| Esc / left click | Release / recapture cursor; Esc also closes F1/F2 |
| F1 | Clickable developer diagnostics menu |
| F2 | In-game session console; Ctrl+C or COPY copies the full session on Windows |
| F3 | Toggle streamed-world frame, render-CPU and movement/look timing reports |
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

### Opt-in section-count streaming budget (native Windows)

The existing `--streamed-world` path loads all required sections at each
section boundary. To compare bounded per-update CPU generation and GPU
mesh/upload work while keeping the initial 95-section bootstrap:

```sh
cargo run --release -- --streamed-world --present-mode mailbox --fps-cap 60 --stream-budget 1
```

`--stream-budget` accepts 1, 2 or 4 new sections per update and is **off
by default**. CPU residency stays at 95 once initially populated; old
sections are replaced incrementally and the log reports the remaining
target-section backlog. Missing cells still sample deterministic source
terrain for collision and neighbor faces. This is a **section-count**
budget, not a strict millisecond guarantee; a single mesh can still
exceed the desired frame interval. See the [pacing evidence and budget
test plan](docs/p3-pacing-results-and-streaming-budget.md).

### Opt-in presentation synchronization A/B (native Windows)

The default remains FIFO with desired maximum frame latency 2. To test
supported alternatives without changing the default:

```sh
cargo run --release -- --streamed-world --present-mode mailbox
cargo run --release -- --streamed-world --present-mode immediate
cargo run --release -- --streamed-world --present-mode fifo --frame-latency 1
```

Accepted modes: `fifo`, `fifo-relaxed`, `mailbox`, `immediate`;
frame latency: 1–3. `immediate` may tear. Unsupported modes fall back
to FIFO, and the startup `PRESENTATION:` line records requested and
effective settings. Follow the [controlled A/B protocol](docs/p3-presentation-mode-ab.md);
the initial Windows results have not yet established a preferred mode.

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

- `src/world.rs`: Q/R/Y cells, bounded demo and opt-in resident chunk window.
- `src/generated_terrain.rs`: deterministic base data for loaded and unloaded sections.
- `src/hex_torus_candidate.rs`: test-only period/precision candidate, not active world wrapping.
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

**Current research:** the headless [space-colonization skeleton](src/space_colonization.rs)
and [technical study](docs/space-colonization-study.md) were merged via
[PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30). The module is **not integrated into
gameplay, geology, caves, materials or water**. A continuous world-space
overlay proof remains planned separately, without requiring SCA.

For visual contributions, use [visual development workflow](docs/visual-contribution-guide.md), then review in the real game.
