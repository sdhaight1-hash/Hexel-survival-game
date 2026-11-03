//! Offline BC7 bake: Rust/Cargo runs this on source changes, never at game startup.
//! The raw RGBA output exists only during the bake and as a CPU decode fallback.

#[path = "src/materials_p0_art.rs"]
mod materials_p0_art;
#[path = "src/texture_codec.rs"]
mod texture_codec;

use std::{env, error::Error, fs, path::PathBuf};
use texture_compressor::{encode, Effort, Format, ImageView, Settings};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/materials_p0_art.rs");
    println!("cargo:rerun-if-changed=src/texture_codec.rs");
    for input in [
        "assets/textures/soil/low_fertility_soil.png",
        "assets/textures/soil/forest_floor.png",
        "assets/textures/rock/metamorphic_igneous/granite.png",
        "assets/textures/rock/sedimentary/shale.png",
        "assets/textures/ore/native_copper.png",
    ] {
        println!("cargo:rerun-if-changed={input}");
    }

    let width = 256u32;
    let height = 256u32;
    let layers = materials_p0_art::generate_textures()?;
    let layer_size = texture_codec::bc7_layer_size(width, height)?;
    let mut packed = Vec::with_capacity(20 + layers.len() * layer_size);
    packed.extend_from_slice(b"HXB7");
    packed.extend_from_slice(&1u32.to_le_bytes());
    packed.extend_from_slice(&width.to_le_bytes());
    packed.extend_from_slice(&height.to_le_bytes());
    packed.extend_from_slice(&(layers.len() as u32).to_le_bytes());

    let mut worst_psnr = f64::INFINITY;
    for (index, rgba) in layers.iter().enumerate() {
        let view = ImageView::rgba8(rgba, width, height, width * 4)
            .map_err(|e| format!("Invalid P0 material layer {index}: {e:?}"))?;
        let compressed = encode(view, &Settings::new(Format::Bc7, Effort::Balanced))
            .map_err(|e| format!("BC7 encoding failed for layer {index}: {e:?}"))?;
        if compressed.len() != layer_size {
            return Err(format!("Incorrect BC7 block size for layer {index}").into());
        }
        // Decode with an independent BC7 decoder before shipping the blocks.
        // This also tests the software fallback with *actual* encoded art.
        let decoded = texture_codec::decode_bc7(&compressed, width, height)?;
        let error: f64 = rgba.chunks_exact(4)
            .zip(decoded.chunks_exact(4))
            .flat_map(|(a, b)| (0..3).map(move |i| (a[i] as f64 - b[i] as f64).powi(2)))
            .sum();
        let mse = error / ((width * height * 3) as f64);
        let psnr = if mse == 0.0 { f64::INFINITY } else { 10.0 * (255.0 * 255.0 / mse).log10() };
        worst_psnr = worst_psnr.min(psnr);
        if psnr < 25.0 {
            return Err(format!("BC7 material layer {index} too lossy ({psnr:.1} dB); do not degrade artwork").into());
        }
        packed.extend_from_slice(&compressed);
    }

    let path = PathBuf::from(env::var("OUT_DIR")?).join("p0_materials.bc7");
    fs::write(&path, &packed)?;
    println!(
        "cargo:warning=P0 material bake: {} BC7 layers, {:.2} MiB -> {:.2} MiB GPU storage (4:1); worst RGB PSNR {:.1} dB",
        layers.len(),
        (layers.len() * width as usize * height as usize * 4) as f64 / 1_048_576.0,
        (layers.len() * layer_size) as f64 / 1_048_576.0,
        worst_psnr,
    );
    Ok(())
}
