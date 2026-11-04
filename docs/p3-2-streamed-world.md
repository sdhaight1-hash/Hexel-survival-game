# P3.2A — first traversable moving hexel streamer

**Status:** opt-in native prototype under `feature/p3-2-moving-chunk-streamer`.
The old material-demo scene remains the default and unchanged, including
its deliberate single-chunk collision boundary. This mode is **not yet**
the finished 1,024 km × 1,024 km toroidal world.

## Launch

From the existing repository directory:

```cmd
cargo run --release -- --streamed-world
```

Without `--streamed-world` the program still launches the known P0/F8
one-chunk gallery, preserving material-acceptance comparisons.

## Implemented end-to-end

1. **World is unbounded test terrain, chunks are resident data.**
   `PrototypeTerrain(seed=42)` samples elevation and block identity from
   absolute signed axial `CellPos`. `World::streamed(42)` exposes this
   source even for cells whose 16³ section is **not** resident.
2. **Moving Q/R/Y window.** `World::stream_around(player.feet)` keeps a
   hexagonal radius-2 set of **19 horizontal section columns**, and
   **five Y sections** (`center.y ± 2`) per column: at most
   **95 resident 16³ sections**. Repeated calls inside the same section
   are no-ops. Added sections generate deterministically, while
   removed sections are evicted from CPU memory. Sections are generated
   nearest-first, in a stable order.
3. **Missing neighbor ≠ air.** `World::get(CellPos)` first checks
   resident blocks, then falls back to the *same deterministic terrain
   source* for unresident cells. The existing mesh surface culling
   consequently sees correct solid terrain on the far side of loaded
   boundaries, rather than rendering false exposed chunk walls.
   When gameplay edits arrive, this query path must also apply
   persistent edits before the base world.
4. **Incremental GPU mesh management.** The renderer now owns a map of
   section GPU vertex buffers. At initialization it uploads only
   nonempty resident meshes. When a section is added it meshes and
   uploads only that section; when evicted it removes its GPU buffer.
   No whole-world or all-resident remesh occurs every frame.
   Fully empty / fully enclosed sections need no vertex buffer.
5. **Traversable collision.** The original `Player::step` movement
   is preserved. In streamed mode, surface height and occupancy come
   from deterministic base terrain even if a section hasn't been
   loaded yet; the demo-edge collision barrier no longer applies.
   It is still a **solid heightfield only** (no caves, overhangs,
   liquids, mining edits or placement).
6. **Observability.** Every three seconds in this mode, `STREAM`
   console lines report resident sections, nonempty GPU chunk meshes,
   submitted vertices, generated/evicted counts, newly created GPU
   mesh buffers, staged vertex-upload MiB, CPU chunk-generation ms
   and CPU mesh/create-buffer ms. These are **not GPU timestamps**.
   Existing F6/F7 UV and F8/F9/F10 material toggles are unchanged.

## Explicit hard boundaries

- **No torus wrap:** The candidate integer axial torus from
  `src/hex_torus_candidate.rs` is test-only. The physical-versus
  geographic 11.56-m period discrepancy still needs approval;
  the streamer does not apply any independent Q/R modulo.
- **No global f32 precision guarantee:** `Player`, `CellPos::center`
  and the current camera/vertex shader use global `f32` metres.
  This is suitable for the local traversal smoke test, not credible
  1-m gameplay geometry a thousand kilometres away.
  Camera-relative rendering and high-precision player/world
  coordinate ownership are separate required milestones before
  wide-world travel.
- **No distant LOD:** A radius-2 chunk resident window may show a
  nearby hard edge where detailed meshes end. Later surface-only
  mid/far LOD must derive from the same elevation/geographic source,
  not another independently seeded world.
- **No persistence:** Unmodified terrain regenerates identically;
  player block edits, mining, placed structures and entity data
  aren't implemented/saved yet. The game does not claim that
  depletion survives unload/restart.
- **No async workers:** CPU section generation and GPU buffer
  creation still run synchronously on the event thread. After the
  initial 95-section bootstrap, the current streamer defaults to
  **one newly resident section per update** (configurable by
  `--stream-budget 1|2|4|unbounded`), substantially reducing the
  observed worst individual CPU streaming spike. This is **not** a
  strict millisecond deadline or background worker pool; refer to the
  [native A/B and hardening report](p3-pacing-results-and-streaming-budget.md).
- **No advanced culling:** All nonempty resident GPU buffers are
  submitted for rendering, not frustum- or occlusion-culled.
  The 95-section resident cap is a **memory** bound, not a claim
  about actual on-screen hexels.

## Acceptance checklist

1. Launch default with `cargo run --release`: same one-chunk
   P0/F8 gallery, boundary behavior, controls and material preview.
2. Launch `--streamed-world`: startup says
   `STREAMED WORLD: 95 resident sections`. Walk across multiple
   horizontal chunk boundaries without hitting an invisible
   chunk wall; the STREAM counts remain bounded at 95.
3. Move around for at least a minute, then return to the start.
   Terrain elevation and textures must match, with no missing
   face seams; record a **native screenshot** if visible artifacts
   occur. Observe a few `+/-` chunk counters after movement.
4. Jump/climb where ground elevation changes. The player must not
   fall through an unloaded chunk or get stuck on a synthetic
   streaming boundary.
5. **Native log needed:** paste 10–20 lines of STREAM output
   covering stillness, crossing several sections and returning.
   Those numbers are CPU activity / vertex-buffer sizes, not GPU time.
6. Benchmark/art validation remains in the separate gallery and
   F2/F3 instrumentation PR chain; no user action is needed
   on those branches to prove this streamer.

## Next slice after native approval

- Camera-relative rendering, high-precision player coordinates
  and a chosen world-manifest/torus mapping.
- Optional background worker queue and an explicit pending/ready
  state for generated and uploaded sections, if validated native
  profiling shows a need beyond the current one-section-per-tick
  synchronous budget; preserve collision-prioritized source sampling.
- Mid/far seed-derived terrain LOD, frustum rejection,
  incremental invalidation and persistent edited blocks.

This PR is a first **playable movement and streaming** experiment,
not a full-scale planet-generation or ecology milestone.
