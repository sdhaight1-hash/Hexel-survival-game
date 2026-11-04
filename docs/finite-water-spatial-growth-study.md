# Finite water and the proposed spatial-growth foundation: hydrology study

**Status (2026-11-03):** research recommendation only. This does **not**
implement a water solver, authoritative water inventory, caves, regional
river routing, ocean reservoirs, or procedural SCA water-path generation.

**Question:** can space colonization help generate water flow, spreading,
and leveling while retaining a finite, globally conserved water inventory?

## Decision

**Yes for selected path/network geometry; no as a standalone liquid solver.**
SCA can propose branching tributary, fracture, rootlike seepage or
subterranean conduit *structures*, under hydrological constraints. It cannot
by itself determine physical water volume, pressure, velocity, hydrostatic
equilibrium, flood depth, flow reversals or conserved transfers.

Keep a physically accountable **water-inventory + transport subsystem**
separate from the geometry-generating **spatial growth / network** subsystem.
Connect them through explicit, versioned hydraulic edge/region interfaces
only when experiments prove useful.

The user's supplied Programming Chaos transcript (video:
https://www.youtube.com/watch?v=Y6QLhjvytXk; 38:50–40:57) shows a
river-*looking* 2D network made by attractor placement, color choice and
random perturbations. It is evidence for a visual branching technique,
**not** runoff, mass conservation, gravitational flow or lake leveling.

## 1. Research and evidence

- US Army Corps of Engineers, HEC-RAS shallow-water equations:
  https://www.hec.usace.army.mil/confluence/hecras/beta/technical-reference/hydraulic-equations/shallow-water-equations
  — depth-integrated mass continuity and momentum; water-surface elevation
  and hydrostatic pressure provide the driving forces.
- USACE finite-volume continuity formulation:
  https://www.hec.usace.army.mil/confluence/hecras/latest/technical-reference/numerical-methods/shallow-water-equations-solver/continuity
  — changes in a cell's water volume equal signed flux through faces plus
  explicit source/sink quantities.
- Simoes (2011), US Geological Survey, finite-volume shallow environmental
  flow: https://www.usgs.gov/publications/finite-volume-model-two-dimensional-shallow-environmental-flow
  — wet/dry fronts and a mass-preserving surface solver on an unstructured grid.
- Caviedes-Voullième et al. (2018), *Cellular Automata and Finite Volume
  solvers converge for 2D shallow flow modelling*:
  https://doi.org/10.1016/j.jhydrol.2018.06.021
  — an appropriate simplified cell-based scheme can closely relate to
  finite-volume formulations; “cellular automata” alone does not guarantee
  physical correctness or conservation.
- Barnes, Lehman & Mulla (2014), Priority-Flood:
  https://doi.org/10.1016/j.cageo.2013.04.024
  — efficient basin/depression analysis and watershed routing; normally
  represents drainage connectivity rather than finite water storage.
- Barnes, Callaghan & Wickert (2021), Fill–Spill–Merge:
  https://doi.org/10.5194/esurf-9-105-2021
  — finite prescribed runoff accumulates in nested depressions, fills to
  pour points, spills and merges via depression hierarchy.
- FlowFill (2019):
  https://doi.org/10.5194/esurf-7-737-2019
  — mass-conserving routing of prescribed runoff while allowing natural
  depressions to retain water.
- USGS on hydraulic head:
  https://pubs.usgs.gov/circ/circ1217/html/boxa.html
  — groundwater moves from greater to lower hydraulic head; water pressure,
  rock conductivity and gradient matter, not tree-root attraction.
- Porous-media drainage networks:
  https://doi.org/10.1002/2016WR019299
  — connectivity in pore networks and multiphase drainage is dynamic and
  controlled by pore geometry/capillary pressure; invasion percolation is
  often a closer model to capillary fingering than SCA.
- Hexagonal finite-volume precedent:
  https://gmd.copernicus.org/articles/7/909/
  — conservative shallow-water methods exist for hexagon-related grids,
  though these spherical geophysical meshes are **not** our chosen
  1m-tall hex-prism world.

## 2. Candidate uses and nonuses

