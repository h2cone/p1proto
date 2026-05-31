//! Input adapter for collecting player input from Godot Input singleton.
//! Separates input collection from movement logic for better testability.

use godot::classes::Input;
use godot::prelude::*;

use super::MovementInput;
use super::aim_indicator::AimInput;

/// Input action names configuration.
/// Allows customization of action names without changing logic.
pub struct InputActions {
    pub walk_left: &'static str,
    pub walk_right: &'static str,
    pub aim_up: &'static str,
    pub aim_down: &'static str,
    pub climb_up: &'static str,
    pub climb_down: &'static str,
    pub jump: &'static str,
    pub drop_through: &'static str,
}

impl Default for InputActions {
    fn default() -> Self {
        Self {
            walk_left: "act_walk_left",
            walk_right: "act_walk_right",
            aim_up: "act_up",
            aim_down: "act_down",
            climb_up: "act_up",
            climb_down: "act_down",
            jump: "act_jump",
            drop_through: "act_down",
        }
    }
}

/// Collect movement input from Godot Input singleton.
pub fn collect_movement_input(actions: &InputActions) -> MovementInput {
    let input = Input::singleton();
    MovementInput {
        direction: input.get_axis(actions.walk_left, actions.walk_right),
        vertical_direction: input.get_axis(actions.climb_up, actions.climb_down),
        jump_just_pressed: input.is_action_just_pressed(actions.jump),
        jump_just_released: input.is_action_just_released(actions.jump),
    }
}

/// Collect four-way aim input from Godot Input singleton.
pub fn collect_aim_input(actions: &InputActions) -> AimInput {
    let input = Input::singleton();
    AimInput {
        horizontal: input.get_axis(actions.walk_left, actions.walk_right),
        vertical: input.get_axis(actions.aim_up, actions.aim_down),
    }
}

/// Check if drop-through action was just pressed.
pub fn is_drop_through_pressed(actions: &InputActions) -> bool {
    Input::singleton().is_action_just_pressed(actions.drop_through)
}

/// Get horizontal push direction for rigid body pushing.
/// Returns -1.0 to 1.0, or 0.0 if below threshold.
pub fn get_push_direction(actions: &InputActions) -> f32 {
    let input = Input::singleton();
    let dir = input.get_axis(actions.walk_left, actions.walk_right);
    if dir.abs() < 0.01 { 0.0 } else { dir }
}
