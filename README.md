# Hexel Survival Game

Foundation for the planned **Rust survival-game runtime** and its synthetic, finite world.

**Status:** bootstrap only. This repository is not yet a playable game or a functioning
planet generator. The first implementation target is a deterministic, periodic geography
index with tests; playable terrain and asset integration are follow-on work.

## Project boundaries

- [Hexel Game Engine](https://github.com/sdhaight1-hash/Hexel-Game-Engine):
  existing playable hexagonal-prism prototype. Its spherical-planet vision conflicts
  with the **toroidal** world chosen here; do not silently combine the topologies.
- [Voxel Game Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine):
  existing Nodes, Presets, Foliage and Assets authoring application, plus the
  [world design decisions](https://github.com/sdhaight1-hash/Voxel-Game-Engine/blob/main/docs/procedural-world-design-decisions.md).
  Keep authoring separate from simulation authority.

## Starting assumptions

- A finite **1,024 × 1,024 km**, flat-coordinate world with wraparound on both axes.
- Synthetic tectonics and coarse climate/hydrology constrain **one-time** terrain refinement.
  Store the finished geography; no diffusion model is required during play.
- Generate local editable 3D cells from the saved global fields on demand. Persist edits,
  resource depletion and environmental state separately.
- Playable hexagonal-prism cells (axial Q/R, vertical Y) are the current **provisional**
  preference, **not** an implemented runtime here. The global geography grid is rectangular.
- Global water is finite and conserved across coarse and local simulations.

## Documentation

- [World-generation foundation](docs/world-generation.md): pipeline and invariants.
- [Architecture and repository integration](docs/integration.md): system ownership.
- [Decision register](docs/decisions.md): decided, provisional and unresolved choices.
- [Implementation roadmap](docs/roadmap.md): measurable milestones.

## Foundation tests

The initial Rust library implements *only periodic raster indexing*.
Run `cargo test` and `cargo fmt --check` after checking out the branch.
This does **not** demonstrate hex-lattice wrapping, terrain generation or water physics.

No production runtime dependencies are introduced at this stage.
