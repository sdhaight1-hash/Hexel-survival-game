# Visual development and art-review workflow

**Scope:** instructions for contributors and agents, art approvals,
review evidence and implementation discipline. This is an operational
guide, not the project's visual style definition.

**Read first:** [canonical style guide](../assets/STYLE_GUIDE.md);
then consult [the documentation map](README.md) for the relevant
technical design, milestone status and integration boundaries.

## Visual task workflow

When starting **any** new visual, vegetation, texture, shader, UI or lighting
task in this repository:

1. **Read the canonical style guide first**, then the relevant existing engine/material/
   flora docs. For tree/vegetation work, also inspect the **current**
   Voxel-Game-Engine authoring source and documentation before proposing
   a new tree generator, material baker or clipping algorithm. Check
   both repositories' actual status, not just earlier chat descriptions.
2. State whether a detail is **established**, a **candidate**, or a
   **future goal**. Do not mislabel a concept sheet as an implemented effect.
3. Translate the task into **testable visual and technical outcomes**,
   including hexel geometry, gameplay readability and performance.
4. Preserve the established world/material separation. **Do not**
   refactor geology, worldgen or the renderer just to produce an art asset.
5. For new images, produce **original** work; don't reproduce Hytale,
   Vintage Story, Luis Royo, or another creator's protected art.
6. Prefer a small **in-game vertical slice** (soil / two rock hosts /
   one readable ore / flora sample) to mass-generating unvalidated files.
7. Keep authored source and baked runtime derivatives separate. Reuse BC7
   and P1 validation rather than inventing an incompatible asset pipeline.
8. For new engine effects, measure a neutral baseline, provide graceful
   low-tier fallback, and never trade away gameplay visual cues.
9. After testing, update the relevant docs and record what **actually**
   changed; do not quietly rewrite the style guide or claim production approval.
10. When a new request conflicts with the style guide, **surface the conflict
    and ask the owner to decide** before treating the new look as canonical.

## Asset review mini-brief

For each new sample, record:

- Stable material/species ID, source/creator and approval state
- Family + geological/ecological role, and why it is recognizable
- Top/side/bottom or plant-geometry requirements; base/overlay split
- For flora: card/3D/hybrid choice, structural algorithm, authored species
  bounds, growth stage, cutout/LOD/wind plan and persistence implications
- Color/value family, macro/mid/micro shape design, physical repeat
- Host-rock/grade/cover variants if applicable
- Screenshots with F6 checker where relevant, normal view and comparison
- Artifact resolution, compression/alpha behavior and performance notes
- Observed defects, open questions and the owner's decision
