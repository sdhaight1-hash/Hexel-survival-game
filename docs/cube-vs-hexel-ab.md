# Cube vs hexel A/B experiment

**This is a separate draft experiment.** It starts from ground-cover PR #77 and
does **not** replace the main or hexel prototype branch.

## Controlled elements

Held constant: seed 42; `PeriodicTerrainCandidate` heights, landforms,
geology and explicit grass/dirt cover; original six ground-cover preview
textures plus inherited P0 materials; daytime cycle, sky and shadow map;
16×16×16 storage sections; streaming budget; LOD selection spacings
1/2/4/8m; four outer LOD bands and near/far residency mask rules;
camera control, collision dimensions and player speed.

Changed: physical mapping of a q/r column to **one meter apart on
orthogonal X and Z axes** instead of hexagonal axial spacing. This is
necessary for genuinely touching 1m×1m×1m cubes; simply putting square
boxes on hex centers would cause holes/overlap. The canonical torus
identities, terrain random seed, and north-period q offset are unchanged,
which means the visual torus footprint is now a sheared square lattice
rather than a hexagonal metric.

- Resident faces: six square cube faces, four horizontal neighbors, 1m
  material UVs. Covered faces are hidden.
- First 64m LOD: square unit column caps and exposed cliff quads, computed
  from the same global surface data, including exposed soil and rock.
  Vertical walls tile the material instead of stretching it.
- Remaining LOD bands: original outer triangle heightfield with the same
  error-refinement strategy and the same material classifier.
- Physics: square-grid coordinate lookup, 1m cubic collision support
  height; normal movement and body size unchanged.
- Local torus origin and rebase: orthogonal 1m units; same canonical
  global cell identity and chunk-aligned floating origin.

## Known nonidentical aspects

Changing lattice shape **necessarily** changes which world positions and
adjacent columns are physically next to each other. q/r seed samples are
the same, but mountains may feel spatially different due to the altered
metric. Chunk-residency layout still uses the original axial-radius budget,
so its physical footprint is diamond/hex-like in the square coordinate
system: this keeps the same number of sections for comparison. The
camera-relative near LOD square band and outer triangle rings are not
guaranteed watertight across every boundary; this experiment should not
be mistaken for an LOD repair.

F12 torus seam guide planes are only approximate with the new sheared
spatial metric. The actual wrapping is determined by canonical q/r
identity, not those overlays.

## Native comparison

In your existing repository directory, run one command at a time:

```bat
git fetch origin
git switch experiment/cubic-voxels-ab
cargo test
cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4
```

For the baseline, switch back to
`feature/ground-cover-diagnostic-palette` and run the same Cargo command.
Use the same spawn, similar camera orientation and 60–120-second
walk toward a distinct ridge. Compare visible holes, cliff silhouette,
near/far transitions, material readability, sun shadows, F2 LOD CPU
generation time, uploaded MiB and frame pacing. Report any first
validation error before attempting to interpret a later Vulkan cleanup
backtrace.

Keep both PRs draft until visual correctness and RTX 2060 performance
are measured.
