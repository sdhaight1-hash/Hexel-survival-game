# P3.2B-0 — Opt-in distant spawn in the current streamed prototype

**Status: first runnable world-scale traversal slice, NOT toroidal seam traversal.** This is intentionally a small change to the existing game, not activation of P3.1A–F topology candidates.

## Usage (Windows)

The existing baseline is unchanged:

```cmd
cargo run --release -- --streamed-world --present-mode mailbox --fps-cap 60
```

Spawn in a distant **signed axial cell** inside the existing unwrapped streamed heightfield:

```cmd
cargo run --release -- --streamed-world --spawn-q 65536 --spawn-r -8192 --present-mode mailbox --fps-cap 60
```

Optionally approach the geographic target's east-end neighborhood:

```cmd
cargo run --release -- --streamed-world --spawn-q 1023984 --spawn-r 0 --present-mode mailbox --fps-cap 60
```

Both `--spawn-q` and `--spawn-r` are required and only permitted together with `--streamed-world`. Negative values work. Values outside conservative i32 headroom are rejected. **The coordinates are unwrapped**: reaching Q=1,024,000 does not teleport or wrap to Q=0, and the terrain across such a line is not guaranteed equal.

The player is placed on top of the deterministic terrain at the requested cell, **before** the unchanged synchronous 95-section bootstrap. Budget 1 remains the streamed update default; controls and F3 metrics are unchanged. Logs record requested spawn coordinates and warn that this is global-f32/unwrapped prototype terrain.

## Verified scope / omissions

Automated tests check CLI validation, default-mode preservation, distant player initialization and 95-section initial residency. The existing streamed-world integration already exercises deterministic chunk unload/reload.

This **does not** solve P3.1F's global-`f32` camera/mesh precision issue. Current world coordinates and emitted vertices still use global `f32`, so significant depth jitter, face gaps or incorrect camera transforms can become visible at large coordinates. The mode is a **diagnostic** to expose that limit, not a claim that 1,024 km navigation is production-ready.

It also does not use `PeriodicTerrainCandidate`, `CandidateCellId`, or `CandidateProjection` in gameplay; physically crossing a torus seam would be incorrect without integrating the periodic source, canonical chunk ownership, GPU/collision neighbor rules and camera-relative geometry. It is therefore deliberately **not** called `--torus-world`.

## Required next end-to-end gate

Before advertising playable 1,024 km toroidal travel, implement stable global ownership plus local-relative camera/vertex generation, select physical hex metric, then introduce a separately enabled periodic world source and seamless both-axis coordinate wrapping. Validate forward crossing/backward crossing, chunk residency, collision, neighboring mesh faces, and edits across seams. Compare native frame and visual behavior with the existing baseline.
