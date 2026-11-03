//! P0 reference artwork and face selection for the existing 16³ demo.
//!
//! These original deterministic 128px swatches are **visual test fixtures**,
//! not the approved geology/flora texture library and not geological authority.
//! The existing 55 atlas-derived images remain untouched in assets/textures.
//! Composite masks are generated independently then baked into GPU layers.
//! Runtime texture-array dimensions here are a *prototype choice*, not the
//! final material-format requirement (see docs/material-format-v2.md).

use hexel_survival_game::{
    material_contract::{CoverDensity, HexFace},
    material_lab::{sample, LabSample, RockHost},
    world::{Block, CellPos},
};

pub const TEXTURE_SIZE: u32 = 128;
pub const REFERENCE_TEXELS_PER_METRE: f32 = 128.0;

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
}

pub const LAYER_COUNT: usize = 12;

pub fn for_face(block: Block, cell: CellPos, face: HexFace) -> Material {
    let is_top = face == HexFace::Top;
    let is_side = matches!(face, HexFace::Side(0..=5));
    if let Some(fixture) = sample(cell.q, cell.r) {
        return match fixture {
            LabSample::Soil(cover) => soil_face(cover, is_top, is_side),
            LabSample::Rock { host: RockHost::Granite, show_copper: true } => Material::GraniteCopper,
            LabSample::Rock { host: RockHost::Granite, show_copper: false } => Material::Granite,
            LabSample::Rock { host: RockHost::Shale, show_copper: true } => Material::ShaleCopper,
            LabSample::Rock { host: RockHost::Shale, show_copper: false } => Material::Shale,
        };
    }
    match block {
        Block::Grass => soil_face(CoverDensity::Full, is_top, is_side),
        Block::Soil => Material::SoilBare,
        Block::Rock => Material::Granite,
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

fn random(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9e37_79b9)
        ^ (y as u32).wrapping_mul(0x85eb_ca6b)
        ^ seed.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

/// Periodic value noise. All generated base/overlay materials tile at 1m.
fn noise(x: f32, y: f32, grid: i32, seed: u32) -> f32 {
    let px = x * grid as f32;
    let py = y * grid as f32;
    let ix = px.floor() as i32;
    let iy = py.floor() as i32;
    let mut tx = px.fract();
    let mut ty = py.fract();
    tx = tx * tx * (3.0 - 2.0 * tx);
    ty = ty * ty * (3.0 - 2.0 * ty);
    let v = |dx: i32, dy: i32| {
        random((ix + dx).rem_euclid(grid), (iy + dy).rem_euclid(grid), seed)
    };
    let a = v(0, 0) * (1.0 - tx) + v(1, 0) * tx;
    let b = v(0, 1) * (1.0 - tx) + v(1, 1) * tx;
    a * (1.0 - ty) + b * ty
}

fn color(r: f32, g: f32, b: f32) -> [u8; 4] {
    [
        r.clamp(0.0, 255.0).round() as u8,
        g.clamp(0.0, 255.0).round() as u8,
        b.clamp(0.0, 255.0).round() as u8,
        255,
    ]
}

fn make_tile(mut pixel: impl FnMut(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            data.extend_from_slice(&pixel(x, y));
        }
    }
    data
}

fn soil_pixel(x: u32, y: u32) -> [u8; 4] {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let coarse = noise(u, v, 7, 11) - 0.5;
    let fine = noise(u, v, 29, 12) - 0.5;
    let grain = random(x as i32, y as i32, 13) - 0.5;
    let n = coarse * 61.0 + fine * 34.0 + grain * 22.0;
    let pebble = random(x as i32 / 3, y as i32 / 3, 14) > 0.978;
    let s = if pebble { 17.0 } else { 0.0 };
    color(97.0 + n + s, 69.0 + n * 0.75 + s, 45.0 + n * 0.55 + s)
}

fn granite_pixel(x: u32, y: u32) -> [u8; 4] {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let n = (noise(u, v, 9, 31) - 0.5) * 48.0
        + (noise(u, v, 39, 32) - 0.5) * 36.0;
    let fleck = random(x as i32 / 2, y as i32 / 2, 33);
    if fleck > 0.88 {
        color(155.0 + n, 143.0 + n, 129.0 + n)
    } else if fleck < 0.12 {
        color(64.0 + n, 71.0 + n, 76.0 + n)
    } else {
        color(117.0 + n, 120.0 + n, 117.0 + n)
    }
}

fn shale_pixel(x: u32, y: u32) -> [u8; 4] {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let warp = noise(u, v, 5, 41) - 0.5;
    let laminations = ((v * 21.0 + warp * 1.9) * std::f32::consts::TAU).sin() * 12.0;
    let n = (noise(u, v, 25, 42) - 0.5) * 29.0 + laminations;
    color(84.0 + n, 91.0 + n, 91.0 + n * 0.92)
}

fn composite(base: &mut [u8], x: u32, y: u32, pigment: [u8; 3], opacity: f32) {
    let i = ((y * TEXTURE_SIZE + x) * 4) as usize;
    let a = opacity.clamp(0.0, 1.0);
    for c in 0..3 {
        base[i + c] = (base[i + c] as f32 * (1.0 - a) + pigment[c] as f32 * a)
            .round() as u8;
    }
}

fn grass_alpha(x: u32, y: u32, density: CoverDensity) -> f32 {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let field = 0.6 * noise(u, v, 9, 51)
        + 0.3 * noise(u, v, 31, 52)
        + 0.1 * random(x as i32, y as i32, 53);
    let threshold = match density {
        CoverDensity::Bare => return 0.0,
        CoverDensity::VerySparse => 0.65,
        CoverDensity::Sparse => 0.49,
        CoverDensity::Full => 0.31,
    };
    ((field - threshold) * 8.0).clamp(0.0, 0.96)
}

fn grass_pigment(x: u32, y: u32) -> [u8; 3] {
    let shade = random(x as i32 / 2, y as i32 / 2, 54);
    [
        (54.0 + shade * 32.0) as u8,
        (107.0 + shade * 47.0) as u8,
        (37.0 + shade * 27.0) as u8,
    ]
}

fn grass_top(soil: &[u8], density: CoverDensity) -> Vec<u8> {
    let mut output = soil.to_vec();
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            composite(&mut output, x, y, grass_pigment(x, y), grass_alpha(x, y, density));
        }
    }
    output
}

