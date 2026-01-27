//! Game entity state persistence: keys, locks, and other collectible/interactive entities.
//! This module contains game-specific logic that tracks entity states across room transitions.

use godot::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;

/// Unique identifier for an entity: (room_x, room_y, pos_x, pos_y)
pub type EntityId = (i32, i32, i32, i32);

/// Create an EntityId from room coordinates and position.
pub fn make_entity_id(room: (i32, i32), position: Vector2) -> EntityId {
    (room.0, room.1, position.x as i32, position.y as i32)
}

#[derive(Default)]
struct EntityStateStore {
    /// Set of unlocked locks (identified by room coords + position)
    unlocked_locks: HashSet<EntityId>,
    /// Set of collected keys (identified by room coords + position)
    collected_keys: HashSet<EntityId>,
    /// Set of collected stars (identified by room coords + position)
    collected_stars: HashSet<EntityId>,
}

thread_local! {
    static STORE: RefCell<EntityStateStore> = RefCell::new(EntityStateStore::default());
}

/// Mark a lock as unlocked (persists across room transitions).
pub fn mark_lock_unlocked(room: (i32, i32), position: Vector2) {
    STORE.with_borrow_mut(|store| {
        let id = make_entity_id(room, position);
        store.unlocked_locks.insert(id);
    });
}

/// Check if a lock has been unlocked.
pub fn is_lock_unlocked(room: (i32, i32), position: Vector2) -> bool {
    STORE.with_borrow(|store| {
        let id = make_entity_id(room, position);
        store.unlocked_locks.contains(&id)
    })
}

/// Mark a key as collected (persists across room transitions).
pub fn mark_key_collected(room: (i32, i32), position: Vector2) {
    STORE.with_borrow_mut(|store| {
        let id = make_entity_id(room, position);
        store.collected_keys.insert(id);
    });
}

/// Check if a key has been collected.
pub fn is_key_collected(room: (i32, i32), position: Vector2) -> bool {
    STORE.with_borrow(|store| {
        let id = make_entity_id(room, position);
        store.collected_keys.contains(&id)
    })
}

/// Mark a star as collected (persists across room transitions).
pub fn mark_star_collected(room: (i32, i32), position: Vector2) {
    STORE.with_borrow_mut(|store| {
        let id = make_entity_id(room, position);
        store.collected_stars.insert(id);
    });
}

/// Check if a star has been collected.
pub fn is_star_collected(room: (i32, i32), position: Vector2) -> bool {
    STORE.with_borrow(|store| {
        let id = make_entity_id(room, position);
        store.collected_stars.contains(&id)
    })
}

/// Get the total number of collected stars.
pub fn get_star_count() -> usize {
    STORE.with_borrow(|store| store.collected_stars.len())
}

/// Reset all entity states (for new game).
pub fn reset() {
    STORE.with_borrow_mut(|store| {
        store.unlocked_locks.clear();
        store.collected_keys.clear();
        store.collected_stars.clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_store() {
        STORE.with_borrow_mut(|store| {
            store.unlocked_locks.clear();
            store.collected_keys.clear();
        });
    }

    #[test]
    fn key_collection_persists() {
        reset_store();

        let room = (1, 2);
        let pos = Vector2::new(10.0, 20.0);

        assert!(!is_key_collected(room, pos));
        mark_key_collected(room, pos);
        assert!(is_key_collected(room, pos));

        // Different position should not be collected
        let other_pos = Vector2::new(30.0, 40.0);
        assert!(!is_key_collected(room, other_pos));
    }

    #[test]
    fn lock_unlock_persists() {
        reset_store();

        let room = (3, 4);
        let pos = Vector2::new(50.0, 60.0);

        assert!(!is_lock_unlocked(room, pos));
        mark_lock_unlocked(room, pos);
        assert!(is_lock_unlocked(room, pos));

        // Different room should not be unlocked
        let other_room = (5, 6);
        assert!(!is_lock_unlocked(other_room, pos));
    }
}
