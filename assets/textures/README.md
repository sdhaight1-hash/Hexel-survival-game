# Hexel Texture Asset Partition

This folder contains the generated texture atlases split into per-material files.

## Structure
- `soil/` — soil and surface organics
- `mineral/` — utility minerals and crystals
- `ore/` — metal ores
- `rock/sedimentary/` — sedimentary rocks
- `rock/metamorphic_igneous/` — metamorphic and igneous rocks
- `rock/special_volcanic/` — special and volcanic rocks
- `atlases/` — original atlas sheets used as source

Each material has:
- `name.png` — cropped tile from the atlas
- `name_32.png` — downscaled 32×32 version

A machine-readable listing is available in `manifest.json`.

## Note
These files were prepared locally from the generated atlas sheets. They have **not** been committed to GitHub in this environment.
