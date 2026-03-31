use godot::classes::{IRigidBody2D, RigidBody2D};
use godot::prelude::*;

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
    #[func]
    fn freeze(&mut self) {
        self.base_mut().set_freeze_enabled(true);
    }

    #[func]
    fn unfreeze(&mut self) {
        self.base_mut().set_freeze_enabled(false);
    }

    #[func]
    fn is_frozen(&self) -> bool {
        self.base().is_freeze_enabled()
    }
}
