use godot::classes::{Area2D, IStaticBody2D, StaticBody2D};
use godot::prelude::*;

use super::plain_key::PlainKey;
use crate::save;

#[derive(GodotClass)]
#[class(base=StaticBody2D)]
pub struct PlainLock {
    #[base]
    base: Base<StaticBody2D>,

    detect_area: OnReady<Gd<Area2D>>,

    #[export]
    room_coords: Vector2i,
}

#[godot_api]
impl IStaticBody2D for PlainLock {
    fn init(base: Base<StaticBody2D>) -> Self {
        Self {
            base,
            detect_area: OnReady::from_node("DetectArea"),
            room_coords: Vector2i::ZERO,
        }
    }

    fn ready(&mut self) {
        godot_print!(
            "[PlainLock] ready at {:?}, room {:?}",
            self.base().get_global_position(),
            self.room_coords
        );

        // Check if already unlocked (query save state for restoration)
        let room = (self.room_coords.x, self.room_coords.y);
        let pos = self.base().get_global_position();
        if save::is_lock_unlocked(room, pos) {
            godot_print!("[PlainLock] already unlocked, queue_free");
            self.base_mut().queue_free();
            return;
        }

        let callable = self.base().callable("on_body_entered");
        self.detect_area.connect("body_entered", &callable);
        godot_print!("[PlainLock] body_entered signal connected");
    }
}

#[godot_api]
impl PlainLock {
    /// Signal emitted when lock is unlocked.
    /// Parameters: room_coords (Vector2i), position (Vector2)
    #[signal]
    fn lock_unlocked(room_coords: Vector2i, position: Vector2);

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        let body_pos = body.get_global_position();
        let detect_pos = self.detect_area.get_global_position();
        let distance = body_pos.distance_to(detect_pos);

        // Filter spurious signals from physics engine edge cases during room transitions.
        const MAX_DETECT_DISTANCE: f32 = 48.0;
        if distance > MAX_DETECT_DISTANCE {
            godot_print!(
                "[PlainLock] on_body_entered IGNORED: body={}, distance={:.1}",
                body.get_name(),
                distance
            );
            return;
        }

        godot_print!("[PlainLock] on_body_entered: body={}", body.get_name());

        if let Some(mut key) = self.find_collected_key() {
            godot_print!("[PlainLock] found collected key, unlocking");
            self.unlock(&mut key);
        } else {
            godot_print!("[PlainLock] no collected key found, ignoring");
        }
    }

    fn find_collected_key(&self) -> Option<Gd<PlainKey>> {
        let mut tree = self.base().get_tree()?;
        let keys = tree.get_nodes_in_group("plain_keys");

        for node in keys.iter_shared() {
            if let Ok(key) = node.try_cast::<PlainKey>() {
                if key.bind().is_collected() {
                    return Some(key);
                }
            }
        }
        None
    }

    fn unlock(&mut self, key: &mut Gd<PlainKey>) {
        godot_print!("[PlainLock] unlocked");

        // Copy values before emitting signal to avoid borrow conflict
        let room_coords = self.room_coords;
        let pos = self.base().get_global_position();

        // Emit signal for SaveService to handle persistence
        self.signals().lock_unlocked().emit(room_coords, pos);

        key.bind_mut().consume();
        self.base_mut().queue_free();
    }
}
