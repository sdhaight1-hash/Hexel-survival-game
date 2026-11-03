//! Original high-resolution source-art processing for the offline P0 bake.
//! This module is used only by build.rs and pixel-quality tests; never per frame.

const TEXTURE_SIZE: u32 = 256;
const LAYER_COUNT: usize = 12;
#[derive(Clone, Copy)]
enum CoverLevel {
    Bare,
    VerySparse,
    Sparse,
    Full,
}

const SEAM_OVERLAP: usize = 40;
pub const N: usize = TEXTURE_SIZE as usize;

/// Decode an original (not downscaled) high-resolution reference image.
struct SourceImage {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

impl SourceImage {
    fn from_png(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
            .read_info()
            .map_err(|e| format!("Could not read reference PNG: {e}"))?;
        let mut data = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut data)
            .map_err(|e| format!("Could not decode reference PNG: {e}"))?;
        if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
            return Err("Reference textures must be 8-bit RGBA PNGs".into());
        }
        data.truncate(info.buffer_size());
        Ok(Self {
            width: info.width as usize,
            height: info.height as usize,
            rgba: data,
        })
    }

    fn sample(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.width + x) * 4;
        [
            self.rgba[i] as f32,
            self.rgba[i + 1] as f32,
            self.rgba[i + 2] as f32,
        ]
    }

    /// Bilinear sample of an original ore image (without tiling its backdrop).
    fn sample_relative(&self, u: f32, v: f32) -> [f32; 3] {
        let x = (8.0 + u * (self.width - 17) as f32).clamp(0.0, (self.width - 1) as f32);
        let y = (8.0 + v * (self.height - 17) as f32).clamp(0.0, (self.height - 1) as f32);
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let top = mix_rgb(self.sample(x0, y0), self.sample(x1, y0), x.fract());
        let bottom = mix_rgb(self.sample(x0, y1), self.sample(x1, y1), x.fract());
        mix_rgb(top, bottom, y.fract())
    }

    /// Use natural source detail, blending *only* a 40px edge overlap.
    /// x=0 samples source x=256; x=255 samples x=255, so the
    /// tile's wrap boundary has ordinary one-pixel neighbor continuity.
    /// The central crop avoids the printed borders of the original atlas.
    fn seamless_tile(&self) -> Result<Vec<u8>, String> {
        let window = N + SEAM_OVERLAP;
        if self.width < window || self.height < window {
            return Err(format!(
                "Reference crop {}x{} is smaller than the {}px seam window",
                self.width, self.height, window
            ));
        }
        let origin_x = (self.width - window) / 2;
        let origin_y = (self.height - window) / 2;
        let mut result = Vec::with_capacity(N * N * 4);
        for y in 0..N {
            let (ay, by, ty) = edge_samples(y);
            for x in 0..N {
                let (ax, bx, tx) = edge_samples(x);
                let top = mix_rgb(
                    self.sample(origin_x + ax, origin_y + ay),
                    self.sample(origin_x + bx, origin_y + ay),
                    tx,
                );
                let bottom = mix_rgb(
                    self.sample(origin_x + ax, origin_y + by),
                    self.sample(origin_x + bx, origin_y + by),
                    tx,
                );
                let result_pixel = mix_rgb(top, bottom, ty);
                result.extend_from_slice(&[
                    result_pixel[0].clamp(0.0, 255.0).round() as u8,
                    result_pixel[1].clamp(0.0, 255.0).round() as u8,
                    result_pixel[2].clamp(0.0, 255.0).round() as u8,
                    255,
                ]);
            }
        }
        Ok(result)
    }
}

fn edge_samples(n: usize) -> (usize, usize, f32) {
    if n < SEAM_OVERLAP {
        (n + N, n, n as f32 / SEAM_OVERLAP as f32)
    } else {
        (n, n, 1.0)
    }
}

fn mix_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
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

/// Small periodic clump noise for optional ground cover, never for rock color.
fn noise(x: f32, y: f32, grid: i32, seed: u32) -> f32 {
    let px = x * grid as f32;
    let py = y * grid as f32;
    let ix = px.floor() as i32;
    let iy = py.floor() as i32;
    let tx = px.fract();
    let ty = py.fract();
    let tx = tx * tx * (3.0 - 2.0 * tx);
    let ty = ty * ty * (3.0 - 2.0 * ty);
    let v = |dx: i32, dy: i32| random((ix + dx).rem_euclid(grid), (iy + dy).rem_euclid(grid), seed);
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
    let mut data = Vec::with_capacity(N * N * 4);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            data.extend_from_slice(&pixel(x, y));
        }
    }
    data
}

fn composite(base: &mut [u8], x: u32, y: u32, pigment: [u8; 3], opacity: f32) {
    let i = ((y * TEXTURE_SIZE + x) * 4) as usize;
    let a = opacity.clamp(0.0, 1.0);
    for c in 0..3 {
        base[i + c] = (base[i + c] as f32 * (1.0 - a) + pigment[c] as f32 * a).round() as u8;
    }
}

fn grass_alpha(x: u32, y: u32, density: CoverLevel) -> f32 {
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let field =
        0.55 * noise(u, v, 11, 71) + 0.25 * noise(u, v, 37, 72) + 0.20 * noise(u, v, 73, 73);
    let threshold = match density {
        CoverLevel::Bare => return 0.0,
        CoverLevel::VerySparse => 0.73,
        CoverLevel::Sparse => 0.51,
        CoverLevel::Full => 0.27,
    };
    ((field - threshold) * 8.0).clamp(0.0, 0.96)
}

