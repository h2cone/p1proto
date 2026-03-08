use godot::{classes::ProjectSettings, prelude::*};

const INPUT_DEADZONE: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Air,
    Floor,
}

#[derive(Default, Clone, Copy)]
pub struct MovementInput {
    pub direction: f32,
    pub jump_just_pressed: bool,
    pub jump_just_released: bool,
}

pub struct MovementConfig {
    pub gravity: f32,
    pub walk_speed: f32,
    pub ground_accel_speed: f32,
    pub ground_decel_speed: f32,
    pub air_accel_speed: f32,
    pub air_decel_speed: f32,
    pub turn_accel_multiplier: f32,
    pub jump_velocity: f32,
    pub jump_buffer_time: f32,
    pub coyote_time: f32,
    pub jump_release_velocity_factor: f32,
    pub min_walk_speed: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        let settings = ProjectSettings::singleton();
        let gravity = settings.get("physics/2d/default_gravity").to::<f64>() as f32;

        Self {
            gravity,
            walk_speed: 120.0,
            ground_accel_speed: 720.0,
            ground_decel_speed: 1080.0,
            air_accel_speed: 540.0,
            air_decel_speed: 720.0,
            turn_accel_multiplier: 1.4,
            jump_velocity: -300.0,
            jump_buffer_time: 0.12,
            coyote_time: 0.10,
            jump_release_velocity_factor: 0.5,
            min_walk_speed: 0.1,
        }
    }
}

pub struct PlayerMovement {
    pub state: MovementState,
    pub config: MovementConfig,
    jump_buffer_timer: f32,
    coyote_timer: f32,
    was_on_floor: bool,
    jumped_this_frame: bool,
    buffered_jump_cut_requested: bool,
}

impl PlayerMovement {
    pub fn new(config: MovementConfig) -> Self {
        Self {
            state: MovementState::Air,
            config,
            jump_buffer_timer: 0.0,
            coyote_timer: 0.0,
            was_on_floor: false,
            jumped_this_frame: false,
            buffered_jump_cut_requested: false,
        }
    }

    pub fn reset_transient_state(&mut self) {
        self.state = MovementState::Air;
        self.jump_buffer_timer = 0.0;
        self.coyote_timer = 0.0;
        self.was_on_floor = false;
        self.jumped_this_frame = false;
        self.buffered_jump_cut_requested = false;
    }

    pub fn physics_process(
        &mut self,
        velocity: Vector2,
        is_on_floor: bool,
        delta: f64,
        input: MovementInput,
    ) -> Vector2 {
        let delta = delta as f32;
        let mut new_velocity = velocity;

        self.jumped_this_frame = false;
        self.tick_timers(delta);

        if input.jump_just_pressed {
            self.jump_buffer_timer = self.config.jump_buffer_time;
            self.buffered_jump_cut_requested = false;
        }

        if input.jump_just_released && self.jump_buffer_timer > 0.0 && !self.can_jump(is_on_floor) {
            self.buffered_jump_cut_requested = true;
        }

        new_velocity.y += self.config.gravity * delta;
        self.apply_walk(&mut new_velocity, delta, input.direction, is_on_floor);

        if self.apply_jump(&mut new_velocity, is_on_floor) {
            self.jumped_this_frame = true;
        }

        if input.jump_just_released || self.buffered_jump_cut_requested {
            self.apply_jump_cut(&mut new_velocity);

            if self.jumped_this_frame || self.jump_buffer_timer <= 0.0 {
                self.buffered_jump_cut_requested = false;
            }
        }

        self.state = if self.jumped_this_frame || !is_on_floor {
            MovementState::Air
        } else {
            MovementState::Floor
        };

        new_velocity
    }

    pub fn post_physics_update(&mut self, is_on_floor: bool) {
        if is_on_floor {
            self.coyote_timer = 0.0;
            if !self.jumped_this_frame {
                self.state = MovementState::Floor;
            }
        } else {
            if self.was_on_floor && !self.jumped_this_frame {
                self.coyote_timer = self.config.coyote_time;
            }
            self.state = MovementState::Air;
        }

        self.was_on_floor = is_on_floor;
        self.jumped_this_frame = false;
    }

