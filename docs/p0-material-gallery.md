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
The candidate textures are intentionally small 128px stylized studies,
resampled to 256px for the test board and **not final or approved art**.

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
baked deterministically. The original source PNGs are never overwritten.

This is still a **quality-refinement prototype**, not a finished licensed,
seam-certified production tile set. The forest-floor luminance is tinted for
a ground-cover sample, not a real grass species model. Copper-rich pixels
are isolated from the original ore art and blended over each host rock.
No ore quantity or geological placement is implied.

See [material format v2](material-format-v2.md) for detailed face mapping,
test coverage, its limitations and the planned asset production contract.
See [production roadmap issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14)
for the full geology and flora texture goal and phases P0–P7.
