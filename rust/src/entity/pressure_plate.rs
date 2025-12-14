use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PressurePlate {
    #[base]
    base: Base<Area2D>,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl IArea2D for PressurePlate {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            sprite: OnReady::from_node("AnimatedSprite2D"),
        }
    }

    fn ready(&mut self) {
        // Basic initialization - sprite setup will be added when implementing logic
    }
}

#[godot_api]
impl PressurePlate {
    // Methods will be added here when implementing specific functionality
}
