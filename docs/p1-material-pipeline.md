# P1 material registry and offline baker

**Implementation status:** v2 authoring/validation, independent BC7 baking,
read-only runtime index validation and a narrow opt-in F8 rock/ore visual
comparison are implemented. **Not** a general-purpose v2 renderer, full
55-material replacement, vegetation renderer, P1 completion claim, or
approved production artwork. The P0 16³ scene, existing F6/F7 controls,
embedded 12-layer BC7 gallery and original 55 reference PNGs remain intact.

## First inventory: 55 original references, zero approved replacements

The new `assets/materials/catalog.v2.json` inventories every source from
`assets/textures/manifest.json`, exactly once:

| Class | Identities | Candidate | Pending | Representation |
| --- | ---: | ---: | ---: | --- |
| Soil | 5 | 0 | 5 | Solid with top/side/bottom face assignments |
| Forest floor | 1 | 0 | 1 | Organic ground cover |
| Rock | 23 | 2 | 21 | Solid; oriented cap/side textures |
| Ore | 16 | 1 | 15 | Transparent inclusions over host rock |
| Minerals | 10 | 0 | 10 | Solid or overlaid seams/crystals |
| **Total** | **55** | **3** | **52** | **Zero approved** |

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

Three original **experimental** 256×256 RGBA image assets are registered:
granite, shale and an ore-only copper alpha overlay. They are deliberately
tagged `candidate`, **not `approved`**. The baker compiles these three
independently and retains the 52 other identities as `needs_art`.
The empty-index behavior still applies when a catalog has no candidates.

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

## Future family/palette extension — proposed, not in v2 schema

A **material-family recipe** is not limited to ores. If validated,
the same abstraction could bind compatible grayscale grain and
weathering detail for rocks, longitudinal ridge/flake patterns for
tree bark, internal pigment/vein detail for leaves, and condition
masks for ground cover. Each use case keeps its own coordinate frame,
geometry/opacity role, source provenance, palette/lighting response
and fallback to bespoke art. Do **not** force base albedo and
leaf-silhouette alpha through an ore-only `C/G` definition or one
shader entry point.

Flora species must refer to **stable approved asset/material IDs**
via the [flora authoring contract](flora-to-assets.md), never temporary
GPU array layers. Color-ramp season changes do not need new full
texture assets, but distinct leaf shapes, bark normals and growth
geometry still need real authored data. Establish the general v2
stable-ID material lookup before extending its schema.



**Decision for research:** compare curated **material/overlay families**
with reusable linear grayscale coverage and tonal-detail fields plus
mineral-specific palette and surface response. The current
`assets/materials/catalog.v2.json` still registers **three candidate
sources and 52 `needs_art` identities**; do **not** silently add
`family`, `palette` or `pattern` fields to its live schema. Its
validator currently rejects unknown fields, and the current baker
produces individual BC7 assets, not a portable array-package format.

A possible *later, versioned* extension would track:
- stable family ID; vetted variant IDs and weights/eligibility;
- optional single-channel **linear** coverage C and tonal detail G,
  or deliberately packed channels with explicit channel semantics;
- a material/species-specific palette ramp (not necessarily flat RGB tint),
  separate roughness/metallic/normal or height response;
- face role, physical repeat, deterministic region/feature selection
  and the provenance/approval of each variant;
- a fallback to a normal authored full-color overlay or existing
  v2 candidate when the representation is unsuitable.

A conceptual family is **not automatically a GPU array**. v2 supports
different source sizes; physical texture array layers require matching
dimensions, block format and mip layout. Candidate formats like BC4
for a scalar could reduce compressed storage from 16 bytes/4×4 in
BC7 to 8 bytes/4×4 in BC4, but BC4 baking/runtime loading is not
implemented. More shader samples and ramps can offset memory savings.
Compare with the existing BC7 gallery before choosing a new format.

**Do not change the catalog or shader yet.** First ship a small
documented comparison that demonstrates one grayscale copper
inclusion pattern over granite and shale, a 3–5-pattern controlled
family, explicit missing-asset fallback, and a second mineral with
distinctive non-color cues. Record actual texture bytes, shader
samples, native GPU timings, visual approval, deterministic reload
behavior and whether colorization harms mineral recognizability.
See [material contract](material-authoring-contract.md) and
[overlay architecture](overlay-visual-architecture.md).

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

## Runtime read-only integration

At game startup, the executable now checks for
`target/materials-v2/index.json` if a v2 bake exists. It validates the
manifest version, stable IDs, declared material face/overlay references,
BC7 dimensions, byte lengths and safe local filenames. A missing index is
expected until candidate artwork is authored; an invalid index is reported
without crashing or replacing the playable P0 material gallery.

**The startup check is not itself GPU binding.** A separate opt-in F8
candidate gallery now exercises three explicitly supported materials
through an independent RGBA preview array. The default renderer retains
the embedded P0 BC7 pack. The v2 check still detects broken caches before
the gallery preview is created.
Use `cargo run --release --bin material-baker -- bake` to create the v2
index; creating the file does not itself activate new textures.

## First in-game v2 candidate comparison (F8)

From the repository root, run the **offline bake** once after updating the
source art, then launch the game:

```sh
cargo run --release --bin material-baker -- bake
cargo run --release
```

Press **F8** to toggle the initial v2 candidate rock/ore view and the original
P0 gallery. P0 still powers the soil/grass fixture and remains available when
a candidate cache is missing, broken or unsupported. The F8 preview uses a
separate uncompressed 12-layer RGBA8 array to keep the original GPU-compressed
P0 array intact; production array packing is **not** solved by this fixture.

Supported v2 recipes in this first slice are only `rock.granite`,
`rock.shale` and `ore.native_copper`. The rock recipe must reuse one sRGB
albedo across top/side/bottom and have no per-face overlay. Copper must use
the same transparent top/side image. Textures may be independently sized, but
for this comparison each must be a 1m × 1m repeat. The preview resamples
each to the pre-existing 256px gallery slots. Unsupported recipes are reported
and the game remains on P0; **this limitation is not a constraint on the
general v2 material format**.

The copper-only alpha mask composites over whichever granite/shale base is
currently selected. This is a first-person visual QA fixture, not a simulated
ore occurrence or approved geological texture. Candidate pixels should be
evaluated at 1m, 3m and 8m under neutral light and in shadow before promoting
anything to `approved`. Neither the source artwork nor the art style has
passed that human review yet.

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
