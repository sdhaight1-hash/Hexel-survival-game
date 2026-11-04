# Production material registry and legacy references

**Current catalogs:** `catalog.v1.json` is the historical 32px prototype.
`catalog.v2.json` is the **new authoring inventory of all 55 source
identities**. Three original exploratory artwork entries (`rock.granite`,
`rock.shale`, `ore.native_copper`) are `candidate` for the F8 gallery,
and 52 remain `needs_art`. The three candidate images were revised in P2.1
at 256×256, with new granite grain, shale bedding and copper inclusion art.
**None is approved** as production art.

The v2 material pipeline accepts RGB albedo and transparent overlays stored
as **RGBA8 PNG**, allows independent rectangular source image dimensions
(multiples of four for offline BC7), and specifies physical repeat scale in
metres. Materials have stable IDs and either:
- solid hexel top/side/bottom face recipes; or
- top/side/bottom overlay recipes for ground cover or ore inclusions, with
  the geological host chosen independently by the world.

Read [P1 material pipeline](../../docs/p1-material-pipeline.md) for the
specific schema, authoring examples, commands and limitations. Read
[material authoring contract](../../docs/material-authoring-contract.md)
for geology and vegetation state ownership.

```powershell
cargo run --bin material-baker -- validate
cargo run --release --bin material-baker -- bake
```

The v1 validation remains separate:
`python3 scripts/validate_material_catalog.py`. No new art may silently
replace original source files under `assets/textures/`.
