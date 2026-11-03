# Repository guidance for coding and asset agents

For **any art, texture, shader, rendering, terrain visuals, flora visuals,
UI visual style or promotional imagery task**, read
[`assets/STYLE_GUIDE.md`](assets/STYLE_GUIDE.md) **before** changing assets
or code. It is the canonical art-direction and visual QA baseline.

Keep implementation status honest: the 16³ P0 gallery and its BC7 path are
real; the P1 authoring catalog/baker is real, but the P1 runtime catalog
loader, new approved material replacements, dynamic style toggle,
ecosystems, planet streaming and cinematic lighting suite are **not**
complete. Do not invent or silently "approve" assets.

For related technical requirements consult:
- `docs/material-authoring-contract.md` and `docs/p1-material-pipeline.md`
- `docs/flora-to-assets.md` and `docs/material-format-v2.md`
- `docs/texture-compression.md` and `docs/world-generation.md`

Prefer small, validated changes; preserve stable IDs, gameplay authority
and existing functionality. For visual work, include first-person
readability and performance/format checks. The user resolves any
conflicts with the canonical style guide.
