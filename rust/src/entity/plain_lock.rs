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
        // Check if already unlocked
        let room = (self.room_coords.x, self.room_coords.y);
        let pos = self.base().get_global_position();
        if save::is_lock_unlocked(room, pos) {
            self.base_mut().queue_free();
            return;
        }

        let callable = self.base().callable("on_body_entered");
        self.detect_area.connect("body_entered", &callable);
    }
}

#[godot_api]
impl PlainLock {
    #[signal]
    fn lock_unlocked();

    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        if let Some(mut key) = self.find_collected_key() {
            self.unlock(&mut key);
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
        godot_print!("Lock unlocked!");

        // Save unlock state
        let room = (self.room_coords.x, self.room_coords.y);
        let pos = self.base().get_global_position();
        save::mark_lock_unlocked(room, pos);

        self.signals().lock_unlocked().emit();
        key.bind_mut().consume();
        self.base_mut().queue_free();
    }
}
