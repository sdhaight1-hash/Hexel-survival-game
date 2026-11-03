# Minimal implementation roadmap

The goal is a **verifiable vertical slice**, not to write all proposed planet
systems before the first playable test.

## M0 — Foundation (this pull request)

- [x] Document current topology, generation ownership and major open decisions.
- [x] Add a dependency-free Rust library and CI formatting/test workflow.
- [x] Add unit tests for periodic rectangular-raster indexing and tile edges.
- [x] Confirm formatting and tests pass on GitHub Actions.

**Acceptance:** `cargo fmt --all -- --check` and `cargo test --all-targets`
pass on GitHub Actions. This is *not* a playable-world milestone.

## M1 — First playable chunk and game handoff

- [x] Add one 16 × 16 × 16 hex-prism chunk and deterministic, bounded demo terrain.
- [x] Add native first-person wgpu rendering and Minecraft-style movement.
- [x] Keep world coordinates, player simulation, and derived rendering separate.
- [ ] Manually verify desktop graphics and input on a target machine.
- [ ] Add editable blocks, persistence, and neighboring chunk streaming.

The current demo **does not** establish toroidal hex-lattice wrapping or a
complete M1 game-state integration; those remain below.

### Remaining M1 work

- [ ] Decide exact playable grid geometry, physical scale and two hex-lattice
      seam translations; tests prove reciprocal adjacency across seams.
- [ ] Add a minimal baked material + foliage + prefab fixture from the existing
      authoring code; write the authoring Z to world Y conversion explicitly.
- [ ] Extend the bounded first-person demo to support block editing and
      persistent authoritative chunk changes.
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
