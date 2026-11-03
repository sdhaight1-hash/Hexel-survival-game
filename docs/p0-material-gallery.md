# In-game P0 material gallery

**Start:** `cargo run --release` from the repository root. You start facing
an elevated sample strip inside the existing one-chunk first-person world.
`F6` toggles the checker for detecting UV distortion and six-side seam
problems; `F7` cycles diagnostic checker scales of 64/128/256 texels/m.

Try looking almost horizontally at the raised row `r=10` to examine the
side faces, then stand back and look down at the caps. The soil bands run
from bare to full grass cover; four adjacent columns show granite, granite
with copper, shale, and shale with the *same* copper inclusion mask.

The gallery is fixed display geometry in `World::demo`. No actual mining,
ore inventory, plant growth, saved edits or ecological generation is added.
Every generated P0 texture is an **original procedural comparison swatch**,
not final artwork and not a rebranding of the existing 55 images.

See [material format v2](material-format-v2.md) for detailed face mapping,
test coverage, its limitations and the planned asset production contract.
See [production roadmap issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14)
for the full geology and flora texture goal and phases P0–P7.
