# Repository guidance for coding and asset agents

Start at [documentation map](docs/README.md) to find **one
canonical source per topic**. For **any** visual, art, texture,
shader, terrain appearance, flora look, UI, lighting or
promotional imagery task, first read
[canonical style guide](assets/STYLE_GUIDE.md).
It is the **visual direction and approval baseline**; do **not**
store implementation architecture, milestone status or work logs
there unless they establish an enduring artistic rule.

For changes to specific systems use:

- [Material contract](docs/material-authoring-contract.md) and
  [P1 pipeline](docs/p1-material-pipeline.md): stable IDs, actual
  authoring/baking interfaces and resource ownership.
- [Overlay visual architecture](docs/overlay-visual-architecture.md)
  and [material/overlay roadmap](docs/material-overlay-roadmap.md):
  current P0–P2.2 status, planned cross-hexel masks and optional SCA
  feature-generator integration.
- [Flora assets](docs/flora-to-assets.md) and
  [vegetation architecture](docs/vegetation-visual-architecture.md):
  species research, authoring references, structure/LOD and
  world-persistence boundaries.
- [Rendering and asset pipeline](docs/rendering-and-asset-pipeline.md):
  hardware, shaders, alpha, compression and performance proposals.
- [World generation](docs/world-generation.md),
  [integration](docs/integration.md), [decisions](docs/decisions.md)
  and [core roadmap](docs/roadmap.md): coordinates, world authority,
  persisted edits, finite resources and future world systems.
- [Visual contribution workflow](docs/visual-contribution-guide.md):
  review evidence, QA checklist and owner approvals.

**Implementation status (2026-11-04):** the 16³ P0 gallery,
BC7 path, P1 catalog/offline baker/index validator, optional F8
gallery and F9/F10 visual diagnostic effects are implemented. The
separate opt-in 95-section **moving streamed-world prototype** has
deterministic terrain, dynamic GPU mesh uploads and F1/F2/F3
diagnostics. It does **not** implement persistent edits, toroidal
wrapping, distant LOD, general v2 GPU material binding, continuous
**production** mineral deposits, living ecology, finite water or
cinematic lighting. Refer to [streaming implementation](docs/p3-2-streamed-world.md)
and [presentation A/B diagnostics](docs/p3-presentation-mode-ab.md)
rather than describing the runtime as one-chunk-only.
The bounded SCA research prototype is now in
[src/space_colonization.rs](src/space_colonization.rs) following
[merged PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30); it is **not** integrated with
world generation, materials or gameplay. The 55-source v2 catalog has **three
candidates, 52 needs-art and zero approved production artwork**.

Prefer small, validated changes. Preserve stable IDs, gameplay
authority, resource conservation, chunk/feature ownership,
source/baked asset boundaries and existing behavior. A green
test suite never constitutes visual approval; require first-person
1m/3m/8m evidence and performance/format checks for visual changes.
Do not silently change canonical decisions; the user resolves
style/architecture conflicts and approves production art.

**Audit/refactor rule:** compare a proposed deletion or abstraction
against a known production call path and relevant tests. Preserve
benchmark log fields, the default FIFO/latency-2 render configuration,
F1/F2/F3 behavior, `Is_Moving`/`Is_Looking` diagnostic groups and
the native Windows testing gate. Code-line reduction is evidence of
maintainability only when semantics stay clear; do not merge old
benchmark/ore branch heads into the streamer without independent review.
See [dated repository audit](docs/repository-audit-2026-11-04.md).
