# P3.1B — Candidate geography ↔ axial-hex coordinate bridge

**Status: test-only candidate; NOT APPROVED world topology or gameplay implementation.** P3.1B makes the metric and identity tradeoff explicit so a later decision is based on verified code instead of implicit modulo arithmetic. The 1,024 km × 1,024 km geographic torus remains the **approved design target**.

## The mathematical incompatibility we must decide

Existing proposed period translations are

- T₁ = **(1,024,000, 0)** axial cells;
- T₂ = **(-591,200, 1,182,400)** axial cells.

Both are exact integers, preserve six-neighbor hex adjacency under translation, and are divisible by the current 16-cell section side. The current **regular-hex** x/z basis maps T₁ to precisely 1,024,000 m east and T₂ to approximately **1,023,988.44 m** north. So the north period is **11.56 m shorter** than the agreed 1,024,000-m geographic period.

A globally **exact square metric**, this **particular** chunk-aligned period basis, and **exactly regular one-metre hexes** cannot all be maintained simultaneously with one unmodified planar metric. Do not claim the numeric difference is floating-point noise or acceptable gameplay distortion without a design decision.

## Alternative A — keep regular hexes and accept a near-square metric

Retain the current `hex_torus_candidate.rs` regular hex geometry, preserve both exact integer periods, and make geographic Z dimension approximately 1,023,988.44 m. This conflicts with the explicitly decided exactly 1,024 × 1,024 km geographic footprint unless a reconciliation rule is approved.

## Alternative B — exact square geography using an affine metric (implemented ONLY as a candidate)

Keep both integer period translations and the exact rectangular geography periods. Define an affine geographic projection:

```text
geo_x(q,r) = q + r/2 metres
geo_z(q,r) = r × (1,024,000 / 1,182,400) metres
```

The northward spacing is slightly stretched relative to the current regular hex basis by a factor of approximately **1.00001129**. That is around 11.56 m across the full north/south period, or roughly 0.011 mm per metre. **It is not perfectly regular hex geometry under the geographic metric.** Physical hex prism geometry/authoring/rendering and collision must be reconciled before choosing this as a playable-world transform. No in-game geometry changes occur in this PR.

For a 32,768 × 32,768 periodic geographic elevation raster, obtain immutable sample addresses **without floating point**:

```text
sample_x = floor((2q+r) × 32768 / (2 × 1024000)) mod 32768
sample_z = floor(r × 32768 / 1182400) mod 32768
```

Euclidean floor/division and signed `i128` intermediates make negative and far `i64` coordinates deterministic, while integer-period translations produce exactly equivalent raster samples across either axis and all four corners. `CandidateCellId(q,r,y)` explicitly keeps **unwrapped Y**; for a chunk-sized identity the original **cell** must be canonicalized before Euclidean division. The candidate `nearest_periodic_image` finds a local representative across both seams using the 3×3 neighboring images **only when both endpoints were previously canonicalized**. For arbitrary multi-period coordinates, normalize first.

Local-relative `Vec3` deltas are derived from signed integer coordinate differences **before converting to f32**, reducing far-origin precision loss. They are not a global camera-rebasing implementation.

## What was validated

Headless tests cover exact 1,024,000-m projected periods, a documented nonunity north-scale ratio, geographic sample invariance at east/west, north/south and corner translations (including negative/far inputs), stable canonical chunk ownership and Y separation, near-neighbor shortest-image distances at seam corners, and camera-relative subchunk movement at 10¹²-cell addresses.

## What is deliberately NOT done

- No geographic tile reader, climate field, geological seed lattice, rivers, mesh LOD or persistent edit keys.
- No terrain revision change, world topology activation, texture/physics distortion or renderer modification.
- The existing `PrototypeTerrain` **does not yet wrap**. Turning on this candidate without sampling the terrain source with the same periodic ownership would produce seam discontinuities. The current coarse-elevation sampler is also **not** a periodic topological world sampler.
- This module is not an approved serialized manifest or save schema. The current `HexTopology::Unresolved` metadata remains unchanged.
- This does not prove all cells across a seam have visually matching terrain/materials, that collisions/fluids are correct, or that the torus is playable at global distance.

## Decision gate before activation

Choose between regular-hex metric fidelity, exact square geographic metric with the tiny affine distortion, or a redesigned hex period/geometry contract. Then approve and version: **coordinate ownership**, **physical hex metrics**, **vertical datum/height preset**, **geographic-to-hex sampling**, and **world seed/revision**. Test periodic base geology/height/biome and exact hex collision/mesh continuity on both seams before first persistent save. Once approved, a separate PR can integrate it into `World`, `PrototypeTerrain`, and/or a new versioned global source; it should not be activated implicitly.
