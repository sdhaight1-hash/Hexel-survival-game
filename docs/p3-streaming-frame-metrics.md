# P3 streaming frame-pacing baseline (diagnostics-only slice)

**Status:** opt-in diagnostic instrumentation on the P3.2 moving-world prototype. No chunk scheduling, generation ordering, geometry, residency radius, collision or GPU rendering algorithm is changed by this slice. This is **not** the P3.3 persistence/correctness milestone already defined in [world-scale foundation](world-scale-foundation.md); rename or sequence any later work explicitly.

## Launch

```cmd
cargo run --release -- --streamed-world
```

Only streamed mode prints these lines; the one-chunk material gallery remains untouched. Every ~3 seconds, existing `STREAM:` totals are followed by:

```text
STREAM FRAME: N update intervals; interval p50/p95/p99 A/B/C ms; >16.7 ms X >33.3 ms Y; K stream updates; worst CPU generation G ms, mesh/buffer M ms, combined S ms (single update).
```

**Interpretation:** intervals are the elapsed wall-clock time between consecutive `about_to_wait` callbacks; they include event loop scheduling/redraw behavior. They are **not** isolated renderer CPU durations, GPU frame timestamps, actual monitor presentation cadence or guaranteed end-to-end frame latency. On a paused/unfocused/occluded window, interval percentiles can be misleading. The three-second totals and per-update peaks are distinct measures. The latter capture CPU work executed synchronously in `stream_around` and `sync_stream`, **not GPU execution**. Nearest-rank p50/p95/p99 are computed separately for each reporting window. The sample buffer resets every report; there is no unbounded history.

## Reproducible comparison

Keep same GPU, driver, window size, resolution, release build, terrain seed and benchmark duration. Capture representative 3-second lines for:

1. Idle 15 seconds after opening streamed mode.
2. Steady WASD walking across multiple section boundaries.
3. Sprint (left Ctrl) across the same route.
4. Reverse direction and revisit loaded terrain.
5. Repeat while crossing diagonal or Y-section boundaries if practical.

Log **resident sections, +/− exchanges, GPU meshes, uploaded MiB, frame-interval p95/p99, >33.3 ms count, and worst single-update generation/mesh/buffer time**. Note observed visible holes, terrain popping, or freezing separately. This is a starting diagnosis; it does not prove whether a spike was CPU, GPU, driver, or OS scheduling.

## Decision gate

- If worst *single-update* mesh/buffer CPU time dominates, first evaluate spreading work across frames or off-thread mesh construction with bounded upload budgets. Do not replace the hex-prism mesher with a cubic greedy mesher without an A/B geometry and shader-quality test.
- If generation dominates, use bounded generation queues and prioritization rather than blindly increasing resident radius.
- If interval spikes occur without corresponding CPU peaks, further instrument render work/GPU/present before changing generation.
- Preserve deterministic cell identity, unloaded-neighbor fallback, and correct collision while any queue is pending. A loaded working set smaller than its desired window must never redefine unloaded terrain as air.

### Verification

Run `cargo test --all-targets` and `cargo fmt --all -- --check` in CI or locally. The unit tests cover per-update peak accounting, window percentile/reset behavior, and ignoring invalid intervals. **Native observation on the target machine is still required** to claim improvement or diagnose a hitch.
