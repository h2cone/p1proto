//! Animation control for platformer characters.
//! Separates animation logic from player physics for reusability.

use godot::classes::AnimatedSprite2D;
use godot::prelude::*;

use super::MovementState;

const INPUT_DIRECTION_DEADZONE: f32 = 0.01;

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

/// Flips the sprite horizontally when moving left/right.
pub fn update_sprite_direction(sprite: &mut Gd<AnimatedSprite2D>, velocity_x: f32) {
    if !velocity_x.is_zero_approx() {
        sprite.set_scale(Vector2::new(velocity_x.signum(), 1.0));
    }
}

/// Prefer player input for facing when available so wall collisions
/// do not make the resolved velocity oscillate the sprite direction.
pub fn resolve_visual_direction_x(input_direction: f32, velocity_x: f32) -> f32 {
    if input_direction.abs() >= INPUT_DIRECTION_DEADZONE {
        input_direction
    } else {
        velocity_x
    }
}

pub fn play_animation_if_changed(sprite: &mut Gd<AnimatedSprite2D>, animation: &str) {
    let animation_name = StringName::from(animation);
    if !animation.is_empty() && animation_name != sprite.get_animation() {
        sprite.set_animation(&animation_name);
        sprite.play();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_direction_prefers_input_over_collision_velocity() {
        assert_eq!(resolve_visual_direction_x(1.0, -0.25), 1.0);
        assert_eq!(resolve_visual_direction_x(-1.0, 0.25), -1.0);
    }

    #[test]
    fn visual_direction_falls_back_to_velocity_without_input() {
        assert_eq!(resolve_visual_direction_x(0.0, 2.0), 2.0);
        assert_eq!(resolve_visual_direction_x(0.0, -2.0), -2.0);
    }
}
