//! Save system module.
//!
//! Split into layers:
//! - `core`: Generic checkpoint slot management (game-agnostic)
//! - `entity_state`: Game-specific entity state persistence (keys, locks)
//! - `service`: Event-driven save handler (Godot node)

mod core;
mod entity_state;
mod exploration;
mod service;

use godot::prelude::*;

// Re-export core save system
pub use core::{
    DEFAULT_SAVE_SLOT, SaveSnapshot, has_save, peek_checkpoint, queue_load, save_checkpoint,
    take_pending_load,
};

// Re-export entity state management
pub use entity_state::{
    get_star_count, is_key_collected, is_lock_unlocked, is_star_collected, mark_key_collected,
    mark_lock_unlocked, mark_star_collected,
};

// Re-export exploration state
pub use exploration::{is_room_explored, list_explored_rooms, mark_room_explored};

// Re-export save service
pub use service::SaveService;

/// Reset all game state (for new game).
pub fn reset_all() {
    core::reset();
    entity_state::reset();
    exploration::reset();
}

/// Godot-facing helper for accessing save state from scenes/scripts.
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
    /// Returns true if the given slot contains data.
    #[func]
    pub fn has_save(&self, slot: i64) -> bool {
        has_save(slot as usize)
    }

    /// Queue loading the specified slot on the next game scene load.
    ///
    /// Returns false if the slot is empty.
    #[func]
    pub fn queue_load(&self, slot: i64) -> bool {
        queue_load(slot as usize)
    }

    /// Clear any pending load flag without removing the saved data itself.
    #[func]
    pub fn clear_pending_load(&self) {
        core::clear_pending_load();
    }

    /// Returns explored room coordinates for world map display.
    #[func]
    pub fn get_explored_rooms(&self) -> Array<Vector2i> {
        let mut rooms = Array::new();
        for (x, y) in list_explored_rooms() {
            rooms.push(Vector2i::new(x, y));
        }
        rooms
    }

    /// Returns whether a room has been explored.
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
        // Save a checkpoint
        save_checkpoint(DEFAULT_SAVE_SLOT, (1, 2), Vector2::new(10.0, 20.0));
        assert!(has_save(DEFAULT_SAVE_SLOT));

        // Mark some entity states
        mark_key_collected((1, 2), Vector2::new(30.0, 40.0));
        mark_lock_unlocked((1, 2), Vector2::new(50.0, 60.0));
        assert!(is_key_collected((1, 2), Vector2::new(30.0, 40.0)));
        assert!(is_lock_unlocked((1, 2), Vector2::new(50.0, 60.0)));

        // Mark exploration state
        mark_room_explored((2, 3));
        assert!(is_room_explored((2, 3)));

        // Reset everything
        reset_all();

        // Verify all state is cleared
        assert!(!has_save(DEFAULT_SAVE_SLOT));
        assert!(!is_key_collected((1, 2), Vector2::new(30.0, 40.0)));
        assert!(!is_lock_unlocked((1, 2), Vector2::new(50.0, 60.0)));
        assert!(!is_room_explored((2, 3)));
    }
}
