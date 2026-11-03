//! Four real 32x32 material assets for the first hexel scene.
//! Material selection in this demo is only a preview, not geology authority.

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    LowSoil = 0,
    MediumSoil = 1,
    ForestFloor = 2,
    Granite = 3,
}

pub const TEXTURE_SIZE: u32 = 32;
pub const TEXTURES: [&[u8]; 4] = [
    include_bytes!("../assets/textures/soil/low_fertility_soil_32.png"),
    include_bytes!("../assets/textures/soil/medium_fertility_soil_32.png"),
    include_bytes!("../assets/textures/soil/forest_floor_32.png"),
    include_bytes!("../assets/textures/rock/metamorphic_igneous/granite_32.png"),
];

pub fn decoded_rgba(png_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    if info.width != TEXTURE_SIZE
        || info.height != TEXTURE_SIZE
        || info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
    {
        return Err("Material must be a 32x32 8-bit RGBA PNG".into());
    }
    Ok(buffer[..info.buffer_size()].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_textures_decode_as_32x32_rgba() {
        for png in TEXTURES {
            assert_eq!(decoded_rgba(png).unwrap().len(), 32 * 32 * 4);
        }
    }
}