    fn tick_timers(&mut self, delta: f32) {
        self.jump_buffer_timer = (self.jump_buffer_timer - delta).max(0.0);
        self.coyote_timer = (self.coyote_timer - delta).max(0.0);

        if self.jump_buffer_timer <= 0.0 {
            self.buffered_jump_cut_requested = false;
        }
    }

    fn apply_walk(
        &mut self,
        velocity: &mut Vector2,
        delta: f32,
        direction: f32,
        is_on_floor: bool,
    ) {
        let accel = self.horizontal_acceleration(velocity.x, direction, is_on_floor);
        velocity.x = move_toward_scalar(
            velocity.x,
            direction * self.config.walk_speed,
            accel * delta,
        );
    }

    fn horizontal_acceleration(&self, velocity_x: f32, direction: f32, is_on_floor: bool) -> f32 {
        let changing_direction = direction.abs() >= INPUT_DEADZONE
            && velocity_x.abs() >= INPUT_DEADZONE
            && direction.signum() != velocity_x.signum();

        let base_accel = if is_on_floor {
            if direction.abs() < INPUT_DEADZONE {
                self.config.ground_decel_speed
            } else {
                self.config.ground_accel_speed
            }
        } else if direction.abs() < INPUT_DEADZONE {
            self.config.air_decel_speed
        } else {
            self.config.air_accel_speed
        };

        if is_on_floor && changing_direction {
            base_accel * self.config.turn_accel_multiplier
        } else {
            base_accel
        }
    }

    fn can_jump(&self, is_on_floor: bool) -> bool {
        self.jump_buffer_timer > 0.0 && (is_on_floor || self.coyote_timer > 0.0)
    }

    fn apply_jump(&mut self, velocity: &mut Vector2, is_on_floor: bool) -> bool {
        let can_jump = self.can_jump(is_on_floor);
        if can_jump {
            velocity.y = self.config.jump_velocity;
            self.jump_buffer_timer = 0.0;
            self.coyote_timer = 0.0;
        }
        can_jump
    }

    fn apply_jump_cut(&self, velocity: &mut Vector2) {
        if velocity.y < 0.0 {
            velocity.y *= self.config.jump_release_velocity_factor;
        }
    }

    pub fn is_walking(&self, velocity: Vector2) -> bool {
        velocity.x.abs() > self.config.min_walk_speed
    }

    pub fn is_walking_or_pressing(&self, velocity: Vector2, input_direction: f32) -> bool {
        input_direction.abs() >= INPUT_DEADZONE || self.is_walking(velocity)
    }
}

