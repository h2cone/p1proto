mod movement;

pub use movement::{MovementConfig, MovementState, PlayerMovement};

use godot::{
    classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            movement: None,
            sprite: OnReady::from_node("AnimatedSprite2D"),
        }
    }

    fn ready(&mut self) {
        let config = MovementConfig {
            walk_speed: 120.0,
            accel_speed: 720.0,
            jump_velocity: -300.0,
            min_walk_speed: 0.1,
            action_walk_left: "act_walk_left".to_string(),
            action_walk_right: "act_walk_right".to_string(),
            action_jump: "act_jump".to_string(),
            ..Default::default()
        };
        self.movement = Some(PlayerMovement::new(config));

        godot_print!("Player ready")
    }

    fn physics_process(&mut self, delta: f64) {
        // Get immutable values first
        let velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

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

        // Update sprite direction and animation
        self.update_sprite_and_animation(new_velocity, state);
    }
}

#[godot_api]
impl Player {
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
