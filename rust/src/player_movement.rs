use godot::{classes::Input, global, prelude::*};

/// Player state machine for movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Air,
    Floor,
}

/// Movement configuration constants
pub struct MovementConfig {
    pub walk_speed: f32,
    pub accel_speed: f32,
    pub jump_velocity: f32,
    pub min_walk_speed: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 120.0,
            accel_speed: 720.0, // walk_speed * 6.0
            jump_velocity: -300.0,
            min_walk_speed: 0.1,
        }
    }
}

/// Reusable player movement controller
pub struct PlayerMovement {
    pub state: MovementState,
    pub gravity: f32,
    pub config: MovementConfig,
}

impl PlayerMovement {
    pub fn new(gravity: f32) -> Self {
        Self {
            state: MovementState::Air,
            gravity,
            config: MovementConfig::default(),
        }
    }

    pub fn with_config(gravity: f32, config: MovementConfig) -> Self {
        Self {
            state: MovementState::Air,
            gravity,
            config,
        }
    }

    /// Main physics update - processes gravity, state transitions, and movement
    /// Returns the new velocity to apply to the body
    pub fn physics_process(&mut self, velocity: Vector2, is_on_floor: bool, delta: f64) -> Vector2 {
        let mut new_velocity = velocity;

        // Apply gravity
        new_velocity.y += self.gravity * delta as f32;

        // State machine logic
        match self.state {
            MovementState::Air => {
                if is_on_floor {
                    self.state = MovementState::Floor;
                    return new_velocity;
                }
                self.try_walk(&mut new_velocity, delta);
                self.try_jump(&mut new_velocity, is_on_floor);
            }
            MovementState::Floor => {
                self.try_walk(&mut new_velocity, delta);
                if !is_on_floor {
                    self.state = MovementState::Air;
                } else if self.try_jump(&mut new_velocity, is_on_floor) {
                    self.state = MovementState::Air;
                }
            }
        }

        new_velocity
    }

    /// Handle horizontal movement input
    fn try_walk(&mut self, velocity: &mut Vector2, delta: f64) {
        let input = Input::singleton();
        let direction = input.get_axis("walk_left", "walk_right");
        velocity.x = global::move_toward(
            velocity.x as f64,
            (direction * self.config.walk_speed) as f64,
            self.config.accel_speed as f64 * delta,
        ) as f32;
    }

    /// Handle jump input
    fn try_jump(&mut self, velocity: &mut Vector2, is_on_floor: bool) -> bool {
        let input = Input::singleton();
        let can_jump = input.is_action_just_pressed("jump") && is_on_floor;
        if can_jump {
            velocity.y = self.config.jump_velocity;
        }
        can_jump
    }

    /// Check if player is moving horizontally (useful for animation)
    pub fn is_walking(&self, velocity: Vector2) -> bool {
        velocity.abs().x > self.config.min_walk_speed
    }

    /// Check if player is rising (useful for animation)
    pub fn is_rising(&self, velocity: Vector2) -> bool {
        velocity.y < 0.0
    }

    /// Check if player is falling (useful for animation)
    pub fn is_falling(&self, velocity: Vector2) -> bool {
        velocity.y > 0.0
    }
}
