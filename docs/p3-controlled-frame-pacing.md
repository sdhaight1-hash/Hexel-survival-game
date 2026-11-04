# P3 controlled frame pacing — native experiment

**Status:** opt-in CPU/event-loop pacing diagnostic. Does not modify game content, streaming layout, GPU shaders or default FIFO/latency-2/uncapped behavior.

## Why

On the RTX 2060 Vulkan Windows test, FIFO latency 2 showed recurring ~50 ms CPU `get_current_texture()` calls; FIFO latency 1 ~33 ms. Mailbox and Immediate eliminated those waits, but the existing Poll loop issued around **3,900 draw calls per second**, without corresponding visible improvement confirmed by the player. CPU callback/draw rates are not display refresh rates.

## Launch matrix

```sh
cargo run --release -- --streamed-world
cargo run --release -- --streamed-world --present-mode mailbox
cargo run --release -- --streamed-world --present-mode mailbox --fps-cap 60
cargo run --release -- --streamed-world --present-mode mailbox --fps-cap 120
```

`--fps-cap` accepts only **60** or **120**; omitting it keeps the existing uncapped `ControlFlow::Poll` behavior. The cap works with any supported presentation mode; unsupported presentation modes retain their existing FIFO fallback. No cap is selected by default. A cap is a **redraw attempt target**, not guaranteed monitor presentation and not a GPU timestamp.

In capped mode, winit uses `ControlFlow::WaitUntil` with a monotonic deadline; early events do not advance the world, request a new redraw, or busy-spin. A late deadline is skipped rather than replaying a burst of catch-up frames. The main update and render paths are unchanged when a scheduled tick occurs. A frame includes time for synchronous chunk generation/meshing; those spikes can delay a deadline independently of swapchain acquisition.

## F2/F3 evidence

The startup `PRESENTATION:` line logs effective mode, `WaitUntil` versus `Poll`, and FPS cap. `PACING:` startup describes the configured cap. With F3 enabled on the streamed world, three-second `PACING:` reports give the successful draw count, achieved **CPU-side draws/s**, p95 interval **between successful draws**, interval sample count and the number of significantly late pacing ticks. Existing `STREAM FRAME:` reports remain update-callback interval metrics, not GPU frame time. `RENDER CPU:` and `LOOK CPU:` remain CPU-call-duration diagnostics. In uncapped mode the pacing report still counts successful draws without imposing a deadline.

Do not call these numbers displayed FPS. On Windows/DWM the monitor's actual scanout, input-to-photon latency, tearing, power consumption and GPU utilization are **not directly measured** by these logs. A deadline can be late when winit or the OS wakes late; precise cadence is best-effort.

## Manual test protocol

Keep resolution, world seed, camera workload, graphics settings and F3 on; close F1/F2 during acquisition. For each mode measure ~15 seconds each at idle, pan-only, W-only, W+pan. Copy full F2 log and record subjective smoothness, artifacts, input lag, fans and crashes. Repeat FIFO baseline at the end. Compare: `PRESENTATION:` effective settings, `PACING:` achieved CPU draws/s and successful-draw interval p95, `STREAM FRAME:` long update intervals, `RENDER CPU:` draw/acquire delays, `LOOK CPU:` samples and nonzero stream-update CPU peaks.

**Acceptance:** caps should materially reduce Mailbox's previously observed ~3,900 draw calls/s toward 60/120 without a new busy loop, with stable world/input behavior. A timing result alone does **not** establish improved perceived smoothness or justify a new production default. Keep FIFO/latency-2/uncapped until a native result supports changing it.

## Automated checks

Unit tests verify default-uncapped behavior, valid/invalid cap arguments, deadline gating, missed-deadline behavior and per-report reset. CI validates format, source catalogs, material baking and all-target Rust tests. Native Windows performance and input remain unverified until recorded.
