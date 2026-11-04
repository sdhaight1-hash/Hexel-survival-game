//! Read-only runtime validation for the offline v2 material handoff.
//! This does not replace the P0 gallery or assign transient GPU texture slots.

use crate::{materials_p0, texture_codec, texture_pack::TexturePack};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    schema_version: u32,
    note: String,
    textures: Vec<Texture>,
    materials: Vec<Material>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Texture {
    id: String,
    path: String,
    format: String,
    width: u32,
    height: u32,
    repeat_metres: [f32; 2],
    wrap: String,
    compressed_bytes: usize,
    rgb_psnr_db: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Material {
    id: String,
    kind: String,
    status: String,
    representation: String,
    faces: Option<Faces>,
    overlays: Option<Overlays>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Faces {
    top: Face,
    side: Face,
    bottom: Face,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Face {
    base: String,
    overlay: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Overlays {
    top: String,
    side: Option<String>,
    bottom: Option<String>,
}

fn valid_id(id: &str) -> bool {
    let mut parts = id.split('.');
    let (Some(first), Some(second), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    [first, second].iter().all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    })
}

fn validated(index: &Index, directory: &Path) -> Result<(usize, usize), String> {
    if index.schema_version != 2 || index.note.trim().is_empty() {
        return Err("Invalid v2 material index header".into());
    }
    let mut texture_ids = HashSet::new();
    for texture in &index.textures {
        if !valid_id(&texture.id) || !texture_ids.insert(texture.id.as_str()) {
            return Err(format!("Invalid/duplicate texture ID: {}", texture.id));
        }
        let expected_path = format!("{}.bc7", texture.id);
        if texture.path != expected_path {
            return Err(format!("Invalid v2 texture path for {}", texture.id));
        }
        if !matches!(texture.format.as_str(), "bc7-rgba-srgb" | "bc7-rgba-linear")
            || !matches!(texture.wrap.as_str(), "repeat" | "clamp")
            || texture.width == 0
            || texture.height == 0
            || texture.width % 4 != 0
            || texture.height % 4 != 0
            || texture
                .repeat_metres
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || !texture.rgb_psnr_db.is_finite()
            || texture.rgb_psnr_db < 25.0
        {
            return Err(format!("Invalid v2 texture metadata: {}", texture.id));
        }
        let expected = (texture.width as usize / 4)
            .checked_mul(texture.height as usize / 4)
            .and_then(|blocks| blocks.checked_mul(16))
            .ok_or_else(|| format!("BC7 size overflow: {}", texture.id))?;
        if texture.compressed_bytes != expected {
            return Err(format!("BC7 byte count mismatch: {}", texture.id));
        }
        let path = directory.join(&texture.path);
        let actual = fs::metadata(&path)
            .map_err(|e| format!("Missing v2 texture {}: {e}", path.display()))?
            .len();
        if actual != expected as u64 {
            return Err(format!("Truncated/invalid BC7 file: {}", texture.id));
        }
    }

    let mut material_ids = HashSet::new();
    for material in &index.materials {
        if !valid_id(&material.id) || !material_ids.insert(material.id.as_str()) {
            return Err(format!("Invalid/duplicate material ID: {}", material.id));
        }
        if !matches!(material.status.as_str(), "candidate" | "approved") {
            return Err(format!("Unapproved material in v2 index: {}", material.id));
        }
        match (
            &material.representation[..],
            &material.faces,
            &material.overlays,
        ) {
            ("solid", Some(faces), None)
                if matches!(material.kind.as_str(), "soil" | "rock" | "mineral") =>
            {
                for face in [&faces.top, &faces.side, &faces.bottom] {
                    require_texture(&texture_ids, &face.base)?;
                    if let Some(id) = &face.overlay {
                        require_texture(&texture_ids, id)?;
                    }
                }
            }
            ("overlay", None, Some(overlays))
                if matches!(material.kind.as_str(), "cover" | "ore" | "mineral") =>
            {
                require_texture(&texture_ids, &overlays.top)?;
                for id in [&overlays.side, &overlays.bottom].into_iter().flatten() {
                    require_texture(&texture_ids, id)?;
                }
            }
            _ => return Err(format!("Invalid v2 recipe: {}", material.id)),
        }
    }
    Ok((index.textures.len(), index.materials.len()))
}

fn require_texture(ids: &HashSet<&str>, id: &str) -> Result<(), String> {
    if ids.contains(id) {
        Ok(())
    } else {
        Err(format!("Unknown v2 texture reference: {id}"))
    }
}

/// Absence of a bake is normal. A present but invalid cache is reported.
fn load_default() -> Result<Option<(Index, PathBuf)>, String> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/materials-v2");
    let path = directory.join("index.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Could not read {}: {error}", path.display())),
    };
    let index: Index =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid v2 index JSON: {e}"))?;
    validated(&index, &directory)?;
    Ok(Some((index, directory)))
}

