//! Room exploration state tracking for world map display.

use std::cell::RefCell;
use std::collections::HashSet;

/// Room identifier as grid coordinates.
pub type RoomId = (i32, i32);

#[derive(Default)]
struct ExplorationStore {
    explored_rooms: HashSet<RoomId>,
}

thread_local! {
    static STORE: RefCell<ExplorationStore> = RefCell::new(ExplorationStore::default());
}

/// Mark a room as explored.
pub fn mark_room_explored(room: RoomId) -> bool {
    STORE.with_borrow_mut(|store| store.explored_rooms.insert(room))
}

/// Check if a room has been explored.
pub fn is_room_explored(room: RoomId) -> bool {
    STORE.with_borrow(|store| store.explored_rooms.contains(&room))
}

/// Return a snapshot of explored rooms.
pub fn list_explored_rooms() -> Vec<RoomId> {
    STORE.with_borrow(|store| store.explored_rooms.iter().copied().collect())
}

/// Reset exploration state (for new game).
pub fn reset() {
    STORE.with_borrow_mut(|store| {
        store.explored_rooms.clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_store() {
        STORE.with_borrow_mut(|store| {
            store.explored_rooms.clear();
        });
    }

    #[test]
    fn room_exploration_persists() {
        reset_store();

        let room = (2, 3);
        assert!(!is_room_explored(room));
        assert!(mark_room_explored(room));
        assert!(is_room_explored(room));

        // Marking again should not add a new entry.
        assert!(!mark_room_explored(room));
    }
}
