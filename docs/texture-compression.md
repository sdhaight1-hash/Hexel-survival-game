# Texture compression pipeline — BC7 reference implementation

**Implemented:** build-time BC7 compression and runtime hardware/CPU decompression
for all 12 P0 art-derived gallery layers. This is the compression foundation
for later versioned material compilation, **not** a finished cross-platform
production asset packer or P1 format migration.

## The full path

```text
high-resolution legacy PNG crops, unchanged (5)
    ↓ offline build.rs (on asset/code changes, not game launch)
seam-reduced 256x256 color textures + grass/ore composites (12 RGBA8 layers)
    ↓ CPU BC7 encoder, Balanced quality, independent CPU decode/PSNR check
embedded versioned HXB7 pack (4x4 blocks, 16 bytes/block)
    ├── GPU supports TEXTURE_COMPRESSION_BC → Bc7RgbaUnormSrgb texture array
    │                                   → GPU samples/decompresses in hardware
    └── otherwise → once-only bcdec_rs CPU decode → Rgba8UnormSrgb texture array
                                             → GPU samples ordinary RGBA
```

The **GPU compressed path never decompresses the entire image on CPU**.
The adapter must advertise `wgpu::Features::TEXTURE_COMPRESSION_BC`; only
then do we request the feature, create a BC7 texture array and upload
encoded blocks. Other adapters get the exact same BC7-compressed artwork
decoded once to RGBA8 before upload. There is no per-frame CPU decode.

## Memory and format accounting

For twelve **256x256** layers, before mipmaps:

| Item | Bytes | MiB |
| --- | ---: | ---: |
| RGBA8 (uncompressed GPU fallback) | 3,145,728 | 3.00 |
| BC7 (GPU-compressed texture) | 786,432 | 0.75 |
| **Native BC7 GPU memory reduction** | **75%** | **2.25 saved** |

BC7 uses exactly 16 bytes per 4x4 pixel block, including RGBA capability.
The entire pack includes a small 20-byte header. Compression is **lossy**.
Build time validates a minimal 25 dB worst-case per-layer RGB PSNR against
the original pre-compression RGBA; this is a low sanity gate, **not**
production visual approval. Actual quality must be reviewed in the native
Windows game at close first-person distances and at seams.

No disk-download or total binary-size reduction is claimed: source PNGs are
already compressed, and the BC7 pack is optimized specifically for **GPU
residency and bandwidth**, not the smallest possible on-disk archive. Offline
packing and compression can increase `cargo build` time when assets change;
normal game startup no longer runs the expensive PNG decode, seam blending,
mask generation or BC7 encode.

## Build and validation

```text
cargo test --all-targets
cargo run --release
```

On compilation, `build.rs` bakes the pack to Cargo's `OUT_DIR` using
`texture-compressor`. It prints size and quality information and fails
the build if block sizes, decode or minimum PSNR validation fail.
The game binary embeds the baked pack, so it doesn't depend on finding
an external `assets/baked` directory on the user's PC.

The runtime `src/texture_pack.rs` checks the pack magic/version, dimensions,
layer count and total length before any GPU upload. The reusable
`src/texture_codec.rs` validates BC7 block dimensions/byte counts and
implements CPU decompression. Automated tests exercise corrupted pack
rejection, every embedded layer's actual CPU decode and the expected 4:1
residency ratio.

At game launch, the console reports **native BC7** or **CPU decode to RGBA
fallback**, and which GPU memory reduction is achieved on that adapter.
The visual F6 UV checker and F7 diagnostic remain unchanged.

## P1 offline production source baker

A separate [P1 material pipeline](p1-material-pipeline.md) now supports
registered *new* variable-resolution RGBA8 inputs (including rectangular
tiles). The `material-baker` tool performs BC7 encoding, independent
software decode/quality checks, and exports stable ID-named blocks plus a
JSON index to `target/materials-v2`. It is **not loaded by the P0 game**:
the existing embedded 12-layer P0 pack and native BC7/CPU fallback remain
untouched. No P1 production sources are approved yet.

## Design limits and next texture-compression work

- This only compresses **sRGB color/albedo** gallery layers; later roughness,
  normal, height and alpha-mask textures should use their appropriate
  *linear/data* formats (e.g. BC4/BC5/BC7), not silently reuse sRGB.
- **BC7 is desktop-first**. Future mobile devices may need ASTC or ETC2,
  with a capability-selected asset variant; the RGBA fallback is correct
  today but costs full uncompressed memory.
- A final v2 packer should write versioned external KTX2/Basis or suitable
  GPU-specific targets, stream packed material pages, and support different
  source resolutions and mip levels with padding. This P0 bake intentionally
  has **one mip level**, matching the previous scene, so it does not introduce
  an unrelated filtering/art-quality change.
- Material IDs, authoritative rock/ore/cover data and save formats do not
  depend on BC7 or transient GPU layer numbers. The 55 source art files and
  flora research registry are unmodified.

Sources: wgpu `TEXTURE_COMPRESSION_BC` and `Bc7RgbaUnormSrgb`,
Microsoft BC7 block format, and the texture-compressor/bcdec_rs Rust APIs.
See [material format v2](material-format-v2.md) and full texture program
[issue #14](https://github.com/sdhaight1-hash/Hexel-survival-game/issues/14).
