use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

use super::switch_door::SwitchDoor;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PressurePlate {
    #[base]
    base: Base<Area2D>,

    /// Is the pressure plate currently pressed?
    pressed: bool,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,

    /// Room coordinates of the target SwitchDoor
    #[export]
    target_room: Vector2i,

    /// NodePath to the target SwitchDoor
    #[export]
    target_id: NodePath,
}

#[godot_api]
impl IArea2D for PressurePlate {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            pressed: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            target_room: Vector2i::default(),
            target_id: NodePath::default(),
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

            // Open target door
            if let Some(mut door) = self.get_target_door() {
                door.bind_mut().open();
            }
        }
    }

    /// Called when a body exits the pressure plate area
    #[func]
    fn on_body_exited(&mut self, _body: Gd<Node2D>) {
        // Defer the check to ensure physics state is fully updated
        self.base_mut().call_deferred("check_deactivation", &[]);
    }

    /// Check if pressure plate should deactivate (called deferred)
    #[func]
    fn check_deactivation(&mut self) {
        // Only deactivate if no bodies remain on the plate
        if self.pressed && self.base().get_overlapping_bodies().is_empty() {
            self.pressed = false;
            self.sprite.set_animation("inactive");
            self.sprite.stop();
            godot_print!("[PressurePlate] deactivated");

            // Close target door
            if let Some(mut door) = self.get_target_door() {
                door.bind_mut().close();
            }
        }
    }

    /// Check if the pressure plate is currently pressed
    #[func]
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Get the target SwitchDoor if configured
    fn get_target_door(&self) -> Option<Gd<SwitchDoor>> {
        if self.target_id.is_empty() {
            return None;
        }

        self.base().try_get_node_as::<SwitchDoor>(&self.target_id)
    }
}
