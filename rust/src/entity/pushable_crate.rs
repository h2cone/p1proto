use godot::classes::{IRigidBody2D, RigidBody2D};
use godot::prelude::*;

/// A crate that can be pushed by the player.
#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct PushableCrate {
    #[base]
    base: Base<RigidBody2D>,

    /// Maximum horizontal push speed.
    #[export]
    max_push_speed: f32,
}

#[godot_api]
impl IRigidBody2D for PushableCrate {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            base,
            max_push_speed: 50.0,
        }
    }

    fn ready(&mut self) {
        godot_print!(
            "[PushableCrate] ready at {:?}",
            self.base().get_global_position()
        );
    }

    fn physics_process(&mut self, _delta: f64) {
        let velocity = self.base().get_linear_velocity();

        // Only clamp if velocity exceeds the limit (avoid interfering with physics)
        if velocity.x.abs() > self.max_push_speed {
            let clamped_x = velocity.x.clamp(-self.max_push_speed, self.max_push_speed);
            let new_velocity = Vector2::new(clamped_x, velocity.y);
            self.base_mut().set_linear_velocity(new_velocity);
        }
    }
}

#[godot_api]
impl PushableCrate {
    /// Freeze the crate in place.
    #[func]
    fn freeze(&mut self) {
        self.base_mut().set_freeze_enabled(true);
    }

    /// Unfreeze the crate.
    #[func]
    fn unfreeze(&mut self) {
        self.base_mut().set_freeze_enabled(false);
    }

    /// Check if frozen.
    #[func]
    fn is_frozen(&self) -> bool {
        self.base().is_freeze_enabled()
    }
}
