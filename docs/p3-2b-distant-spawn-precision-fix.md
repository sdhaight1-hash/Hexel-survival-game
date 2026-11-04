# P3.2B-0.1 — Distant-spawn hex-face precision repair

**Cause:** P3.2B-0 enabled spawning around ~1,024,000m while the renderer still constructed each hex's top/side vertices by adding 0.577m corner offsets to a **global f32 centre**. At that magnitude, mesh corners are snapped onto coarse f32 increments, opening visible holes between adjacent faces. This is a rendering-precision defect; all 95 resident CPU sections and a zero streaming backlog do **not** rule it out.

**Fix (opt-in distant-spawn path only):** The renderer now fixes a stable integer axial origin from `--spawn-q`/`--spawn-r`. Both bootstrap meshes and subsequent streamed GPU meshes use `build_chunk_mesh_relative`, computing signed `i64` axial **differences first**, then converting to a local metre-space `f32` vertex before adding hex ring corners. The camera view and fragment eye-position uniform subtract the *same* physical origin. Default demo/streamed runs (no spawn flags) preserve their original mesh and camera coordinates and material behavior.

**Explicit remaining limitation:** player motion, world surface lookup, and streamer target conversion are still global `f32`. This fixes the most visible vertex quantization but does **not** implement globally precise movement or guarantee zero jitter in camera motion/collision at 1,024km. The stable origin also assumes movement remains near the chosen spawn; a real world-scale game needs global integer/fixed ownership plus dynamic camera-relative render rebasing. No toroidal wrap is enabled, and `PrototypeTerrain` is still nonperiodic.

## Native visual verification

Run the exact reproduction after updating `main`:

```cmd
cargo run --release -- --streamed-world --spawn-q 1023984 --spawn-r 0 --present-mode mailbox --fps-cap 60
```

Compare against the screenshot with widely separated grass/soil strips and sky gaps. Expected: hex top and side faces form a continuous local terrain mesh, with no large sky-colored breaks caused by vertex precision. Move and look around to check for any remaining small jitter; this candidate does **not** claim perfect movement.

For baseline compatibility:

```cmd
cargo run --release -- --streamed-world --present-mode mailbox --fps-cap 60
```

CI validates scene construction, bounded relative mesh vertex positions, default-mode compatibility and Rust material/pacing tests. The visual claim requires the native reproduction to confirm it. This change should **not** be used as evidence that the full 1,024km torus can be traversed.
