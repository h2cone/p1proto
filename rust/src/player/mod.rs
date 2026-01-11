mod animation;
mod input_adapter;
mod movement;

pub use animation::AnimationNames;
pub use input_adapter::InputActions;
pub use movement::{MovementConfig, MovementInput, MovementState, PlayerMovement};

use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, RigidBody2D,
    },
    prelude::*,
};

const MOVING_PLATFORM_LAYER: i32 = 4;
const DROP_THROUGH_DURATION: f64 = 0.35;
const PUSH_SPEED: f32 = 80.0;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    input_actions: InputActions,
    animation_names: AnimationNames,
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
            input_actions: InputActions::default(),
            animation_names: AnimationNames::default(),
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

        // 1. Collect input (using input_adapter module)
        let movement_input = input_adapter::collect_movement_input(&self.input_actions);

        // 2. Process movement and get new velocity and state
        let (new_velocity, state) = if let Some(movement) = &mut self.movement {
            let new_velocity =
                movement.physics_process(velocity, is_on_floor, delta, movement_input);
            (new_velocity, movement.state)
        } else {
            return;
        };

        // 3. Update physics
        self.base_mut().set_velocity(new_velocity);
        self.base_mut().move_and_slide();

        // 4. Push rigid bodies (e.g., pushable crates)
        self.push_rigid_bodies();

        // 5. Update animation (using animation module)
        let is_walking = self
            .movement
            .as_ref()
            .map(|m| m.is_walking(new_velocity))
            .unwrap_or(false);
        animation::update_sprite_direction(&mut self.sprite, new_velocity.x);
        let anim =
            animation::get_animation_name(state, new_velocity, is_walking, &self.animation_names);
        animation::play_animation_if_changed(&mut self.sprite, anim);
    }
}

#[godot_api]
impl Player {
    fn update_drop_through(&mut self, is_on_floor: bool, delta: f64) {
        if is_on_floor
            && self.drop_through_timer <= 0.0
            && self.is_standing_on_moving_platform()
            && input_adapter::is_drop_through_pressed(&self.input_actions)
        {
            self.start_drop_through();
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

    /// Push rigid bodies we collided with during move_and_slide
    fn push_rigid_bodies(&mut self) {
        let input_dir = input_adapter::get_push_direction(&self.input_actions);
        if input_dir.abs() < 0.01 {
            return;
        }

        let collision_count = self.base().get_slide_collision_count();
        for i in 0..collision_count {
            let Some(collision) = self.base_mut().get_slide_collision(i) else {
                continue;
            };
            let Some(collider) = collision.get_collider() else {
                continue;
            };
            if let Ok(mut rigid_body) = collider.try_cast::<RigidBody2D>() {
                let crate_vel = rigid_body.get_linear_velocity();
                rigid_body.set_linear_velocity(Vector2::new(input_dir * PUSH_SPEED, crate_vel.y));

                // If crate velocity was near zero despite pushing, it's stuck.
                // Force move the crate by directly adjusting its position.
                if crate_vel.x.abs() < 1.0 {
                    let current_pos = rigid_body.get_global_position();
                    let push_delta = input_dir.signum() * 0.5;
                    rigid_body.set_global_position(Vector2::new(
                        current_pos.x + push_delta,
                        current_pos.y,
                    ));
                }
            }
        }
    }
}
