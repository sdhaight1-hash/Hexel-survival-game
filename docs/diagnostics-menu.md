# F1 developer diagnostics menu (prototype)

**Status:** an in-game clickable diagnostics panel for the existing
material/mesh benchmark controls. This is **not** a player options
menu, inventory, game pause, new material pipeline, graphics profiler,
or a general GUI toolkit.

## Reference and decision

The older [Hexel-Game-Engine](https://github.com/sdhaight1-hash/Hexel-Game-Engine)
repository has useful **custom wgpu HUD / hotbar** logic in
`src/renderer.rs` around its HUD vertex generation and crosshair
overlay. That prototype uses hand-generated screen-space vertices;
its inventory was still labeled `UI pending`, and it does **not**
supply a complete clickable options screen to transplant.

Hexal Survival follows that small **screen-space overlay** principle,
with its own deliberately simple input hitboxes and bitmap text.
No engine source, production assets or menu images are copied. No
`egui`, `imgui`, lockfile or new runtime dependencies are required
by this first experiment. Revisit a mature GUI library when we have
real inventory, tooltips, scrollable asset browsers, keyboard focus,
scaling/localization or accessibility requirements. A bitmap debug
font should **not** be interpreted as the final game UI style.

## Usage

- Press **F1** to open the developer menu and release the mouse.
- Click a row to change the same runtime setting as its displayed F-key.
- **F1 or Esc** closes the menu and recaptures first-person mouse look.
- **F3, F4, F6, F7, F8, F9, F10 and F11** remain working hotkeys.
- The menu shows the current value directly from the renderer / Game:
  F3 log ON/OFF; F4 nearby culling or N/A; F6 UV checker ON/OFF;
  F7 64/128/256 texels/metre; F8 materials ON/OFF; F9 Dry/Wet/
  Moss/Frost/Combined; F10 metal-glint ON/OFF; F11 gray/color.
- Disabled F4 in the one-chunk scene and disabled F8/F9/F10/F11
  when source art is unavailable cannot be activated by clicking.
  F11 requires **F8 ON** and a compatible copper source. F9/F10
  can prepare a preview state with F8 off, preserving the existing
  hotkey contract; the state only renders when F8 is selected.
- Clicks outside actionable rows do not accidentally recapture the
  cursor or change the world. Cursor move changes a row highlight.

**Important:** Opening the menu stops directional input and mouse
look but **does not pause the game's simulation clock or gravity**.
No terrain/edit/resource state is changed by the menu. For benchmark
runs, **close the menu before timing** so the debug overlay itself
does not distort the frame results.

## Implementation boundaries

- `src/debug_menu.rs`: single pure layout/hit-test/vertex generator
  and view-only snapshot type; 5×7 fixed bitmap debug glyphs.
- `src/debug_menu.wgsl`: simple 2D alpha-blended color triangles.
- `src/main.rs`: one `apply_debug_action` path for hotkeys and clicks;
  F1 and Esc mouse capture flow; current hovered/clicked row.
- `src/render.rs`: one separate overlay pipeline. A vertex buffer
  is cached until menu state, hovered row or window size changes;
  closed-menu terrain rendering is otherwise unchanged.
- No change to `World`, materials, textures, vertex mesh recipes,
  saved IDs, ecology, time progression, or gameplay behavior.

## First native Windows run — observed behavior (2026-11-03)

The project owner supplied a console trace from the **200-chunk**
`feature/diagnostics-menu` session. The visible record supports the
following outcomes:

- Scene initialized **200 resident chunks**, **1,960,656 mesh
  vertices**, **67.3 MiB** resident vertex buffers, and
  **61.4 ms** reported CPU build/upload time.
- `F1 diagnostics menu: OPEN` appeared; afterward F8, F11,
  F3, F6, F4, F7, F9 and F10 actions all logged expected state
  transitions, including multiple changes to the color/grayscale
  copper comparison and the full surface-preset cycle.
- `F1 diagnostics menu: CLOSED` appeared; subsequent chunk
  submission counts varied with player position, consistent with
  resumed first-person movement and distance culling.
- F4 switched between **200/200** submitted chunks
  (**1,960,656 vertices**) and **132/200** submitted chunks
  (**1,281,522 vertices**) at the initial position, matching
  the previous baseline. After returning to movement, nearby
  submitted counts ranged around **129–132**.
- Most FPS samples remained around **59.6–60.8 FPS** with
  **~16.2–16.6 ms CPU draw/acquire/present** in the log, whether
  F4 or F11 was selected. These numbers remain **V-sync limited**
  and do not isolate the cost of the UI, the copper shader,
  or the GPU.

**Interpretation:** this is a successful **native functional smoke
test**: no crash or unexpected error is shown, and the diagnostic
actions drive visible console state changes while F1 is open and
after it closes. However, the console cannot distinguish a mouse
click from the matching keyboard shortcut. No screenshot, recorded
mouse hitbox position, resize behavior, 1-chunk disabled-row review,
or explicit keyboard-vs-click equivalence test was provided, so
these remain **visual/input acceptance gates**, not approved features.

**Benchmark caution:** F3 logging was enabled with the menu open for
much of this trace, and multiple material/overlay modes changed
within measurement windows. Do **not** treat the nearly identical
V-sync-limited FPS as proof that the menu is free or that copper
material shaders have equal GPU cost. Close F1 for valid frame-time
comparisons, then use an opt-in uncapped presentation mode and,
where supported, actual GPU timestamps in a separate slice.

## Acceptance checks

1. Both the default one-chunk and `--benchmark-200` scenes launch,
   with F1 menu initially closed.
2. Open F1, click every available row, verify matching console
   feedback and the value displayed; verify the hotkey for the
   same row produces the same state transition.
3. Toggle F8 then F11 (disabled when F8 is off); cycle F9 fully;
   test F4 N/A in one-chunk mode and available in 200 chunks.
4. Esc closes the menu and returns mouse movement; clicking the
   viewport while the menu is open does **not** move the player
   or unexpectedly capture the cursor.
5. Resize the window, mouse-hit rows still correspond to visible
   rows and text remains readable on typical desktop sizes.
6. Keep F3 benchmarking off or close the menu for repeatable
   F8/F11/F4 measurements. The panel is not an on-screen GPU
   profiler and does not remove FIFO/V-sync masking.

This is an interface convenience while the team adds proper GPU
benchmark instrumentation and real material-family candidates.