| System | SCA role | Actual water authority |
| --- | --- | --- |
| River creation at world-build | Optional candidate network seeded by sources and constrained by *saved* terrain elevation, basin and outlet | Hydrological routing, rainfall budget, flow accumulation and channel hydraulics |
| Tributary/streambed pattern | Optional supplementary branch geometry and visual meanders | Slope, discharge, bankfull capacity, sediment/erosion constraints |
| Fracture-fed springs or cave streams | Optional connected graph of *possible* conduits through eligible fractures/rock | Hydraulic head, Darcy/conduit resistance, pressure, finite aquifer/cave inventory |
| Water seepage along roots, cracks and porous bands | Optional visually coherent preferred path or mapped conductivity structure | Soil retention, capillarity, permeability, flux and mass transfers |
| Water settling and equalizing across adjacent hexels | **Not suitable** | Conservative flux over shared faces; water surface elevation and/or pressure head |
| Filling lakes and spilling into neighboring lakes | **Not suitable** | Depression hierarchy, storage-volume curves, spill elevations and mass accounting |
| Dam breaches / flash floods | **Not suitable** for the physics | Dynamic finite-volume shallow-water solver with momentum, wet/dry handling and controlled timesteps |
| Ocean/wave cosmetics | Potential only in stationary pattern authoring | Dedicated animation/rendering; ocean volume remains accounted for |

A standalone SCA skeleton is usually a parent-linked tree. It cannot
represent bidirectional flow, reservoir storage, loops, backwater or
temporarily reversed currents without a separate hydraulic graph and solver.
A waterfall or flowing river does not “consume” attraction points as water
passes through it. Attraction targets are geometry-generation parameters,
**not** actual packets of water or an inventory of fluid.

## 3. Fundamental conservation contract

For each authoritative water compartment `i`:

```text
volume_new[i] = volume_old[i]
              + Σ(inbound accepted transfers)
              - Σ(outbound accepted transfers)
              + explicit phase-transfer / boundary bookkeeping
```

For every internal liquid exchange between compartments `i` and `j`,
**book one transfer once**, with `-q` to donor and `+q` to recipient.
Don't compute a separate possibly inconsistent transfer from either side.
Flux timestep and donor-capacity limits must prevent negative water.
Use stable edge ordering; if several edges request more than the donor
holds, allocate/scale consistently rather than clamping independently.

Planetary inventory invariant:

```text
total_water_mass =
  oceans + rivers + lakes + groundwater + soil + snow/ice
  + atmosphere/clouds + organisms + player-stored water + other reservoirs
```

Evaporation, rain, melting, infiltration, pumping and chunk unloading are
**signed transfers** between compartments, not creates/deletes. If models
use volumes instead of mass, handle ice/water/vapor density conversion
explicitly so a phase change doesn't break mass conservation.

The gameplay prototype's actual geometry uses 1m *across flats*, 1m
high regular hexagonal prisms. Its full hexel cross-sectional area is
`sqrt(3)/2 m² ≈ 0.8660254 m²`, so a completely unobstructed 1m-high
cell has `≈ 0.8660254 m³` of space, and a 0.5m column `≈ 0.433m³`.
These are *geometric capacities*, not a universal amount of water per cell.
Solid geometry and porosity change storage capacity. Six lateral hex faces
plus top/bottom neighbor relations are defined in `src/world.rs`; they
must not be treated as cube faces.

For an unobstructed open water column at bottom elevation `z`,
surface level `η = z + h`, water volume `V = A*h` where `h ≥ 0`.
Neighbor flux is governed by potential/head and hydraulic resistance,
not a direction toward randomly sampled attractors. When open connected
water rests, surface levels equalize, **not** water depths. At a shared
wall higher than either surface, no overflow occurs. Hydrostatic column
flow differs from a fully submerged pressurized pipe/cave, which needs
pressure-head physics and can flow contrary to local bed slope.

A candidate simple first validation fixture can exchange bounded volumes
between two adjacent surface hexel columns using a symmetric
water-surface/head-based flux. It is **not** a production-grade wave solver:
do not promise correct momentum or dam-break behavior from it.
For active fast runoff or floods, select and validate a conservative
finite-volume shallow-water formulation and stable CFL timestep.
For 3D caves, trapped air, siphons and submerged pressure passages, use
a separate limited 3D/cavern hydraulic model or explicitly document
gameplay simplifications.

## 4. Basins and leveling: stronger match than SCA

For finite surface water over terrain, *Fill–Spill–Merge* and the related
depression hierarchy are a better fit for large, mostly steady waterbodies:

1. Store a basin's bottom geometry, storage curve `V(level)` and spill sill.
2. Route prescribed, finite runoff into the basin and subtract it from
   upstream compartments.
