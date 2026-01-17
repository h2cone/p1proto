use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

use crate::save;

/// Offset position for the key when following the player (above head)
const FOLLOW_OFFSET: Vector2 = Vector2::new(0.0, -20.0);

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PlainKey {
    #[base]
    base: Base<Area2D>,

    collected: bool,

    sprite: OnReady<Gd<AnimatedSprite2D>>,

    follow_target: Option<Gd<Node2D>>,

    /// Whether reparent is pending (deferred to avoid physics callback issues)
    pending_reparent: bool,

    #[export]
    room_coords: Vector2i,

    /// Original position for save state identification
    original_position: Vector2,
}

#[godot_api]
impl IArea2D for PlainKey {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            collected: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            follow_target: None,
            pending_reparent: false,
            room_coords: Vector2i::ZERO,
            original_position: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.original_position = self.base().get_global_position();
        godot_print!(
            "[PlainKey] ready at {:?}, room {:?}",
            self.original_position,
            self.room_coords
        );

        // Check if already collected (query save state for restoration)
        let room = (self.room_coords.x, self.room_coords.y);
        if save::is_key_collected(room, self.original_position) {
            godot_print!("[PlainKey] already collected, queue_free");
            self.base_mut().queue_free();
            return;
        }

        self.sprite.play();
        self.base_mut().add_to_group("plain_keys");

        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
        godot_print!("[PlainKey] added to plain_keys group");
    }

    fn process(&mut self, _delta: f64) {
        let Some(target) = &self.follow_target else {
            return;
        };

        // Handle deferred reparent (must be done outside physics callback)
        if self.pending_reparent {
            self.pending_reparent = false;
            godot_print!("[PlainKey] reparenting to player");
            if let Some(mut old_parent) = self.base().get_parent() {
                old_parent.remove_child(&self.to_gd());
            }
            target.clone().add_child(&self.to_gd());
            godot_print!("[PlainKey] reparent complete");
        }

        let target_pos = target.get_global_position();
        self.base_mut()
            .set_global_position(target_pos + FOLLOW_OFFSET);
    }
}

#[godot_api]
impl PlainKey {
    /// Signal emitted when key is collected.
    /// Parameters: room_coords (Vector2i), position (Vector2)
    #[signal]
    pub(crate) fn key_collected(room_coords: Vector2i, position: Vector2);

    #[signal]
    pub(crate) fn key_used();

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if self.collected {
            return;
        }
        self.collect(body);
    }

    fn collect(&mut self, body: Gd<Node2D>) {
        self.collected = true;
        self.follow_target = Some(body);
        self.pending_reparent = true;
        godot_print!(
            "[PlainKey] collected! pos={:?}, will reparent to player",
            self.base().get_global_position()
        );

        // Copy values before emitting signal to avoid borrow conflict
        let room_coords = self.room_coords;
        let original_position = self.original_position;

        // Emit signal for SaveService to handle persistence
        self.signals()
            .key_collected()
            .emit(room_coords, original_position);

        self.base_mut()
            .set_deferred("monitoring", &false.to_variant());
    }

    #[func]
    pub fn consume(&mut self) {
        godot_print!(
            "[PlainKey] consume called! pos={:?}",
            self.base().get_global_position()
        );
        self.signals().key_used().emit();
        self.base_mut().queue_free();
    }

    #[func]
    pub fn is_collected(&self) -> bool {
        self.collected
    }
}
