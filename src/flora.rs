//! Headless ecological eligibility contracts for future world generation.
//!
//! No plants are spawned by this module. The flora data catalog is research
//! material, not game balance. Worldgen, established-plant survival, growth,
//! and local establishment must use distinct rule sets in later milestones.

/// All normalized climate fractions are in [0, 1], NOT raw 0..255.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HabitatSample {
    pub mean_temperature_c: f32,
    pub annual_rain_fraction: f32,
    pub forestation_fraction: f32,
    /// A *future Hexel Survival* normalized terrain-elevation input, not
    /// Vintage Story's height or world Y index.
    pub elevation_fraction: f32,
}

impl HabitatSample {
    pub fn valid(self) -> bool {
        self.mean_temperature_c.is_finite()
            && [
                self.annual_rain_fraction,
                self.forestation_fraction,
                self.elevation_fraction,
            ]
            .into_iter()
            .all(|x| x.is_finite() && (0.0..=1.0).contains(&x))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InclusiveRange {
    pub min: f32,
    pub max: f32,
}

impl InclusiveRange {
    pub fn valid(self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.min <= self.max
    }

    pub fn includes(self, value: f32) -> bool {
        self.valid() && value.is_finite() && value >= self.min && value <= self.max
    }
}

/// An unspecified rule is NOT the same as permission to spawn anywhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClimateConstraint {
    /// Source doesn't establish an acceptable interval.
    Unknown,
    /// Explicitly reviewed to impose no restriction on this dimension.
    Unconstrained,
    Interval(InclusiveRange),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HabitatRules {
    pub mean_temperature_c: ClimateConstraint,
    pub annual_rain_fraction: ClimateConstraint,
    pub forestation_fraction: ClimateConstraint,
    pub elevation_fraction: ClimateConstraint,
}

/// Research-only species can never silently enter the actual generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleApproval {
    ResearchOnly,
    Approved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eligibility {
    RequiresApproval,
    InvalidInput,
    InvalidRule,
    OutsideRange,
    MissingRule,
    Suitable,
}

impl HabitatRules {
    pub fn assess(self, sample: HabitatSample, approval: RuleApproval) -> Eligibility {
        if approval != RuleApproval::Approved {
            return Eligibility::RequiresApproval;
        }
        if !sample.valid() {
            return Eligibility::InvalidInput;
        }
        let constraints = [
            (self.mean_temperature_c, sample.mean_temperature_c, false),
            (self.annual_rain_fraction, sample.annual_rain_fraction, true),
            (self.forestation_fraction, sample.forestation_fraction, true),
            (self.elevation_fraction, sample.elevation_fraction, true),
        ];
        // Validate every interval before checking suitability, so a
        // malformed registry record can never become a normal habitat.
        for &(rule, _, is_fraction) in &constraints {
            if let ClimateConstraint::Interval(range) = rule {
                if !range.valid() || (is_fraction && (range.min < 0.0 || range.max > 1.0)) {
                    return Eligibility::InvalidRule;
                }
            }
        }
        let mut unknown = false;
        for (rule, value, _) in constraints {
            match rule {
                ClimateConstraint::Unknown => unknown = true,
                ClimateConstraint::Unconstrained => {}
                ClimateConstraint::Interval(range) if !range.includes(value) => {
                    return Eligibility::OutsideRange;
                }
                ClimateConstraint::Interval(_) => {}
            }
        }
        if unknown {
            Eligibility::MissingRule
        } else {
            Eligibility::Suitable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HabitatSample {
        HabitatSample {
            mean_temperature_c: 12.0,
            annual_rain_fraction: 0.6,
            forestation_fraction: 0.45,
            elevation_fraction: 0.3,
        }
    }

    #[test]
    fn research_bands_never_authorize_world_generation() {
        let rules = HabitatRules {
            mean_temperature_c: ClimateConstraint::Interval(InclusiveRange {
                min: 0.0,
                max: 24.0,
            }),
            annual_rain_fraction: ClimateConstraint::Unconstrained,
            forestation_fraction: ClimateConstraint::Unconstrained,
            elevation_fraction: ClimateConstraint::Unconstrained,
        };
        assert_eq!(
            rules.assess(sample(), RuleApproval::ResearchOnly),
            Eligibility::RequiresApproval
        );
        assert_eq!(
            rules.assess(sample(), RuleApproval::Approved),
            Eligibility::Suitable
        );
    }

    #[test]
    fn unknown_is_not_equivalent_to_unrestricted() {
        let rules = HabitatRules {
            mean_temperature_c: ClimateConstraint::Unknown,
            annual_rain_fraction: ClimateConstraint::Unconstrained,
            forestation_fraction: ClimateConstraint::Unconstrained,
            elevation_fraction: ClimateConstraint::Unconstrained,
        };
        assert_eq!(
            rules.assess(sample(), RuleApproval::Approved),
            Eligibility::MissingRule
        );
    }

    #[test]
    fn habitat_bounds_are_inclusive_and_fail_closed() {
        let rules = HabitatRules {
            mean_temperature_c: ClimateConstraint::Interval(InclusiveRange {
                min: 12.0,
                max: 18.0,
            }),
            annual_rain_fraction: ClimateConstraint::Interval(InclusiveRange {
                min: 0.6,
                max: 0.9,
            }),
            forestation_fraction: ClimateConstraint::Unconstrained,
            elevation_fraction: ClimateConstraint::Unconstrained,
        };
        assert_eq!(
            rules.assess(sample(), RuleApproval::Approved),
            Eligibility::Suitable
        );
        let dry = HabitatSample {
            annual_rain_fraction: 0.59,
            ..sample()
        };
        assert_eq!(
            rules.assess(dry, RuleApproval::Approved),
            Eligibility::OutsideRange
        );
        let invalid = HabitatSample {
            annual_rain_fraction: f32::NAN,
            ..sample()
        };
        assert_eq!(
            rules.assess(invalid, RuleApproval::Approved),
            Eligibility::InvalidInput
        );
    }

    #[test]
    fn inverted_reference_range_is_not_eligible() {
        let rules = HabitatRules {
            mean_temperature_c: ClimateConstraint::Unconstrained,
            annual_rain_fraction: ClimateConstraint::Interval(InclusiveRange {
                min: 0.5,
                max: 0.1,
            }),
            forestation_fraction: ClimateConstraint::Unconstrained,
            elevation_fraction: ClimateConstraint::Unconstrained,
        };
        assert_eq!(
            rules.assess(sample(), RuleApproval::Approved),
            Eligibility::InvalidRule
        );
    }

    #[test]
    fn raw_255_rainfall_must_be_normalized_before_evaluation() {
        assert!((95.0_f32 / 255.0 - 0.372_549).abs() < 0.00001);
        let invalid = HabitatSample {
            annual_rain_fraction: 95.0,
            ..sample()
        };
        assert!(!invalid.valid());
    }
}
