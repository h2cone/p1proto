//! Core save system: checkpoint slots and pending load management.
//! This module is game-agnostic and can be reused across different projects.

use godot::prelude::*;
use std::cell::RefCell;

/// Default save slot index. Designed for easy expansion to multiple slots later.
pub const DEFAULT_SAVE_SLOT: usize = 0;

/// Snapshot of player progress for a single save slot.
#[derive(Copy, Clone, Debug)]
pub struct SaveSnapshot {
    pub room: (i32, i32),
    pub position: Vector2,
}

impl SaveSnapshot {
    pub fn new(room: (i32, i32), position: Vector2) -> Self {
        Self { room, position }
    }
}

#[derive(Default)]
struct CoreStore {
    slots: Vec<Option<SaveSnapshot>>,
    /// Which slot should be loaded on the next game start.
    pending_load_slot: Option<usize>,
}

impl CoreStore {
    fn ensure_slot(&mut self, slot: usize) {
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }
    }
}

thread_local! {
    static STORE: RefCell<CoreStore> = RefCell::new(CoreStore::default());
}

/// Save checkpoint data into the specified slot.
pub fn save_checkpoint(slot: usize, room: (i32, i32), position: Vector2) -> SaveSnapshot {
    STORE.with_borrow_mut(|store| {
        store.ensure_slot(slot);
        let snapshot = SaveSnapshot::new(room, position);
        store.slots[slot] = Some(snapshot);
        snapshot
    })
}

/// Peek at the saved checkpoint for a slot without consuming it.
pub fn peek_checkpoint(slot: usize) -> Option<SaveSnapshot> {
    STORE.with_borrow(|store| store.slots.get(slot).and_then(|&slot_data| slot_data))
}

/// Check if a slot currently has data.
pub fn has_save(slot: usize) -> bool {
    STORE.with_borrow(|store| {
        store
            .slots
            .get(slot)
            .and_then(|slot_data| slot_data.as_ref())
            .is_some()
    })
}

/// Mark a slot to be loaded on the next game scene load.
pub fn queue_load(slot: usize) -> bool {
    STORE.with_borrow_mut(|store| {
        store.ensure_slot(slot);
        if store.slots[slot].is_some() {
            store.pending_load_slot = Some(slot);
            true
        } else {
            false
        }
    })
}

/// Consume the pending load request, returning the snapshot if present.
pub fn take_pending_load() -> Option<SaveSnapshot> {
    STORE.with_borrow_mut(|store| {
        let slot = store.pending_load_slot.take()?;
        store.ensure_slot(slot);
        store.slots.get(slot).and_then(|&slot_data| slot_data)
    })
}

/// Clear pending load flag without removing saved data.
pub fn clear_pending_load() {
    STORE.with_borrow_mut(|store| {
        store.pending_load_slot = None;
    });
}

/// Reset core save state (for new game).
pub fn reset() {
    STORE.with_borrow_mut(|store| {
        store.slots.clear();
        store.pending_load_slot = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_store() {
        STORE.with_borrow_mut(|store| {
            store.slots.clear();
            store.pending_load_slot = None;
        });
    }

    #[test]
    fn save_and_queue_load_in_single_slot() {
        reset_store();

        let slot = DEFAULT_SAVE_SLOT;
        let room = (1, 2);
        let position = Vector2::new(10.0, 20.0);

        assert!(!has_save(slot));
        let saved = save_checkpoint(slot, room, position);
        assert_eq!(saved.room, room);
        assert_eq!(saved.position, position);
        assert!(has_save(slot));

        assert!(queue_load(slot));
        let pending = take_pending_load().expect("pending load should exist");
        assert_eq!(pending.room, room);
        assert_eq!(pending.position, position);

        // Pending flag is consumed after take_pending_load
        assert!(take_pending_load().is_none());
    }

    #[test]
    fn peek_checkpoint_returns_saved_data_without_consuming() {
        reset_store();

        let slot = DEFAULT_SAVE_SLOT;
        let room = (3, 4);
        let position = Vector2::new(5.0, 6.0);

        assert!(peek_checkpoint(slot).is_none());

        let saved = save_checkpoint(slot, room, position);
        let peeked = peek_checkpoint(slot).expect("expected saved checkpoint");
        assert_eq!(peeked.room, saved.room);
        assert_eq!(peeked.position, saved.position);

        // Ensure data remains after peeking
        assert!(peek_checkpoint(slot).is_some());

        // Queue and take pending load should not clear saved checkpoint
        assert!(queue_load(slot));
        assert!(take_pending_load().is_some());
        let still_saved = peek_checkpoint(slot).expect("checkpoint should persist after load");
        assert_eq!(still_saved.room, room);
        assert_eq!(still_saved.position, position);
    }
}
