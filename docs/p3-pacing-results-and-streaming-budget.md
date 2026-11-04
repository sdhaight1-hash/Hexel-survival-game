# P3 CPU presentation A/B findings and streaming follow-up (2026-11-04)

**Source:** native Windows F2 logs supplied after PR #54, RTX 2060 Vulkan, i5-9400F, 95-section streamed-world workload. All rates below are **successful CPU draw calls per second**, not monitor/display scanout FPS or GPU timestamps. `Is_Moving` measures input intent and `Is_Looking` accumulated mouse motion.

## Presentation experiment outcome

| Configuration | Source observation | Interpretation |
| --- | --- | --- |
| FIFO / latency 2, uncapped | Previous trial: repeated ~50 ms CPU `get_current_texture()` waits, ~60 successful draws/s | CPU acquire wait is strongly associated with FIFO presentation/sync; it does **not** prove 50 ms gaps in monitor scanout |
| FIFO / latency 1 | Previous trial: recurring ~33 ms CPU acquire waits, ~60 draws/s | Shorter waits but still periodic blocking |
| Mailbox / uncapped | Previous trial: nearly zero acquire waits, about 3,890 draws/s | Removes measured acquire stalls but wastes CPU scheduling/render submissions; unverified displayed frames |
| Immediate / uncapped | Previous trial: nearly zero acquire waits, about 3,930 draws/s | Similar CPU behavior, with possible tearing |
| Mailbox / 120 cap | Current trial: ~112–115 draws/s; ~8.8 ms median update and ~9.2–9.4 ms p95 successful-draw interval; idle long-interval count 0 | CPU pacing works and avoids uncontrolled draw submission. Target interval is 8.33 ms; small systematic cap undershoot remains |
| Mailbox / 60 cap | Current trial: usually ~58.0–58.6 draws/s (one initial 45.6 startup window); ~17.2 ms update p50 and ~17.6–18.0 ms successful-draw interval p95 | CPU pacing works and avoids uncontrolled draw submission. Target 16.67 ms; startup window is not representative steady state |

**Visual observation:** the player reported all original presentation modes felt approximately the same and could not confidently identify tearing. There is **not enough justification to switch the production default** from FIFO latency 2 or uncapped operation.

**Diagnostic caveat:** the `>16.7 ms` counter flags most successful 60-cap intervals by design because the observed interval is ~17.2 ms; it should not be interpreted as a missed frame at this cap. Zero-sample `Is_Moving`/`Is_Looking` p95 reports of 0.00 ms do not represent measured instantaneous work.

## Specific evidence isolating streaming spikes

- At capped **120**, one movement window shows a **40.80 ms** maximum combined CPU stream update, with draw CPU p95 roughly 1.3–1.4 ms and occasional >33.3 ms **update** intervals. Multiple windows show per-update stream cost 15–40 ms.
- At capped **60**, the heaviest 3-second period reports **294 added / 294 removed chunks**, **60.75 ms aggregate CPU generation** and **169.97 ms aggregate mesh/buffer calls**, distributed over 12 stream updates; worst **single** combined stream update **34.44 ms**.
- Another capped 60 movement period shows the successful-draw interval p95 rise to **27.72 ms** while generation/meshing is active, despite otherwise stable ~17.7 ms interval p95. Streaming synchronization—not measured CPU draw/acquire work—is the leading candidate for these transient main-thread spikes.
- The original `World::stream_around()` materializes all missing CPU chunks on center movement before returning. `Renderer::sync_stream()` meshes/uploads every newly added section in the same tick. Both execute in `about_to_wait` on the same UI/game thread. CPU costs accumulate across many new sections.

**Scope decision:** implement **opt-in section-count budgeting** first, not a speculative hard millisecond deadline. A single chunk generation or GPU mesh operation is not preemptible; a time-only check after an operation cannot guarantee a frame-time maximum. Preserve the existing full-update default and the first 95-section bootstrap. Bound **additional** per-tick newly resident chunks and corresponding GPU meshing/upload count; retain outside-window residents until a replacement is ready, so the resident set stays at 95 during ordinary traversals. Existing procedural sampling must remain authoritative for unloaded neighbors/collision. Avoid graphics shader changes, threading, and async worker pools at this stage.

## Acceptance / next native test

- Explicit budget on launch, leave baseline behavior unchanged when omitted.
- New sections installed incrementally, with matched GPU removals/additions.
- No duplicate/resurrected stale sections when player changes direction before a window finishes; deterministic world revisit unchanged.
- During ordinary travel, preserve 95 resident CPU sections; queue/backlog observable in F2 logs; chunk updates per tick do not exceed the selected count.
- Unit tests exercise startup, transitions, repeated ticks and mid-transition reversal. CI passes. Native Windows test must compare moving/looking `STREAM FRAME` long intervals and single-update peaks against original baseline at identical trajectory.
- Do **not** claim this section-count budget is a strict millisecond cap or a complete fix for isolated single-mesh spikes. A future async/budgeted mesh worker design should depend on native measurements.
