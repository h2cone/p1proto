use godot::{classes::ProjectSettings, global, prelude::*};

/// Player state machine for movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Air,
    Floor,
}

/// Abstract movement input, decoupled from input source.
/// Can be driven by player input, AI, or replay systems.
#[derive(Default, Clone, Copy)]
pub struct MovementInput {
    /// Horizontal direction: -1.0 (left) to 1.0 (right)
    pub direction: f32,
    /// Whether jump was just pressed this frame
    pub jump_just_pressed: bool,
}

/// Movement configuration
pub struct MovementConfig {
    pub gravity: f32,
    pub walk_speed: f32,
    pub accel_speed: f32,
    pub jump_velocity: f32,
    pub min_walk_speed: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        let settings = ProjectSettings::singleton();
        let gravity = settings.get("physics/2d/default_gravity").to::<f64>() as f32;

        Self {
            gravity,
            walk_speed: 120.0,
            accel_speed: 720.0,
            jump_velocity: -300.0,
            min_walk_speed: 0.1,
        }
    }
}

/// Player movement controller
pub struct PlayerMovement {
    pub state: MovementState,
    pub config: MovementConfig,
}

impl PlayerMovement {
    pub fn new(config: MovementConfig) -> Self {
        Self {
            state: MovementState::Air,
            config,
        }
    }

    /// Main physics update - processes gravity, state transitions, and movement
    /// Returns the new velocity to apply to the body
    pub fn physics_process(
        &mut self,
        velocity: Vector2,
        is_on_floor: bool,
        delta: f64,
        input: MovementInput,
    ) -> Vector2 {
        let mut new_velocity = velocity;

        // Apply gravity
        new_velocity.y += self.config.gravity * delta as f32;

        // State machine logic
        match self.state {
            MovementState::Air => {
                if is_on_floor {
                    self.state = MovementState::Floor;
                    return new_velocity;
                }
                self.apply_walk(&mut new_velocity, delta, input.direction);
                self.apply_jump(&mut new_velocity, is_on_floor, input.jump_just_pressed);
            }
            MovementState::Floor => {
                self.apply_walk(&mut new_velocity, delta, input.direction);
                if !is_on_floor {
                    self.state = MovementState::Air;
                } else if self.apply_jump(&mut new_velocity, is_on_floor, input.jump_just_pressed) {
                    self.state = MovementState::Air;
                }
            }
        }

        new_velocity
    }

    /// Handle horizontal movement
    fn apply_walk(&mut self, velocity: &mut Vector2, delta: f64, direction: f32) {
        velocity.x = global::move_toward(
            velocity.x as f64,
            (direction * self.config.walk_speed) as f64,
            self.config.accel_speed as f64 * delta,
        ) as f32;
    }

    /// Handle jump
    fn apply_jump(
        &mut self,
        velocity: &mut Vector2,
        is_on_floor: bool,
        jump_pressed: bool,
    ) -> bool {
        let can_jump = jump_pressed && is_on_floor;
        if can_jump {
            velocity.y = self.config.jump_velocity;
        }
        can_jump
    }

    /// Check if player is moving horizontally
    pub fn is_walking(&self, velocity: Vector2) -> bool {
        velocity.abs().x > self.config.min_walk_speed
    }
}
