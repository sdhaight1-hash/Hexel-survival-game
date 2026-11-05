//! Opt-in V4 PNG texture loader. Source art is never altered or treated as approved.
use std::{fs::File, io::BufReader, path::PathBuf};

pub const ROCKS: [(&str, &str); 23] = [
    ("sedimentary", "bauxite"),
    ("sedimentary", "chalk"),
    ("sedimentary", "chert"),
    ("sedimentary", "claystone"),
    ("sedimentary", "conglomerate"),
    ("sedimentary", "limestone"),
    ("sedimentary", "sandstone"),
    ("sedimentary", "shale"),
    ("metamorphic", "phyllite"),
    ("metamorphic", "slate"),
    ("metamorphic", "green_marble"),
    ("metamorphic", "red_marble"),
    ("metamorphic", "white_marble"),
    ("igneous_volcanic", "andesite"),
    ("igneous_volcanic", "granite"),
    ("igneous_volcanic", "kimberlite"),
    ("igneous_volcanic", "peridotite"),
    ("igneous_volcanic", "basalt"),
    ("igneous_volcanic", "obsidian"),
    ("igneous_volcanic", "scoria"),
    ("igneous_volcanic", "travertine"),
    ("igneous_volcanic", "tuff"),
    ("igneous_volcanic", "suevite"),
];
pub const SIZE: u32 = 256;
pub const LAYERS: u32 = 69;

/// Order matches the mesh mapping: index*3 + (top=0, side=1, bottom=2).
/// Files must be extracted from the V4 ZIP by scripts/install_rock_gallery_v4.py.
pub fn load_layers() -> Result<Vec<Vec<u8>>, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/rock-gallery-v4");
    let mut layers = Vec::with_capacity(LAYERS as usize);
    for (family, rock) in ROCKS {
        for face in ["top", "side", "bottom"] {
            let path = root.join(family).join(rock).join(format!("{rock}_{face}.png"));
            let file = File::open(&path).map_err(|e| format!(
                "Missing V4 gallery texture {}: {e}. Run scripts/install_rock_gallery_v4.py with the V4 ZIP.",
                path.display()
            ))?;
            let decoder = png::Decoder::new(BufReader::new(file));
            let mut reader = decoder.read_info().map_err(|e| format!("{}: {e}", path.display()))?;
            let mut buffer = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buffer)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if info.width != SIZE || info.height != SIZE
                || info.color_type != png::ColorType::Rgba
                || info.bit_depth != png::BitDepth::Eight
            {
                return Err(format!("{}: expected 256x256 RGBA8 PNG", path.display()));
            }
            layers.push(buffer[..info.buffer_size()].to_vec());
        }
    }
    Ok(layers)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gallery_has_one_material_per_repo_rock() {
        assert_eq!(ROCKS.len(), 23);
        assert_eq!(ROCKS.len() * 3, LAYERS as usize);
    }
}
