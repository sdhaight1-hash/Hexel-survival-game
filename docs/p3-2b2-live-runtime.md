# P3.2B-2 live global torus integration (experimental)

## Run
```
cargo run --release -- --torus-test-ne --present-mode mailbox --fps-cap 60
cargo run --release -- --torus-test-ne --torus-spawn-q 1023984 --torus-spawn-r 1182384 --present-mode mailbox --fps-cap 60
```

This opt-in mode uses global `i64` axial identity for periodic terrain and a
small `f32` local world frame for movement, collision, streamed 16³ sections
and mesh vertex coordinates. The existing non-torus demo and
`--streamed-world` modes retain their existing behavior.

When the player moves 512m away from local origin, a *whole-section* integer
rebase adjusts the player frame, invalidates all local resident CPU sections,
regenerates the 95-section window and rebuilds all GPU vertex buffers.
The camera remains in the same local frame as the mesh.
The F2/terminal/session log records TORUS REBASE separately from TORUS WRAP.
Wrap events compare *global period index before and after* and include
canonical coordinates, directions and counters, including corner crossings.
Red/green guide planes use the nearest period boundary relative to the frame.

**Limitations**:
- Initial whole-window rebase is intentionally expensive; not seamless large
  distance streaming yet. Profiling is required.
- No persisted edits, globally canonical section residency, caves, ocean,
  planet-scale LOD or save format. The canonical terrain sample is invariant
  even though the working-set cache is local.
- Candidate geography retains the regular-hex ≈11.56m shorter north period.
- Windows visual/collision behavior must be verified after CI; successful unit
  tests alone do not prove no visual hitches.

## Review tests
`cargo fmt --check`; `cargo test`; `cargo run --release -- --torus-test-ne`.
Try north/east reverse/corner and a 512m local displacement in multiple
directions, then check canonical identity, REBASE records, 95 resident sections
and contiguous terrain geometry. No edits of existing demo materials.
