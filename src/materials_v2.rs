//! Read-only runtime validation for the offline v2 material handoff.
//! This does not replace the P0 gallery or assign transient GPU texture slots.

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

fn validated(index: Index, directory: &Path) -> Result<(usize, usize), String> {
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
            || texture.repeat_metres.iter().any(|v| !v.is_finite() || *v <= 0.0)
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
        match (&material.representation[..], &material.faces, &material.overlays) {
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

/// Absence of an index is normal until original production art is authored.
/// An invalid *present* index is never silently accepted.
pub fn inspect_default() -> Result<Option<(usize, usize)>, String> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/materials-v2");
    let path = directory.join("index.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Could not read {}: {error}", path.display())),
    };
    let index: Index = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Invalid v2 index JSON: {e}"))?;
    validated(index, &directory).map(Some)
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
        assert_eq!(validated(empty(), Path::new(".")).unwrap(), (0, 0));
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
                top: Face { base: "rock.missing".into(), overlay: None },
                side: Face { base: "rock.missing".into(), overlay: None },
                bottom: Face { base: "rock.missing".into(), overlay: None },
            }),
            overlays: None,
        });
        assert!(validated(index, Path::new(".")).is_err());
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
            overlays: Some(Overlays { top: "ore.copper".into(), side: None, bottom: None }),
        });
        assert!(validated(index, Path::new(".")).is_err());
        assert!(!valid_id("rock.Granite"));
        assert!(!valid_id("rock.granite.extra"));
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
        assert!(validated(index, Path::new(".")).is_err());
    }
}
