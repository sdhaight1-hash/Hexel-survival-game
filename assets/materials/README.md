# Material catalog: face-aware authoring

The source/reference image library remains under `assets/textures/`. The
catalog in `catalog.v1.json` holds explicit stable material IDs and
top/side/bottom image paths. Its current entries are **legacy-preview only**:
they are not proof that the art is seamless, that the side face has the correct
physical texel density, or that it contains vegetation transitions.

Future approved texture work follows `docs/material-authoring-contract.md`:

- Substrates and ore hosts stay separate from cover and individual plants.
- A soil top may have grass cover, while a side uses grass-to-soil transition
  and a bottom remains soil.
- An ore is shown through an *ore-only transparent overlay* of its actual
  geological host rock; existing opaque ore crops are unsuitable overlays.
- Maintain the 32x32 RGBA8 baseline and record stable material IDs and status.
- Preserve the six hexagon-side seams and use ~0.577m side width vs 1m height.

Run `python3 scripts/validate_material_catalog.py` from the repository root
before pushing. The checks run automatically in GitHub Actions.
