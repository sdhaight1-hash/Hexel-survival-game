> **Historical first textured demo (pre-P0 quality refinement).**
> This document describes the initial 32px rendering experiment, not the
> current source-art-derived 256px gallery. For the current scene, use
> [P0 material gallery](p0-material-gallery.md) and
> [material format v2](material-format-v2.md). The retained examples are
> engineering history, not approved production textures.

# First-person scale and hexel material preview

The first 16×16×16 chunk remains a **local visual prototype**, not the
global toroidal world or geology pipeline.

## Dimensions

- A hexel is a hexagonal prism **1.0 m tall** and **1.0 m across flats**.
- The player body is **5/3 m (about 1.667 m)** tall.
- This makes one hexel **3/5 of a standing character's height**.
- The eye position is **1.52 m above the player's feet**.
- The camera uses a 70° vertical field of view; three stacked hexels still
  form a 3-metre wall, taller than a player, as expected.

The simulation owns the feet position and body dimensions. The renderer uses
the eye position from `Player`. Do not scale rendered hexels independently
to make them appear shorter; physical dimensions must agree with collision
and eventual construction/asset sizes.

## Currently visible material mapping

The demo now displays **real 32×32 PNGs** from `assets/textures`:

| Face in the demo | Material asset |
| --- | --- |
| Surface cap | `soil/low_fertility_soil_32.png` |
| Temporary small surface preview patch | `soil/forest_floor_32.png` |
| Exposed soil and surface-cell sides | `soil/medium_fertility_soil_32.png` |
| Rock | `rock/metamorphic_igneous/granite_32.png` |

These four PNGs are embedded in the binary and decoded once when building
the GPU **2D texture array**; each mesh vertex carries UV and a material layer
ID. Nearest-neighbor filtering preserves the 32px appearance. Hex caps use
X/Z projection into the square tile, while each of six side quads has its own
U/V coordinates. **No cube UV layout is used**.

The other soil, rock, ore and utility-mineral assets remain in their existing
categories, available for later resource mapping. The demo **does not** scatter
ores or infer host rocks: rendering only a granite prototype is not a
geological rule. When real strata are implemented, each cell's geological
material must be part of authoritative world data, not decided by the
renderer. A texture or a GPU buffer is not a mineral-stock inventory.

## Quality limits

The atlas-derived textures are downsampled to 32×32 but have not been
certified fully seamless. Seams or directional motifs in the original art
need separate art review; clamping hides border spill, not seam artifacts.
The PR's automated checks cover asset decoding and generated UV bounds,
but actual GPU visuals and camera feel need a Windows smoke test.
