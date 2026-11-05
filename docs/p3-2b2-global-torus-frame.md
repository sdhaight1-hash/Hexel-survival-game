# P3.2B-2: Global torus frame foundation (incremental)

This change adds a globally addressable `TorusFrame` which does not embed
the northeast test origin in global-world identity. A global `i64` axial
origin, a small local `f32` player frame, deterministic canonical IDs,
and a chunk-aligned rebasing operation are represented independently.

## Invariants
- Every local cell maps to raw global `i64` axial coordinates, then to the
  existing candidate period translations.
- Period indices and canonical identity do not depend on current frame.
- At spawn anywhere (including negative and far positions), local coordinates
  remain bounded.
- Rebase preserves raw/canonical identity without changing world terrain.
- A returned shift requires a **coordinated runtime** cache/mesh rebuild.
  CPU sections, GPU buffers, collision and HUD must not use different frames.

## Relationship to the playable northeast experiment
The user has a local v2 runtime patch applied on Windows and has validated
18 directional axis crossings, including two corner transitions, without
visible terrain holes or obvious hiccups. That patch is not yet on `main`.
This PR only lands the foundation and its unit tests; it does **not** activate
runtime rebasing or unrestricted torus travel.

## Remaining integration gates
1. Bring the validated northeast runtime patch into the review branch.
2. Make the periodic world source own `TorusFrame` rather than a constant NE offset.
3. Add a coordinated full-resident rebuild after each rebase; update camera and
   debug guides in the same frame, preserve player velocity.
4. Add arbitrary global spawn flags and exercise east/west/north/south corners.
5. Run CI and native Windows visual/collision validation.

Keep saved coordinate versioning, edits, physics precision and geographic north
metric as explicit open design decisions. Regular-hex physical north period is
about 11.56m shorter than the nominal 1,024km square geography.
