//! Appearance-only surface conditions, independent of substrate and ore IDs.
//!
//! The current game uses these as explicit gallery presets. Future weather,
//! ecology and persistence systems must own their authoritative values.
//! Values are cover/intensity fractions, not texture or GPU layer indices.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceConditions {
    pub wetness: f32,
    pub moss: f32,
    pub frost: f32,
}

impl SurfaceConditions {
    pub fn is_valid(self) -> bool {
        [self.wetness, self.moss, self.frost]
            .into_iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
    }
}

/// Deterministic diagnostic states, not simulated weather or ecology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfacePreset {
    #[default]
    Dry,
    Wet,
    Moss,
    Frost,
    Mixed,
}

impl SurfacePreset {
    pub fn next(self) -> Self {
        match self {
            Self::Dry => Self::Wet,
            Self::Wet => Self::Moss,
            Self::Moss => Self::Frost,
            Self::Frost => Self::Mixed,
            Self::Mixed => Self::Dry,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dry => "Dry",
            Self::Wet => "Wet",
            Self::Moss => "Moss / lichen",
            Self::Frost => "Frost",
            Self::Mixed => "Combined",
        }
    }

    pub fn conditions(self) -> SurfaceConditions {
        match self {
            Self::Dry => SurfaceConditions::default(),
            Self::Wet => SurfaceConditions {
                wetness: 0.85,
                ..SurfaceConditions::default()
            },
            Self::Moss => SurfaceConditions {
                moss: 0.85,
                ..SurfaceConditions::default()
            },
            Self::Frost => SurfaceConditions {
                frost: 0.85,
                ..SurfaceConditions::default()
            },
            Self::Mixed => SurfaceConditions {
                wetness: 0.55,
                moss: 0.55,
                frost: 0.35,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_cycle_without_changing_material_identity() {
        let mut current = SurfacePreset::Dry;
        let mut visited = Vec::new();
        for _ in 0..5 {
            visited.push(current);
            current = current.next();
        }
        assert_eq!(
            visited.as_slice(),
            &[
                SurfacePreset::Dry,
                SurfacePreset::Wet,
                SurfacePreset::Moss,
                SurfacePreset::Frost,
                SurfacePreset::Mixed
            ]
        );
        assert_eq!(current, SurfacePreset::Dry);
    }

    #[test]
    fn condition_families_are_independent_and_bounded() {
        for preset in [
            SurfacePreset::Dry,
            SurfacePreset::Wet,
            SurfacePreset::Moss,
            SurfacePreset::Frost,
            SurfacePreset::Mixed,
        ] {
            assert!(preset.conditions().is_valid());
        }
        assert_eq!(SurfacePreset::Wet.conditions().moss, 0.0);
        assert_eq!(SurfacePreset::Moss.conditions().wetness, 0.0);
        assert_eq!(SurfacePreset::Frost.conditions().moss, 0.0);
        let mixed = SurfacePreset::Mixed.conditions();
        assert!(mixed.wetness > 0.0 && mixed.moss > 0.0 && mixed.frost > 0.0);
        assert!(!SurfaceConditions {
            wetness: f32::NAN,
            ..mixed
        }
        .is_valid());
        assert!(!SurfaceConditions { moss: 1.2, ..mixed }.is_valid());
    }
}
