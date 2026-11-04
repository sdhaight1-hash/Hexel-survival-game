# Material format v2 — historical design note and current contracts

**Classification: historical/conceptual document.** The early P0
gallery and a speculative v2 JSON recipe were originally documented
together here. The **actual implemented v2 catalog, stable-ID
schema, validation and BC7 authoring/bake contract** are now defined
by [P1 material pipeline](p1-material-pipeline.md),
[material authoring contract](material-authoring-contract.md) and the
checked-in `assets/materials/catalog.v2.json`. Those are authoritative;
do not implement the old illustrative JSON shape as a new parallel
format.

## Current implementation

- The native P0 gallery has 12 × 256² derived reference layers
  embedded as build-time BC7 or CPU-decoded RGBA fallback.
- The v2 catalog inventories all 55 original material identities,
  with **three `candidate` (granite, shale, transparent native copper)**,
  **52 `needs_art` and zero `approved`**.
- The v2 material baker validates rectangular or square RGBA8
  source images, physical repeat and face/overlay recipes, then
  writes stable-ID-sorted BC7 files and a versioned index.
- The runtime checks an available v2 index at startup and can
  show a narrow F8 candidate comparison. The index is **not yet**
  a general-purpose v2 GPU material binding.
- P2.2 F9/F10 preview conditions use diagnostic shader values and
  localized copper metallic coverage, not a production layering
  contract, real weather or resource quantities.

## Valid design principles carried forward

1. A 1m across-flats, 1m tall hexagonal prism has **two hexagonal
   caps and six rectangular sides**. Base and overlay textures
   may be conventional square/rectangular PNGs; geometry and
   metre-based UVs determine the projection.
2. A side edge is approximately 0.57735m wide; do **not** stretch
   a full 1m texture repeat equally onto each side. The special
   top-edge grass/soil transition must not wrap to the bottom.
3. Stable material IDs, authored art, host geology, grade,
   actual resource quantity, environmental conditions and GPU
   texture-layer numbers are distinct responsibilities.
4. Source art need not have one resolution. Color textures are
   sRGB; masks/roughness/normal data use suitable **linear**
   representations and must not be decoded as albedo.
5. Future mipmaps, atlas padding, physical repeat, border
   continuity, signed alpha-mask semantics, array grouping
   and hardware formats require measured tests and explicit
   versioned recipes. They are **not already implemented**.

The old illustrative `physical.texels_per_metre` /
`variations.authored_revision` recipe and earlier **128px**
candidate examples were proposals, not today's `catalog.v2.json`
contract. Current art candidates are 256px. Do not adopt
obsolete example fields or images from an old review.

## Where the content moved

- [P0 in-game gallery](p0-material-gallery.md): exact sample cells,
  physical UV mapping, controls, scale and gallery acceptance.
- [P1 material pipeline](p1-material-pipeline.md): implemented
  schema, source dimensions, baker, cache/index and GPU handoff.
- [Material authoring contract](material-authoring-contract.md):
  host/cover/ore/plant semantics and face roles.
- [Texture compression](texture-compression.md): BC7 encode/decode,
  measured sample residency, CPU fallback and format limitations.
- [P2.2 reactive preview](p2-2-reactive-surfaces.md) and
  [overlay architecture](overlay-visual-architecture.md):
  prototype state, surface-mask roles and feature continuity.
- [Current material/overlay roadmap](material-overlay-roadmap.md):
  what exists, what's not approved and the next implementation gates.

This compatibility page exists because older documents and PRs link
to `material-format-v2.md`. It deliberately does not duplicate
the live schema or a long implementation checklist.
