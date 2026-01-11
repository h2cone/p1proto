//! Animation control for platformer characters.
//! Separates animation logic from player physics for reusability.

use godot::classes::AnimatedSprite2D;
use godot::prelude::*;

use super::MovementState;

/// Animation names for platformer character.
/// Allows customization without changing animation logic.
pub struct AnimationNames {
    pub idle: &'static str,
    pub walk: &'static str,
    pub jump: &'static str,
    pub fall: &'static str,
}

impl Default for AnimationNames {
    fn default() -> Self {
        Self {
            idle: "idle",
            walk: "walk",
            jump: "jump",
            fall: "fall",
        }
    }
}

/// Determine animation name based on movement state and velocity.
pub fn get_animation_name(
    state: MovementState,
    velocity: Vector2,
    is_walking: bool,
    names: &AnimationNames,
) -> &'static str {
    match state {
        MovementState::Floor => {
            if is_walking {
                names.walk
            } else {
                names.idle
            }
        }
        MovementState::Air => {
            if velocity.y > 0.0 {
                names.fall
            } else {
                names.jump
            }
        }
    }
}

/// Update sprite direction based on horizontal velocity.
/// Flips the sprite horizontally when moving left/right.
pub fn update_sprite_direction(sprite: &mut Gd<AnimatedSprite2D>, velocity_x: f32) {
    if !velocity_x.is_zero_approx() {
        sprite.set_scale(Vector2::new(velocity_x.signum(), 1.0));
    }
}

/// Play animation if different from current.
pub fn play_animation_if_changed(sprite: &mut Gd<AnimatedSprite2D>, animation: &str) {
    let animation_name = StringName::from(animation);
    if !animation.is_empty() && animation_name != sprite.get_animation() {
        sprite.set_animation(&animation_name);
        sprite.play();
    }
}
