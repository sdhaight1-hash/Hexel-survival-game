# V4 rock cluster gallery (opt-in candidate-art review)

This branch adds a **separate** `--rock-cluster-gallery` mode.
The default P0 gallery, streamed terrain, geology and saved state are not changed.

## Install the existing V4 PNGs

Download `hexel_rocks_v4_pixel_geology.zip` from the ChatGPT conversation.
From the repository root:

```powershell
python scripts/install_rock_gallery_v4.py "C:\path\to\hexel_rocks_v4_pixel_geology.zip"
cargo run --release -- --rock-cluster-gallery
```

If Python is invoked using `py` on your system, substitute `py` for `python`.
The installer reads 69 V4 RGBA PNG files and writes only to
`assets/rock-gallery-v4/<family>/<rock>/<rock>_<face>.png`.
It does not overwrite `assets/textures/`, `assets/production/` or the v2 material catalog.
If a candidate file is missing, the game **fails explicitly** instead of showing
a misleading P0 rock.

## Layout

- 23 pads, in canonical rock order.
- Each pad is 10 axial Q by 10 axial R hexels, three hexels high.
- Two empty axial columns between pads.
- Walkway runs along R = 12..14, next to the exposed front pad wall.
- Player starts near (Q=5, R=13), facing along the lineup.
- Rock 01 at Q=0..9, rock 02 at Q=12..21, ... rock 23 at Q=264..273.
- Four-texture layers are **not** baked: each rock has top/side/bottom
  and the 70th layer is a neutral walkway.
- Source images are sampled at 1m x 1m repeat; side UVs use measured physical
  metres, not a 0..1 stretch per hex side.
- The gallery retains the existing 200-metre far plane; walk the strip to see
  pads beyond the initial camera range.

Rock order: bauxite, chalk, chert, claystone, conglomerate, limestone,
sandstone, shale, phyllite, slate, green marble, red marble, white marble,
andesite, granite, kimberlite, peridotite, basalt, obsidian, scoria,
travertine, tuff, suevite.

## Inspection protocol

1. **At 1m:** stand close to pad 01. Examine left- and right-facing side
   orientations. Look for a vertical line on one side of each prism,
   top rim leakage, blur/stretching, and high-contrast pixel clusters.
2. **At 3m:** walk a few steps back and scan the entire 10x10 top area. Look
   for repeated stamp shapes, obviously repeating squares, dark seams,
   or pattern discontinuities crossing neighboring hexels.
3. **At 8m:** examine whether a rock is still identifiable and whether detail
   starts crawling or shimmering as the camera moves slowly.
4. **UV diagnostic:** press F6 and inspect six sides + top. Each physical
   metre must have the same checker density. Check whether the original
   *left-side line* persists when the texture is replaced by the checker.
   If it does, investigate mesh UVs, face lighting, or geometry first.
   F7 cycles checker scale.
5. **Camera test:** slowly rotate on a single spot. Note whether lines stay
   on the same world-side face, follow screen orientation, or disappear
   with a different view angle.
6. **Performance:** compare idle FPS/frame CPU timing at the starting pad,
   center of gallery and last pad; F3 and F2 collect existing diagnostics.
   Do **not** interpret F3's CPU render-stage numbers as GPU timestamps.
7. **Texture comparison:** compare shale vs slate, granite vs andesite,
   sandstone vs limestone, basalt vs obsidian, conglomerate vs suevite,
   and the three marble variants.

For any defect note: **rock name, pad number, viewing distance, face
(top/side), checker ON/OFF, whether the camera was moving**, and screenshot.
Capture F2 session text if a hitch or a rendering error occurs.

### Limitations

This is a material-review fixture, not a terrain biome or ore simulator.
V4 PNGs are **candidate art** and not approved. Existing side-face tint
differs slightly by orientation and can mimic a seam; F6 helps isolate it.
There are no generated mipmaps in this debug array, so distant shimmer is
a candidate limitation of the preview, not yet a final production verdict.
The runtime gallery uses uncompressed RGBA source textures for rapid review,
not the production BC7 material runtime handoff.

CI or local Rust compile and a native walkthrough remain required to
declare this fixture working; adding files to GitHub alone is not that test.