fn grass_side(soil: &[u8], density: CoverDensity) -> Vec<u8> {
    let mut output = soil.to_vec();
    for y in 0..TEXTURE_SIZE {
        let depth_metres = y as f32 / TEXTURE_SIZE as f32;
        for x in 0..TEXTURE_SIZE {
            // Transparent grass coverage is applied only at the UPPER rim.
            let u = x as f32 / TEXTURE_SIZE as f32;
            let edge_depth = 0.09 + 0.075 * noise(u, 0.0, 8, 55);
            let edge_mask = ((edge_depth - depth_metres) / 0.045).clamp(0.0, 1.0);
            let top_density = grass_alpha(x, 16, density);
            let alpha = top_density * edge_mask;
            composite(&mut output, x, y, grass_pigment(x, y), alpha);
        }
    }
    output
}

fn copper_alpha(x: u32, y: u32) -> f32 {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let vein = (std::f32::consts::TAU
        * (u * 3.0 - v * 2.0 + 0.10 * noise(u, v, 6, 61)))
        .sin()
        .abs();
    let pocket = noise(u, v, 12, 62);
    ((0.16 - vein) * 5.0).clamp(0.0, 0.9) * 0.75
        + ((pocket - 0.69) * 3.0).clamp(0.0, 0.65)
}

fn ore_composite(rock: &[u8]) -> Vec<u8> {
    let mut output = rock.to_vec();
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let green = random(x as i32 / 3, y as i32 / 3, 63) > 0.86;
            let pigment = if green { [79, 139, 99] } else { [206, 111, 48] };
            composite(&mut output, x, y, pigment, copper_alpha(x, y));
        }
    }
    output
}

fn checker_pixel(x: u32, y: u32) -> [u8; 4] {
    if x % 16 == 0 || y % 16 == 0 {
        color(32.0, 33.0, 36.0)
    } else if (x / 16 + y / 16) % 2 == 0 {
        color(230.0, 219.0, 179.0)
    } else {
        color(79.0, 105.0, 134.0)
    }
}

/// Upload these layers as one small 128² RGBA array, in Material enum order.
/// The source surfaces/overlays remain independently generated before baking.
pub fn generate_textures() -> Vec<Vec<u8>> {
    let soil = make_tile(soil_pixel);
    let granite = make_tile(granite_pixel);
    let shale = make_tile(shale_pixel);
    let result = vec![
        soil.clone(),
        grass_top(&soil, CoverDensity::VerySparse),
        grass_top(&soil, CoverDensity::Sparse),
        grass_top(&soil, CoverDensity::Full),
        grass_side(&soil, CoverDensity::VerySparse),
        grass_side(&soil, CoverDensity::Sparse),
        grass_side(&soil, CoverDensity::Full),
        granite.clone(),
        shale.clone(),
        ore_composite(&granite),
        ore_composite(&shale),
        make_tile(checker_pixel),
    ];
    debug_assert_eq!(result.len(), LAYER_COUNT);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_p0_textures_have_exactly_one_rgba_layer_each() {
        let layers = generate_textures();
        assert_eq!(layers.len(), LAYER_COUNT);
        assert!(layers.iter().all(|pixels| pixels.len() == (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize));
    }

    #[test]
    fn bottom_never_receives_grass_but_side_does() {
        let cell = CellPos::new(9, 10, 2);
        assert_eq!(for_face(Block::Grass, cell, HexFace::Top), Material::SoilTopFull);
        assert_eq!(for_face(Block::Grass, cell, HexFace::Side(0)), Material::SoilSideFull);
        assert_eq!(for_face(Block::Grass, cell, HexFace::Side(5)), Material::SoilSideFull);
        assert_eq!(for_face(Block::Grass, cell, HexFace::Bottom), Material::SoilBare);
    }

    #[test]
    fn identical_copper_mask_changes_two_distinct_host_rocks() {
        let layers = generate_textures();
        let g = &layers[Material::Granite as usize];
        let s = &layers[Material::Shale as usize];
        let cg = &layers[Material::GraniteCopper as usize];
        let cs = &layers[Material::ShaleCopper as usize];
        assert_ne!(g, s);
        assert_ne!(cg, cs);
        let changed = |a: &[u8], b: &[u8]| {
            a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(x, y)| x != y).count()
        };
        assert!(changed(g, cg) > 100);
        assert!(changed(s, cs) > 100);
        // No ore-only texture can silently replace host-rock identity.
        assert!(changed(cg, cs) > 100);
    }

    #[test]
    fn grass_cover_is_denser_in_order_and_side_rim_is_not_bottom_grass() {
        let layers = generate_textures();
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
        assert_eq!(&base[side.len() - row_bytes..], &side[side.len() - row_bytes..]);
    }
}
