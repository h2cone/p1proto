use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

use crate::save::{self, DEFAULT_SAVE_SLOT, SaveSnapshot};

/// Distance threshold for matching a saved checkpoint to a scene instance
const POSITION_MATCH_EPSILON: f32 = 1.0;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Checkpoint {
    #[base]
    base: Base<Area2D>,

    /// Has this checkpoint been activated?
    activated: bool,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,

    /// Grid coordinates of the room containing this checkpoint (set from Godot)
    #[export]
    room_coords: Vector2i,
}

#[godot_api]
impl IArea2D for Checkpoint {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            activated: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            room_coords: Vector2i::ZERO,
        }
    }

    fn ready(&mut self) {
        // Show the unchecked frame without autoplaying the animation
        self.sprite.set_animation("unchecked");
        self.sprite.stop();

        // Connect body_entered signal without string literals
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);

        self.restore_if_saved();
    }
}

#[godot_api]
impl Checkpoint {
    /// Signal emitted when checkpoint is activated.
    /// Parameters: room_coords (Vector2i), position (Vector2)
    #[signal]
    pub(crate) fn checkpoint_activated(room_coords: Vector2i, position: Vector2);

    /// Called when a body enters the checkpoint area
    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        // Only activate once
        if self.activated {
            return;
        }
        self.activate();
    }

    /// Activate the checkpoint
    #[func]
    fn activate(&mut self) {
        if self.activated {
            return;
        }

        self.activated = true;
        godot_print!("[Checkpoint] activated");

        // Immediately switch to checked loop
        self.sprite.set_animation("checked");
        self.sprite.play();

        // Copy values before emitting signal to avoid borrow conflict
        let room_coords = self.room_coords;
        let position = self.base().get_global_position();

        // Emit signal for SaveService to handle persistence
        self.signals()
            .checkpoint_activated()
            .emit(room_coords, position);
    }

    /// Check if checkpoint has been activated
    #[func]
    fn is_activated(&self) -> bool {
        self.activated
    }

    /// Reset checkpoint to unchecked state (for testing/debugging)
    #[func]
    fn reset(&mut self) {
        self.activated = false;
        self.sprite.set_animation("unchecked");
        self.sprite.stop();
    }

    fn restore_if_saved(&mut self) {
        if let Some(snapshot) = save::peek_checkpoint(DEFAULT_SAVE_SLOT) {
            if self.matches_saved_checkpoint(&snapshot) {
                self.apply_saved_state(&snapshot);
            }
        }
    }

    fn matches_saved_checkpoint(&self, snapshot: &SaveSnapshot) -> bool {
        let room_matches = snapshot.room == (self.room_coords.x, self.room_coords.y);
        if !room_matches {
            return false;
        }

        let checkpoint_position = self.base().get_global_position();
        checkpoint_position.distance_to(snapshot.position) <= POSITION_MATCH_EPSILON
    }

    fn apply_saved_state(&mut self, snapshot: &SaveSnapshot) {
        self.activated = true;
        self.sprite.set_animation("checked");
        self.sprite.play();
        godot_print!(
            "[Checkpoint] restored from slot {} at room {:?}, position {:?}",
            DEFAULT_SAVE_SLOT,
            snapshot.room,
            snapshot.position
        );
    }
}
