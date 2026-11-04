# Core game/world implementation roadmap

The goal is a **verifiable vertical slice**, not to write all proposed planet
systems before the first playable test. This M0–M3 track covers the
**game/world foundation**, while [material/overlay milestones P0–P7](
material-overlay-roadmap.md) cover art, assets and rendering.

## M0 — Foundation (delivered baseline)

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

## Material and overlay track — P0–P7 (cross-cutting)

The **reconciled, status-checked** art/texture program is documented in
[Material and overlay roadmap](material-overlay-roadmap.md), with the
[canonical art direction](../assets/STYLE_GUIDE.md) as the visual gate.

- **Delivered prototypes, not approved art:** P0 in-game BC7 gallery;
  P1 v2 catalog/BC7 baker/startup index check plus narrow F8 preview;
  revised P2.1 granite/shale/copper candidates; P2.2 F9 wet/moss/frost
  and F10 copper-highlight diagnostic gallery.
- **Next high-value work:** native visual approval for reference rock/ore
  assets; general v2 runtime material binding; a deterministic
  **world-space feature-mask** proof on adjacent hexel faces (P2.3).
- **Optional, subsequent:** compare a bounded space-colonization graph
  against a simpler vein/spline baseline (P2.4). The bounded research solver is in
  [src/space_colonization.rs](../src/space_colonization.rs), merged via
  [PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30), but is not a production
  texture or world generator.
- **Later:** resource/cover state authority and save/reload; staged
  replacement of all 55 identities and approved flora; environment-driven
  effects; measured mip/streaming/LOD optimization.

**Exit condition for the next art/overlay slice:** a player can identify
candidate copper on granite/shale at 1m/3m/8m and inspect one continuous
feature across neighboring hexel caps/sides without false ore identity,
discontinuous masks or loss of P0 fallback. No dependence on SCA is required
for ordinary materials, wetness or general overlay composition.

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