pub fn inspect_default() -> Result<Option<(usize, usize)>, String> {
    load_default()
        .map(|loaded| loaded.map(|(index, _)| (index.textures.len(), index.materials.len())))
}

/// Read an individually baked texture, mapped to the P0 gallery's 256px
/// one-metre UV grid. Other physical repeat sizes are explicitly unsupported
/// in this temporary preview, not silently stretched.
fn gallery_texture(index: &Index, directory: &Path, id: &str) -> Result<Vec<u8>, String> {
    let texture = index
        .textures
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("Unknown v2 gallery texture {id}"))?;
    if texture.format != "bc7-rgba-srgb"
        || texture.wrap != "repeat"
        || texture.repeat_metres != [1.0, 1.0]
    {
        return Err(format!(
            "V2 gallery supports only sRGB 1m-repeat color assets: {id}"
        ));
    }
    let blocks = fs::read(directory.join(&texture.path))
        .map_err(|e| format!("Cannot read v2 texture {id}: {e}"))?;
    let source = texture_codec::decode_bc7(&blocks, texture.width, texture.height)?;
    let size = materials_p0::TEXTURE_SIZE;
    let mut output = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let src_x = (x as u64 * texture.width as u64 / size as u64) as usize;
            let src_y = (y as u64 * texture.height as u64 / size as u64) as usize;
            let source_offset = (src_y * texture.width as usize + src_x) * 4;
            let dest_offset = ((y * size + x) * 4) as usize;
            output[dest_offset..dest_offset + 4]
                .copy_from_slice(&source[source_offset..source_offset + 4]);
        }
    }
    Ok(output)
}

/// The first gallery experiment uses one base texture across all eight
/// rock faces. Distinct face recipes remain valid in v2 but require a
/// future mesh/material-ID extension before this F8 comparison can show them.
fn gallery_rock(index: &Index, directory: &Path, id: &str) -> Result<Option<Vec<u8>>, String> {
    let Some(material) = index.materials.iter().find(|item| item.id == id) else {
        return Ok(None);
    };
    if material.kind != "rock" || material.representation != "solid" {
        return Err(format!("Unexpected v2 rock recipe: {id}"));
    }
    let faces = material
        .faces
        .as_ref()
        .ok_or_else(|| format!("{id}: missing faces"))?;
    let same_base = faces.top.base == faces.side.base && faces.top.base == faces.bottom.base;
    let no_overlay = [&faces.top, &faces.side, &faces.bottom]
        .iter()
        .all(|face| face.overlay.is_none());
    if !same_base || !no_overlay {
        return Err(format!(
            "{id}: F8 gallery needs a single unlayered rock base"
        ));
    }
    gallery_texture(index, directory, &faces.top.base).map(Some)
}

fn gallery_copper(index: &Index, directory: &Path) -> Result<Option<Vec<u8>>, String> {
    let Some(material) = index.materials.iter().find(|m| m.id == "ore.native_copper") else {
        return Ok(None);
    };
    let overlays = material
        .overlays
        .as_ref()
        .ok_or("Native copper v2 recipe has no overlay")?;
    if overlays.side.as_deref() != Some(overlays.top.as_str()) || overlays.bottom.is_some() {
        return Err("F8 copper preview needs one shared top/side overlay".into());
    }
    gallery_texture(index, directory, &overlays.top).map(Some)
}

