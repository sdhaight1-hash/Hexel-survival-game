//! P0 reference artwork and face selection for the existing 16³ demo.
//!
//! These source-art-derived 256px tiles are **offline-baked test fixtures**,
//! not the approved geology/flora texture library and not geological authority.
//! The existing 55 atlas-derived images remain untouched in assets/textures.
//! Four high-resolution source crops supply natural soil, granite, shale and
//! copper detail; derived tiles remove hard edge seams before GPU upload.
//! Composite masks are generated independently then baked into GPU layers.
//! Runtime texture-array dimensions here are a *prototype choice*, not the
//! final material-format requirement (see docs/material-format-v2.md).

use hexel_survival_game::{
    material_contract::{CoverDensity, HexFace},
    material_lab::{sample, surface_y, LabSample, RockHost},
    world::{Block, CellPos},
};

pub const TEXTURE_SIZE: u32 = 256;
pub const REFERENCE_TEXELS_PER_METRE: f32 = 256.0;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    SoilBare = 0,
    SoilTopVerySparse = 1,
    SoilTopSparse = 2,
    SoilTopFull = 3,
    SoilSideVerySparse = 4,
    SoilSideSparse = 5,
    SoilSideFull = 6,
    Granite = 7,
    Shale = 8,
    GraniteCopper = 9,
    ShaleCopper = 10,
    Checker = 11,
    DiagnosticLushGrass = 12,
    DiagnosticDryGrass = 13,
    DiagnosticSparseGrass = 14,
    DiagnosticBareDirt = 15,
    DiagnosticStonyDirt = 16,
    DiagnosticDarkShale = 17,
}

pub const LAYER_COUNT: usize = 18;

pub fn for_face(block: Block, cell: CellPos, face: HexFace) -> Material {
    let is_top = face == HexFace::Top;
    let is_side = matches!(face, HexFace::Side(0..=5));
    // Only the top displayed cell gets a surface-cover or ore swatch.
    // Underlying cells retain their real demo soil/rock appearance.
    if let Some(fixture) = sample(cell.q, cell.r).filter(|_| cell.y == surface_y(cell.r)) {
        return match fixture {
            LabSample::Soil(cover) => soil_face(cover, is_top, is_side),
            LabSample::Rock {
                host: RockHost::Granite,
                show_copper: true,
            } => Material::GraniteCopper,
            LabSample::Rock {
                host: RockHost::Granite,
                show_copper: false,
            } => Material::Granite,
            LabSample::Rock {
                host: RockHost::Shale,
                show_copper: true,
            } => Material::ShaleCopper,
            LabSample::Rock {
                host: RockHost::Shale,
                show_copper: false,
            } => Material::Shale,
        };
    }
    match block {
        Block::Grass => soil_face(CoverDensity::Full, is_top, is_side),
        Block::Soil => Material::SoilBare,
        Block::Rock => Material::Granite,
        Block::Shale => Material::Shale,
        Block::Gravel => {
            if is_top {
                Material::SoilTopVerySparse
            } else {
                Material::SoilBare
            }
        }
        Block::Air => unreachable!("Only solid blocks have faces"),
    }
}

fn soil_face(density: CoverDensity, top: bool, side: bool) -> Material {
    if top {
        match density {
            CoverDensity::Bare => Material::SoilBare,
            CoverDensity::VerySparse => Material::SoilTopVerySparse,
            CoverDensity::Sparse => Material::SoilTopSparse,
            CoverDensity::Full => Material::SoilTopFull,
        }
    } else if side {
        match density {
            CoverDensity::Bare => Material::SoilBare,
            CoverDensity::VerySparse => Material::SoilSideVerySparse,
            CoverDensity::Sparse => Material::SoilSideSparse,
            CoverDensity::Full => Material::SoilSideFull,
        }
    } else {
        // A covered soil block has an ordinary, ungrassed bottom.
        Material::SoilBare
    }
}

