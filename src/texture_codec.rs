//! Safe BC7 block decoding for GPUs without native BC texture support.
//! Offline encoding is provided by the build-time asset baker, not by gameplay.

pub const BC7_BLOCK_BYTES: usize = 16;

pub fn bc7_layer_size(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 || width % 4 != 0 || height % 4 != 0 {
        return Err("BC7 width/height must be nonzero multiples of 4".into());
    }
    let blocks_w = usize::try_from(width / 4).map_err(|e| e.to_string())?;
    let blocks_h = usize::try_from(height / 4).map_err(|e| e.to_string())?;
    blocks_w
        .checked_mul(blocks_h)
        .and_then(|n| n.checked_mul(BC7_BLOCK_BYTES))
        .ok_or_else(|| "BC7 layer byte count overflow".into())
}

/// Decode GPU BC7 blocks into the same RGBA8 layout used by our fallback.
/// The GPU path never calls this; it samples the compressed blocks directly.
pub fn decode_bc7(bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let expected = bc7_layer_size(width, height)?;
    if bytes.len() != expected {
        return Err(format!("BC7 byte count mismatch: expected {expected}, got {}", bytes.len()));
    }
    let width = width as usize;
    let height = height as usize;
    let stride = width.checked_mul(4).ok_or("RGBA row size overflow")?;
    let count = stride.checked_mul(height).ok_or("RGBA buffer size overflow")?;
    let mut pixels = vec![0u8; count];
    let blocks_w = width / 4;
    for by in 0..height / 4 {
        for bx in 0..blocks_w {
            let input = (by * blocks_w + bx) * BC7_BLOCK_BYTES;
            let mut block = [0u8; 4 * 4 * 4];
            bcdec_rs::bc7(&bytes[input..input + BC7_BLOCK_BYTES], &mut block, 16);
            for row in 0..4 {
                let target = (by * 4 + row) * stride + bx * 16;
                pixels[target..target + 16].copy_from_slice(&block[row * 16..row * 16 + 16]);
            }
        }
    }
    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bc7_validates_block_layout() {
        assert_eq!(bc7_layer_size(256, 256).unwrap(), 65536);
        assert!(bc7_layer_size(255, 256).is_err());
        assert!(bc7_layer_size(0, 4).is_err());
        assert!(decode_bc7(&[0; 12], 4, 4).is_err());
    }
}
