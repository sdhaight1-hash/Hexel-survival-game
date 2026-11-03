# Minimal implementation roadmap

The goal is a **verifiable vertical slice**, not to write all proposed planet
systems before the first playable test.

## M0 — Foundation (this pull request)

- [x] Document current topology, generation ownership and major open decisions.
- [x] Add a dependency-free Rust library and CI formatting/test workflow.
- [x] Add unit tests for periodic rectangular-raster indexing and tile edges.
- [ ] Confirm checks complete on GitHub Actions.

**Acceptance:** `cargo fmt --all -- --check` and `cargo test --all-targets`
pass on GitHub Actions. This is *not* a playable-world milestone.

## M1 — Topology and minimal game handoff

- [ ] Decide exact playable grid geometry, physical scale and two hex-lattice
      seam translations; tests prove reciprocal adjacency across seams.
- [ ] Add a minimal baked material + foliage + prefab fixture from the existing
      authoring code; write the authoring Z to world Y conversion explicitly.
- [ ] Reuse or adapt only required pieces from the existing hexel prototype to
      display one bounded world with first-person editing.
- [ ] Save a modified cell and harvested tree, unload/reload and prove
      non-regeneration with automated tests.

**Acceptance:** A test/demo can change a cell or tree, close and reopen its
world, and observe the same persistent change.

## M2 — Repeatable global geography

- [ ] Persist a versioned world manifest (seed, topology, height, revisions).
- [ ] Implement tested periodic, checksummed elevation tile read/write.
- [ ] Build a tiny synthetic tectonic/coarse-height prototype; verify seed
      determinism and seam continuity.
- [ ] Evaluate Terrain Diffusion conditioning offline; only integrate after
      a small stitched-tile seam test passes.
- [ ] Define and validate geographic elevation → playable Y mapping.

**Acceptance:** A saved tiny geography dataset can be reloaded and produces
bit-for-bit identical unmodified local terrain on repeated runs.

## M3 — Causal environment and conservation

- [ ] Condition geology and soil on tectonic region; finite ore removal remains
      removed after reload.
- [ ] Establish periodic coarse temperature, winds and precipitation with
      measurable rain-shadow cases.
- [ ] Drive vegetation placement from saved climate/soil/water inputs.
- [ ] Implement a small reservoir-based hydrology test with conservation
      assertions during precipitation, flow, damming and unload/reload.

**Acceptance:** deterministic repeatability, seam checks and water/resource
accounting tests pass before scaling the simulation.

## Out of initial scope

Planet-wide high-resolution generation, continuous atmospheric simulation,
new multiplayer/database infrastructure, total engine rewrite, full game UI,
live diffusion inference, and copies of existing asset-editor features.