fn srgb_to_linear(value: u8) -> f32 {
    let v = value as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> u8 {
    let converted = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (converted * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Composite a transparent ore-only mask over the actual displayed host.
fn composite_ore(base: &[u8], overlay: &[u8]) -> Vec<u8> {
    let mut result = base.to_vec();
    for (dst, ore) in result.chunks_exact_mut(4).zip(overlay.chunks_exact(4)) {
        let alpha = ore[3] as f32 / 255.0;
        // F8 preview only: preserve the ore footprint in the otherwise
        // opaque composite layer's alpha for localized material shading.
        // The terrain shader still writes fully opaque output.
        dst[3] = ore[3];
        if alpha == 0.0 {
            continue;
        }
        if alpha == 1.0 {
            dst[..3].copy_from_slice(&ore[..3]);
        } else {
            for c in 0..3 {
                let host = srgb_to_linear(dst[c]);
                let mineral = srgb_to_linear(ore[c]);
                dst[c] = linear_to_srgb(host * (1.0 - alpha) + mineral * alpha);
            }
        }
    }
    result
}

/// Extract two independent concepts from a colored ore-only input:
/// alpha is mineral coverage; linear-light luminance is internal tonal detail.
/// Store tonal detail as sRGB grayscale because the temporary F8 preview
/// array is Rgba8UnormSrgb. Alpha remains linear on texture sampling.
fn grayscale_copper_tile(overlay: &[u8]) -> Vec<u8> {
    let mut result = overlay.to_vec();
    for pixel in result.chunks_exact_mut(4) {
        let tone = 0.2126 * srgb_to_linear(pixel[0])
            + 0.7152 * srgb_to_linear(pixel[1])
            + 0.0722 * srgb_to_linear(pixel[2]);
        let gray = linear_to_srgb(tone);
        pixel[..3].fill(gray);
        // Keep the original coverage byte, including for dark minerals.
    }
    result
}

/// Create a *separate*, opt-in RGBA preview array of the original 12 display
/// slots as P0, plus a shared grayscale copper source at layer 12 when
/// a compatible candidate exists. The default P0 pack remains unchanged.
/// Only granite, shale and shared copper-mask candidates are currently wired.
pub fn preview_layers(p0: &TexturePack<'_>) -> Result<Option<Vec<Vec<u8>>>, String> {
    let Some((index, directory)) = load_default()? else {
        return Ok(None);
    };
    let granite = gallery_rock(&index, &directory, "rock.granite")?;
    let shale = gallery_rock(&index, &directory, "rock.shale")?;
    let copper = gallery_copper(&index, &directory)?;
    if granite.is_none() && shale.is_none() && copper.is_none() {
        return Ok(None);
    }
    let mut layers: Vec<Vec<u8>> = (0..p0.layers)
        .map(|layer| p0.decode_layer(layer))
        .collect::<Result<_, _>>()?;
    use materials_p0::Material;
    if let Some(granite) = granite {
        layers[Material::Granite as usize] = granite;
    }
    if let Some(shale) = shale {
        layers[Material::Shale as usize] = shale;
    }
    if let Some(mask) = copper {
        for (host, composite) in [
            (Material::Granite, Material::GraniteCopper),
            (Material::Shale, Material::ShaleCopper),
        ] {
            layers[composite as usize] = composite_ore(&layers[host as usize], &mask);
        }
        // One source mask serves *both* rock hosts. Layer 12 is only sampled
        // in the shader-driven F11 comparison, never as world/resource data.
        layers.push(grayscale_copper_tile(&mask));
    } else {
        // No v2 ore source: keep the P0 copper coloration but mark its
        // F8 preview as nonmetallic rather than highlighting an entire face.
        for layer in [Material::GraniteCopper, Material::ShaleCopper] {
            for pixel in layers[layer as usize].chunks_exact_mut(4) {
                pixel[3] = 0;
            }
        }
    }
    Ok(Some(layers))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Index {
        Index {
            schema_version: 2,
            note: "Test".into(),
            textures: vec![],
            materials: vec![],
        }
    }

    #[test]
    fn valid_empty_index_leaves_p0_available() {
        assert_eq!(validated(&empty(), Path::new(".")).unwrap(), (0, 0));
    }

    #[test]
    fn refuses_unresolved_material_references() {
        let mut index = empty();
        index.materials.push(Material {
            id: "rock.granite".into(),
            kind: "rock".into(),
            status: "candidate".into(),
            representation: "solid".into(),
            faces: Some(Faces {
                top: Face {
                    base: "rock.missing".into(),
                    overlay: None,
                },
                side: Face {
                    base: "rock.missing".into(),
                    overlay: None,
                },
                bottom: Face {
                    base: "rock.missing".into(),
                    overlay: None,
                },
            }),
            overlays: None,
        });
        assert!(validated(&index, Path::new(".")).is_err());
    }

    #[test]
    fn excludes_unapproved_or_malformed_recipes() {
        let mut index = empty();
        index.materials.push(Material {
            id: "ore.native_copper".into(),
            kind: "ore".into(),
            status: "needs_art".into(),
            representation: "overlay".into(),
            faces: None,
            overlays: Some(Overlays {
                top: "ore.copper".into(),
                side: None,
                bottom: None,
            }),
        });
        assert!(validated(&index, Path::new(".")).is_err());
        assert!(!valid_id("rock.Granite"));
        assert!(!valid_id("rock.granite.extra"));
    }

    #[test]
    fn baked_candidate_gallery_composes_distinct_hosts_when_index_exists() {
        let pack = TexturePack::embedded().unwrap();
        let Some(layers) = preview_layers(&pack).unwrap() else {
            // A clean checkout can run tests before any optional offline bake.
            return;
        };
        use materials_p0::Material;
        assert_eq!(layers.len(), materials_p0::LAYER_COUNT + 1);
        let gray = &layers[materials_p0::LAYER_COUNT];
        assert!(gray.chunks_exact(4).all(|pixel| pixel[0] == pixel[1]
            && pixel[1] == pixel[2]));
        assert!(gray.chunks_exact(4).any(|pixel| pixel[3] > 32));
        assert!(gray.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert_ne!(
            layers[Material::Granite as usize],
            layers[Material::Shale as usize]
        );
        assert_ne!(
            layers[Material::Granite as usize],
            layers[Material::GraniteCopper as usize]
        );
        assert_ne!(
            layers[Material::Shale as usize],
            layers[Material::ShaleCopper as usize]
        );
        assert!(layers[Material::Granite as usize]
            .chunks_exact(4)
            .all(|pixel| pixel[3] == 255));
        // The ore mask survives the composite as coverage, not as an
        // opaque rock-swapping texture or a fully shiny surface.
        for layer in [Material::GraniteCopper, Material::ShaleCopper] {
            let pixels = &layers[layer as usize];
            let covered = pixels.chunks_exact(4).filter(|pixel| pixel[3] > 32).count();
            assert!(covered > 100);
            assert!(covered < pixels.len() / 4 / 2);
            assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] == 0));
        }
    }

    #[test]
    fn grayscale_copper_preserves_coverage_independent_of_tonal_detail() {
        let input = [
            255, 255, 255, 0, // transparent bright pigment
            0, 0, 0, 255,     // fully covered black pigment
            255, 0, 0, 128,   // partially covered red pigment
        ];
        let gray = grayscale_copper_tile(&input);
        assert_eq!(gray.len(), input.len());
        for (src, out) in input.chunks_exact(4).zip(gray.chunks_exact(4)) {
            assert_eq!(src[3], out[3], "coverage is never inferred from tone");
            assert_eq!(out[0], out[1]);
            assert_eq!(out[1], out[2]);
        }
        assert_eq!(gray[4], 0);
        assert_eq!(gray[7], 255, "dark mineral is still fully covered");
        assert!(gray[8] > 0 && gray[8] < 255);
        assert_eq!(gray[11], 128);
    }

    #[test]
    fn grayscale_copper_luminance_is_gamma_correct_and_deterministic() {
        let input = [128, 128, 128, 200, 128, 128, 128, 200];
        let one = grayscale_copper_tile(&input);
        let two = grayscale_copper_tile(&input);
        assert_eq!(one, two);
        assert_eq!(one, input);
    }

    #[test]
    fn transparent_copper_preserves_host_and_opaque_copper_replaces_color() {
        let base = [60u8, 80, 100, 255, 60, 80, 100, 255];
        let overlay = [200u8, 110, 50, 0, 200, 110, 50, 255];
        let out = composite_ore(&base, &overlay);
        assert_eq!(&out[..4], &[60, 80, 100, 0]);
        assert_eq!(&out[4..], &[200, 110, 50, 255]);
    }

    #[test]
    fn semi_transparent_ore_keeps_its_mask_without_replacing_host() {
        let base = [60u8, 80, 100, 255];
        let overlay = [200u8, 110, 50, 128];
        let result = composite_ore(&base, &overlay);
        assert_eq!(result[3], 128);
        assert_ne!(&result[..3], &base[..3]);
        assert_ne!(&result[..3], &overlay[..3]);
    }

    #[test]
    fn rejects_path_traversal_without_opening_file() {
        let mut index = empty();
        index.textures.push(Texture {
            id: "rock.granite".into(),
            path: "../private.bc7".into(),
            format: "bc7-rgba-srgb".into(),
            width: 4,
            height: 4,
            repeat_metres: [1.0, 1.0],
            wrap: "repeat".into(),
            compressed_bytes: 16,
            rgb_psnr_db: 30.0,
        });
        assert!(validated(&index, Path::new(".")).is_err());
    }
}
