use godot::prelude::*;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

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

/// Unique identifier for an entity (room_x, room_y, pos_x, pos_y)
type EntityId = (i32, i32, i32, i32);

#[derive(Default)]
struct SaveStore {
    slots: Vec<Option<SaveSnapshot>>,
    /// Which slot should be loaded on the next game start.
    pending_load_slot: Option<usize>,
    /// Set of unlocked locks (identified by room coords + position)
    unlocked_locks: HashSet<EntityId>,
    /// Set of collected keys (identified by room coords + position)
    collected_keys: HashSet<EntityId>,
}

impl SaveStore {
    fn ensure_slot(&mut self, slot: usize) {
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }
    }
}

fn store() -> &'static Mutex<SaveStore> {
    static STORE: OnceLock<Mutex<SaveStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(SaveStore::default()))
}

/// Save checkpoint data into the specified slot.
pub fn save_checkpoint(slot: usize, room: (i32, i32), position: Vector2) -> SaveSnapshot {
    let mut store = store().lock().expect("save store poisoned");
    store.ensure_slot(slot);

    let snapshot = SaveSnapshot::new(room, position);
    store.slots[slot] = Some(snapshot);
    snapshot
}

/// Peek at the saved checkpoint for a slot without consuming it.
pub fn peek_checkpoint(slot: usize) -> Option<SaveSnapshot> {
    let store = store().lock().expect("save store poisoned");
    store.slots.get(slot).and_then(|&slot_data| slot_data)
}

/// Check if a slot currently has data.
pub fn has_save(slot: usize) -> bool {
    let store = store().lock().expect("save store poisoned");
    store
        .slots
        .get(slot)
        .and_then(|slot_data| slot_data.as_ref())
        .is_some()
}

/// Mark a slot to be loaded on the next game scene load.
pub fn queue_load(slot: usize) -> bool {
    let mut store = store().lock().expect("save store poisoned");
    store.ensure_slot(slot);

    if store.slots[slot].is_some() {
        store.pending_load_slot = Some(slot);
        true
    } else {
        false
    }
}

/// Consume the pending load request, returning the snapshot if present.
pub fn take_pending_load() -> Option<SaveSnapshot> {
    let mut store = store().lock().expect("save store poisoned");
    let slot = store.pending_load_slot.take()?;
    store.ensure_slot(slot);
    store.slots.get(slot).and_then(|&slot_data| slot_data)
}

/// Mark a lock as unlocked (persists across room transitions)
pub fn mark_lock_unlocked(room: (i32, i32), position: Vector2) {
    let mut store = store().lock().expect("save store poisoned");
    let id = (room.0, room.1, position.x as i32, position.y as i32);
    store.unlocked_locks.insert(id);
}

/// Check if a lock has been unlocked
pub fn is_lock_unlocked(room: (i32, i32), position: Vector2) -> bool {
    let store = store().lock().expect("save store poisoned");
    let id = (room.0, room.1, position.x as i32, position.y as i32);
    store.unlocked_locks.contains(&id)
}

/// Mark a key as collected (persists across room transitions)
pub fn mark_key_collected(room: (i32, i32), position: Vector2) {
    let mut store = store().lock().expect("save store poisoned");
    let id = (room.0, room.1, position.x as i32, position.y as i32);
    store.collected_keys.insert(id);
}

/// Check if a key has been collected
pub fn is_key_collected(room: (i32, i32), position: Vector2) -> bool {
    let store = store().lock().expect("save store poisoned");
    let id = (room.0, room.1, position.x as i32, position.y as i32);
    store.collected_keys.contains(&id)
}

/// Reset all game state (for new game)
pub fn reset_all() {
    let mut store = store().lock().expect("save store poisoned");
    store.slots.clear();
    store.pending_load_slot = None;
    store.unlocked_locks.clear();
    store.collected_keys.clear();
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
        let mut store = store().lock().expect("save store poisoned");
        store.pending_load_slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_queue_load_in_single_slot() {
        // Ensure deterministic state for the test
        {
            let mut store = store().lock().expect("save store poisoned");
            store.slots.clear();
            store.pending_load_slot = None;
        }

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
        // Reset store state
        {
            let mut store = store().lock().expect("save store poisoned");
            store.slots.clear();
            store.pending_load_slot = None;
        }

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