fn move_toward_scalar(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> MovementConfig {
        MovementConfig {
            gravity: 0.0,
            walk_speed: 120.0,
            ground_accel_speed: 720.0,
            ground_decel_speed: 1080.0,
            air_accel_speed: 540.0,
            air_decel_speed: 720.0,
            turn_accel_multiplier: 1.4,
            jump_velocity: -300.0,
            jump_buffer_time: 0.12,
            coyote_time: 0.10,
            jump_release_velocity_factor: 0.5,
            min_walk_speed: 0.1,
        }
    }

    #[test]
    fn buffers_jump_until_landing() {
        let mut movement = PlayerMovement::new(test_config());
        let delta = 0.016;

        let airborne_velocity = movement.physics_process(
            Vector2::new(0.0, 80.0),
            false,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );
        movement.post_physics_update(false);
        assert_eq!(airborne_velocity.y, 80.0);

        let jumped_velocity =
            movement.physics_process(airborne_velocity, true, delta, MovementInput::default());

        assert_eq!(jumped_velocity.y, movement.config.jump_velocity);
        assert_eq!(movement.state, MovementState::Air);
    }

    #[test]
    fn allows_coyote_jump_after_walking_off_ledge() {
        let mut movement = PlayerMovement::new(test_config());
        let delta = 0.016;

        movement.post_physics_update(true);
        movement.physics_process(Vector2::ZERO, true, delta, MovementInput::default());
        movement.post_physics_update(false);

        let jumped_velocity = movement.physics_process(
            Vector2::ZERO,
            false,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );

        assert_eq!(jumped_velocity.y, movement.config.jump_velocity);
        assert_eq!(movement.state, MovementState::Air);
    }

    #[test]
    fn jump_release_cuts_upward_speed() {
        let mut movement = PlayerMovement::new(test_config());

        let velocity = movement.physics_process(
            Vector2::new(0.0, -200.0),
            false,
            0.016,
            MovementInput {
                jump_just_released: true,
                ..Default::default()
            },
        );

        assert_eq!(velocity.y, -100.0);
    }

    #[test]
    fn leaving_floor_via_jump_does_not_restore_coyote_time() {
        let mut movement = PlayerMovement::new(test_config());
        let delta = 0.016;

        movement.post_physics_update(true);

        let jumped_velocity = movement.physics_process(
            Vector2::ZERO,
            true,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );
        movement.post_physics_update(false);
        assert_eq!(jumped_velocity.y, movement.config.jump_velocity);

        let second_jump_attempt = movement.physics_process(
            Vector2::new(0.0, -180.0),
            false,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );

        assert_eq!(second_jump_attempt.y, -180.0);
    }

    #[test]
    fn reset_transient_state_clears_buffered_jump() {
        let mut movement = PlayerMovement::new(test_config());
        let delta = 0.016;

        let airborne_velocity = movement.physics_process(
            Vector2::new(0.0, 80.0),
            false,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );
        movement.post_physics_update(false);
        movement.reset_transient_state();

        let grounded_velocity =
            movement.physics_process(airborne_velocity, true, delta, MovementInput::default());

        assert_eq!(grounded_velocity.y, 80.0);
        assert_eq!(movement.state, MovementState::Floor);
    }

    #[test]
    fn buffered_jump_release_cuts_jump_on_landing() {
        let mut movement = PlayerMovement::new(test_config());
        let delta = 0.016;

        let airborne_velocity = movement.physics_process(
            Vector2::new(0.0, 80.0),
            false,
            delta,
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
        );
        movement.post_physics_update(false);

        let released_velocity = movement.physics_process(
            airborne_velocity,
            false,
            delta,
            MovementInput {
                jump_just_released: true,
                ..Default::default()
            },
        );
        movement.post_physics_update(false);
        assert_eq!(released_velocity.y, 80.0);

        let jumped_velocity =
            movement.physics_process(released_velocity, true, delta, MovementInput::default());

        assert_eq!(jumped_velocity.y, movement.config.jump_velocity * 0.5);
        assert_eq!(movement.state, MovementState::Air);
    }

    #[test]
    fn turn_acceleration_is_snappier_on_ground() {
        let mut movement = PlayerMovement::new(test_config());
        let mut baseline_config = test_config();
        baseline_config.turn_accel_multiplier = 1.0;
        let mut baseline = PlayerMovement::new(baseline_config);

        let tuned_velocity = movement.physics_process(
            Vector2::new(120.0, 0.0),
            true,
            0.1,
            MovementInput {
                direction: -1.0,
                ..Default::default()
            },
        );

        let baseline_velocity = baseline.physics_process(
            Vector2::new(120.0, 0.0),
            true,
            0.1,
            MovementInput {
                direction: -1.0,
                ..Default::default()
            },
        );

        assert!(tuned_velocity.x < baseline_velocity.x);
    }

    #[test]
    fn walking_or_pressing_treats_wall_push_as_walk_intent() {
        let movement = PlayerMovement::new(test_config());

        assert!(movement.is_walking_or_pressing(Vector2::ZERO, 1.0));
        assert!(movement.is_walking_or_pressing(Vector2::new(5.0, 0.0), 0.0));
        assert!(!movement.is_walking_or_pressing(Vector2::ZERO, 0.0));
    }
}
