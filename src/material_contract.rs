//! Renderer-independent material/vegetation contracts, not live gameplay.
//! Substrate, cover, grass plants and ore occurrences remain separate facts.

pub type TextureId = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureLayers {
    pub base: TextureId,
    /// Transparent overlay over an existing host or substrate texture.
    pub overlay: Option<TextureId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceTextures {
    pub top: TextureLayers,
    pub side: TextureLayers,
    pub bottom: TextureLayers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexFace {
    Top,
    Side(u8),
    Bottom,
}

impl FaceTextures {
    /// Hexagonal prisms have two caps and *six* vertical sides.
    pub fn for_face(&self, face: HexFace) -> Option<&TextureLayers> {
        match face {
            HexFace::Top => Some(&self.top),
            HexFace::Side(0..=5) => Some(&self.side),
            HexFace::Side(_) => None,
            HexFace::Bottom => Some(&self.bottom),
        }
    }
}

/// Ordinal ground-cover density, not a soil fertility or plant height value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoverDensity {
    #[default]
    Bare,
    VerySparse,
    Sparse,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundCoverKind {
    Grass,
    Moss,
    LeafLitter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceAppearance {
    pub substrate_id: &'static str,
    pub cover_kind: Option<GroundCoverKind>,
    pub density: CoverDensity,
}

impl SurfaceAppearance {
    pub fn is_consistent(&self) -> bool {
        self.cover_kind.is_none() == (self.density == CoverDensity::Bare)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrassHeight {
    VeryShort,
    Short,
    MediumShort,
    Medium,
    Tall,
    VeryTall,
}

/// Grass tufts are independent vegetation, never soil-block texture variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrassPlant {
    pub species_id: &'static str,
    pub height: GrassHeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OreGrade {
    Poor,
    Medium,
    Rich,
    Bountiful,
}

/// Mineable ore data identifies both the host rock and the ore.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OreOccurrence {
    pub host_rock_id: &'static str,
    pub ore_id: &'static str,
    pub grade: OreGrade,
}

/// Compose an ore-only alpha overlay over a particular rock-face texture.
/// Existing opaque ore swatches are NOT suitable as these overlays.
pub fn ore_face(host_rock_texture: TextureId, ore_overlay_texture: TextureId) -> TextureLayers {
    TextureLayers {
        base: host_rock_texture,
        overlay: Some(ore_overlay_texture),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_hex_faces_use_explicit_recipes_and_invalid_sides_are_rejected() {
        let faces = FaceTextures {
            top: TextureLayers {
                base: "soil/top",
                overlay: Some("grass/cover_top"),
            },
            side: TextureLayers {
                base: "soil/side",
                overlay: Some("grass/side_transition"),
            },
            bottom: TextureLayers {
                base: "soil/bottom",
                overlay: None,
            },
        };
        assert_eq!(faces.for_face(HexFace::Top), Some(&faces.top));
        assert_eq!(faces.for_face(HexFace::Bottom), Some(&faces.bottom));
        for i in 0..6 {
            assert_eq!(faces.for_face(HexFace::Side(i)), Some(&faces.side));
        }
        assert_eq!(faces.for_face(HexFace::Side(6)), None);
        assert_ne!(faces.top, faces.side);
        assert!(faces.bottom.overlay.is_none());
    }

    #[test]
    fn soil_fertility_does_not_change_when_ground_cover_changes() {
        let bare = SurfaceAppearance {
            substrate_id: "soil.low_fertility",
            cover_kind: None,
            density: CoverDensity::Bare,
        };
        let grassy = SurfaceAppearance {
            cover_kind: Some(GroundCoverKind::Grass),
            density: CoverDensity::Full,
            ..bare
        };
        assert!(bare.is_consistent());
        assert!(grassy.is_consistent());
        assert_eq!(bare.substrate_id, grassy.substrate_id);
        assert!(!SurfaceAppearance {
            cover_kind: Some(GroundCoverKind::Grass),
            ..bare
        }.is_consistent());
    }

    #[test]
    fn grass_species_and_height_are_separate_from_soil_and_cover() {
        let short = GrassPlant {
            species_id: "grass.redtop",
            height: GrassHeight::Short,
        };
        let tall = GrassPlant {
            height: GrassHeight::VeryTall,
            ..short
        };
        assert_eq!(short.species_id, tall.species_id);
        assert_ne!(short.height, tall.height);
    }

    #[test]
    fn ore_overlay_preserves_selected_host_material() {
        let occurrence = OreOccurrence {
            host_rock_id: "rock.granite",
            ore_id: "ore.native_copper",
            grade: OreGrade::Rich,
        };
        let granite = ore_face("rock/granite", "ore/native_copper_alpha");
        let shale = ore_face("rock/shale", "ore/native_copper_alpha");
        assert_eq!(occurrence.ore_id, "ore.native_copper");
        assert_eq!(granite.overlay, shale.overlay);
        assert_ne!(granite.base, shale.base);
    }

    #[test]
    fn one_hex_side_is_narrower_than_one_metre_of_height() {
        // 1m across flats => hex side length ~0.577m, not 1m.
        assert!(crate::world::HEX_RADIUS < crate::world::HEXEL_HEIGHT);
        assert!((crate::world::HEX_RADIUS * 3.0_f32.sqrt() - 1.0).abs() < 0.0001);
    }
}
