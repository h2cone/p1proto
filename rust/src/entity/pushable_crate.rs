use godot::classes::{IRigidBody2D, RigidBody2D};
use godot::prelude::*;

/// A crate that can be pushed by the player.
#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct PushableCrate {
    #[base]
    base: Base<RigidBody2D>,
}

#[godot_api]
impl IRigidBody2D for PushableCrate {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self { base }
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
