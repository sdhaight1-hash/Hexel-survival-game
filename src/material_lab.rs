//! Authoritative *demo fixture only*: fixed gallery labels for P0 material QA.
//! No species, ore deposits, biome or world-generation rule is inferred here.
use crate::material_contract::CoverDensity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RockHost {
    Granite,
    Shale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabSample {
    Soil(CoverDensity),
    Rock { host: RockHost, show_copper: bool },
}

/// A flat reference strip facing the existing first-person spawn, r=9..12.
/// The rock display is an artistic composite, *not* finite mineable ore.
pub fn surface_y(r: i32) -> i32 {
    // Slightly raised middle row exposes all six-facing grass/ore materials.
    if r == 10 {
        3
    } else {
        2
    }
}

pub fn sample(q: i32, r: i32) -> Option<LabSample> {
    if !(9..=12).contains(&r) {
        return None;
    }
    match q {
        2..=4 => Some(LabSample::Soil(CoverDensity::Bare)),
        5..=6 => Some(LabSample::Soil(CoverDensity::VerySparse)),
        7..=8 => Some(LabSample::Soil(CoverDensity::Sparse)),
        9..=10 => Some(LabSample::Soil(CoverDensity::Full)),
        11 => Some(LabSample::Rock {
            host: RockHost::Granite,
            show_copper: false,
        }),
        12 => Some(LabSample::Rock {
            host: RockHost::Granite,
            show_copper: true,
        }),
        13 => Some(LabSample::Rock {
            host: RockHost::Shale,
            show_copper: false,
        }),
        14 => Some(LabSample::Rock {
            host: RockHost::Shale,
            show_copper: true,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_contains_four_cover_states_and_both_host_rocks() {
        assert_eq!(sample(2, 9), Some(LabSample::Soil(CoverDensity::Bare)));
        assert_eq!(
            sample(5, 9),
            Some(LabSample::Soil(CoverDensity::VerySparse))
        );
        assert_eq!(sample(7, 9), Some(LabSample::Soil(CoverDensity::Sparse)));
        assert_eq!(sample(9, 9), Some(LabSample::Soil(CoverDensity::Full)));
        assert_eq!(
            sample(12, 9),
            Some(LabSample::Rock {
                host: RockHost::Granite,
                show_copper: true
            })
        );
        assert_eq!(
            sample(14, 9),
            Some(LabSample::Rock {
                host: RockHost::Shale,
                show_copper: true
            })
        );
        assert_eq!(sample(8, 8), None);
        assert_eq!(sample(15, 9), None);
    }
}
