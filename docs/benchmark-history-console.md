# F2 bottom console, benchmark history and machine/scene census

**Scope:** diagnostic improvements on top of the 200-chunk benchmark,
F1 menu and F5 GPU instrumentation. **No source art, world generation,
resource inventory, gameplay UI or game simulation changes.**

## Startup report

At game startup, `target/benchmark-session.log` records:

- OS/platform and architecture, processor model (Windows registry
  where available, with an environment fallback), logical processor
  count, installed/available **physical RAM in GiB**.
- Selected wgpu adapter name/type/vendor/device, GPU backend and
  driver/version string, optional BC7, timestamp-query and Immediate
  presentation support. This is **not a VRAM capacity or usage claim**.
- Window's physical **render viewport** width/height, swapchain
  format and presentation setting.
- 200 resident chunk allocations and total cell slots, meshed
  **solid hexels**, meshed **exposed hexels**, top/bottom **cap faces**,
  lateral **side faces**, total exposed faces, generated triangles
  and vertices, GPU vertex-buffer bytes and CPU mesh build/upload
  time.

A **solid hexel** is a world cell containing a solid block. An
**exposed hexel** has one or more faces whose neighbor is air/absent,
according to the existing world-aware mesh code. A cap is one hex
with six triangles and a side is a quad with two triangles. Geometry
is counted during meshing, not by extra world scans each frame.

## Controls

- **F1** opens the existing diagnostics/settings panel, including
  an **F2 LOG CONSOLE** row.
- **F2** toggles a bottom-docked, scroll-free live console and
  releases the mouse. The viewport keeps rendering behind it.
  **F2 or Esc** closes and returns mouse-look.
- **Ctrl+C** while F2 is open, or click **COPY LOG CTRL+C**, copies
  the **complete current session** to the Windows clipboard. Paste
  directly into the ChatGPT conversation. The existing console
  window remains fully usable.
- The complete log is also saved continuously to
  **`target/benchmark-session.log`**, so it survives closing the
  game. The on-screen overlay retains only the latest 250 source
  log lines and shows the newest wrapped rows. Copy is from
  the **complete file**, not that bounded display.
- **F3** toggles continuous benchmark samples. One compact
  **BENCH** line is logged every ~3 seconds. Each cycle also
  emits **AVG ROLLING 30S** (time-weighted across recent complete
  samples, actual measured span shown) and **AVG TRIAL**
  (cumulative, frame/sample-weighted mean since F3 was enabled
  or the last benchmark setting changed).
- On a material/culling/presentation/overlay setting change, a
  **TRIAL FINAL** mean is logged under the *old* mode label,
  followed by a clean new trial. F3 OFF also finalizes.
  Opening/closing F1/F2 splits trials to avoid treating menu
  rendering as comparable to unobstructed gameplay.

## How to interpret the output

**BENCH** reports completed-frame wall FPS; frame-interval
p50/p95/p99; CPU draw/acquire/present time; the **sampled GPU
render-pass** mean where supported; submitted/total chunks,
**submitted meshed-exposed hexels**, submitted caps/sides/faces,
triangles and vertices. Each resident chunk currently issues one
draw call when submitted.

**AVG ROLLING 30S** and **AVG TRIAL** report *weighted* wall
FPS, average CPU draw/present and average **available GPU pass
samples** over the interval; they do **not** pretend that a GPU
sample exists for every frame. If timestamps are unsupported
or absent, GPU data is explicitly N/A.

The submitted geometry metrics are exact for the selected chunk
meshes but **NOT actual camera-frustum-visible, occlusion-visible or
rasterized hexels/faces**. Distance culling only rejects whole
chunks outside its radius. GPU overdraw, fragment/texture
bandwidth, draw-call cost, dynamic mesh changes, 1% lows over
multiple minutes, actual resident VRAM and true GPU frame latency
remain future experiments; this implementation does not invent
these values.

**Benchmarking protocol:** choose `--benchmark-200`, F5
Immediate/unlocked where available, F8 ON, F9 Dry, F10 constant
and F6 checker OFF. Stand still at one viewpoint; enable F3
and close F1/F2 for several 30-second windows per condition.
Compare F4 culling and F11 material modes separately. Open F2
*after* the timing interval to copy the captured session and
avoid measuring the overlay itself.

## Source of evidence

User's RTX 2060 native Windows trace from the preceding F5
milestone confirmed **Immediate** and **GPU timestamps**
available, around **1,700 FPS** wall and **0.30 ms sampled
GPU render-pass** in many F8/F11 samples with ~132 submitted
chunks. Those values are device- and scene-specific, not fixed
test thresholds. A later segment varied between roughly
1,455–1,784 FPS and ~0.26–0.42 ms GPU pass, which is why
longer averages and frame-time distributions are useful.

## First native Windows evidence — 2026-11-03

The project owner pasted a complete copy from the F2 logger, confirming
that the **Windows clipboard workflow works on actual hardware**.
This verifies copy/paste and log persistence up to the moment of
copying; it does not by itself verify mouse hitbox positions or
visual/layout quality. F2 reported the successful copy event.

**Machine / viewport from this session:**

