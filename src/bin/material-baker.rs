//! P1 offline authoring tool: validate stable material IDs and bake independent
//! color/overlay images to BC7. The playable P0 renderer is intentionally
//! unchanged. No gameplay logic, ecology, ore quantities or GPU uploads here.

#[path = "../texture_codec.rs"]
mod texture_codec;

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};
use texture_compressor::{encode, Effort, Format, ImageView, Settings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema_version: u32,
    notes: String,
    texture_assets: Vec<TextureAsset>,
    materials: Vec<Material>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TextureAsset {
    id: String,
    path: String,
    usage: String,
    color_space: String,
    /// How many metres of world surface one tile repetition covers.
    repeat_metres: [f32; 2],
    wrap: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Material {
    id: String,
    kind: String,
    status: String,
    legacy_reference: String,
    representation: Option<String>,
    faces: Option<Faces>,
    overlays: Option<Overlays>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Faces {
    top: FaceLayer,
    side: FaceLayer,
    bottom: FaceLayer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FaceLayer {
    base: String,
    overlay: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Overlays {
    top: String,
    side: Option<String>,
    bottom: Option<String>,
}

#[derive(Deserialize)]
struct Legacy {
    name: String,
    category: String,
    crop_path: String,
}

#[derive(Serialize)]
struct BakedTexture {
    id: String,
    path: String,
    format: &'static str,
    width: u32,
    height: u32,
    repeat_metres: [f32; 2],
    wrap: String,
    compressed_bytes: usize,
    rgb_psnr_db: f64,
}

#[derive(Serialize)]
struct BakeIndex {
    schema_version: u32,
    note: &'static str,
    textures: Vec<BakedTexture>,
    // Authoring identities, never transient GPU layer indices.
    materials: Vec<BakedMaterial>,
}

#[derive(Serialize)]
struct BakedMaterial {
    id: String,
    kind: String,
    status: String,
    representation: String,
    faces: Option<Faces>,
    overlays: Option<Overlays>,
}

/// Export only art-backed candidate/approved recipes, ordered by stable ID.
/// The validated catalog ensures every referenced texture exists.
fn material_recipes(catalog: &Catalog) -> Vec<BakedMaterial> {
    let mut recipes: Vec<_> = catalog
        .materials
        .iter()
        .filter(|material| material.status != "needs_art")
        .map(|material| BakedMaterial {
            id: material.id.clone(),
            kind: material.kind.clone(),
            status: material.status.clone(),
            representation: material.representation.clone().unwrap_or_default(),
            faces: material.faces.clone(),
            overlays: material.overlays.clone(),
        })
        .collect();
    recipes.sort_by(|a, b| a.id.cmp(&b.id));
    recipes
}

fn invalid(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}

fn id_valid(id: &str) -> bool {
    let mut parts = id.split('.');
    let first = parts.next();
    let second = parts.next();
    first.is_some()
        && second.is_some()
        && parts.next().is_none()
        && [first.unwrap(), second.unwrap()].into_iter().all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == b'_')
        })
}

fn reference_id(row: &Legacy) -> (String, String) {
    let kind = match row.category.as_str() {
        "soil" if row.name == "forest_floor" => "cover",
        "soil" => "soil",
        "ore" => "ore",
        "mineral" => "mineral",
        _ if row.category.starts_with("rock/") => "rock",
        _ => "invalid",
    };
    let name = if kind == "soil" {
        row.name.strip_suffix("_soil").unwrap_or(&row.name)
    } else {
        &row.name
    };
    (format!("{kind}.{name}"), kind.to_string())
}

fn safe_relative(path: &str, prefix: &str) -> bool {
    path.starts_with(prefix)
        && path.ends_with(".png")
        && !path.contains('\\')
        && !path.contains(':')
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn check_reference(root: &Path, relative: &str) -> Result<()> {
    if !safe_relative(relative, "assets/textures/") {
        return Err(invalid(format!("Invalid legacy source path: {relative}")));
    }
    if !root.join(relative).is_file() {
        return Err(invalid(format!(
            "Missing original reference texture: {relative}"
        )));
    }
    Ok(())
}

fn asset<'a>(
    assets: &'a BTreeMap<&str, &'a TextureAsset>,
    id: &str,
    usage: &str,
) -> Result<&'a TextureAsset> {
    let item = assets
        .get(id)
        .ok_or_else(|| invalid(format!("Unknown texture asset ID: {id}")))?;
    if item.usage != usage {
        return Err(invalid(format!(
            "{id}: expected {usage}, found {}",
            item.usage
        )));
    }
    Ok(item)
}

fn check_faces(faces: &Faces, assets: &BTreeMap<&str, &TextureAsset>) -> Result<()> {
    for (role, face) in [
        ("top", &faces.top),
        ("side", &faces.side),
        ("bottom", &faces.bottom),
    ] {
        let base = asset(assets, &face.base, "albedo")?;
        if let Some(overlay) = &face.overlay {
            let mask = asset(assets, overlay, "overlay")?;
            if base.repeat_metres != mask.repeat_metres {
                return Err(invalid(format!(
                    "{role}: base/overlay repetitions differ in metres; resampling must be explicit"
                )));
            }
        }
    }
    Ok(())
}

fn validate(catalog: &Catalog, legacy: &[Legacy], root: &Path) -> Result<()> {
    if catalog.schema_version != 2 || catalog.notes.trim().is_empty() {
        return Err(invalid("Catalog must declare v2 and explanatory notes"));
    }
    if legacy.len() != 55 || catalog.materials.len() != legacy.len() {
        return Err(invalid(
            "Catalog must cover all 55 original source identities exactly",
        ));
    }

    let mut assets = BTreeMap::new();
    for a in &catalog.texture_assets {
        if !id_valid(&a.id) || assets.insert(a.id.as_str(), a).is_some() {
            return Err(invalid(format!(
                "Invalid or duplicate texture ID: {}",
                a.id
            )));
        }
        if !safe_relative(&a.path, "assets/production/") {
            return Err(invalid(format!(
                "Production asset outside approved folder: {}",
                a.path
            )));
        }
        if !matches!(a.usage.as_str(), "albedo" | "overlay") {
            return Err(invalid(format!("Invalid texture usage: {}", a.usage)));
        }
        if !matches!(a.color_space.as_str(), "srgb" | "linear")
            || !matches!(a.wrap.as_str(), "repeat" | "clamp")
        {
            return Err(invalid(format!("Bad color space or wrap: {}", a.id)));
        }
        if a.repeat_metres
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0 || *x > 1024.0)
        {
            return Err(invalid(format!(
                "Invalid physical repeat dimensions: {}",
                a.id
            )));
        }
        let path = root.join(&a.path);
        if !path.is_file() {
            return Err(invalid(format!("Missing production art: {}", a.path)));
        }
        let (width, height, rgba) = read_png(&path)?;
        if width < 4
            || height < 4
            || width > 4096
            || height > 4096
            || width % 4 != 0
            || height % 4 != 0
        {
            return Err(invalid(format!(
                "{}: BC7 requires 4-pixel-aligned dimensions, 4..4096; got {width}x{height}",
                a.id
            )));
        }
        if a.usage == "overlay" && rgba.chunks_exact(4).all(|pixel| pixel[3] == 255) {
            return Err(invalid(format!("{}: overlay has no transparency", a.id)));
        }
    }

    let expected: BTreeMap<_, _> = legacy
        .iter()
        .map(|row| {
            let (id, kind) = reference_id(row);
            (id, (kind, row.crop_path.as_str()))
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut used_assets = BTreeSet::new();
    for material in &catalog.materials {
        let wanted = expected.get(&material.id).ok_or_else(|| {
            invalid(format!(
                "Unknown original material identity: {}",
                material.id
            ))
        })?;
        if !id_valid(&material.id)
            || !seen.insert(material.id.as_str())
            || material.kind != wanted.0
            || material.legacy_reference != wanted.1
        {
            return Err(invalid(format!(
                "Material ID, kind or reference mismatch: {}",
                material.id
            )));
        }
        check_reference(root, &material.legacy_reference)?;
        match material.status.as_str() {
            "needs_art" => {
                if material.representation.is_some()
                    || material.faces.is_some()
                    || material.overlays.is_some()
                {
                    return Err(invalid(format!(
                        "{}: pending art cannot masquerade as ready",
                        material.id
                    )));
                }
            }
            "candidate" | "approved" => match material.representation.as_deref() {
                Some("solid") if matches!(material.kind.as_str(), "soil" | "rock" | "mineral") => {
                    if material.overlays.is_some() {
                        return Err(invalid(format!(
                            "{}: solid has unexpected ore/cover overlay",
                            material.id
                        )));
                    }
                    let faces = material.faces.as_ref().ok_or_else(|| {
                        invalid(format!(
                            "{}: missing top, side, bottom recipes",
                            material.id
                        ))
                    })?;
                    check_faces(faces, &assets)?;
                    for face in [&faces.top, &faces.side, &faces.bottom] {
                        used_assets.insert(face.base.as_str());
                        if let Some(overlay) = &face.overlay {
                            used_assets.insert(overlay.as_str());
                        }
                    }
                }
                Some("overlay")
                    if matches!(material.kind.as_str(), "cover" | "ore" | "mineral") =>
                {
                    if material.faces.is_some() {
                        return Err(invalid(format!(
                            "{}: overlay cannot own a host-rock face set",
                            material.id
                        )));
                    }
                    let overlays = material.overlays.as_ref().ok_or_else(|| {
                        invalid(format!("{}: missing overlay surfaces", material.id))
                    })?;
                    for id in [
                        Some(&overlays.top),
                        overlays.side.as_ref(),
                        overlays.bottom.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        asset(&assets, id, "overlay")?;
                        used_assets.insert(id.as_str());
                    }
                }
                _ => {
                    return Err(invalid(format!(
                        "{}: invalid or missing representation for {}",
                        material.id, material.kind
                    )))
                }
            },
            _ => {
                return Err(invalid(format!(
                    "Unrecognized approval state: {}",
                    material.status
                )))
            }
        }
    }
    for id in assets.keys() {
        if !used_assets.contains(*id) {
            return Err(invalid(format!("Unreferenced production texture: {id}")));
        }
    }
    Ok(())
}

fn read_png(path: &Path) -> Result<(u32, u32, Vec<u8>)> {
    let mut reader = png::Decoder::new(File::open(path)?)
        .read_info()
        .map_err(|e| invalid(format!("PNG header {}: {e}", path.display())))?;
    let mut output = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut output)
        .map_err(|e| invalid(format!("PNG data {}: {e}", path.display())))?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(invalid(format!(
            "{}: production art must be 8-bit RGBA PNG",
            path.display()
        )));
    }
    output.truncate(info.buffer_size());
    Ok((info.width, info.height, output))
}

