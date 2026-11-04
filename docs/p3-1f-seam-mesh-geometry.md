# P3.1F — Candidate hex-mesh seam geometry validation

**Status: headless test-only mesh geometry contract.** No change to playable renderer, collision, streaming, live world coordinates or save identities. The world target remains a 1,024 × 1,024 km geographic torus; physical topology still needs approval.

## Existing mesh facts and constraint

`src/mesh.rs` constructs each regular hex prism's six side edges using a corner radius of 1/sqrt(3), with corners at 30°, 90°, ... and axial side-neighbor ordering `[(0,1),(-1,1),(-1,0),(0,-1),(1,-1),(1,0)]`. A neighbor side and its opposite must share **both exact edge endpoints** (reversed winding), not merely terrain values or occupancy. The renderer currently derives mesh vertices from global `f32` cell centers and is **not** precision-safe at megametre offsets.

The P3.1B proposed geographic metric maps northward axial row distance to `1_024_000 / 1_182_400` metres, rather than `sqrt(3)/2` used for the current regular hex basis. If future gameplay adopts that physical metric, applying it **only to cell centers** while retaining the original unscaled corner ring creates small adjacent-face gaps and overlaps. The same affine transform must apply to **centers and corners**, and collision, interaction and raycasting geometry must be consistent.

## Headless acceptance checks

`src/seam_mesh_geometry_candidate_tests.rs` reproduces the *position* construction of the six existing mesh side edges without compiling or running GPU code. For eight representative east/west/north/south/corner seam coordinates and six neighbor directions:

1. Find the closest periodic image of the canonical neighbor.
2. Compute both side-edge endpoints locally using integer axial coordinate differences converted into `f64`.
3. Compare each edge against the opposite neighbor edge with reversed endpoints.
4. Assert coincidence under either fully **regular-hex** or fully **geographic affine** geometry.
5. Confirm a deliberately incorrect hybrid of affine centers with untransformed regular-hex corners does **not** align.
6. Check vertically neighboring candidate cells remain exactly one metre apart in Y and local `f32` deltas preserve one-metre motion at 10^12-cell absolute coordinates.

Tests ensure the metric must be applied consistently, not that an affine world is automatically correct for player collision or art authoring. No visual mesh is emitted or displayed, and no production code calls this test fixture.

## Decision and remaining end-to-end gate

**Do not activate the projection as world physics or render geometry yet.** Choose and version a single physical metric, its compatible hex-prism shape/collider and global/local coordinate conversion. Then migrate one vertical traversal slice from global `f32` positions to canonical signed-global ownership + local-relative render positions, while verifying seam crossing, section eviction/reload and unchanged terrain. That integration requires actual mesh upload, collision and player movement tests; the P3.1F tests only prove face-corner geometry and local coordinate precision at selected points.

This keeps the stable 95-section streamer and current gameplay behavior untouched.
