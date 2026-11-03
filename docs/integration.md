# System ownership and integration boundaries

**Status:** integration plan; no libraries from the other repositories have
been imported into this game.

## Existing code to build upon

| Repository | What exists | Intended role |
| --- | --- | --- |
| [Hexel Game Engine](https://github.com/sdhaight1-hash/Hexel-Game-Engine) | Rust/wgpu playable flat hexel prototype, 16³ chunks, camera, collisions, editing, some tree and geology data | Candidate runtime techniques and gameplay prototype; validate code before reusing |
| [Voxel Game Engine](https://github.com/sdhaight1-hash/Voxel-Game-Engine) | Rust procedural asset editor (Nodes, Presets, Foliage, Assets), wgpu asset preview and authoring exports | Authoring tools and reusable **baked** assets, not world-simulation authority |
| **This repository** | Bootstrap Rust geography indexing and project documents | Future game runtime integration, persistent world state and global-field consumers |

**Topological conflict to resolve:** Hexel Game Engine's README still describes
a future **sphere**, while the latest supplied world-generation specification
selects a **flat-coordinate torus**. Adopt neither spherical gameplay topology
nor spherical surface IDs by accident. A controlled, explicitly tested migration
is needed before connecting engine gameplay to this world's coordinates.

## Data contracts (not yet implemented)

```text
synthetic seed + generation revisions
     -> periodic tectonics / climate / constrained terrain refinement
     -> validated immutable global geographic tiles
     -> deterministic local 3D generation
     -> editable, streamed chunks / physics / renderer
                         |
               persistent world-state overlay
               (edits, finite resources, water)
```

Parallel content pipeline:

```text
authoring Nodes/Presets/Foliage/Assets
     -> validate + bake + version
     -> shared game-ready asset registry
     -> biome/geology/world placement + rendering
```

The runtime decides **where** a tree or rock occurs, its quantities, and what
changes after harvest; authored assets define **what** it is and how it looks.
Do not evaluate CPU material graphs per terrain cell or run ecology inside the
asset editor. Meshes and GPU buffers are derived caches.

## Coordinates and authority

- The global geography layer uses a periodic **rectangular X/Z raster**.
- Proposed playable hexels use **Q/R/Y**. The authoring editor currently uses
  **Q/R/Z** for local hex occupancy; mapping and physical scale must be explicit.
- Determine hex-lattice period translations, orientation, exact geometry and
  world-to-hex transforms before claiming seamless player travel.
- Keep immutable base geography separate from player modifications, resource
  depletion, ecology and conserved water stores.
- Persist world seed, settings, asset IDs/versions and generator revisions so
  reloading or upgrading never silently regenerates mined/harvested resources.
- One 1 m across-flats authored foliage hexel is approximately 0.866 m³;
  never assume it is a one-cubic-metre water container.

## Integration acceptance slice

Prove a **single tiny test world**, not a full planet:

1. Load one material, one tree and one hexel asset using a documented baked
   source-to-runtime handoff.
2. Place a prefab with explicit authoring-Z to world-Y conversion.
3. Edit/harvest it, unload and reload the section, and assert the change
   remains while immutable source assets remain unchanged.
4. Only then connect global geographic fields to local placement.

No new database, network server, AI inference service or game renderer is
required to exercise these first contracts.
