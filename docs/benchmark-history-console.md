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

See [F5 GPU benchmarking](benchmark-instrumentation.md),
[200-chunk scene](benchmark-200-chunks.md),
and [F1 diagnostic panel](diagnostics-menu.md).
