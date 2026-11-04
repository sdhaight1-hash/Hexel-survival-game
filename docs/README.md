# Documentation map and source-of-truth guide

**Scope:** Hexel Survival Game repository. This index distinguishes
**art direction**, **architecture contracts**, **implementation records**,
**active roadmaps** and **historical experiments**. It does not declare
any feature complete that the runtime cannot demonstrate.

## Read in this order

1. [Project README](../README.md) for the actual playable demo, controls
   and build/test commands.
2. [Canonical style guide](../assets/STYLE_GUIDE.md) **only for visual
   direction and art approval**: palette, material identity, silhouettes,
   atmosphere and gameplay-readable presentation.
3. [Architecture/ownership](integration.md) and
   [world design](world-generation.md) for global-local boundaries,
   coordinates, finite resources and long-term planned systems.
4. The *specific* design or milestone document below for the task.
5. [Contribution and visual QA](visual-contribution-guide.md) and
   [AGENTS.md](../AGENTS.md) for the implementation/review workflow.

Do not substitute an older milestone report for current runtime behavior.
Code, registry records and validated current CI are the evidence of
implementation; the project owner approves art and deliberate design changes.

## Canonical topic ownership

| Question | Start here | Other evidence |
| --- | --- | --- |
| How should Hexal **look**? | [Style guide](../assets/STYLE_GUIDE.md) | [Original visual reference posters](../assets/style-infographics/README.md) |
| What has **shipped** for materials? | [Material/overlay roadmap](material-overlay-roadmap.md) | [P0 gallery](p0-material-gallery.md), [P1 baker](p1-material-pipeline.md), [P2.1 review](p2-1-texture-review.md), [P2.2 surface preview](p2-2-reactive-surfaces.md) |
| How do base, ore, cover and conditions differ? | [Material authoring contract](material-authoring-contract.md) | [Overlay architecture](overlay-visual-architecture.md) |
| How do continuous feature masks and optional SCA fit? | [Overlay architecture](overlay-visual-architecture.md) | [Material roadmap](material-overlay-roadmap.md), [SCA research study](space-colonization-study.md) |
| How are material IDs, files and BC7 baked? | [P1 material pipeline](p1-material-pipeline.md) | [Texture compression](texture-compression.md), [v2 format history](material-format-v2.md), [asset catalog](../assets/materials/README.md) |
| How does vegetation look and render? | [Style guide](../assets/STYLE_GUIDE.md) for look; [vegetation architecture](vegetation-visual-architecture.md) for structure | [Flora research/assets](flora-to-assets.md), [flora registry](../assets/flora/README.md) |
| What rendering effects are built or planned? | [Rendering and asset pipeline](rendering-and-asset-pipeline.md) | [P2.2 surface preview](p2-2-reactive-surfaces.md), [compression](texture-compression.md) |
| How do we move from the static demo to a 1,024 km streamed world? | [World-scale foundation](world-scale-foundation.md) | [World generation](world-generation.md) |
| What is the selected world and water invariant? | [World generation](world-generation.md) | [Decisions](decisions.md), [Integration](integration.md) |
| What is the next game/world milestone? | [Core roadmap](roadmap.md) | [Material/overlay roadmap](material-overlay-roadmap.md) |
| How should visual contributors validate work? | [Visual contribution workflow](visual-contribution-guide.md) | [Style QA](../assets/STYLE_GUIDE.md) |
| What does the first playable gallery do? | [P0 gallery](p0-material-gallery.md) | [First-person/texture history](first-person-textures.md) |

## Illustrated documentation

- [Original art-direction posters](../assets/style-infographics/README.md):
  visual identity and geology remain in the style guide; the flora
  architecture poster now appears in vegetation design.
- [Editable technical infographics](../assets/diagrams/README.md):
  material/overlay composition, optional SCA consumers, finite-water
  conservation, and world generation with persistent edits.

These are **illustrative diagrams**. The canonical prose and tested
implementation determine actual contracts, quality and shipping status.

## Classification and status discipline

- **Decided:** project direction or invariant agreed, not necessarily coded.
- **Implemented prototype:** verified, scoped runnable/headless feature;
  does not imply full production pipeline, persistence or acceptance.
- **Candidate:** visually or technically proposed, not approved.
- **Planned / research:** unimplemented; experiments still required.
- **Historical:** preserved design context. It cannot override a newer
  verified specification or current code.

**Current material snapshot:** the bounded one-chunk P0 gallery, embedded
BC7 fallback, v2 catalog/BC7 baker/index validator, F8 candidate gallery
and F9/F10 diagnostic surfaces exist. Three source art records are
`candidate`, 52 are `needs_art`, **zero are `approved`**. There is no
general v2 material GPU binding, persistent ore/cover system, world
streaming, living flora or finite-water solver.

**SCA status:** the [bounded headless prototype](../src/space_colonization.rs),
[space-colonization study](space-colonization-study.md) and
[finite-water study](finite-water-spatial-growth-study.md) are now in
`main`, merged via [PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30). None is a
production flora/ore/cave/water generator. The planned P2.3 overlay
mask system does not depend on SCA.

## Documentation lifecycle

- **Canonical art:** edit the style guide only when artistic standards
  change. Technical implementations must not quietly redefine the look.
- **Architecture:** record types, inputs, outputs, ownership, constraints,
  precision/seam issues and validation criteria in the relevant design
  document. Avoid copying the full art direction into it.
- **Status and work:** update the two roadmaps and the relevant milestone
  note when code changes; preserve unpassed native visual gates.
- **History:** first-person 32px experiment and the early material-format
  draft remain for context, clearly labeled and linking to the current
  source of truth. Do not use their old settings as current instructions.
- **Other repositories:** Voxel Game Engine and Hexel Game Engine are
  reference implementations, not dependencies or sources of game-state
  authority. Their docs must not be assumed to match this runtime.

The [master art epic #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14)
tracks P0–P7 tasks. The [main roadmap](roadmap.md) tracks M0–M3.
Neither supersedes this index or the style guide's art-review criteria.
