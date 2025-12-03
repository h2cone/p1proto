use godot::{
    classes::{Input, ProjectSettings},
    global,
    prelude::*,
};

/// Player state machine for movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Air,
    Floor,
}

/// Movement configuration
pub struct MovementConfig {
    pub gravity: f32,
    pub walk_speed: f32,
    pub accel_speed: f32,
    pub jump_velocity: f32,
    pub min_walk_speed: f32,
    pub action_walk_left: String,
    pub action_walk_right: String,
    pub action_jump: String,
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
            action_walk_left: "walk_left".to_string(),
            action_walk_right: "walk_right".to_string(),
            action_jump: "jump".to_string(),
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
    pub fn physics_process(&mut self, velocity: Vector2, is_on_floor: bool, delta: f64) -> Vector2 {
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
        let direction = input.get_axis(
            self.config.action_walk_left.as_str(),
            self.config.action_walk_right.as_str(),
        );
        velocity.x = global::move_toward(
            velocity.x as f64,
            (direction * self.config.walk_speed) as f64,
            self.config.accel_speed as f64 * delta,
        ) as f32;
    }

    /// Handle jump input
    fn try_jump(&mut self, velocity: &mut Vector2, is_on_floor: bool) -> bool {
        let input = Input::singleton();
        let can_jump =
            input.is_action_just_pressed(self.config.action_jump.as_str()) && is_on_floor;
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
