# F1/F2 debug-console recovery on the moving streamer

**Status (2026-11-04): MERGED** via [PR #47](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/47), with the F2 Ctrl-key fix in [PR #48](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/48). The F1/F2 screen-space overlay and copyable session log run on the current `main` streamed-world renderer. No 200-chunk static benchmark or experimental ore shader branches were merged.

## Expected behavior

- **F1:** clickable debug menu. F2/F3/F6–F10 are active; F4/F5/F11 show N/A/disabled because their old benchmark/ore implementations are not ported.
- **F2:** on-screen session-log console. **Ctrl+C** or the on-screen copy button copies the full session via Windows `clip.exe`. On-screen history keeps 250 lines; the full log is written to `target/benchmark-session.log`.
- **F3:** toggles additional `STREAM FRAME` interval p50/p95/p99 and CPU-streaming peaks; `RENDER CPU` draw-stage timing; and `LOOK CPU` four-category moving/panning analyses. Ordinary `STREAM:` aggregates remain logged every three seconds. These are CPU/event-loop measurements, **not GPU execution timestamps**.
- **Esc/F1/F2:** close overlays / restore mouse capture; F1 and F2 are mutually exclusive. F6–F10 work via either keyboard or the menu.
- **Dynamic streamer:** remains a 95-section moving Q/R/Y working set with unchanged chunk generation and mesh synchronization. The default one-chunk gallery is unchanged.

## Native QA

1. Launch `cargo run --release`; press F1; click F6/F7/F8/F9/F10; press F2; Ctrl+C and paste log; Esc returns to mouse look. Resize; check menu hitboxes and console copy button.
2. Launch `cargo run --release -- --streamed-world`; confirm initial 95 resident sections and moving chunk exchanges. Open F1/F2; ensure world draws behind overlay and pressing keys does not move player while it is open. Enable F3, walk and sprint for multiple boundaries; verify `STREAM FRAME` is in the F2 log/file and remains comparable to previous branch values.
3. Check focus loss and cursor recapture; the menu isn't a game pause. F4/F5/F11 are intentionally disabled.
4. Run CI formatting and all-target Rust tests. Visual/input verification needs the actual Windows device.

**Source heritage:** `src/benchmark_log.rs`, `src/debug_menu.rs`, and `src/debug_menu.wgsl` originate from the tested `feature/benchmark-history-console` branch. The renderer and event loop were adapted to current streamed-world architecture, rather than replacing it with the fixed 200-chunk renderer.

**Not yet implemented:** GPU timestamps, interactive F5 present-mode switching, F4 culling and the old rich F3 `AVG ROLLING 30S` benchmark system. Standalone **launch-time presentation-mode A/B settings** are available via [PR #52](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/52); they are not the old F5 menu feature. The remaining experimental recovery work is tracked separately in [issue #46](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/46). Recovery of those is separate and tracked in [issue #46](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/46).
