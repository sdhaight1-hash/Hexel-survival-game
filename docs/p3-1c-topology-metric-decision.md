# P3.1C — Hex-lattice metric decision evidence

**Status: tested design comparison; topology still NOT approved for gameplay.** Follow-up to [P3.1B projection candidate](p3-1b-axial-geography-projection-candidate.md). The approved design objective remains a **1,024,000 m × 1,024,000 m flat geographic torus**, with playable 1-m-across-flats hexagonal prisms as a provisional geometry preference.

## Constraint calculation

The regular flat-hex basis used by the current prototype is:

```text
x(q,r) = q + r/2                         metres
z(q,r) = (sqrt(3)/2) × r                  metres
T_east = (1,024,000, 0)                  axial
T_north = (-591,200, 1,182,400)         axial
```

For an *exactly north-directed* period, its horizontal component must cancel: `q = -r/2`. Both axial components of a period are required to be divisible by 16, the currently implemented section dimension; hence `r` must be divisible by **32**. The closest legal R values around the geographic target are:

| North R | North Q | Regular-hex physical north period | Error from 1,024,000 m |
| ---: | ---: | ---: | ---: |
| 1,182,368 | -591,184 | 1,023,960.724622 m | −39.275378 m |
| **1,182,400** | **−591,200** | **1,023,988.437435 m** | **−11.562565 m** |
| 1,182,432 | −591,216 | 1,024,016.150248 m | +16.150248 m |

The Rust test enumerates chunk-aligned orthogonal northward candidates within ±3,200 R of the current period and confirms the existing period minimizes absolute length error there. The general constraint also implies nearest multiples of 32 minimize error globally since physical north length is linear and monotonic in R.

**Boundaries of this result:** no claim that the current basis is globally optimal across sheared periods, changed hex dimensions, alternative orientations or independently scaled global coordinates. The existing geographic grid (32,768² at 31.25m spacing) is separate from the playable hex lattice.

## Three engineering choices

| Choice | What stays exact | What changes / risk |
| --- | --- | --- |
| A. Regular hexes and current period | Perfect local regular 1-m-across-flats hex prisms; integer chunk-aligned seam equivalence | Physical N/S period short by 11.562565m. Geographic square coordinates need a documented scale mapping. |
| **B. Exact square geographic metric and current integer periods (recommended for further tests)** | Approved 1,024km × 1,024km geographic metric, integer seam equivalence, raster sample ownership | World-coordinate N/S pitch becomes `1,024,000 / 1,182,400`m, **~1.00001129×** regular lattice pitch. Physics/render geometry must agree if that metric is designated physical. |
| C. Change lattice basis/period constraints | Potentially different compromise | Could violate 16-cell section alignment, change metre-scale hex shape, introduce period shear or demand rework of world identifiers; no identified advantage yet. |

Recommendation **for an isolated test-only implementation**, not an irreversible gameplay decision: keep B as the working projection candidate because the geographic square dimensions are already decided and the metric difference is tiny. Prove seam-aware physics, geometry and terrain continuity first. Do not enable B just because the numerical discrepancy looks small.

## Evidence still missing to approve topology

1. A single selected and versioned transform tying global signed/fixed coordinates, physical hex dimensions, geographic metre positions and camera-relative rendering together. If an affine geographic metric is chosen, collision radii, raycasts and authored geometry must use it consistently.
2. A periodic **authoritative terrain source** (not current nonwrapping `PrototypeTerrain`) whose elevation, climate and geology agree at both seams. Independent Q/R modulo is not a substitute for axial period equivalence.
3. Seams/corners tests for mesh faces, collision, walking/planting/pathfinding, chunk eviction/reload, edit-owner identities and fluid transfers.
4. A concrete vertical Y mapping and save-compatible manifest/version rules. Neither follows from horizontal period tests.
5. Native precision and performance tests before increasing travel distance; f32 global player/render positions must not own distant identity.

The current PR adds only one headless proof of the restricted regular-lattice metric choice and documents the decision criteria. It changes **no live terrain, presentation, streamer, renderer, physics, world size or player motion**.
