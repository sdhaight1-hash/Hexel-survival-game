# P3 CPU rendering-stage timing diagnostic

**Status:** opt-in F3 diagnostic on `--streamed-world`, added after the `Is_Moving` baseline. CPU measurement only; **not a GPU profiler or a fix for the ~50 ms event-loop interval pattern**.

## Why

Native Windows samples show `about_to_wait` callback interval p95 near 50–51 ms both idle and moving, with no streaming updates in many of those windows. Movement sometimes causes additional single-update synchronous generation/meshing work up to ~38 ms. We must measure actual renderer CPU call durations before guessing that chunk generation or the GPU is responsible.

## Output

Every ~3 seconds **while F3 is enabled**, the existing `STREAM:` and `STREAM FRAME:` lines are followed by:

```text
RENDER CPU: N successful draws; draw p95 moving M ms (Nm), idle I ms (Ni); peak draw total T ms; independent CPU peaks prepare P, acquire A, encode E, submit S, present V ms (not GPU timings).
```

- `moving` / `idle` use the same captured W/A/S/D/Space **input-intent** flag as the stream frame metrics; not actual player velocity.
- `draw total`: wall-clock CPU duration of the `Renderer::draw` function on successful calls.
- `prepare`: overlay cache changes, camera calculation, uniform buffer write.
- `acquire`: blocking CPU duration of `surface.get_current_texture()`. Can reflect presentation pacing/driver waits, but **not GPU execution time**.
- `encode`: surface view, command encoder, render pass and `encoder.finish()`.
- `submit`: CPU duration of `queue.submit()`. May block on driver or resource scheduling.
- `present`: CPU duration of calling `frame.present()`. CPU call time **does not** mean a frame reached the display.
- Stage peaks are **independent maxima across draws**, so they may not sum to peak `draw total`. Both draw p95 populations include sample counts. Zero sample group p95 is a sentinel, not measured zero latency.
- Samples are collected for the current three-second interval and reset on report whether F3 is enabled or disabled, retaining bounded memory. Failed/outdated surface draws do not contribute a successful draw.
- Full result is recorded in the existing `target/benchmark-session.log` and F2 clipboard workflow.
- The streamer's deterministic terrain, 95-section residency, gameplay physics, existing P0/F8 material appearance and renderer's submission strategy are **unchanged**.

## Mouse-pan experiment (Is_Looking)

The new `Is_Looking` flag is **relative mouse movement**, not mouse capture or character motion. Only captured `DeviceEvent::MouseMotion` contributes; the absolute `|dx| + |dy|` from all mouse events since the previous update is sampled, and at least 1.0 raw mouse-delta unit marks that update as looking. Movement while F1/F2 is open or capture is released does not count. This threshold is for noise suppression, not a rotation-speed estimate. The last sampled flag is shown on the three-second `STREAM:` line; it is **only the last update**, not a label for the entire three seconds.

With F3, each successful `Renderer::draw` is categorized by the latest movement-input state and mouse-look state, giving **four exclusive groups** in `LOOK CPU:`:

| Group | Keyboard movement input | Relative mouse input |
| --- | --- | --- |
| idle+still | None | None |
| idle+look | None | Panning |
| move+still | W/A/S/D or Space | None |
| move+look | W/A/S/D or Space | Panning |

Each group reports sample count, draw CPU p95 and **surface acquisition CPU p95**. The report also gives looking vs no-look draw p95. The samples are reset every reporting window and the full record is copied by F2. Empty-group p95 is printed as 0.00 by the current diagnostic convention; **that means no data, not zero latency**. Mouse motion is accumulated from event callbacks and sampled at the update callback, so exact event-to-draw alignment is approximate. Changes in camera view or GPU work can still affect surface acquisition, but this comparison alone cannot establish causality or measure actual GPU time.

## Test

Use the same machine/window resolution and F3 enabled:
1. Stand still for at least four windows, with F1/F2 closed.
2. Walk and sprint over chunk boundaries; note `STREAM:` exchanges, `RENDER CPU` stage peaks, moving/idle draw p95 and `STREAM FRAME` p95.
3. Stop moving; compare with the original idle baseline.
4. Open F1/F2 briefly to assess how debug geometry changes timings, but don't blend those UI-on samples into a gameplay-only comparison.
5. Recheck default gallery without `--streamed-world`, F1/F2 controls, and Windows clipboard.
6. CI: `cargo fmt --all -- --check` and `cargo test --all-targets`.

**Next decision:** If acquire/present/submit CPU calls account for the recurring interval spike, investigate wgpu presentation mode, event-loop timing and driver synchronization *before* redesigning chunk streaming. If draw CPU times are short but callback intervals stay ~50 ms, instrument callback timestamps and redraw scheduling separately. If synchronous mesh updates cause isolated stalls, evaluate a per-frame work budget in a separate implementation milestone.
