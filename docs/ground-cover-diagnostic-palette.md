# Diagnostic ground-cover palette — experiment

This branch is an opt-in visual test layered on the V4 terrain experiment,
not a change to production Hexel art or the approved style guide.

## Original preview artwork

Six original 16×16 RGBA pixel-art tiles were derived from the approved
temporary preview images and are checked into `assets/diagnostic_ground/`:

| Atlas layer | File | Purpose |
| --- | --- | --- |
| 12 | lush_grass.png | Dense cool/green grass |
| 13 | dry_grass.png | Muted yellow/olive grass |
| 14 | sparse_grass.png | Mostly soil with scattered grass |
| 15 | bare_dirt.png | Entirely exposed soil |
| 16 | stony_dirt.png | Soil with gravel/stones |
| 17 | dark_shale.png | Cracked blue-gray shale |

The build script expands 16×16 pixel-art tiles to 256×256 with nearest
sampling and adds them to the offline BC7 bake. Existing 12 reference
layers retain their exact indices and original sources. Approximate
increase to BC7 storage: 0.375 MiB for six additional layers.

## World ownership and visual behavior

The authoritative torus-periodic source first computes height/geology and
slope. On soil-capable ground it then samples broad moisture, connected
ground-cover patches and small variation. Exposed top blocks can be **bare Soil**, Gravel, rich Grass, dry
GrassDry or thin GrassSparse. These are explicit seed-owned preview
block states, not per-frame random color assignments. This is a
reproducible procedural preview, not a climate/vegetation simulation.

Nearby resident world sections and seed-derived distant LOD share the
exact same surface identity and canonical torus coordinates. In opt-in
distant-terrain mode the fragment shader samples the corresponding
actual atlas layers for lush, dry and sparse grass, bare soil, gravel
and shale. No camera-relative color-region logic is required.
These new block identities are experimental, with no migration policy
for persisted saved worlds.

## Native validation

Run `git fetch origin` and `git switch feature/ground-cover-diagnostic-palette`
from the existing repository, then `cargo test` and

`cargo run --release -- --torus-test-ne --distant-terrain --stream-budget 4`

Confirm that the `TERRAIN V4` survey prints *bare soil* alongside grass
and other geology. Look for irregular connected brown patches while
walking, not alternating random single-hex colors. Approach a patch
from the outer rings and check whether its position stays consistent.
Also look for the previously reported overhead triangles, missing
faces, and near/far seam artifacts; this change does not claim they
are repaired. Capture F2 LOD worker timing and uploaded MiB.

## Follow-on work intentionally not hidden in this PR

- Geometry validation for overhead triangles and true watertight hex/LOD
  seam ownership
- Stable quantitative camera-path silhouette error and world-region
  terrain composition benchmarks
- Real regional climate, erosion, hydrology and vegetation simulation
- High-resolution production-grade, material-specific geological art
- Source-of-truth terrain edits and persistent distant-LOD invalidation

Do not merge this experimental palette until it renders and performs
acceptably on native RTX 2060 Vulkan.
