use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PressurePlate {
    #[base]
    base: Base<Area2D>,

    /// Is the pressure plate currently pressed?
    pressed: bool,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl IArea2D for PressurePlate {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            pressed: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
        }
    }

    fn ready(&mut self) {
        // Set initial animation to inactive
        self.sprite.set_animation("inactive");
        self.sprite.stop();

        // Connect signals
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);

        self.signals()
            .body_exited()
            .connect_self(Self::on_body_exited);
    }
}

#[godot_api]
impl PressurePlate {
    /// Called when a body enters the pressure plate area
    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        // Only activate if not already pressed
        if !self.pressed {
            self.pressed = true;
            self.sprite.set_animation("active");
            self.sprite.play();
            godot_print!("[PressurePlate] activated");
        }
    }

    /// Called when a body exits the pressure plate area
    #[func]
    fn on_body_exited(&mut self, _body: Gd<Node2D>) {
        // Only deactivate if no bodies remain on the plate
        if self.pressed && self.base().get_overlapping_bodies().is_empty() {
            self.pressed = false;
            self.sprite.set_animation("inactive");
            self.sprite.stop();
            godot_print!("[PressurePlate] deactivated");
        }
    }

    /// Check if the pressure plate is currently pressed
    #[func]
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }
}
