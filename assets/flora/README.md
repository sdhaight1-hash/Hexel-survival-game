# Flora research and eventual visual assets

- `reference_registry.v1.json`: **208 research rows**, including game variants
  and names from older mod releases; **not** 208 approved in-game species.
- Every `approval` is `research_only`, all unknown inputs are explicit nulls,
  and `visuals: null` means **no actual flora textures/models are linked**.
- Historical worldgen constraints are not Hexel Survival gameplay balance.
- `../../docs/flora-to-assets.md`: the planned connection from approved
  species and growth stage to authored bark/leaf/grass/fungus/crop materials.
- `../../docs/material-authoring-contract.md`: face roles for solid hexels,
  independent soil cover, grass tufts and ore-host overlays.

Current terrain texture previews under `assets/textures` do not magically
become vegetation assets. Publish actual plant assets before binding those IDs.
Run `python3 scripts/validate_flora_registry.py` at the repository root.
