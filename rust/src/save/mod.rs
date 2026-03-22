use godot::prelude::*;

use crate::core::progress;
#[cfg(test)]
use crate::core::progress::PersistentEntityKind;

pub use crate::core::progress::{
    DEFAULT_SAVE_SLOT, clear_pending_load, get_star_count, has_save, is_room_explored,
    list_explored_rooms, mark_room_explored, queue_load,
};

#[cfg(test)]
pub fn mark_lock_unlocked(room: (i32, i32), position: Vector2) {
    let _marked = progress::mark_entity(PersistentEntityKind::Lock, room, position);
}

#[cfg(test)]
pub fn is_lock_unlocked(room: (i32, i32), position: Vector2) -> bool {
    progress::has_entity(PersistentEntityKind::Lock, room, position)
}

#[cfg(test)]
pub fn mark_key_collected(room: (i32, i32), position: Vector2) {
    let _marked = progress::mark_entity(PersistentEntityKind::Key, room, position);
}

#[cfg(test)]
pub fn is_key_collected(room: (i32, i32), position: Vector2) -> bool {
    progress::has_entity(PersistentEntityKind::Key, room, position)
}

pub fn reset_all() {
    progress::reset_all();
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct SaveApi {
    #[base]
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for SaveApi {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl SaveApi {
    #[func]
    pub fn has_save(&self, slot: i64) -> bool {
        has_save(slot as usize)
    }

    #[func]
    pub fn queue_load(&self, slot: i64) -> bool {
        queue_load(slot as usize)
    }

    #[func]
    pub fn clear_pending_load(&self) {
        clear_pending_load();
    }

    #[func]
    pub fn get_explored_rooms(&self) -> Array<Vector2i> {
        let mut rooms = Array::new();
        for (x, y) in list_explored_rooms() {
            rooms.push(Vector2i::new(x, y));
        }
        rooms
    }

    #[func]
    pub fn is_room_explored(&self, room: Vector2i) -> bool {
        crate::save::is_room_explored((room.x, room.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_all_clears_everything() {
        progress::save_checkpoint(DEFAULT_SAVE_SLOT, (1, 2), Vector2::new(10.0, 20.0));
        assert!(progress::has_save(DEFAULT_SAVE_SLOT));

        mark_key_collected((1, 2), Vector2::new(30.0, 40.0));
        mark_lock_unlocked((1, 2), Vector2::new(50.0, 60.0));
        assert!(is_key_collected((1, 2), Vector2::new(30.0, 40.0)));
        assert!(is_lock_unlocked((1, 2), Vector2::new(50.0, 60.0)));

        mark_room_explored((2, 3));
        assert!(is_room_explored((2, 3)));

        reset_all();

        assert!(!progress::has_save(DEFAULT_SAVE_SLOT));
        assert!(!is_key_collected((1, 2), Vector2::new(30.0, 40.0)));
        assert!(!is_lock_unlocked((1, 2), Vector2::new(50.0, 60.0)));
        assert!(!is_room_explored((2, 3)));
    }
}
