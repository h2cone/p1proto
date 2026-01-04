mod movement;

pub use movement::{MovementConfig, MovementState, PlayerMovement};

use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input, RigidBody2D,
    },
    prelude::*,
};

const MOVING_PLATFORM_LAYER: i32 = 3;
const WALK_LEFT_ACTION: &str = "act_walk_left";
const WALK_RIGHT_ACTION: &str = "act_walk_right";
const JUMP_ACTION: &str = "act_jump";
const DROP_THROUGH_ACTION: &str = "act_down";
const DROP_THROUGH_DURATION: f64 = 0.35;
const PUSH_FORCE: f32 = 80.0;
const PUSH_INPUT_DEADZONE: f32 = 0.01;
const PUSH_NORMAL_EPS: f32 = 0.01;
const PUSH_POSITION_EPS: f32 = 0.01;

fn compute_horizontal_push_impulse(
    input_axis: f32,
    collision_normal: Vector2,
    player_pos: Vector2,
    body_pos: Vector2,
    push_force: f32,
) -> Option<Vector2> {
    if input_axis.abs() < PUSH_INPUT_DEADZONE {
        return None;
    }

    let input_sign = input_axis.signum();
    let normal_ok =
        collision_normal.x.abs() > PUSH_NORMAL_EPS && (-collision_normal.x).signum() == input_sign;

    let delta_x = body_pos.x - player_pos.x;
    let position_ok = delta_x.abs() > PUSH_POSITION_EPS && delta_x.signum() == input_sign;

    if normal_ok || position_ok {
        Some(Vector2::new(input_sign * push_force, 0.0))
    } else {
        None
    }
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    drop_through_timer: f64,
    moving_platform_mask_default: bool,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            movement: None,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            drop_through_timer: 0.0,
            moving_platform_mask_default: true,
        }
    }

    fn ready(&mut self) {
        let config = MovementConfig {
            walk_speed: 120.0,
            accel_speed: 720.0,
            jump_velocity: -300.0,
            min_walk_speed: 0.1,
            action_walk_left: WALK_LEFT_ACTION.to_string(),
            action_walk_right: WALK_RIGHT_ACTION.to_string(),
            action_jump: JUMP_ACTION.to_string(),
            ..Default::default()
        };
        self.movement = Some(PlayerMovement::new(config));

        self.moving_platform_mask_default =
            self.base().get_collision_mask_value(MOVING_PLATFORM_LAYER);

        godot_print!("[Player] ready")
    }

    fn physics_process(&mut self, delta: f64) {
        // Get immutable values first
        let velocity = self.base().get_velocity();
        let mut is_on_floor = self.base().is_on_floor();

        self.update_drop_through(is_on_floor, delta);
        if self.drop_through_timer > 0.0 {
            is_on_floor = false;
        }

        // Process movement and get new velocity and state
        let (new_velocity, state) = if let Some(movement) = &mut self.movement {
            let new_velocity = movement.physics_process(velocity, is_on_floor, delta);
            (new_velocity, movement.state)
        } else {
            return;
        };

        // Update physics
        self.base_mut().set_velocity(new_velocity);
        self.base_mut().move_and_slide();

        // Push rigid bodies (e.g., pushable crates)
        self.push_rigid_bodies();

        // Update sprite direction and animation
        self.update_sprite_and_animation(new_velocity, state);
    }
}

#[godot_api]
impl Player {
    fn update_drop_through(&mut self, is_on_floor: bool, delta: f64) {
        if is_on_floor && self.drop_through_timer <= 0.0 && self.is_standing_on_moving_platform() {
            let input = Input::singleton();
            if input.is_action_just_pressed(DROP_THROUGH_ACTION) {
                self.start_drop_through();
            }
        }

        if self.drop_through_timer > 0.0 {
            self.drop_through_timer -= delta;
            if self.drop_through_timer <= 0.0 {
                self.stop_drop_through();
            }
        }
    }

    fn start_drop_through(&mut self) {
        self.drop_through_timer = DROP_THROUGH_DURATION;
        self.base_mut()
            .set_collision_mask_value(MOVING_PLATFORM_LAYER, false);
    }

    fn stop_drop_through(&mut self) {
        self.drop_through_timer = 0.0;
        let mask_default = self.moving_platform_mask_default;
        self.base_mut()
            .set_collision_mask_value(MOVING_PLATFORM_LAYER, mask_default);
    }

    fn is_standing_on_moving_platform(&mut self) -> bool {
        let Some(collision) = self.base_mut().get_last_slide_collision() else {
            return false;
        };

        let normal = collision.get_normal();
        let is_floor_hit = normal.dot(Vector2::new(0.0, -1.0)) > 0.7;
        if !is_floor_hit {
            return false;
        }

        let Some(collider) = collision.get_collider() else {
            return false;
        };

        if let Ok(body) = collider.try_cast::<CollisionObject2D>() {
            body.get_collision_layer_value(MOVING_PLATFORM_LAYER)
        } else {
            false
        }
    }

