# Technical design infographics

These **editable, original SVG diagrams** illustrate currently proposed
architectural boundaries. They complement the Markdown contracts; no
diagram is an implemented feature proof, a game screenshot, a tested
shader output, or an exact hexel UV/terrain template.

| Diagram | Placement | What it communicates |
| --- | --- | --- |
| [material-overlay-stack.svg](material-overlay-stack.svg) | [Overlay visual architecture](../../docs/overlay-visual-architecture.md) | Authored substrate vs structural feature vs condition vs material response; world-space copper continuity and authoritative ore state |
| [spatial-growth-consumers.svg](spatial-growth-consumers.svg) | [Overlay architecture](../../docs/overlay-visual-architecture.md) and [vegetation architecture](../../docs/vegetation-visual-architecture.md) | Shared SCA geometric skeleton as an optional input; specialized consumers keep their own rules |
| [finite-water-hierarchy.svg](finite-water-hierarchy.svg) | [World generation](../../docs/world-generation.md) | Global, regional and local finite-water compartments with balanced transfers; SCA does not solve water pressure |
| [worldgen-persistence.svg](worldgen-persistence.svg) | [World generation](../../docs/world-generation.md) and [integration](../../docs/integration.md) | Immutable generated geography vs authoritative persistent player edits and disposable render caches |

## Accuracy and use

- Each diagram has an accessible SVG `title` and `desc`, text labels and
  no linked external image or font resources.
- Text is **editable and source-controlled**, so engineering changes
  can update labels and diagrams surgically without regenerating an
  entire painted poster.
- The planar linked hexagons illustrate **true hexagonal caps**, but
  do not specify the final global hexel-coordinate torus seam mapping.
- SCA research currently resides on
  [draft PR #30](https://github.com/sdhaight1-hash/Hexel-survival-game/pull/30);
  the overlay compositor, world generator, water solver and real
  geology/ecology remain **proposed**. Do not promote their labels
  to `implemented` until the code and tests prove them.
- Diagram palette echoes Hexal's restrained warm/cool color intent,
  but is not a numeric gameplay texture palette.
- The only canonical art-direction document is
  [STYLE_GUIDE.md](../STYLE_GUIDE.md). Review visuals there; use
  [documentation map](../../docs/README.md) for engineering ownership.

The older painted WebP reference posters are retained separately in
[style-infographics](../style-infographics/README.md), and are **not**
technical contracts or precise illustrations of a 1m hexel renderer.
