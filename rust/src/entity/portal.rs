use godot::classes::{AnimatedSprite2D, Area2D, IArea2D, Input, Label};
use godot::prelude::*;

const ACTIVATE_ACTION: &str = "act_up";

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Portal {
    #[base]
    base: Base<Area2D>,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,

    /// Hint label shown when player is in range
    hint_label: OnReady<Gd<Label>>,

    /// Grid coordinates of the room containing this portal
    #[export]
    room_coords: Vector2i,

    /// Target room coordinates to teleport to
    #[export]
    destination_room: Vector2i,

    /// Whether player is currently in portal area
    player_in_area: bool,
}

#[godot_api]
impl IArea2D for Portal {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            hint_label: OnReady::from_node("HintLabel"),
            room_coords: Vector2i::ZERO,
            destination_room: Vector2i::ZERO,
            player_in_area: false,
        }
    }

    fn ready(&mut self) {
        self.sprite.play();

        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
        self.signals()
            .body_exited()
            .connect_self(Self::on_body_exited);
    }

    fn process(&mut self, _delta: f64) {
        if !self.player_in_area {
            return;
        }

        let input = Input::singleton();
        if input.is_action_just_pressed(ACTIVATE_ACTION) {
            self.activate_teleport();
        }
    }
}

#[godot_api]
impl Portal {
    /// Signal emitted when portal teleport is activated
    #[signal]
    fn teleport_requested(destination_room: Vector2i);

    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        self.player_in_area = true;
        self.hint_label.set_visible(true);
    }

    #[func]
    fn on_body_exited(&mut self, _body: Gd<Node2D>) {
        self.player_in_area = false;
        self.hint_label.set_visible(false);
    }

    fn activate_teleport(&mut self) {
        let destination = self.destination_room;
        godot_print!(
            "[Portal] teleport activated: {:?} -> {:?}",
            self.room_coords,
            destination
        );
        self.signals().teleport_requested().emit(destination);
    }

    /// Get the global position of this portal (used by RoomManager to find spawn point)
    #[func]
    pub fn get_spawn_position(&self) -> Vector2 {
        self.base().get_global_position()
    }
}
