//! Versioned, embedded BC7 asset pack. All heavy compression is done by build.rs.
//! Native GPU sampling transparently decompresses BC7; non-BC GPUs use a one-time
//! CPU decode to RGBA8, avoiding a slow per-frame or per-launch re-encode.

use crate::texture_codec;

const MAGIC: &[u8; 4] = b"HXB7";
const HEADER_BYTES: usize = 20;
const DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/p0_materials.bc7"));

pub struct TexturePack<'a> {
    bytes: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub layers: usize,
    layer_bytes: usize,
}

impl<'a> TexturePack<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_BYTES || &bytes[..4] != MAGIC {
            return Err("Missing P0 BC7 asset pack header".into());
        }
        let word = |offset: usize| {
            u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("fixed header size"),
            )
        };
        if word(4) != 1 {
            return Err("Unsupported P0 BC7 pack version".into());
        }
        let width = word(8);
        let height = word(12);
        let layers = word(16) as usize;
        if layers == 0 || layers > 4096 {
            return Err("Invalid P0 BC7 layer count".into());
        }
        let layer_bytes = texture_codec::bc7_layer_size(width, height)?;
        let expected = layers
            .checked_mul(layer_bytes)
            .and_then(|n| n.checked_add(HEADER_BYTES))
            .ok_or("P0 BC7 pack size overflow")?;
        if bytes.len() != expected {
            return Err(format!(
                "P0 BC7 pack length mismatch: expected {expected}, got {}",
                bytes.len()
            ));
        }
        Ok(Self {
            bytes: &bytes[HEADER_BYTES..],
            width,
            height,
            layers,
            layer_bytes,
        })
    }

    pub fn embedded() -> Result<TexturePack<'static>, String> {
        TexturePack::parse(DATA)
    }

    pub fn bc7_layer(&self, layer: usize) -> &[u8] {
        assert!(layer < self.layers, "Texture layer out of range");
        let offset = layer * self.layer_bytes;
        &self.bytes[offset..offset + self.layer_bytes]
    }

    pub fn decode_layer(&self, layer: usize) -> Result<Vec<u8>, String> {
        texture_codec::decode_bc7(self.bc7_layer(layer), self.width, self.height)
    }

    pub fn rgba_bytes(&self) -> usize {
        self.width as usize * self.height as usize * 4 * self.layers
    }

    pub fn bc7_bytes(&self) -> usize {
        self.layer_bytes * self.layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_pack_decodes_every_material_layer() {
        let pack = TexturePack::embedded().unwrap();
        assert_eq!((pack.width, pack.height, pack.layers), (256, 256, 12));
        assert_eq!(pack.bc7_bytes() * 4, pack.rgba_bytes());
        for layer in 0..pack.layers {
            let decoded = pack.decode_layer(layer).unwrap();
            assert_eq!(decoded.len(), (pack.width * pack.height * 4) as usize);
            assert!(decoded.chunks_exact(4).all(|pixel| pixel[3] == 255));
        }
    }

    #[test]
    fn corrupt_pack_headers_are_rejected() {
        let mut bad = DATA.to_vec();
        bad[4] = 99;
        assert!(TexturePack::parse(&bad).is_err());
        bad[4] = 1;
        bad.truncate(bad.len() - 16);
        assert!(TexturePack::parse(&bad).is_err());
    }
}