// Build-time art generation is deliberately excluded from shipped runtime code.
#[cfg(test)]
#[path = "materials_p0_art.rs"]
mod art;
#[cfg(test)]
use art::{generate_textures, N};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_derived_materials_decode_and_have_256px_rgba_layers() {
        let layers = generate_textures().expect("valid source PNGs");
        assert_eq!(layers.len(), LAYER_COUNT);
        assert!(layers
            .iter()
            .all(|pixels| pixels.len() == (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize));
    }

    #[test]
    fn source_derived_base_tiles_wrap_without_visible_hard_image_borders() {
        let layers = generate_textures().expect("valid source PNGs");
        for &layer in &[Material::SoilBare, Material::Granite, Material::Shale] {
            let image = &layers[layer as usize];
            let mut seam: u64 = 0;
            for i in 0..N {
                for c in 0..3 {
                    let left = image[(i * N) * 4 + c] as i32;
                    let right = image[(i * N + N - 1) * 4 + c] as i32;
                    let top = image[i * 4 + c] as i32;
                    let bottom = image[((N - 1) * N + i) * 4 + c] as i32;
                    seam += left.abs_diff(right) as u64 + top.abs_diff(bottom) as u64;
                }
            }
            let average_channel_delta = seam as f32 / (N * 6) as f32;
            assert!(
                average_channel_delta < 26.0,
                "{layer:?} source tile has an obvious image-wrap seam: {average_channel_delta}"
            );
        }
    }

    #[test]
    fn ore_masks_preserve_most_original_host_pixels() {
        let layers = generate_textures().expect("valid source PNGs");
        for (base, composite) in [
            (Material::Granite, Material::GraniteCopper),
            (Material::Shale, Material::ShaleCopper),
        ] {
            let unchanged = layers[base as usize]
                .chunks_exact(4)
                .zip(layers[composite as usize].chunks_exact(4))
                .filter(|(a, b)| a == b)
                .count();
            assert!(unchanged > N * N / 2);
        }
    }

    #[test]
    fn bottom_never_receives_grass_but_side_does() {
        let cell = CellPos::new(9, 10, 3);
        assert_eq!(
            for_face(Block::Grass, cell, HexFace::Top),
            Material::SoilTopFull
        );
        assert_eq!(
            for_face(Block::Grass, cell, HexFace::Side(0)),
            Material::SoilSideFull
        );
        assert_eq!(
            for_face(Block::Grass, cell, HexFace::Side(5)),
            Material::SoilSideFull
        );
        assert_eq!(
            for_face(Block::Grass, cell, HexFace::Bottom),
            Material::SoilBare
        );
    }

    #[test]
    fn identical_copper_mask_changes_two_distinct_host_rocks() {
        let layers = generate_textures().expect("valid source PNGs");
        let g = &layers[Material::Granite as usize];
        let s = &layers[Material::Shale as usize];
        let cg = &layers[Material::GraniteCopper as usize];
        let cs = &layers[Material::ShaleCopper as usize];
        assert_ne!(g, s);
        assert_ne!(cg, cs);
        let changed = |a: &[u8], b: &[u8]| {
            a.chunks_exact(4)
                .zip(b.chunks_exact(4))
                .filter(|(x, y)| x != y)
                .count()
        };
        assert!(changed(g, cg) > 100);
        assert!(changed(s, cs) > 100);
        // No ore-only texture can silently replace host-rock identity.
        assert!(changed(cg, cs) > 100);
    }

    #[test]
    fn appended_ground_reference_tiles_are_distinct_from_production_art() {
        let layers = generate_textures().expect("baked diagnostic ground tiles");
        assert_eq!(layers.len(), LAYER_COUNT);
        assert_eq!(LAYER_COUNT, 18);
        for layer in 12..18 {
            assert_eq!(
                layers[layer].len(),
                (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize
            );
            assert_ne!(
                layers[layer], layers[0],
                "diagnostic layer must not alias bare soil"
            );
            for previous in 12..layer {
                assert_ne!(
                    layers[layer], layers[previous],
                    "each diagnostic tile should be visually distinct"
                );
            }
        }
    }

    #[test]
    fn grass_cover_is_denser_in_order_and_side_rim_is_not_bottom_grass() {
        let layers = generate_textures().expect("valid source PNGs");
        let base = &layers[Material::SoilBare as usize];
        let coverage = |layer: Material| {
            base.chunks_exact(4)
                .zip(layers[layer as usize].chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count()
        };
        assert!(coverage(Material::SoilTopVerySparse) < coverage(Material::SoilTopSparse));
        assert!(coverage(Material::SoilTopSparse) < coverage(Material::SoilTopFull));
        let side = &layers[Material::SoilSideFull as usize];
        let row_bytes = (TEXTURE_SIZE * 4) as usize;
        assert_ne!(&base[..row_bytes], &side[..row_bytes]);
        assert_eq!(
            &base[side.len() - row_bytes..],
            &side[side.len() - row_bytes..]
        );
    }
}
