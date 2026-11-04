# P3 presentation synchronization A/B diagnostic

**Status:** opt-in diagnostic on the existing native renderer. No change to the default, gameplay, chunk streamer, shaders or redraw scheduling. This is not the P3.3 persistence/correctness milestone.

## What changed

The existing render surface used `wgpu::PresentMode::Fifo` and `desired_maximum_frame_latency: 2`, with winit `ControlFlow::Poll` continuously requesting redraws in `about_to_wait`. CPU timings show repeated ~50 ms calls to `surface.get_current_texture()`, including when `Is_Moving=false` and `Is_Looking=false`. While mouse-look samples often show shorter acquire waits, this does not establish causality or actual display refresh cadence.

This slice makes the surface presentation mode and desired maximum frame latency configurable **at launch**, preserving the baseline:

```cmd
cargo run --release -- --streamed-world
cargo run --release -- --streamed-world --present-mode mailbox
cargo run --release -- --streamed-world --present-mode immediate
cargo run --release -- --streamed-world --present-mode fifo-relaxed
cargo run --release -- --streamed-world --present-mode fifo --frame-latency 1
cargo run --release -- --streamed-world --present-mode fifo --frame-latency 3
```

Default is `fifo` with `--frame-latency 2`. Accepted latencies are **1, 2, or 3**; invalid values fail before startup. Mode choices are `fifo`, `fifo-relaxed`, `mailbox`, `immediate`. **Immediate may tear; it is never selected automatically.** Modes not present in the active adapter's reported surface capabilities are replaced with **FIFO**, and that fallback is clearly logged rather than allowing surface configuration to fail or pretending the requested mode was tested.

On startup, `PRESENTATION:` is written to the existing console and `target/benchmark-session.log` with GPU adapter, requested/effective mode, supported modes, desired maximum frame latency, and a note that redraw control remains `ControlFlow::Poll`. The effective configuration is reused when resizing. Tests must verify the **effective** rather than merely requested mode.

## Protocol

Keep the same machine, driver, monitor refresh rate, window resolution, terrain seed, F3 settings, and workload for each run. Close F1/F2 while collecting measurements. For each supported mode, collect at least 15 seconds each in:
1. Idle, no mouse activity.
2. Pan only.
3. Move only.
4. Move plus pan.

Run baseline **FIFO latency 2** first and again last to detect drift. Then change only **one setting at a time**: first test each available present mode at latency 2, then if useful test FIFO latency 1 and 3. Exit and relaunch between trials so the surface configuration is unambiguous.

Compare `STREAM FRAME` interval p95/p99 and long-interval counts, `RENDER CPU` draw and acquire peaks, `LOOK CPU` four-way draw/acquire p95 and sample sizes, plus visible tearing, latency, frame jitter and any surface errors. No 0-sample group should be interpreted as measured zero latency.

**Interpretation:** `get_current_texture()` is a CPU blocking-call duration, not GPU execution or confirmed display scanout. Mailbox and immediate may trade synchronization or tearing characteristics, subject to GPU driver/OS support. A lower acquire p95 alone is not proof of smoother perceived presentation. Do not change production defaults based solely on one run or a mode that fell back.

## Follow-up decision gate

- If a supported alternate present mode eliminates recurrent long acquisitions without unacceptable artifacts, follow up with measured native A/B verification before discussing any default change.
- If long waits persist in every mode, instrument redraw scheduling / event-loop callback cadence in a *separate* experiment; don't conflate scheduling with swapchain modes.
- Keep synchronous chunk generation/meshing CPU spikes as a separate streaming-budget investigation, not a presumed cause of idle acquisition waits.

## Automated validation

CI must pass Rust formatting, material registry/bake and all-target tests. Unit tests cover default settings, valid flags, invalid arguments and unsupported-mode fallback. Native Windows rendering/presentation quality requires user observation.