fn bake(catalog: &Catalog, root: &Path, out: &Path) -> Result<usize> {
    let mut textures = Vec::new();
    let mut sources: Vec<_> = catalog.texture_assets.iter().collect();
    sources.sort_by(|a, b| a.id.cmp(&b.id));
    if sources.is_empty() {
        println!("No production textures yet: all 55 references still need artwork.");
    }
    fs::create_dir_all(out)?;
    for src in sources {
        let (width, height, rgba) = read_png(&root.join(&src.path))?;
        let view = ImageView::rgba8(&rgba, width, height, width as usize * 4)
            .map_err(|e| invalid(format!("{}: {e:?}", src.id)))?;
        let blocks = encode(view, &Settings::new(Format::Bc7, Effort::Balanced))
            .map_err(|e| invalid(format!("{}: BC7 compression failed: {e:?}", src.id)))?;
        if blocks.len() != texture_codec::bc7_layer_size(width, height)? {
            return Err(invalid(format!(
                "{}: invalid compressed block count",
                src.id
            )));
        }
        let decoded = texture_codec::decode_bc7(&blocks, width, height)?;
        let squared_error: f64 = rgba
            .chunks_exact(4)
            .zip(decoded.chunks_exact(4))
            .flat_map(|(a, b)| (0..3).map(move |i| (a[i] as f64 - b[i] as f64).powi(2)))
            .sum();
        let mse = squared_error / (width as f64 * height as f64 * 3.0);
        let psnr = if mse == 0.0 {
            f64::INFINITY
        } else {
            10.0 * (255.0_f64.powi(2) / mse).log10()
        };
        if psnr < 25.0 {
            return Err(invalid(format!(
                "{}: BC7 quality rejected ({psnr:.1} dB)",
                src.id
            )));
        }
        let name = format!("{}.bc7", src.id);
        fs::write(out.join(&name), &blocks)?;
        textures.push(BakedTexture {
            id: src.id.clone(),
            path: name,
            format: if src.color_space == "srgb" {
                "bc7-rgba-srgb"
            } else {
                "bc7-rgba-linear"
            },
            width,
            height,
            repeat_metres: src.repeat_metres,
            wrap: src.wrap.clone(),
            compressed_bytes: blocks.len(),
            rgb_psnr_db: (psnr.min(99.0) * 100.0).round() / 100.0,
        });
    }
    let index = BakeIndex {
        schema_version: 2,
        note: "Offline BC7 sources and validated material recipes; runtime GPU layout is separate.",
        textures,
        materials: material_recipes(catalog),
    };
    fs::write(out.join("index.json"), serde_json::to_vec_pretty(&index)?)?;
    Ok(index.textures.len())
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let catalog: Catalog =
        serde_json::from_slice(&fs::read(root.join("assets/materials/catalog.v2.json"))?)?;
    let legacy: Vec<Legacy> =
        serde_json::from_slice(&fs::read(root.join("assets/textures/manifest.json"))?)?;
    validate(&catalog, &legacy, &root)?;
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("validate"), None, None) => {
            let pending = catalog
                .materials
                .iter()
                .filter(|x| x.status == "needs_art")
                .count();
            println!("P1 material catalog valid: {} identities, {pending} pending artwork, {} candidate/approved textures.",
                catalog.materials.len(), catalog.texture_assets.len());
        }
        (Some("bake"), None, None) => {
            let count = bake(&catalog, &root, &root.join("target/materials-v2"))?;
            println!("Baked {count} production texture(s) to target/materials-v2.");
        }
        _ => {
            return Err(invalid(
                "Usage: cargo run --bin material-baker -- [validate|bake]",
            ))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, category: &str) -> Legacy {
        Legacy {
            name: name.to_owned(),
            category: category.to_owned(),
            crop_path: format!("assets/textures/{category}/{name}.png"),
        }
    }

    #[test]
    fn exact_source_id_mapping_preserves_geology_and_organic_cover() {
        assert_eq!(
            reference_id(&entry("forest_floor", "soil")).0,
            "cover.forest_floor"
        );
        assert_eq!(
            reference_id(&entry("low_fertility_soil", "soil")).0,
            "soil.low_fertility"
        );
        assert_eq!(
            reference_id(&entry("native_copper", "ore")).0,
            "ore.native_copper"
        );
        assert_eq!(
            reference_id(&entry("granite", "rock/metamorphic_igneous")).0,
            "rock.granite"
        );
        assert!(!id_valid("rock.Granite"));
        assert!(!safe_relative(
            "assets/production/../textures/ore.png",
            "assets/production/"
        ));
    }

    #[test]
    fn offline_baker_accepts_mixed_rectangular_and_square_resolution_assets() {
        let root = std::env::temp_dir().join(format!(
            "hexel-v2-baker-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_dir = root.join("assets/production/test");
        fs::create_dir_all(&source_dir).unwrap();
        let mut items = Vec::new();
        for (id, width, height) in [("test.small", 64, 128), ("test.large", 256, 256)] {
            let name = format!("{id}.png");
            let bytes: Vec<u8> = (0..height)
                .flat_map(|y| {
                    (0..width).flat_map(move |x| {
                        // Smooth gradients compress well and retain distinguishable texels.
                        let red = (x * 130 / width + 55) as u8;
                        let green = (y * 120 / height + 60) as u8;
                        [red, green, 90, 255]
                    })
                })
                .collect();
            let path = source_dir.join(&name);
            let mut png = png::Encoder::new(File::create(&path).unwrap(), width, height);
            png.set_color(png::ColorType::Rgba);
            png.set_depth(png::BitDepth::Eight);
            png.write_header()
                .unwrap()
                .write_image_data(&bytes)
                .unwrap();
            items.push(TextureAsset {
                id: id.to_owned(),
                path: format!("assets/production/test/{name}"),
                usage: "albedo".to_string(),
                color_space: "srgb".to_string(),
                repeat_metres: if width == 64 { [0.5, 1.0] } else { [1.0, 1.0] },
                wrap: "repeat".to_owned(),
            });
        }
        let catalog = Catalog {
            schema_version: 2,
            notes: "Synthetic compression test".to_owned(),
            materials: Vec::new(),
            texture_assets: items,
        };
        let out = root.join("target/v2-baked");
        let result = bake(&catalog, &root, &out);
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap(), 2);
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("index.json")).unwrap()).unwrap();
        assert_eq!(index["materials"].as_array().unwrap().len(), 0);
        assert_eq!(index["textures"][0]["id"], "test.large");
        assert_eq!(index["textures"][1]["id"], "test.small");
        assert_eq!(index["textures"][0]["width"], 256);
        assert_eq!(index["textures"][1]["height"], 128);
        assert_eq!(index["textures"][0]["repeat_metres"][0], 1.0);
        assert_eq!(index["textures"][1]["repeat_metres"][0], 0.5);
        assert_eq!(
            fs::metadata(out.join("test.large.bc7")).unwrap().len(),
            65536
        );
        assert_eq!(
            fs::metadata(out.join("test.small.bc7")).unwrap().len(),
            8192
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn baked_manifest_preserves_stable_faces_and_reusable_ore_overlays() {
        let face = FaceLayer {
            base: "rock.granite_albedo".into(),
            overlay: None,
        };
        let mut catalog = Catalog {
            schema_version: 2,
            notes: "Synthetic recipes".into(),
            texture_assets: vec![],
            materials: vec![
                Material {
                    id: "rock.granite".into(),
                    kind: "rock".into(),
                    status: "candidate".into(),
                    legacy_reference: String::new(),
                    representation: Some("solid".into()),
                    faces: Some(Faces {
                        top: face.clone(),
                        side: face.clone(),
                        bottom: face,
                    }),
                    overlays: None,
                },
                Material {
                    id: "ore.native_copper".into(),
                    kind: "ore".into(),
                    status: "candidate".into(),
                    legacy_reference: String::new(),
                    representation: Some("overlay".into()),
                    faces: None,
                    overlays: Some(Overlays {
                        top: "ore.copper_mask".into(),
                        side: Some("ore.copper_mask".into()),
                        bottom: None,
                    }),
                },
                Material {
                    id: "rock.shale".into(),
                    kind: "rock".into(),
                    status: "needs_art".into(),
                    legacy_reference: String::new(),
                    representation: None,
                    faces: None,
                    overlays: None,
                },
            ],
        };
        let recipes = material_recipes(&catalog);
        assert_eq!(recipes.len(), 2);
        assert_eq!(recipes[0].id, "ore.native_copper");
        assert_eq!(recipes[1].id, "rock.granite");
        let json = serde_json::to_value(&recipes).unwrap();
        assert_eq!(json[0]["overlays"]["side"], "ore.copper_mask");
        assert_eq!(json[1]["faces"]["top"]["base"], "rock.granite_albedo");
        assert_eq!(
            json[1]["faces"]["bottom"]["overlay"],
            serde_json::Value::Null
        );
        // Authoring order does not affect the runtime index.
        catalog.materials.reverse();
        assert_eq!(
            serde_json::to_value(material_recipes(&catalog)).unwrap(),
            json
        );
    }

    #[test]
    fn candidate_art_has_seam_continuity_and_no_short_period_duplication() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for (relative, ore) in [
            ("assets/production/rock/granite_candidate.png", false),
            ("assets/production/rock/shale_candidate.png", false),
            ("assets/production/ore/native_copper_candidate.png", true),
        ] {
            let (width, height, rgba) = read_png(&root.join(relative)).unwrap();
            assert_eq!((width, height), (256, 256), "{relative}");
            let side = width as usize;
            let pixel = |x: usize, y: usize| &rgba[(y * side + x) * 4..(y * side + x + 1) * 4];
            let mut seam_sum = 0usize;
            for i in 0..side {
                for channel in 0..4 {
                    seam_sum += pixel(0, i)[channel].abs_diff(pixel(side - 1, i)[channel]) as usize;
                    seam_sum += pixel(i, 0)[channel].abs_diff(pixel(i, side - 1)[channel]) as usize;
                }
            }
            let seam_delta = seam_sum as f32 / (side * 8) as f32;
            assert!(
                seam_delta < 8.0,
                "{relative}: excessive average wrap seam delta {seam_delta:.2}"
            );

            if ore {
                let visible = rgba.chunks_exact(4).filter(|pixel| pixel[3] > 32).count();
                let coverage = visible as f32 / (side * side) as f32;
                assert!(
                    (0.06..0.30).contains(&coverage),
                    "{relative}: ore-only alpha coverage {coverage:.3} is not usable"
                );
                assert!(
                    rgba.chunks_exact(4).any(|pixel| pixel[3] == 0),
                    "{relative}: overlay must retain a fully clear background"
                );
            } else {
                assert!(
                    rgba.chunks_exact(4).all(|pixel| pixel[3] == 255),
                    "{relative}: host rock must be opaque"
                );
                // A duplicated 32/64px image patch must not masquerade as
                // a complete 1m authored material. This cannot assess taste.
                for step in [32, 64] {
                    let mut difference = 0usize;
                    for y in 0..side {
                        for x in 0..side {
                            for channel in 0..3 {
                                difference += pixel(x, y)[channel]
                                    .abs_diff(pixel((x + step) % side, (y + step) % side)[channel])
                                    as usize;
                            }
                        }
                    }
                    let average = difference as f32 / (side * side * 3) as f32;
                    assert!(
                        average > 4.0,
                        "{relative}: suspicious {step}px period (mean delta {average:.2})"
                    );
                }
            }
        }
    }

    #[test]
    fn repo_catalog_has_55_sources_and_only_three_unapproved_candidates() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog: Catalog = serde_json::from_slice(
            &fs::read(root.join("assets/materials/catalog.v2.json")).unwrap(),
        )
        .unwrap();
        let legacy: Vec<Legacy> =
            serde_json::from_slice(&fs::read(root.join("assets/textures/manifest.json")).unwrap())
                .unwrap();
        validate(&catalog, &legacy, root).unwrap();
        assert_eq!(legacy.len(), 55);
        assert_eq!(
            catalog
                .materials
                .iter()
                .filter(|item| item.status == "needs_art")
                .count(),
            52
        );
        assert_eq!(
            catalog
                .materials
                .iter()
                .filter(|item| item.status == "candidate")
                .count(),
            3
        );
        assert!(catalog
            .materials
            .iter()
            .all(|item| item.status != "approved"));
        assert_eq!(catalog.texture_assets.len(), 3);
        assert_eq!(material_recipes(&catalog).len(), 3);
    }
}
