# In-game P0 material gallery

**Start:** `cargo run --release` from the repository root. You start facing
an elevated sample strip inside the existing one-chunk first-person world.
`F6` toggles the checker for detecting UV distortion and six-side seam
problems; `F7` cycles diagnostic checker scales of 64/128/256 texels/m.

**F8 candidate comparison:** First run
`cargo run --release --bin material-baker -- bake`; then launch
`cargo run --release`. F8 toggles an independent preview of newly
authored **candidate** granite, shale and transparent copper over both
rock hosts. P0 soil and grass previews are unchanged. F8 is unavailable
if the offline bake has not been produced or fails validation.
The revised candidates are original 256×256 studies with clustered granite
grain, uneven shale bedding and localized native-copper inclusions. They now
match the 256px test-board layer dimensions, so no preview upscaling is needed.
They are still **not final or approved production art**. See the
[P2.1 visual-review record](p2-1-texture-review.md) for explicit QA gates.

Try looking almost horizontally at the raised row `r=10` to examine the
side faces, then stand back and look down at the caps. The soil bands run
from bare to full grass cover; four adjacent columns show granite, granite
with copper, shale, and shale with the *same* copper inclusion mask.

The gallery is fixed display geometry in `World::demo`. No actual mining,
ore inventory, plant growth, saved edits or ecological generation is added.
P0 now uses the **original high-resolution soil, granite, shale, forest-floor
and native-copper crops already committed to GitHub** instead of simplistic
procedural rocks. Cropped source detail is seam-reduced into 256px materials
at startup, while grass-cover masks and copper-to-host composites are
baked deterministically at **build time**, not while launching the game.
The original source PNGs are never overwritten.

This is still a **quality-refinement prototype**, not a finished licensed,
seam-certified production tile set. The forest-floor luminance is tinted for
a ground-cover sample, not a real grass species model. Copper-rich pixels
are isolated from the original ore art and blended over each host rock.
No ore quantity or geological placement is implied.

## Physical UV and fixture details

The gallery's regular hexels are **1m across flats and 1m tall**.
Their vertical edge width is approximately **0.5773503m**, so applying
U=0..1 across each of six sides stretches apparent texel size.
P0 cap UVs project local X/Z metres; side U runs along perimeter in
physical metres and V spans one vertical metre. A half-texel side-V
inset prevents the *top-edge-only* grass rim from repeating onto the
underside. These are correct demonstration choices, not a universal
material format for directional strata or huge-world absolute UVs.

The display strip lies around axial `r=9..12`, with row `r=10`
raised by 1m for side inspection. Its rock portion is approximately:

| Q | Fixture |
| --- | --- |
| 11 | Granite |
| 12 | Granite with transparent native-copper inclusion |
| 13 | Shale |
| 14 | Shale with the **same** copper-only mask |

The preceding cells show bare/very sparse/sparse/full soil cover.
These are **fixed display cells**, not naturally generated strata
or a persisted ore occurrence. P0's 12 layers use 256×256 RGBA
color masters baked to the embedded BC7 pack at build time; the
checker F7 cycles 64/128/256 texels per metre **only for diagnostics**.
Actual production resolution is not fixed to 256px.

See [material format v2](material-format-v2.md) for detailed face mapping,
test coverage, its limitations and the planned asset production contract.
See [production roadmap issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14)
for the full geology and flora texture goal and phases P0–P7.
