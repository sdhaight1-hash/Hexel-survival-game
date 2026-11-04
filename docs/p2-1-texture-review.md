# P2.1 — geology candidate art review

**State:** first revised candidates committed; **native visual acceptance pending**.
No material is approved. This is an art iteration, not new game simulation,
geology placement, lighting, mesh format, or vegetation.

## Baseline vs revised source art

| Material | Before (initial F8 candidate) | Revised P2.1 candidate | Gate still open |
| --- | --- | --- | --- |
| Granite | Obvious large Voronoi/paving-stone polygons and flat color islands | Mottled interlocking quartz/feldspar-like groups with darker mica-like pockets, modest fractures and secondary grain | Distinct crystals without blurry stone or repeating motifs |
| Shale | Regular continuous sinusoidal contour bands | Unequal-thickness sedimentary beds, occasional cleavages, subdued surface variation | Natural-looking layer interruptions, cap orientation and seam quality |
| Native copper | High-contrast orange squiggles over the host rock | Sparse branching fracture fills and grouped localized nodules; mostly transparent overlay, occasional greenish oxidation | Poor ore visible without glow or colored paint, geology credible |

**Before evidence:** the project owner's November 3, 2026 first-person F8
screenshot (in the project conversation) establishes the visual problems above.
It is **not** stored as a repository screenshot.

**After evidence:** native Windows F8 screenshot is **not yet captured**.
Automated CI, source PNG previews or GitHub art files cannot replace a
same-camera GPU screenshot. Do not claim P2.1's art direction has passed.

## Physical and technical constraints

- Three replacement PNG sources remain at their existing paths under
  `assets/production/`, are **256×256 RGBA8**, and retain exactly the same
  registered stable IDs and **1 m × 1 m physical repeats**.
- Granite and shale are opaque. Copper's source is a **transparent ore-only**
  layer; both host rocks remain separate, and the same mask is used over both.
- Artwork derives from original controlled shapes and periodic noise, without
  importing copyrighted game textures. Procedural construction does **not**
  imply that the art is hand-painted or approved.
- The PNGs are current editable color masters. BC7 images are regenerated
  caches with the existing `material-baker` tool, not checked-in masters.
- The existing F8 preview, P0 fallback and hexel UV code are unchanged.
- A Rust test checks PNG decoding, average opposite-edge seam differences,
  absence of exact short-period repetition in rock colors, and an actual
  transparent/opaque split for the copper layer. These are **sanity checks**,
  not a score for geological accuracy or beauty.

## Required native comparison to close the art gate

1. From the repo's `main` build, run the existing v2 baker and game, stand
   in the same gallery position and photograph the granite, shale and both
   copper hosts with **F8 ON** and **F6 OFF**.
2. Compare against the November 3 baseline at close (~1 m), medium (~3 m),
   and distance (~8 m). Walk around to inspect all six rectangular sides,
   the true hexagon caps, top edge joins and nearby tile repeats.
3. Record separate neutral-light and shadow screenshots. If ore is readable
   only because of its saturation, or granite/shale look pasted together
   rather than physically distinct, revise the source assets.
4. Only after these visual checks and explicit art approval may any of the
   catalog records change from `candidate` to `approved`.

**Current review decision: NEEDS IN-GAME REVIEW.** Do not scale the pattern
generation into 23 rock species or 16 ore types yet.
