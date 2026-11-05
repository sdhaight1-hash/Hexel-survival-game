# V5.5 — One palette per hexel, narrow contact-only blending

**Prototype of the user's color concept, not a claim that it exactly matches
the final intended method.** Branch derived from V5.4. Only the V5 rock
gallery changes; geological structure and disruption overlays stay in
world-space 3D coordinates.

## Design

Each granite, shale, or basalt hexel has one of **five rock-specific RGB
palette multipliers**. A stable hash of axial (q,r,y) and rock family
selects its variant. The hash subtracts the A/B/C/D pad offset so the
same local cells have matching assignments across stages.

- **Center of each exposed face:** retain *this hexel's* assigned palette.
- **Only within 0.14m of a face boundary:** blend with a neighbor *if it
  is actually solid and has the exact same gallery pad/material family*.
- **Air, gaps, water and unrelated rock pads:** do not contribute color.
  A boundary facing air retains the own-cell palette.
- At corner junctions and vertical layer joins, weights normalize the
  contributing same-rock palettes. Actual hidden shared faces are
  still culled; visible *border strips* carry the interpolation.
- The previous **5m world-scale color gradient is removed**. Structural
  details such as shale bedding and basalt fracture overlays remain
  world-space and are not hashed independently per cell.

This is an intentionally literal test of color ownership at hexel scale,
not a procedural geologic map or production host-rock contact system.

## Geometry tradeoff

To keep the interior unblended, the V5 gallery alone subdivides visible
top/bottom caps (3 triangles per wedge rather than 1) and side quads into
three by three strips (18 triangles instead of 2). This makes a visible
14cm border without changing the world simulation or expanding the
normal-world meshes. **CPU mesh generation and GPU triangle count will
increase significantly in this gallery**. Measure before considering
production. A shader-driven blend or packed per-face palette attributes
could later reduce this cost.

## Visual controls

F5 cycles:
1. **Normal** — regular grayscale geology with the V5.5 per-cell palette
   tints and the existing V5.4 world-space overlays.
2. **Raw IDs** — five bright diagnostic colors, a single ID per hexel.
   No blending in this view.
3. **Blend view** — exaggerated display of the *actual mesh-based*
   boundary interpolation; hexel interiors should be solid in color,
   with narrow transition zones around contacts.

Leave F6 UV checker OFF when inspecting the palette diagnostics.
Pads 1 / 5 / 9 remain neutral grayscale controls. Pads B, C, D of
each rock show palette assignment and contact blending.

## Windows commands

```cmd
git fetch origin experiment/v5-5-hexel-edge-palette
git switch experiment/v5-5-hexel-edge-palette
cargo test --bin hexel-survival-game rock_palette_edges
cargo test --bin hexel-survival-game p2_surface_shader_parses_and_validates
cargo run --release -- --rock-v5-gallery
```

If Git cannot switch because of uncommitted local edits, stop before
resetting or deleting anything; leave Cargo.lock alone.

## Test

1. At pad 2, use F5 raw IDs. Confirm each hexel is distinctly colored
   with five repeatable colors and no camera-dependent flicker.
2. Press F5 again. The *interior* should keep its own tint. Only **narrow
   strips near solid same-rock joins** should transition to a neighbor.
3. At the exterior outline, inspect exposed air edges: no borrowed
   outside color. Inspect 3-way corners and horizontal stacked-hex joins.
4. Return to normal view. Compare A vs B, then C vs D at 1/3/8m.
   Report any checkerboard effect or obvious bands along the hex seams.
5. Walk sideways and measure perceived responsiveness versus V5.4.
   This prototype has much higher V5 gallery triangle counts, so check
   whether the extra mesh cost is acceptable.

## Scope limits

This method intentionally does **not** blend different rock species,
simulate fractures, preserve torus-canonical palette IDs, or use
physically based lighting. Neighbor blending is restricted to equal pad
IDs so independent gallery pads never affect one another.

GitHub source has been checked; the shader validation and native
Windows compilation/runtime are not yet verified.
