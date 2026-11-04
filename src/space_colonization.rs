//! Deterministic, headless space-colonization skeleton prototype.
//!
//! This produces connected line segments, not terrain, plants, caves or ore.
//! Callers own attractor sampling, geological/habitat constraints, masks,
//! resource quantities, and any persisted changes. Coordinates are local
//! Cartesian metres (Y up), not hexel indices or toroidal geography.

use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct GrowthParams {
    pub step_length: f32,
    pub influence_radius: f32,
    pub kill_radius: f32,
    pub tropism: Vec3,
    pub max_iterations: usize,
    pub max_nodes: usize,
}

impl GrowthParams {
    fn valid(self) -> bool {
        self.step_length.is_finite()
            && self.step_length > 0.0
            && self.influence_radius.is_finite()
            && self.influence_radius > 0.0
            && self.kill_radius.is_finite()
            && self.kill_radius > 0.0
            && self.kill_radius <= self.influence_radius
            && self.tropism.is_finite()
            && self.max_iterations > 0
            && self.max_nodes > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowthNode {
    pub position: Vec3,
    /// The root has no parent; every other node belongs to a connected edge.
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    AllAttractorsReached,
    NoGrowth,
    IterationLimit,
    NodeLimit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrowthResult {
    /// Parents always refer to earlier entries in this vector.
    pub nodes: Vec<GrowthNode>,
    pub remaining_attractors: usize,
    pub stop_reason: StopReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrowthError {
    InvalidParameters,
    InvalidPosition,
}

/// Grow a rooted graph toward preselected attractors.
///
/// Each attractor votes for its nearest node when that node lies within the
/// influence radius. Influenced nodes extend one segment in the averaged vote
/// direction, optionally biased by tropism. Reached attractors are removed.
///
/// `segment_allowed` must check the *entire* candidate segment if obstacle
/// avoidance matters; checking only its endpoint can cross solid rock. The
/// closure must be deterministic for deterministic output. This reference
/// implementation scans all nodes for every attractor, so use small bounded
/// offline fixtures until spatial indexing and benchmarks exist.
pub fn grow(
    root: Vec3,
    attractors: &[Vec3],
    params: GrowthParams,
    segment_allowed: impl Fn(Vec3, Vec3) -> bool,
) -> Result<GrowthResult, GrowthError> {
    if !params.valid() {
        return Err(GrowthError::InvalidParameters);
    }
    if !root.is_finite() || attractors.iter().any(|point| !point.is_finite()) {
        return Err(GrowthError::InvalidPosition);
    }

    let mut nodes = vec![GrowthNode {
        position: root,
        parent: None,
    }];
    let mut remaining = attractors.to_vec();
    let influence_sq = params.influence_radius * params.influence_radius;
    let kill_sq = params.kill_radius * params.kill_radius;
    // Prevent a node from repeatedly adding essentially coincident children.
    let separation_sq = (params.step_length * 0.25).powi(2);
    let mut stop_reason = StopReason::IterationLimit;

    'iterations: for _ in 0..params.max_iterations {
        let mut votes = vec![Vec3::ZERO; nodes.len()];
        let mut affected = vec![false; nodes.len()];
        let mut survivors = Vec::with_capacity(remaining.len());

        for point in remaining.drain(..) {
            let mut closest = 0;
            let mut distance_sq = f32::INFINITY;
            // Stable iteration order determines ties; no HashMap ordering.
            for (index, node) in nodes.iter().enumerate() {
                let candidate_sq = point.distance_squared(node.position);
                if candidate_sq < distance_sq {
                    distance_sq = candidate_sq;
                    closest = index;
                }
            }
            if distance_sq <= kill_sq {
                continue;
            }
            if distance_sq <= influence_sq {
                votes[closest] += (point - nodes[closest].position).normalize_or_zero();
                affected[closest] = true;
            }
            survivors.push(point);
        }
        remaining = survivors;
        if remaining.is_empty() {
            stop_reason = StopReason::AllAttractorsReached;
            break;
        }

        let previous_node_count = nodes.len();
        let mut added = 0;
        for index in 0..previous_node_count {
            if !affected[index] {
                continue;
            }
            let direction = votes[index].normalize_or_zero();
            if direction.length_squared() < 1e-12 {
                continue;
            }
            let direction = (direction + params.tropism).normalize_or_zero();
            if direction.length_squared() < 1e-12 {
                continue;
            }

            let start = nodes[index].position;
            let end = start + direction * params.step_length;
            if !end.is_finite() || !segment_allowed(start, end) {
                continue;
            }
            if nodes
                .iter()
                .any(|node| node.position.distance_squared(end) < separation_sq)
            {
                continue;
            }
            if nodes.len() >= params.max_nodes {
                stop_reason = StopReason::NodeLimit;
                break 'iterations;
            }
            nodes.push(GrowthNode {
                position: end,
                parent: Some(index),
            });
            added += 1;
        }
        if added == 0 {
            stop_reason = StopReason::NoGrowth;
            break;
        }
    }

    // Also account for attractors reached by the final iteration's new nodes.
    remaining.retain(|point| {
        !nodes
            .iter()
            .any(|node| point.distance_squared(node.position) <= kill_sq)
    });
    if remaining.is_empty() {
        stop_reason = StopReason::AllAttractorsReached;
    }
    Ok(GrowthResult {
        nodes,
        remaining_attractors: remaining.len(),
        stop_reason,
    })
}

/// Squared world-space distance to a finite skeleton edge.
///
/// Useful for later hexel occupancy or cross-hexel appearance masks. This
/// function deliberately does not alter substrate or resource inventory.
pub fn point_segment_distance_squared(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let edge = end - start;
    let length_sq = edge.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance_squared(start);
    }
    let fraction = ((point - start).dot(edge) / length_sq).clamp(0.0, 1.0);
    point.distance_squared(start + edge * fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> GrowthParams {
        GrowthParams {
            step_length: 1.0,
            influence_radius: 6.0,
            kill_radius: 0.25,
            tropism: Vec3::ZERO,
            max_iterations: 20,
            max_nodes: 100,
        }
    }

    #[test]
    fn straight_growth_is_connected_and_consumes_its_target() {
        let result = grow(Vec3::ZERO, &[Vec3::new(3.0, 0.0, 0.0)], params(), |_, _| {
            true
        })
        .unwrap();
        assert_eq!(result.stop_reason, StopReason::AllAttractorsReached);
        assert_eq!(result.remaining_attractors, 0);
        assert_eq!(result.nodes.len(), 4);
        assert_eq!(result.nodes[0].parent, None);
        for index in 1..result.nodes.len() {
            assert_eq!(result.nodes[index].parent, Some(index - 1));
            assert_eq!(result.nodes[index].position.x, index as f32);
        }
    }

    #[test]
    fn obstacles_block_growth_without_disconnected_edges() {
        let result = grow(
            Vec3::ZERO,
            &[Vec3::new(4.0, 0.0, 0.0)],
            params(),
            |_, end| end.x <= 1.0,
        )
        .unwrap();
        assert_eq!(result.stop_reason, StopReason::NoGrowth);
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.remaining_attractors, 1);
    }

    #[test]
    fn repeated_inputs_have_identical_results() {
        let points = [
            Vec3::new(4.0, 1.0, 0.0),
            Vec3::new(2.0, 3.0, 1.0),
            Vec3::new(-2.0, 2.0, 0.5),
        ];
        let first = grow(Vec3::ZERO, &points, params(), |_, _| true).unwrap();
        let second = grow(Vec3::ZERO, &points, params(), |_, _| true).unwrap();
        assert_eq!(first, second);
        for (index, node) in first.nodes.iter().enumerate().skip(1) {
            assert!(node.parent.unwrap() < index);
        }
    }

    #[test]
    fn node_budget_is_enforced() {
        let mut limited = params();
        limited.max_nodes = 2;
        let result = grow(Vec3::ZERO, &[Vec3::new(5.0, 0.0, 0.0)], limited, |_, _| {
            true
        })
        .unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.stop_reason, StopReason::NodeLimit);
    }

    #[test]
    fn rejects_malformed_inputs_and_preserves_an_empty_fixture() {
        let mut invalid = params();
        invalid.kill_radius = 10.0;
        assert_eq!(
            grow(Vec3::ZERO, &[], invalid, |_, _| true),
            Err(GrowthError::InvalidParameters)
        );
        assert_eq!(
            grow(Vec3::ZERO, &[Vec3::NAN], params(), |_, _| true),
            Err(GrowthError::InvalidPosition)
        );
        let empty = grow(Vec3::ZERO, &[], params(), |_, _| true).unwrap();
        assert_eq!(empty.nodes.len(), 1);
        assert_eq!(empty.stop_reason, StopReason::AllAttractorsReached);
    }

    #[test]
    fn edge_distance_is_finite_and_clamped_to_endpoints() {
        let start = Vec3::ZERO;
        let end = Vec3::new(2.0, 0.0, 0.0);
        assert_eq!(
            point_segment_distance_squared(Vec3::new(1.0, 1.0, 0.0), start, end),
            1.0
        );
        assert_eq!(
            point_segment_distance_squared(Vec3::new(4.0, 0.0, 0.0), start, end),
            4.0
        );
        assert_eq!(
            point_segment_distance_squared(Vec3::new(0.0, 2.0, 0.0), start, start),
            4.0
        );
    }
}