fn grass_pigment(forest: &[u8], x: u32, y: u32) -> [u8; 3] {
    // Reuse genuine forest-floor microstructure to break up the synthetic
    // green. This is a tinted *preview*, not species-specific grass artwork.
    let i = ((y * TEXTURE_SIZE + x) * 4) as usize;
    let light = 0.29 * forest[i] as f32 + 0.59 * forest[i + 1] as f32 + 0.12 * forest[i + 2] as f32;
    let organic = (light - 50.0) * 0.32;
    let u = x as f32 / TEXTURE_SIZE as f32;
    let v = y as f32 / TEXTURE_SIZE as f32;
    let detail = noise(u, v, 36, 88) - 0.5;
    [
        (54.0 + organic + detail * 27.0).clamp(0.0, 255.0) as u8,
        (103.0 + organic * 1.4 + detail * 33.0).clamp(0.0, 255.0) as u8,
        (37.0 + organic * 0.65 + detail * 18.0).clamp(0.0, 255.0) as u8,
    ]
}

fn grass_top(soil: &[u8], forest: &[u8], density: CoverLevel) -> Vec<u8> {
    let mut output = soil.to_vec();
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            composite(
                &mut output,
                x,
                y,
                grass_pigment(forest, x, y),
                grass_alpha(x, y, density),
            );
        }
    }
    output
}

fn grass_side(soil: &[u8], forest: &[u8], density: CoverLevel) -> Vec<u8> {
    let mut output = soil.to_vec();
    for y in 0..TEXTURE_SIZE {
        let depth_metres = y as f32 / TEXTURE_SIZE as f32;
        for x in 0..TEXTURE_SIZE {
            // A shallow, irregular organic turf rim, NOT green paint running
            // down the full metre-high dirt side. At most ~0.075m deep.
            let u = x as f32 / TEXTURE_SIZE as f32;
            let edge_depth = 0.035 + 0.040 * noise(u, 0.0, 14, 85);
            let edge_mask = ((edge_depth - depth_metres) / 0.028).clamp(0.0, 1.0);
            composite(
                &mut output,
                x,
                y,
                grass_pigment(forest, x, y),
                grass_alpha(x, 5, density) * edge_mask,
            );
        }
    }
    output
}

/// Extract only copper-colored mineral details from our original ore crop.
/// Transparent/absent areas leave the actual host rock fully visible.
fn ore_composite(rock: &[u8], source: &SourceImage) -> Vec<u8> {
    let mut output = rock.to_vec();
    for y in 0..TEXTURE_SIZE {
        let v = y as f32 / (TEXTURE_SIZE - 1) as f32;
        for x in 0..TEXTURE_SIZE {
            let u = x as f32 / (TEXTURE_SIZE - 1) as f32;
            let copper = source.sample_relative(u, v);
            let red_difference = copper[0] - copper[1];
            let selective_alpha = ((red_difference - 34.0) / 50.0).clamp(0.0, 1.0)
                * ((copper[0] - 130.0) / 65.0).clamp(0.0, 1.0);
            // Do not leave a rectangular brown border around the ore.
            let edge_x = x.min(TEXTURE_SIZE - 1 - x) as f32;
            let edge_y = y.min(TEXTURE_SIZE - 1 - y) as f32;
            let fade = (edge_x / 23.0).clamp(0.0, 1.0) * (edge_y / 23.0).clamp(0.0, 1.0);
            let pigment = [copper[0] as u8, copper[1] as u8, copper[2] as u8];
            composite(&mut output, x, y, pigment, selective_alpha * fade * 0.95);
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

/// Bake the gallery from *actual* original high-resolution source textures.
/// Keeping these compositions in separate layers preserves the existing
/// shader and its diagnostic controls without changing game world state.
pub fn generate_textures() -> Result<Vec<Vec<u8>>, String> {
    let soil = SourceImage::from_png(include_bytes!(
        "../assets/textures/soil/low_fertility_soil.png"
    ))?
    .seamless_tile()?;
    let granite = SourceImage::from_png(include_bytes!(
        "../assets/textures/rock/metamorphic_igneous/granite.png"
    ))?
    .seamless_tile()?;
    let shale = SourceImage::from_png(include_bytes!(
        "../assets/textures/rock/sedimentary/shale.png"
    ))?
    .seamless_tile()?;
    let forest = SourceImage::from_png(include_bytes!("../assets/textures/soil/forest_floor.png"))?
        .seamless_tile()?;
    let copper = SourceImage::from_png(include_bytes!("../assets/textures/ore/native_copper.png"))?;

    let result = vec![
        soil.clone(),
        grass_top(&soil, &forest, CoverLevel::VerySparse),
        grass_top(&soil, &forest, CoverLevel::Sparse),
        grass_top(&soil, &forest, CoverLevel::Full),
        grass_side(&soil, &forest, CoverLevel::VerySparse),
        grass_side(&soil, &forest, CoverLevel::Sparse),
        grass_side(&soil, &forest, CoverLevel::Full),
        granite.clone(),
        shale.clone(),
        ore_composite(&granite, &copper),
        ore_composite(&shale, &copper),
        make_tile(checker_pixel),
    ];
    debug_assert_eq!(result.len(), LAYER_COUNT);
    Ok(result)
}
