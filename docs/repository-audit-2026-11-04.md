# Hexel Survival — repository architecture, documentation and branch audit

**Audit date:** 2026-11-04 (America/Los_Angeles). **Baseline:** `main` after PR #52 (presentation-mode A/B), before this audit PR. **Scope:** remote GitHub tree, branch ancestry, all open PRs and issues, Markdown local-link resolution, targeted Rust/WGSL architecture review and standard CI. **Not performed:** local Windows GPU execution, a native benchmark A/B run, exhaustive proof of runtime equivalence for every old branch, screenshot approval, security/dependency advisory scan or deletion of non-merged work.

**Executive finding:** The actively used code is modest, but the Git history and status documents are substantially more complicated than the runtime. The main risks are (1) stale descriptions of the now-running moving streamer and debug/benchmark tools, (2) eight older stacked PRs based on pre-streamer code, (3) treating historical/squash-divergent branches as safely mergeable or deletable, and (4) overinterpreting CPU surface-acquisition waits as GPU frame time. Avoid an engine-wide rewrite; preserve known-good defaults and finish native presentation A/B testing.

## Inventory

- **220 tracked files**, **24 directory entries**, roughly **62.5 MB** of tracked file content (Git object sizes, not Git pack size).
- **27 Rust source files**, **3 WGSL shaders**, **37 Markdown files**, 2 Python validators. `src/` ~267 KB; `docs/` ~225 KB.
- **136 asset files** (~34.2 MB) and **17 test screenshots** (~27.8 MB). Images are review artifacts/source art; large binaries are not evidence of redundant code. No original assets were deleted.
- **43 remote branches** including `main`; of the other 42, **20 have no commits ahead of `main`** and **22 have divergent history**. An ahead count after a squash merge does **not** prove unique functionality.
- **8 open draft PRs (#32–39)**, all in the older benchmark/material/ore stack; **9 open non-PR issues** (#2, #3, #4, #9, #10, #11, #13, #14, #46).
- GitHub Actions `.github/workflows/rust.yml`: `cargo fmt --all -- --check`, material catalog and flora validators, v2 registry validation, candidate bake, then `cargo test --all-targets`. This does not exercise Windows swapchain behavior, clipboard, input, or visual quality.
- Checked local Markdown link targets in **all 37 repository Markdown files** at this baseline. No missing tracked-relative Markdown links were detected by the static link scan. This does not verify external URLs or image rendering.

## Verified current runtime (separate from planned features)

- Default: single bounded 16³ first-person gallery, P0 embedded materials, candidate F8 gallery with F9/F10 visual toggles.
- Opt-in `--streamed-world`: deterministic seeded local moving Q/R/Y working set, **95 resident sections**, incremental disposable GPU meshes. No production 1,024 km torus, persisted edits, approved hex-lattice seam mapping, distant LOD, caves or finite resource simulation.
- F1 clickable diagnostics; F2 250-line overlay with full on-disk `target/benchmark-session.log`, Windows clipboard command; F3 event-loop interval p50/p95/p99, synchronous streaming CPU peak, CPU renderer stage timings, and per-sample `Is_Moving` / `Is_Looking` comparisons.
- `--present-mode fifo|fifo-relaxed|mailbox|immediate` and `--frame-latency 1|2|3` are opt-in. **Default remains FIFO / frame latency 2**. Unsupported modes fall back to FIFO and requested/effective modes are logged. Immediate can tear. No official A/B verdict yet.
- Source art: three v2 candidates, 52 identities needing art, none approved; validated authoring assets do not mean production visual approval.

## Code architecture findings

| Severity | Observation | Action / gate |
| --- | --- | --- |
| High | `src/main.rs` combines winit application state, input, native debug UI, streamed-world updates and reporting. Render submission/camera/materials/menus are also in `src/render.rs` (~958 lines). | Do **not** split solely for line count; a future split must demonstrate independent ownership, tests and unchanged input/mouse-capture behavior. |
| High | Legacy stacked PRs alter `main.rs`, renderer, shaders and world at a pre-streamer baseline; whole-head merges are unsafe despite apparently related feature labels. | Preserve unique experiments; only port narrow F4/F5 / rolling histories / ore mask work via separate, tested slices if actually needed. |
| Medium | `src/benchmark_log.rs` included dormant `BenchSample` / `BenchHistory` rolling aggregate code and tests. Neither live main nor renderer called these types. | **Removed 128 lines** of inactive implementation/tests on audit branch; retained working `ConsoleLog` and system report. Old research remains recoverable from draft PR #37 / `feature/benchmark-history-console`. This is not a claim that rolling F3 benchmarking was implemented. |
| Medium | `src/stream_metrics.rs` and `src/render_cpu_metrics.rs` retain overlapping moving/idle/looking sample arrays and a small duplicated percentile helper. | Deferred. Removing arrays or sharing a utility would introduce an aggregate-on-report change for little demonstrable gain; keep samples explicit until performance or maintenance data justifies it. |
| Medium | `src/render.rs` uses per-frame blocking `surface.get_current_texture()` and winit `ControlFlow::Poll`. Native logs show recurrent ~49–50 ms acquisition call p95 even at idle, while encoding, submission and terrain-generation CPU timings are generally far shorter in idle windows. | First compare supported presentation configurations in identical native workloads; do not assume CPU wait = GPU execution time. Only then isolate redraw/event scheduling as separate experiment. |
| Medium | Synchronous world `stream_around` + `sync_stream` can make CPU spikes during chunk-boundary crossings. | Budgeted streaming is a separate future milestone; keep correctness and unloaded-neighbor fallback; avoid conflating with idle surface acquisition waits. |
| Medium | Benchmarks include empty groups as `0.00 ms`; `Is_Moving` measures held gameplay input, and `Is_Looking` measures accumulated mouse delta. | Always interpret sample counts; a zero-sample group's p95 is not evidence of zero latency. Treat group comparisons as observational and mouse-event alignment as approximate. |
| Low | Active Rust modules are small and intentionally separated for world, player, procedural generation, authoring contract, materials, mesh, debug, and diagnostics. | No generic “reduce all files” initiative; preserve meaningful abstraction boundaries and deterministic tests. |
| Low | `Cargo.lock` may be locally modified on Windows, outside GitHub visibility. | Never issue destructive reset/clean to make a branch switch work. |

### Code reduction gate

The **128-line reduction** is confined to unreachable-at-runtime legacy benchmark aggregation. It does not change the compiled code paths that produce F2 console output, streamed F3 statistics, draw CPU timing, presentation modes, material catalogs, world state, or rendering. CI is required before merge; native mouse/clipboard/graphics regressions remain outside CI. Other possible shortening was *not* committed because it would trade explicit, verifiable behavior for fragile abstractions or altered sample aggregation.

## Documentation audit and changes

**Corrected during this audit:**

1. `README.md`: now documents F1/F2/F3, session log, movement/look CPU diagnostics and opt-in presentation modes/latency with tearing/fallback caveats.
2. `docs/README.md`: removes the false statement that world streaming is absent; links current diagnostics and this dated audit from the canonical documentation map.
3. `docs/integration.md`: distinguishes the running 95-section local streamer from the still-proposed saved/global world data contracts.
4. `docs/roadmap.md`: marks moving section-streaming prototype separately as done, without marking editing/persistence or torus seam decisions done.
5. `docs/debug-console-streamer-integration.md`: updates stale “integration candidate” status to actual PRs #47/#48; distinguishes launcher A/B flags (#52) from unported interactive F5 and historical rolling averages.
6. `AGENTS.md`: current-state guardrails for code/art agents; explicitly preserve existing render defaults and native QA.
7. This report: dated inventory, code risks, branch dispositions, issues, tests and next steps.

**Preserved intentionally:** the 31 other documentation/reference records, `assets/STYLE_GUIDE.md`, authored diagrams, user-approved design decisions and historical studies. A concept study is not disposable solely because it does not describe live runtime behavior. The style guide remains visual direction rather than a place to store CI and branch status.

## Remote branch inventory and disposition

The two categories below reflect **Git commit ancestry compared with main**, not patch-equivalence analysis.

### 20 behind-only branches — eligible for later administrative pruning

Each has `ahead_by=0`, i.e. its head is reachable from `main`. They do not need an additional merge for code reachability. **Not deleted** because branch deletion was not performed in this audit.

| Category | Branches |
| --- | --- |
| Art/style/docs already integrated | `assets/add-style-guide-infographics`, `docs/canonical-hexel-art-style-guide`, `docs/vegetation-architecture-style-guide` |
| Early core/material foundations | `feature/bc7-asset-bake-gpu-fallback`, `feature/first-playable-hexel-chunk`, `feature/hex-coordinate-contract`, `feature/p0-hexel-material-test-board`, `feature/p0-source-art-quality-refinement`, `feature/p1-material-registry-baker`, `feature/player-scale-hexel-textures` |
| World/streamed work | `feature/world-scale-core`, `feature/p3-2-moving-chunk-streamer`, `feature/p3-3-streaming-frame-metrics` |
| Fixes/integration | `fix/crosshair-depth-renderpass`, `fix/f2-console-modifier-input`, `integration/restore-debug-console-on-streamer`, `feature/benchmark-is-moving-input` |
| Foundation contracts | `foundation/face-materials-and-vegetation`, `foundation/flora-registry-v1`, `foundation/periodic-world-core` |

**Recommendation:** after owner review, these 20 can be deleted as stale branch pointers without losing commits reachable from `main`. PR/title references and historical commit links remain for provenance; no forced deletion is appropriate.

### 22 divergent branches — preserve pending patch/asset equivalence

| Group | Branches | Disposition |
| --- | --- | --- |
| Old **open draft PRs #32–39** | `docs/grayscale-material-families-study`, `feature/grayscale-copper-gallery`, `feature/benchmark-200-chunks`, `feature/diagnostics-menu`, `feature/uncapped-gpu-benchmark`, `feature/benchmark-history-console`, `feature/ore-pattern-family-gallery`, `feature/p2-4-worldspace-ore-veins` | **Retain:** contains historical rolling benchmark, 200-chunk, F4/F5, F11, grayscale ore-family and continuous ore-mask experiments. Do not merge entire heads, close PRs as wholly superseded or delete source until unique work is reviewed. |
| Squash-merged **recent diagnostics** | `feature/p3-render-cpu-stage-timings`, `feature/p3-is-looking-benchmark`, `feature/p3-presentation-mode-ab` | **Patch-equivalence review later:** core features are already in `main` via PRs #50–52; divergent commits are expected from squash merge and do not alone prove missing functionality. |
| Material/art history | `art/p2-1-geology-texture-refinement`, `feat/p2-2-reactive-material-preview`, `feat/v2-baked-material-recipes`, `feat/v2-rock-ore-gallery-preview`, `feat/v2-runtime-pack-validation` | **Retain pending comparison:** current candidate materials already exist in `main`; verify asset SHA and behavior before pruning. No production art approval is implied. |
| Research/older architecture | `docs/embed-visual-infographics`, `docs/embed-visual-style-infographics`, `docs/material-overlay-roadmap-refresh`, `docs/worldgen-reference-synthesis-2026-11`, `feature/world-scale-terrain-foundation`, `research/space-colonization-foundation` | **Retain for provenance / possible unique drafts.** Some contain extra image files or older merged research. Check unique art copyrights/originality and new document content before any deletion. |

**No branch was deleted or forcibly rebased.** A branch with a different commit after a squash merge may be functionally redundant or genuinely contain unique material. The audit deliberately did not equate ancestry with runtime equivalence. Reference screenshots/assets were not touched.

## Open pull requests / issue audit

### Eight old draft PRs

- **#32:** material variation research. Compare unique `docs/material-variation-industry-research.md` to canonical style/overlay docs. No automatic style-guide overwrite.
- **#33:** F11 grayscale copper candidate. Preserve until visual approval and decision on whether an optional gallery is worthwhile.
- **#34:** optional static 200-chunk benchmark. Keep distinct from 95-section streamer; assess whether a stand-alone perf fixture adds actionable coverage.
- **#35:** F1 UI core already recovered by #47, but old branch may have F4/F5/F11 controls. Do not close as fully redundant yet.
- **#36:** GPU timestamps / uncapped presentation / culling. Current launch-time mode A/B is **not** the original F5 implementation.
- **#37:** F2 console is recovered, but rich rolling/trial benchmark is not. Historical accumulator removed from active code because unused; original PR remains source if deliberately recovered.
- **#38:** five grayscale ore pattern families. Retain as unapproved experiment.
- **#39:** world-space ore vein visual diagnostic. Retain until independent overlay/host/stock correctness tests; not a finite ore deposit implementation.

**No old PR closed or blindly retargeted**: each is still a recoverable record of work, and the 8-PR stack has nontrivial parent dependencies.

### Nine open issues

| Issue | Decision |
| --- | --- |
| #2 — exact playable torus lattice | **Keep open.** Candidate math/tests do not constitute an approved production coordinate wrap. |
| #3 — world manifest / periodic tile IO | **Keep open.** No versioned persistent geography. |
| #4 — edit/harvest and reload | **Keep open.** No authoritative save-state proof. |
| #9 — production v2 face binding | **Keep open.** F8 candidate gallery is not stable-ID general material dispatch. |
| #10 — persistent six-stage grass instances | **Keep open.** Research types and cover fixture are not living grass gameplay. |
| #11 — ore host/continuous mineral field | **Keep open.** Old experimental PR #39 does not satisfy production acceptance or ore stock authority. |
| #13 — approved grass/tree visual assets | **Keep open.** Flora registry is research; approvals absent. |
| #14 — art epic | **Keep open.** P0/P1 engineering baseline is not P0–P7 production completion. |
| #46 — debug/benchmark stack consolidation | **Keep open, update status.** F1/F2, CPU timing, movement/look flags and opt-in present-mode A/B have now reached `main`; the original F4/F5/rolling/F11/ore experimental stack remains unresolved. |

No issue was closed simply for age, and no duplicate was proven to be fully satisfied by shipping code.

## Risk-ranked engineering backlog (do not silently turn these into features during cleanup)

1. **P0 verification:** Native Windows presentation A/B, F1/F2 hitbox and clipboard sanity check, renderer/streamer visual stability. Do not change default settings until results exist.
2. **P1 correctness:** Defined, tested hex-lattice torus seam and world-state persistence, without regressing generated-but-unloaded-neighbor collision fallback.
3. **P1 production art contract:** stable v2 face IDs, explicit source/baked/approved boundary and true ore host/remaining-stock authority; native QA for rock/ore/grass first.
4. **P2 streaming performance:** isolate CPU chunk/mesh spikes and frame work-budget options after event-loop/presentation A/B.
5. **P2 experimental triage:** review #32–39 individually; port demonstrably useful pieces only. Avoid blindly reviving 200-chunk assumptions.
6. **P3 maintenance:** once active codepaths stabilize, consider clearer lifecycle separation inside `main.rs` and `render.rs` and reducing overlapping diagnostic arrays if a test proves equivalent sampling/performance. Do not chase line count alone.
7. **P3 repository size:** screenshots/atlases account for most tracked bytes; consider an intentional asset archive / artifact policy, **not automatic deletion of style/visual QA evidence**.

## Validation matrix and limitations

| Gate | Evidence |
| --- | --- |
| Code review / inventory | GitHub `main` tracked tree, current source files, branch comparisons, open issues and PR metadata inspected. |
| Markdown relative links | All 37 Markdown files checked against tracked paths; no missing tracked-relative links detected. External web link health not established. |
| Functional reduction | Dormant `BenchSample` / `BenchHistory` and their self-tests removed; 128 lines removed from `benchmark_log.rs`. Live logging API retained. |
| Automated CI | **PASSED** on audit PR #53 (GitHub Actions Rust foundation run [#608](https://github.com/sdhaight1-hash/Hexel-survival-game/actions/runs/41827556074)): formatter, material/flora validators, v2 registry, candidate art bake and Rust all-target tests. Native Windows tests not included. |
| Native Windows GPU/UI | **Not run in audit**; cannot conclude that presentation modes improve frame pacing, F2 clipboard works on every Windows config, or visual assets are approved. |
| Branch deletion | **None performed**. Twenty behind-only branch pointers are candidates, twenty-two divergent ones require care. |

## Tomorrow's non-negotiable native test reminder

**Thursday, November 5, 2026 — Presentation A/B test.** A one-time morning reminder has been scheduled.

1. Update local `main` carefully; do **not** discard locally modified `Cargo.lock`.
2. Run `cargo run --release -- --streamed-world` for **FIFO / latency 2** baseline. Enable **F3** and leave F1/F2 closed during measurements.
3. Compare supported `--present-mode mailbox`, `--present-mode immediate` (**may tear**), optional `fifo-relaxed`, and `--frame-latency 1` / `3` for FIFO. **Read the startup `PRESENTATION:` line** to confirm each *effective* mode.
4. For each run collect at least 15 seconds each of idle/still, pan-only, movement-only and movement-plus-pan. Repeat the FIFO baseline afterward.
5. Use **F2** to copy `target/benchmark-session.log`; compare `STREAM FRAME` long-interval counts, `RENDER CPU` acquire/draw, and `LOOK CPU` samples. Record visual tearing, jitter, input responsiveness and errors separately.
6. Report outcomes before changing the default presentation mode or redraw loop.

**Decision rule:** The goal is a repeatable improvement in actual user-visible fluidity with no unacceptable tearing or instability, not merely smaller CPU acquire calls. Current evidence implicates acquisition/presentation waits but does not settle cause or optimum configuration.
