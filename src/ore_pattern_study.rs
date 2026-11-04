//! Diagnostic grayscale mineral morphology sources, not world ore deposits.
//! Five deliberately distinct, periodic 256px C/G tiles are generated once
//! for the optional F8 gallery. Coverage (alpha) is independent from tone (RGB).

pub const FAMILY_NAMES: [&str; 7] = [
    "SOURCE",
    "HAIRLINE",
    "NODULES",
    "STOCKWORK",
    "RIBBONS",
    "SCATTER",
    "WORLD VEIN",
];
pub const PALETTE_NAMES: [&str; 3] = ["COPPER", "HEMATITE", "MOONSILVER"];
pub const GENERATED_PATTERNS: usize = 5;
const TAU: f32 = std::f32::consts::TAU;
const TILE_SIZE: usize = 256;

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut value = (x as u32).wrapping_mul(0x9e37_79b9)
        ^ (y as u32).wrapping_mul(0x85eb_ca6b)
        ^ seed.wrapping_mul(0xc2b2_ae35);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    (value & 0x00ff_ffff) as f32 / 16_777_215.0
}

/// A repeating value-noise lattice: both axes wrap at precisely 1.0,
/// even if material UVs cross a tile edge on a neighboring hexel.
fn noise(u: f32, v: f32, grid: i32, seed: u32) -> f32 {
    let x = u * grid as f32;
    let y = v * grid as f32;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let fx = fx * fx * (3.0 - 2.0 * fx);
    let fy = fy * fy * (3.0 - 2.0 * fy);
    let sample =
        |dx: i32, dy: i32| hash((ix + dx).rem_euclid(grid), (iy + dy).rem_euclid(grid), seed);
    let bottom = sample(0, 0) * (1.0 - fx) + sample(1, 0) * fx;
    let top = sample(0, 1) * (1.0 - fx) + sample(1, 1) * fx;
    bottom * (1.0 - fy) + top * fy
}

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn line_distance(value: f32) -> f32 {
    let f = value.rem_euclid(1.0);
    f.min(1.0 - f)
}

/// Returns (linear grayscale tone, mineral coverage). The five cases have
/// distinct topology; changing palette does not regenerate/alter coverage.
fn sample(pattern: usize, u: f32, v: f32) -> (f32, f32) {
    let broad = noise(u, v, 5, 13);
    let detail = noise(u, v, 23, 97);
    let warp = (broad - 0.5) * 0.08;
    let a = u + warp;
    let b = v - warp;
    let coverage = match pattern {
        // Sparse twisting veins.
        1 => {
            let vein = line_distance(a * 6.0 + (v * TAU * 2.0).sin() * 0.24);
            1.0 - smoothstep(0.009, 0.032, vein)
        }
        // Rounded, clustered inclusions.
        2 => smoothstep(0.59, 0.79, noise(a, b, 11, 211)),
        // Intersecting cracks and branching stockwork mineral veins.
        3 => {
            let x = line_distance(a * 8.0 + (v * TAU * 3.0).sin() * 0.18);
            let y = line_distance(b * 7.0 + (u * TAU * 2.0).sin() * 0.16);
            (1.0 - smoothstep(0.012, 0.05, x.min(y))) * 0.90
        }
        // Sedimentary-style irregular ribbons (a visual fixture only).
        4 => {
            let wave = (b * TAU * 9.0 + (u * TAU * 3.0).sin() * 1.0).sin();
            smoothstep(0.32, 0.76, wave)
        }
        // Fine scattered flecks in sparse, low-frequency clusters.
        5 => {
            let speck = noise(a, b, 33, 419);
            smoothstep(0.68, 0.87, speck) * smoothstep(0.25, 0.63, broad)
        }
        _ => 0.0,
    };
    // Tone remains grayscale, independent from its coverage and palette.
    let tone = (0.16 + 0.55 * (0.55 * broad + 0.45 * detail)).clamp(0.0, 1.0);
    (tone, coverage.clamp(0.0, 1.0))
}

fn linear_to_srgb(value: f32) -> u8 {
    let encoded = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

/// One RGBA8-sRGB layer, with tonal detail encoded in RGB and coverage
/// in linear alpha, as required by the existing optional preview binding.
pub fn generate(pattern: usize) -> Vec<u8> {
    assert!((1..=GENERATED_PATTERNS).contains(&pattern));
    let mut bytes = Vec::with_capacity(TILE_SIZE * TILE_SIZE * 4);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let u = x as f32 / TILE_SIZE as f32;
            let v = y as f32 / TILE_SIZE as f32;
            let (tone, coverage) = sample(pattern, u, v);
            let gray = linear_to_srgb(tone);
            bytes.extend_from_slice(&[gray, gray, gray, (coverage * 255.0).round() as u8]);
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_patterns_have_distinct_coverage_and_grayscale_detail() {
        let patterns: Vec<_> = (1..=GENERATED_PATTERNS).map(generate).collect();
        for (i, pattern) in patterns.iter().enumerate() {
            assert_eq!(pattern.len(), TILE_SIZE * TILE_SIZE * 4);
            let pixels: Vec<_> = pattern.chunks_exact(4).collect();
            assert!(pixels.iter().all(|p| p[0] == p[1] && p[1] == p[2]));
            let covered = pixels.iter().filter(|p| p[3] >= 64).count();
            assert!(covered > 100, "pattern {} has no useful coverage", i + 1);
            assert!(
                covered < pixels.len() / 2,
                "pattern {} covers too much host",
                i + 1
            );
            assert!(pixels.iter().any(|p| p[3] == 0));
            assert_eq!(*pattern, generate(i + 1));
        }
        for a in 0..patterns.len() {
            for b in a + 1..patterns.len() {
                assert_ne!(patterns[a], patterns[b]);
            }
        }
    }

    #[test]
    fn generated_patterns_are_periodic_at_tile_seams() {
        for pattern in 1..=GENERATED_PATTERNS {
            for t in [0.0, 0.13, 0.27, 0.73] {
                let x0 = sample(pattern, 0.0, t);
                let x1 = sample(pattern, 1.0, t);
                let y0 = sample(pattern, t, 0.0);
                let y1 = sample(pattern, t, 1.0);
                assert!((x0.0 - x1.0).abs() < 0.00001);
                assert!((x0.1 - x1.1).abs() < 0.00001);
                assert!((y0.0 - y1.0).abs() < 0.00001);
                assert!((y0.1 - y1.1).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn pattern_and_palette_lists_are_explicitly_bounded() {
        // Six existing texture masks remain unchanged; the seventh mode
        // is generated in WGSL from world position, not a texture layer.
        assert_eq!(FAMILY_NAMES.len(), GENERATED_PATTERNS + 2);
        assert_eq!(FAMILY_NAMES[6], "WORLD VEIN");
        assert_eq!(PALETTE_NAMES.len(), 3);
    }
}