    /// Apply impulse to rigid bodies we collided with during move_and_slide
    /// Based on: https://kidscancode.org/godot_recipes/4.x/physics/character_vs_rigid/
    fn push_rigid_bodies(&mut self) {
        let input = Input::singleton();
        let input_axis = input.get_axis(WALK_LEFT_ACTION, WALK_RIGHT_ACTION);
        if input_axis.abs() < PUSH_INPUT_DEADZONE {
            return;
        }

        let player_pos = self.base().get_global_position();
        let collision_count = self.base().get_slide_collision_count();
        for i in 0..collision_count {
            let Some(collision) = self.base_mut().get_slide_collision(i) else {
                continue;
            };
            let Some(collider) = collision.get_collider() else {
                continue;
            };
            if let Ok(mut rigid_body) = collider.try_cast::<RigidBody2D>() {
                let normal = collision.get_normal();
                let body_pos = rigid_body.get_global_position();
                let Some(impulse) = compute_horizontal_push_impulse(
                    input_axis, normal, player_pos, body_pos, PUSH_FORCE,
                ) else {
                    continue;
                };
                rigid_body
                    .apply_central_impulse_ex()
                    .impulse(impulse)
                    .done();
            }
        }
    }

    /// Update sprite direction based on velocity and play appropriate animation
    fn update_sprite_and_animation(&mut self, velocity: Vector2, state: MovementState) {
        // Get the appropriate animation for current state before mutably borrowing sprite
        let animation = self.get_animation_name(velocity, state);

        // Flip sprite based on horizontal velocity
        if !velocity.x.is_zero_approx() {
            self.sprite
                .set_scale(Vector2::new(velocity.x.signum(), 1.0));
        }

        // Play animation if it's different from current one
        if !animation.is_empty() && animation != self.sprite.get_animation() {
            self.sprite.set_animation(&animation);
            self.sprite.play();
        }
    }

    /// Determine which animation to play based on state and velocity
    fn get_animation_name(&self, velocity: Vector2, state: MovementState) -> StringName {
        let animation_str = match state {
            MovementState::Floor => {
                if let Some(movement) = &self.movement {
                    if movement.is_walking(velocity) {
                        "walk"
                    } else {
                        "idle"
                    }
                } else {
                    "idle"
                }
            }
            MovementState::Air => {
                if velocity.y > 0.0 {
                    "fall"
                } else {
                    "jump"
                }
            }
        };
        StringName::from(animation_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_impulse_is_horizontal_and_matches_input() {
        let input_axis = 1.0;
        let normal = Vector2::new(-1.0, 0.0);
        let player_pos = Vector2::new(0.0, 0.0);
        let body_pos = Vector2::new(10.0, 0.0);

        let impulse =
            compute_horizontal_push_impulse(input_axis, normal, player_pos, body_pos, PUSH_FORCE)
                .expect("expected an impulse");

        assert_eq!(impulse, Vector2::new(PUSH_FORCE, 0.0));
    }

    #[test]
    fn push_impulse_falls_back_to_position_when_normal_is_vertical() {
        let input_axis = -1.0;
        let normal = Vector2::new(0.0, -1.0);
        let player_pos = Vector2::new(10.0, 0.0);
        let body_pos = Vector2::new(0.0, 0.0);

        let impulse =
            compute_horizontal_push_impulse(input_axis, normal, player_pos, body_pos, PUSH_FORCE)
                .expect("expected an impulse");

        assert_eq!(impulse, Vector2::new(-PUSH_FORCE, 0.0));
    }

    #[test]
    fn push_impulse_requires_horizontal_input() {
        let input_axis = 0.0;
        let normal = Vector2::new(-1.0, 0.0);
        let player_pos = Vector2::new(0.0, 0.0);
        let body_pos = Vector2::new(10.0, 0.0);

        let impulse =
            compute_horizontal_push_impulse(input_axis, normal, player_pos, body_pos, 1.0);
        assert!(impulse.is_none());
    }

    #[test]
    fn push_impulse_does_not_push_bodies_behind_player() {
        let input_axis = 1.0;
        let normal = Vector2::new(0.0, -1.0);
        let player_pos = Vector2::new(10.0, 0.0);
        let body_pos = Vector2::new(0.0, 0.0);

        let impulse =
            compute_horizontal_push_impulse(input_axis, normal, player_pos, body_pos, PUSH_FORCE);

        assert!(impulse.is_none());
    }
}
