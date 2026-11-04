# F5 uncapped presentation and GPU timestamp benchmark

**Status:** opt-in diagnostics added on top of the existing F1 menu and
static 200-chunk fixture. No production material, terrain generation,
resource state, shader palette, world streaming, or game render-quality
preset was changed.

## Why

The first native Windows F3 captures at 200 resident chunks,
F4=132/200 and 200/200 submissions, and F8/F11 looked essentially
identical at around **60 FPS / 16.4 ms CPU draw/present**. This is
inconclusive because the normal surface uses **FIFO V-sync**, and the
CPU draw call may spend much of its time waiting for acquisition or
presentation rather than doing work.

This implementation adds two independent measurements:

1. **F5 presentation**: FIFO (V-sync, safe default) versus **Immediate**
   presentation (unlocked/tearing possible) *only* when the active
   surface lists Immediate as supported, and only in `--benchmark-200`
   mode. It never labels Mailbox or a fallback as truly unlocked.
2. **GPU render-pass timestamps**: if the chosen adapter supports
   `wgpu::Features::TIMESTAMP_QUERY`, F3 samples the world render pass
   asynchronously roughly every 250 ms. Two per-pass GPU timestamps
   are resolved into a staging buffer and read back via a callback
   channel and nonblocking `Device::poll(Maintain::Poll)`. No
   `Maintain::Wait`, GPU stalls, or synchronous readbacks are inserted
   into the normal loop. Missing hardware capabilities display N/A.

**Render-pass time is not full GPU frame latency or isolated fragment
shader time.** It includes the world draw submission, crosshair and
the F1 overlay if open, but excludes window presentation and other
CPU/event-loop work. Time values are GPU hardware deltas multiplied
by the adapter's reported timestamp period. They can vary with
driver clocks and workloads.

## Run and controls

```cmd
git fetch origin && git switch -C feature/uncapped-gpu-benchmark origin/feature/uncapped-gpu-benchmark && cargo run --release -- --benchmark-200
```

Press **F1** to open the clickable menu. It includes **F5
PRESENTATION**, showing **VSYNC**, **UNLOCKED** or **N/A**. F3 continues
to toggle 3-second benchmark output. F4/F6–F11 retain their original
behavior. The menu makes inactive F9/F10 settings explicit as
`DRY SET`, `ON SET` and similarly staged values while F8 is OFF.
When F8 is ON, the values show without `SET`.

After activating F3, close F1, hold a fixed position/camera, leave
F9 dry, F10 constant and F6 checker off, then record 3 or more
consecutive windows for each condition:

| Comparison | Hold constant | Change |
| --- | --- | --- |
| Frame pacing | Same camera, culling, materials | F5 VSYNC / UNLOCKED |
| Geometry cost | F5 UNLOCKED, same art/view | F4 132-ish / 200 submissions |
| Copper shader cost | F5 UNLOCKED, F4 fixed, F8 ON, F9 Dry, F10 unchanged | F11 full-color / grayscale |
| Material baseline | F5 UNLOCKED, F4 fixed, no checker | F8 P0 / V2 |

**F3 print columns**:
- **Wall FPS**: actual completed redraws per three-second wall clock;
  includes scheduling and present mode effects.
- **Frame p50 / p95 / p99 (ms)**: percentiles of observed intervals
  between successful completed frames. These include all CPU/GPU
  scheduling effects; they are **not GPU timestamps**.
- **CPU draw (ms)**: average time spent in `Renderer::draw`, including
  `get_current_texture()` and `present()`. Not isolated CPU render
  work.
- **GPU pass (ms, N samples)**: average of asynchronously sampled
  world-pass timestamp deltas, or pending/N/A. F3 switches profiling
  on/off; while profiling is off the renderer doesn't issue new
  timestamp queries.
- **Submitted chunks/vertices**: scene workload indicator from the
  same frame; neither frustum-culled nor GPU-visible triangles.

Toggling a diagnostic control resets the three-second logging window
so comparisons aren't mixed. A timestamp result can arrive slightly
later than the corresponding frame; discard the first interval after
a major mode change if needed. The menu itself is a real GPU overlay:
**close it for all timing windows**.

## Explicit limitations

- Immediate presentation may not be supported by a given
  Windows/driver/adapter/backend combination. N/A is correct;
  **never pretend F5 is unlocked** if it cannot switch to Immediate.
- If GPU timestamps aren't supported, F3 still reports wall FPS,
  percentiles, CPU draw/present time and submitted geometry. That
  is useful but **not** an isolated GPU comparison.
- GPU queries are **periodic samples**, not a continuous GPU timing
  history. GPU pass time is measured for a render pass including the
  optional menu if open, not the copper shader exclusively.
- V-sync remains the default on each launch. Immediate may tear
  and should not become a gameplay visual preference by accident.
- Tests/CI can compile the feature and validate percentile logic,
  but real GPU timestamp accuracy and stable Immediate presentation
  require native runs on the target Windows adapter.

The [benchmark history and console](benchmark-history-console.md)
adds 30-second/whole-trial weighted means, startup system/adapter
details, accurate meshed-exposed cell/face counts, and a Windows
copyable F2 running log. These are separate from isolated GPU
shader timing or actual camera visibility.

See [200-chunk benchmark](benchmark-200-chunks.md) and
[F1 diagnostics menu](diagnostics-menu.md).
