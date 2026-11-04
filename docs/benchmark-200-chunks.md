# Static 200-chunk visual benchmark scene

**Scope:** experimental **20 × 10 horizontal chunks × 1 vertical section**:
200 *resident* 16×16×16 hexel sections, exactly 819,200 allocated cell
slots (many air), world seed 42. This is **not** streaming,
long-distance LOD, toroidal seam correctness, persistence, authentic
ore distribution, climate, water or a complete procedural world.

This test is deliberately downstream of the optional F8/F11 copper
material gallery; it is a **repeatable render/mesh comparison fixture**.

## Why this exists

The first F11 grayscale copper comparison worked in code, but reused
the **same** copper pattern and therefore showed only slight color
changes. The original 16³ showcase was too small to expose repeated
texture tiles over a landscape or compare material shading under many
visible rock faces. User-observed art quality is still **unapproved**.

The next separate material-family experiment will introduce
**3–5 genuinely different grayscale pattern designs**, independently
selectable mineral palettes (copper, a morphologically appropriate
second mineral, and a fictional metal only if artistically justified),
and compatible response rules. *This benchmark does not claim those
pattern families exist now.*

## Run

Use the current branch, with the existing v2 cache baked once:

```powershell
cargo run --release --bin material-baker -- bake
cargo run --release -- --benchmark-200
```

The original reference baseline is still:

```powershell
cargo run --release
```

The benchmark is **opt-in** to preserve the old one-chunk visual and
physics tests. The first chunk at q=0/r=0 uses exactly the original
material_lab display strip. Surrounding chunks use simple, deterministic
low-frequency height/rock/grass patches. Rock faces use 8-cell
stable patches of granite, shale and their *display-only* copper
composites, making repeated patterns visible without pretending to
generate real ore. The terrain cells and visual selection do not
change on a rebuild.

## Controls and measurement

| Key | Action |
| --- | --- |
| F3 | Toggle a log every ~3 seconds with sampled wall FPS, average **CPU draw path** milliseconds, submitted nearby chunks and submitted vertices |
| F4 | Toggle conservative nearby-chunk culling against drawing **all resident chunks**; availability only in 200-chunk mode |
| F8 | Default P0 versus optional candidate preview (requires baked cache) |
| F11 | Shader grayscale copper versus original full-color copper, **F8 only** |
| F9 / F10 | Diagnostic wet/moss/frost and copper glint comparisons |
| F6 / F7 | UV checker/physical texel density |

**Measure responsibly.** Wait for mesh startup to complete, stand
still and use an unchanged viewpoint/graphics resolution. Run F3 for
several sampling windows with F8 on and F11 OFF, then F11 ON with
F9 dry and F10 constant. Compare same orientation and camera location.
Repeat looking toward the material-rich surrounding rocks, then with
F4 culling disabled. Keep native captures for 1m, 3m and 8m art reviews.

The renderer reports initial resident-chunk count, exposed-vertex count,
estimated GPU vertex-buffer memory and CPU mesh build/upload wall time.
The F3 FPS includes **vsync**, windowing and event-loop scheduling; draw
CPU milliseconds include surface acquisition/presentation and can block.
Neither is a **GPU timestamp, isolated shader cost, true GPU memory
profiler, or proof that one texture format is faster**.

## First native Windows observation — 2026-11-03

User-provided console log from `cargo run --release -- --benchmark-200`
on the benchmark feature branch (full hardware/GPU identification,
fixed camera pose and screen capture not supplied):

| Reported metric | Native result |
| --- | --- |
| Resident chunks | **200 / 200** |
| Startup exposed vertices | **1,960,656** |
| Resident vertex buffers | **67.3 MiB** |
| CPU mesh build/upload wall time | **71.1 ms** |
| Chunks submitted per logged sample (F4 initial nearby mode) | **132 / 200** |
| Vertices submitted per logged sample | **1,281,522** |
| P0 FPS | Generally **~60 FPS**; one **30.2 FPS** sample |
| F8 full-color FPS | Generally **~60 FPS** |
| F11 grayscale FPS | Generally **~60 FPS** |
| Reported CPU draw/present wall time | Generally **~16.4–16.6 ms** for all three |

**Interpretation:** static scene generation and the F8/F11 rendering
controls work in this native run, and the submitted counts remain
stable while the camera remains at the tested pose. There is **no
measurable shader win or regression demonstrated here**. The
application uses FIFO/V-sync and the CPU draw wall time includes
surface acquisition/present: the near-identical ~60 FPS and ~16.5 ms
numbers are presentation-limited, **not isolated GPU frame times**.
A transient 30.2 FPS sample cannot be assigned a root cause without
frame/GPU timing data.

**Missing evidence:** F4 was **not** toggled in the submitted log,
so there is no 132-vs-200-chunk comparison yet. Several intervals
also changed F9/F10/F11/F6/F7 settings during a measurement window.
No fixed-camera screenshots, independent native GPU timestamp, GPU
memory measurement or longer frametime distribution were supplied.
The repeated `F11 unavailable` messages happened with F8/P0 active,
as designed. Existing harmless Rust dead-code warnings and a locally
modified `Cargo.lock` do not indicate benchmark failure.

**Next meaningful performance gate:** run fixed-pose F3 windows with
F9=Dry, F10 fixed, F6 off and F8 on; compare F11 OFF/ON, then F4
distance culling ON/OFF, separately. For a real shader comparison,
add an **opt-in non-V-sync benchmark present mode** plus, if the
adapter supports them, GPU timestamp queries or another trusted
GPU frame profiler. Preserve normal V-sync behavior for gameplay
and report CPU vs GPU measurements distinctly. Consider true frustum
culling/mesh compression after valid measurements; these are not
required to accept the static scene itself.

## Technical bounds

- Original `World::demo(42)` remains unchanged, including P0 reference
  coordinates and local player spawn. `World::benchmark(42)` is a
  separate static world constructor.
- All 200 sections are materialized and meshed **once at startup**. Meshes
  use the same neighbor-aware exposed-face code across chunk seams.
- Each nonempty chunk has its own vertex buffer; these are **not 200
  texture copies**. They share existing material bindings and shaders.
- F4 uses intentionally conservative **horizontal distance culling**,
  with a 112m center-distance threshold; it is not frustum,
  occlusion, streaming or scene-graph LOD.
- The collision/ground lookup still assumes solid columns and one
  vertical section. It is intentionally not a cave/overhang or
  deformable-terrain physics solution.
- 200 loaded sections are not the number of simultaneously drawn
  sections: F3 reports the count actually submitted each frame.

## Verification gates

1. **Determinism:** same seed, chunk coordinates and block samples
   independent of hash-map order; origin chunk identical to old demo.
2. **Culling/mesh:** each neighboring chunk reuses world-aware
   visibility and exposes valid vertex positions; F4 changes submitted
   draw calls, never world state.
3. **Visual controls:** F8/F11/F9/F10 still work from a stationary
   viewpoint, including repeated granite/shale/copper appearances
   outside origin.
4. **Performance:** compare same fixed viewpoint, resolution, seed,
   exposed geometry and preview settings. Measure actual GPU time
   later if adding GPU timestamp queries is warranted.
5. **Retirement:** do not use this static fixture as proof of toroidal
   terrain, region ownership, water conservation, functional mining,
   infinite chunks, or actual material-family art approval.

See [F11 grayscale gallery notes](p2-3-grayscale-copper-gallery.md),
[material overlay roadmap](material-overlay-roadmap.md), and
[world-generation contracts](world-generation.md).