3. Invert the monotonic storage curve to obtain surface level.
4. Once the sill is reached, send only excess water into the connected
   neighbor; merge when the connecting sill is submerged.
5. Keep the overall water volume and downstream recipient explicit.

Unlike a DEM algorithm that preemptively fills all depressions to create
continuous drainage paths, **retain real ponds/lakes**. In Hexal, lake water
might never reach the spill sill. The player could dig a channel, mine
through a divider, build a dam or pump some water away, changing both
topology and water levels. Do not mistake offline priority-flood terrain
conditioning for physically adding water.

On Hexal's **finite flat-coordinate torus**, there is no true world *edge*
for a conventional watershed algorithm to drain out of. Define stable
ocean/base-level reservoirs and periodic drainage connectivity, and seed
watershed routing accordingly. Every spill into the ocean becomes a
**balanced ledger transfer to ocean storage**, never water removed from
the simulation. Regional and local water quantities must be reconciled
during chunk streaming.

## 5. Scale-specific execution boundary

```text
World creation:
  periodic tectonic surface + climate + basin masks + fractures
    -> terrain-constrained drainage / depression hierarchy
    -> optional SCA side-channel / conduit STRUCTURE proposals
    -> geological/hydrological validation and immutable geography data

Gameplay:
  GLOBAL: ocean + atmosphere + phase/reservoir mass ledger
    ⇅ balanced precipitation, evaporation and transfers
  REGIONAL: basins, lakes, rivers, aquifers and stream-network flow
    ⇅ reconciled reservoir ↔ local flux boundaries
  LOCAL: active hexel surface volumes, horizontal face fluxes,
         vertical seepage / pumps / cavern water where applicable
    -> renderer reads depth, flow direction, turbulence/wetness
```

This is compatible with the currently documented hierarchy in
`docs/world-generation.md`, but **none of these solvers are implemented**
in the current one-chunk gallery. `src/space_colonization.rs` stays a
headless research graph generator, not a new simulation authority.

## 6. First validation slice and exact tests

Start **without SCA** so the finite-water rules are proven independently:

1. **Two connected hexel columns:** transfer an exact volume until their
   surfaces match; verify no negative volume, no mass loss, and correct
   difference between equal depth vs equal elevation.
2. **Barrier / spill sill:** hold water behind a higher wall, increase
   finite volume, detect overflow only at sill elevation; dig the wall
   and ensure exactly the held inventory redistributes.
3. **Two-level basin hierarchy:** water fills an inner depression before
   the neighboring one, merges when level crosses connecting sill; no
   spontaneous filling of dry basins.
4. **Three or more recipients:** donor requests exceed supply; assert
   total accepted outgoing volume ≤ donor volume and stable distribution.
5. **Unload / reload:** transfer exact authoritative water across a
   local→regional→local boundary without a recreated waterfall source.
6. **Toroidal seam:** validate periodic field routing only after
   hex-lattice seam translations are decided; test no duplicate or
   dropped edge exchange.
7. **Global accounting:** precipitation, evaporation, pumping, freezing,
   soil exchange and ocean transfers conserve mass over repeated ticks.
8. **Optional SCA assessment:** propose fracture-constrained cave conduit
   edges, convert to a bidirectional hydraulic graph, pass fixed volumes
   through them using pressure/head and flow capacity. Compare the
   resulting appearance and spatial performance against a simpler
   fracture spline/terrain routing; reject SCA if it adds complexity
   without a measurable improvement.

Do not integrate full planet runoff and 3D caves in the first pass.
A tiny headless solver with water budgets, directed temporary fluxes and
well-balanced resting-water tests is the right next technical proof,
**separate from the current ore/vegetation SCA draft PR**.

## Recommendation

Keep **shared graph/spatial primitives** potentially reusable for water
conduits, cave passages, veins, roots and other networks. Keep the **water
solver, finite inventory, hydrostatic leveling, river discharge and phase
transfers** independently authoritative. For rivers, derive principal
drainage from slopes and reservoirs rather than SCA appearances. For
regional lakes, investigate volume-preserving Fill–Spill–Merge. For
active near-player flooding, research a modest mass-conserving
finite-volume or zero-inertia/diffusion-wave hex-face solver before
considering full shallow-water dynamics. Treat underground SCA-assisted
conduits as an experimental second consumer, not a shipping promise.
