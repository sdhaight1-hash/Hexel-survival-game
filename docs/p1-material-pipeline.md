# P1 material registry and offline baker

**Implementation status:** P1 *authoring/validation and independent BC7 source baking*
foundation. **Not** a runtime v2 material loader, full 55-material visual
replacement, vegetation renderer, P1 completion claim, or new approved artwork.
The P0 16³ scene, F6/F7 controls, 12-layer compressed gallery and original
55 reference PNGs remain unchanged.

## First inventory: 55 original references, zero approved replacements

The new `assets/materials/catalog.v2.json` inventories every source from
`assets/textures/manifest.json`, exactly once:

| Class | Pending identities | Representation when new art is ready |
| --- | ---: | --- |
| Soil | 5 | Solid with top/side/bottom face assignments |
| Forest floor | 1 | Organic *ground cover*, not an intrinsic soil fertility |
| Rock | 23 | Solid; oriented strata and appropriate cap/side textures |
| Ore | 16 | Transparent inclusions over the **actual** host rock |
| Minerals | 10 | Either solid strata or overlaid seams/crystals, explicitly selected |
| **Total** | **55** | **All currently `needs_art`** |

No source is silently reclassified as production-ready. IDs such as
`soil.low_fertility`, `rock.granite`, `ore.native_copper` and
`cover.forest_floor` are stable logical identities, unrelated to GPU
array offsets, texture filenames or the amount of mineable material.

The 208 flora research records remain **research-only**. Once species have
approved visuals, flora assets will reference stable IDs in a separate
flora-to-visual binding, never substitute a generic soil texture.

## Authoring contract

Each *newly authored* source texture is registered under
`assets/production/...` with an ID, file path, usage, color-space,
wrap mode and physical `repeat_metres: [x,y]`. This allows **64×128,
256×256, 512×256, etc.**, without a universal square resolution.

- Production inputs must be valid 8-bit **RGBA PNGs**, dimensions multiples
  of 4 for BC7, within a provisional 4..4096px limit.
- Physical repeat is in metres; a 1-m-tall hexel has **0.57735-m sides**.
  Source dimensions do *not* determine how wide the hexel is.
- Albedo is color data; transparent overlay textures are separate assets.
  If a base and overlay share one face, their `repeat_metres` must match,
  even if the two images have different pixel dimensions.
- For a **solid** soil/rock/mineral material, `faces` must define
  `top`, `side`, `bottom`, each with a base ID and optional overlay ID.
  The side face represents all six sides of a regular hexagonal prism.
- For a **cover** or **ore** overlay material, `overlays` names a top
  overlay and optional side/bottom overlays. It never embeds a host-rock ID
  or ore quantity. The actual geological host comes from game state.
- `needs_art` records are required to have **null** representation, face and
  overlay recipes. Only `candidate` or `approved` records may bind new
  production artwork. Approval is an authoring decision, not implied by
  passing a file/header check.
- Production asset paths are restricted to `assets/production/*.png`
  below the repository, with no path traversal or absolute paths.

A **future** candidate material definition (illustrative; these art files
do not exist yet):

```json
{
  "id": "soil.low_fertility",
  "kind": "soil",
  "status": "candidate",
  "legacy_reference": "assets/textures/soil/low_fertility_soil.png",
  "representation": "solid",
  "faces": {
    "top": {"base": "soil.low_fertility_albedo", "overlay": "cover.grass_top"},
    "side": {"base": "soil.low_fertility_albedo", "overlay": "cover.grass_side"},
    "bottom": {"base": "soil.low_fertility_albedo", "overlay": null}
  },
  "overlays": null
}
```

Ground cover *density*, grass species and seasonal variation are not
implicitly stored in that soil ID. The example merely illustrates a face
assignment that an eventual renderer may select based on real state.

## Commands

From the game repository root:

```powershell
cargo run --bin material-baker -- validate
cargo run --release --bin material-baker -- bake
```

The first command checks all 55 references against the original inventory,
state/role validity, paths, actual RGBA PNG data and physical repeat fields.
The second produces a **deterministically ID-sorted** set of individual
`target/materials-v2/<texture-id>.bc7` files and an `index.json`
manifest for registered candidate/approved source textures **and** their
stable material recipes. Each exported material recipe retains its
`id`, `kind`, `status`, `representation`, and explicit top/side/bottom
face or transparent-overlay texture IDs. Pending `needs_art` records are
excluded from the runtime-facing recipe list without losing them from
the authoring catalog.

**Currently there are no registered production texture assets.** The
bake command therefore reports that no new art is available and writes an
**empty v2 index** without manufacturing replacement images.

As soon as one new material source is authored, the same tool can compress
rectangular or square color/overlay inputs with a **BC7 encoder**. It decodes
the compressed blocks to verify the byte count and RGB quality (25 dB
minimum PSNR), and records image dimensions, color space, wrapping,
physical repeat and compressed size in the output index.

### Automated tests

`cargo test --all-targets` includes a **real BC7 offline-bake test**
with synthetic 64×128 and 256×256 RGBA inputs and a strict 55-source
inventory test. GitHub Actions also runs
`cargo run --quiet --bin material-baker -- validate` on each push/PR.

The original `scripts/validate_material_catalog.py` still checks the
historical 32px gallery. It is not a production-resolution rule.

## Why P0 remains separate for now

The current playable scene uses `src/materials_p0.rs` and an embedded
12-layer BC7 pack with a native GPU compressed path and CPU RGBA fallback.
The P1 tool writes **individual** textures with different dimensions;
those cannot be placed in one fixed-size GPU array unchanged.

A subsequent **separate, testable renderer slice** must consume this v2
index (which now contains the necessary material-to-texture face mappings),
group assets by compatible texture formats/dimensions or use
atlases, and map stable material/overlay IDs to GPU resources. Only then
should we replace the P0 gallery's hardcoded material selection.

Do not conflate an offline BC7 file with a streamed GPU texture or a
persisted geological resource: the files are generated caches.

## Next acceptance gates

1. Author **one** approved-quality soil base with top/side transition and
   a grass-coverage overlay, in `assets/production/`.
2. Author granite and shale base surfaces plus a genuine **ore-only**
   transparent native-copper overlay.
3. Register them as candidates in v2 and bake, inspect BC7 quality and all
   eight actual hex-prism faces in-game. Only then approve the artwork.
4. Integrate **stable v2 IDs** with the runtime using a backward-compatible
   fallback to the current P0 materials; avoid editing world geology.
5. Complete the remaining 55-material re-authoring list and independent
   flora visuals, then add streaming/mips/LOD under master
   [issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14).

See [material authoring contract](material-authoring-contract.md),
[flora-to-assets](flora-to-assets.md) and
[texture compression](texture-compression.md).