| Field | Observed |
| --- | --- |
| OS / architecture | Windows x86_64 |
| CPU | Intel Core i5-9400F @ 2.90 GHz, 6 logical threads |
| RAM | 15.9 GiB physical, 9.0 GiB available at startup |
| GPU | NVIDIA GeForce RTX 2060, Vulkan backend |
| Driver / features | NVIDIA 610.88; BC7, timestamps and Immediate supported |
| Render viewport | 1280×800, Bgra8UnormSrgb |
| Scene | 200 resident chunks, 819,200 cell slots; 293,226 solid hexels |
| Meshing | 106,050 exposed hexels; 102,400 exposed caps + 19,576 sides |
| Mesh triangles / vertices | **653,552** / **1,960,656** |
| Terrain vertex buffers | 67.3 MiB |
| CPU mesh build/upload | 81.0 ms in this session |

The face accounting reconciles exactly: 102,400 × 6 +
19,576 × 2 = **653,552 triangles**, times three vertices =
**1,960,656**. At the recorded nearby-culling position:
132/200 submitted chunks, **69,268** submitted exposed hexels,
67,584 caps + 10,835 sides = 78,419 faces, **427,174**
triangles and **1,281,522** vertices.

**Measured unlocked trial summaries (F1 was OPEN throughout):**

| Mode | Length | Wall FPS | CPU draw/present | Sampled GPU world pass |
| --- | ---: | ---: | ---: | ---: |
| P0 | 21.3 s | 2,028.0 | 0.442 ms | 0.266 ms (85 samples) |
| F8 authored full-color copper | 9.1 s | 1,925.0 | 0.470 ms | 0.301 ms (37 samples) |
| F11 grayscale copper, Dry | 13.0 s | 1,910.1 | 0.471 ms | 0.302 ms (52 samples) |
| F11 grayscale copper, later Dry | 18.6 s | 1,888.9 | 0.478 ms | 0.305 ms (74 samples) |

The copper F8/F11 GPU numbers differ by around **0.001 ms**
(one microsecond) in the short comparable first pair; this is
not strong evidence of a real shader regression. P0's GPU pass
is around **0.035 ms** below F8's, but this comparison changes
art/texture paths. FPS differences may also involve scheduling,
CPU frame work and GPU clocks. The F2-open segment subsequently
ran about 1,755–1,794 FPS with ~0.311–0.322 ms sampled GPU pass,
but that is **not a controlled F2-vs-F1 isolated UI-cost test**.

**Important limitations and resulting small fix:**
- The source log begins `F1 diagnostics menu: OPEN` and does not
  close F1 before the P0, F8 or F11 trials. Every BENCH sample
  correctly reports `UI OPEN`; none qualifies as a *closed UI*
  renderer-only benchmark.
- Despite a longer-term averaging implementation, the completed
  trials here are all **shorter than 30 seconds**. Rolling values
  were accumulating correctly but never contained a full
  continuous 30-second window.
- The first implementation labeled all F11 runs identically even
  when F9 moved through Wet/Moss/Frost/Combined, and `TRIAL FINAL`
  omitted UI status. The next logging refinement now captures
  **F4 culling, F9 preset, F10 glints, F6 checker, F7 density,
  and F1/F2/closed state** in both sample and final-trial labels.
  This is a *new code change* that still needs its own CI/native
  verification, not a property of the supplied log.
- The captured log cannot establish actual **camera-visible** faces
  or GPU pixel overdraw, and no new VRAM usage claim is made.

**Next clean comparison:** F1/F2 closed, F3 running, stationary
camera, F5 Immediate, F8 ON, F9 Dry, F10 held constant, F6 OFF,
F4 constant. Gather at least **30 seconds each** for F11 OFF/ON.
No need to re-test startup CPU/GPU reporting or Windows F2 copy
unless the new branch changes those capabilities.

## Acceptance

1. Both one-chunk default and 200-chunk benchmark launch.
2. Startup system/adapter/scene counts are accurate on native
   Windows and no unsupported GPU VRAM claim is shown.
3. F2/Esc/copy click and Ctrl+C work while F1/F3/F4/F5/F8–F11
   remain functional; clipboard content is the **full session**.
4. Console HUD wraps and fits its bottom area on the 1280×800
   default viewport, and reflows after resize. It does not mutate
   world/texture/render source data.
5. F3 emits BENCH + weighted rolling ~30s + cumulative trial.
   Toggle F4/F11; confirm previous-trial final lines are labeled
   correctly and different settings do not pollute each other.
6. Submitted face/triangle/vertex counts agree with the
   mesher's exact top/side emission ratios.
7. Normal gameplay still defaults to FIFO and diagnostics OFF.
   CI passing alone does not constitute native mouse/clipboard
   approval.

The subsequent [ore-pattern family study](ore-pattern-family-gallery.md)
adds explicit F12 pattern and P palette identities to the benchmark
trial labels so unlike material variants cannot be silently mixed.

See [F5 GPU benchmarking](benchmark-instrumentation.md),
[200-chunk scene](benchmark-200-chunks.md),
and [F1 diagnostic panel](diagnostics-menu.md).
