//! Headless first-person movement for a finite local demonstration chunk.
//!
//! Feet position is game-world space, independent of the graphics camera.
//! A real streamed world will replace the demo boundary with chunk lookups.

use crate::world::World;
use glam::Vec3;

const EYE_HEIGHT: f32 = 1.62;
const BODY_HEIGHT: f32 = 1.8;
const BODY_RADIUS: f32 = 0.27;
const GRAVITY: f32 = 22.0;
const JUMP_VELOCITY: f32 = 7.6;
const LOOK_SENSITIVITY: f32 = 0.0022;

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub sneak: bool,
}

#[derive(Debug)]
pub struct Player {
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
    vertical_velocity: f32,
}

impl Player {
    pub fn spawn(world: &World) -> Self {
        let center = crate::world::CellPos::new(8, 7, 0).center();
        let ground = world.support_top(center.x, center.z).unwrap_or(1.0);
        Self {
            feet: Vec3::new(center.x, ground, center.z),
            yaw: 0.0,
            pitch: -0.12,
            grounded: true,
            vertical_velocity: 0.0,
        }
    }

    pub fn eye_position(&self) -> Vec3 {
        self.feet + Vec3::Y * EYE_HEIGHT
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * LOOK_SENSITIVITY;
        self.pitch = (self.pitch - dy * LOOK_SENSITIVITY).clamp(-1.52, 1.52);
    }

    pub fn step(&mut self, world: &World, input: MoveInput, dt: f32) {
        let dt = dt.clamp(0.0, 0.04);
        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        // Camera right for a right-handed look_to_rh view.
        let right = Vec3::new(-self.yaw.cos(), 0.0, self.yaw.sin());
        let mut direction = Vec3::ZERO;
        if input.forward {
            direction += forward;
        }
        if input.backward {
            direction -= forward;
        }
        if input.right {
            direction += right;
        }
        if input.left {
            direction -= right;
        }

        let speed = if input.sneak {
            1.8
        } else if input.sprint {
            6.0
        } else {
            4.3
        };
        let delta = direction.normalize_or_zero() * speed * dt;
        let candidate = self.feet + delta;
        if can_occupy(world, candidate) {
            self.feet.x = candidate.x;
            self.feet.z = candidate.z;
        } else {
            // Axis-separated fallback slides along walls and avoids corner sticking.
            if can_occupy(world, Vec3::new(candidate.x, self.feet.y, self.feet.z)) {
                self.feet.x = candidate.x;
            }
            if can_occupy(world, Vec3::new(self.feet.x, self.feet.y, candidate.z)) {
                self.feet.z = candidate.z;
            }
        }

        if input.jump && self.grounded {
            self.vertical_velocity = JUMP_VELOCITY;
            self.grounded = false;
        }

        self.vertical_velocity -= GRAVITY * dt;
        self.feet.y += self.vertical_velocity * dt;
        let ground = footprint(world, self.feet.x, self.feet.z)
            .filter(|&(x, z)| world.contains_demo_footprint(x, z))
            .filter_map(|(x, z)| world.support_top(x, z))
            .fold(0.0_f32, f32::max);
        if self.vertical_velocity <= 0.0 && self.feet.y <= ground {
            self.feet.y = ground;
            self.vertical_velocity = 0.0;
            self.grounded = true;
        }
    }
}

fn footprint(x: f32, z: f32) -> impl Iterator<Item = (f32, f32)> {
    let diagonal = BODY_RADIUS * std::f32::consts::FRAC_1_SQRT_2;
    [
        (0.0, 0.0),
        (BODY_RADIUS, 0.0),
        (-BODY_RADIUS, 0.0),
        (0.0, BODY_RADIUS),
        (0.0, -BODY_RADIUS),
        (diagonal, diagonal),
        (diagonal, -diagonal),
        (-diagonal, diagonal),
        (-diagonal, -diagonal),
    ]
    .into_iter()
    .map(move |(dx, dz)| (x + dx, z + dz))
}

fn can_occupy(world: &World, position: Vec3) -> bool {
    footprint(position.x, position.z).all(|(x, z)| {
        if !world.contains_demo_footprint(x, z) {
            return false;
        }
        let Some(top) = world.support_top(x, z) else {
            return false;
        };
        // Since bootstrap terrain is solid to its surface and has no caves,
        // this surface test is equivalent to the player's upright body not
        // intersecting terrain. Replace with individual-cell collision when
        // caves, overhangs, placed blocks and dug terrain are introduced.
        position.y >= top - 0.02 && position.y + BODY_HEIGHT >= top
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CellPos;

    #[test]
    fn spawn_is_grounded_above_terrain() {
        let world = World::demo(42);
        let player = Player::spawn(&world);
        assert!(player.grounded);
        assert!(can_occupy(&world, player.feet));
    }

    #[test]
    fn jump_leaves_ground_and_lands() {
        let world = World::demo(42);
        let mut player = Player::spawn(&world);
        let start = player.feet.y;
        player.step(
            &world,
            MoveInput {
                jump: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert!(!player.grounded);
        assert!(player.feet.y > start);
        for _ in 0..180 {
            player.step(&world, MoveInput::default(), 1.0 / 60.0);
        }
        assert!(player.grounded);
        assert!((player.feet.y - start).abs() < 0.01);
    }

    #[test]
    fn look_has_bounded_pitch_and_movement_is_normalized() {
        let world = World::demo(42);
        let mut a = Player::spawn(&world);
        let mut b = Player::spawn(&world);
        a.look(0.0, -100_000.0);
        assert_eq!(a.pitch, 1.52);
        a.step(
            &world,
            MoveInput {
                forward: true,
                ..Default::default()
            },
            0.02,
        );
        b.step(
            &world,
            MoveInput {
                forward: true,
                right: true,
                ..Default::default()
            },
            0.02,
        );
        let origin = CellPos::new(8, 7, 0).center();
        let da = (Vec3::new(a.feet.x, 0.0, a.feet.z) - Vec3::new(origin.x, 0.0, origin.z)).length();
        let db = (Vec3::new(b.feet.x, 0.0, b.feet.z) - Vec3::new(origin.x, 0.0, origin.z)).length();
        assert!((da - db).abs() < 0.001);
    }

    #[test]
    fn cannot_walk_outside_loaded_demo_chunk() {
        let world = World::demo(42);
        let player = Player::spawn(&world);
        let outside = CellPos::new(-1, 7, 0).center();
        assert!(!can_occupy(
            &world,
            Vec3::new(outside.x, player.feet.y, outside.z)
        ));
    }
}
