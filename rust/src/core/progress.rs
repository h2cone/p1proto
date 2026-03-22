use godot::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;

use super::world::RoomId;

pub const DEFAULT_SAVE_SLOT: usize = 0;

pub type SaveSlot = usize;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PersistentKey {
    Explicit(String),
    Legacy { room: RoomId, position: (i32, i32) },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistentEntityKind {
    Key,
    Lock,
    Star,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveSnapshot {
    pub room: RoomId,
    pub position: Vector2,
    checkpoint_key: Option<PersistentKey>,
}

impl SaveSnapshot {
    pub fn new(room: RoomId, position: Vector2) -> Self {
        Self {
            room,
            position,
            checkpoint_key: None,
        }
    }

    pub fn with_checkpoint_key(
        room: RoomId,
        position: Vector2,
        checkpoint_key: PersistentKey,
    ) -> Self {
        Self {
            room,
            position,
            checkpoint_key: Some(checkpoint_key),
        }
    }

    pub fn matches_checkpoint(
        &self,
        room: RoomId,
        position: Vector2,
        match_epsilon: f32,
        checkpoint_key: Option<&PersistentKey>,
    ) -> bool {
        if self.room != room {
            return false;
        }

        if let (Some(saved_key), Some(current_key)) = (self.checkpoint_key.as_ref(), checkpoint_key)
        {
            return saved_key == current_key;
        }

        position.distance_to(self.position) <= match_epsilon
    }
}

#[derive(Default)]
pub struct ProgressProfile {
    unlocked_locks: HashSet<PersistentKey>,
    collected_keys: HashSet<PersistentKey>,
    collected_stars: HashSet<PersistentKey>,
    explored_rooms: HashSet<RoomId>,
}

impl ProgressProfile {
    fn entity_set(&self, kind: PersistentEntityKind) -> &HashSet<PersistentKey> {
        match kind {
            PersistentEntityKind::Key => &self.collected_keys,
            PersistentEntityKind::Lock => &self.unlocked_locks,
            PersistentEntityKind::Star => &self.collected_stars,
        }
    }

    fn entity_set_mut(&mut self, kind: PersistentEntityKind) -> &mut HashSet<PersistentKey> {
        match kind {
            PersistentEntityKind::Key => &mut self.collected_keys,
            PersistentEntityKind::Lock => &mut self.unlocked_locks,
            PersistentEntityKind::Star => &mut self.collected_stars,
        }
    }

    pub fn mark_entity_key(&mut self, kind: PersistentEntityKind, key: PersistentKey) -> bool {
        self.entity_set_mut(kind).insert(key)
    }

    pub fn has_entity_key(&self, kind: PersistentEntityKind, key: &PersistentKey) -> bool {
        self.entity_set(kind).contains(key)
    }

    pub fn mark_room_explored(&mut self, room: RoomId) -> bool {
        self.explored_rooms.insert(room)
    }

    pub fn is_room_explored(&self, room: RoomId) -> bool {
        self.explored_rooms.contains(&room)
    }

    pub fn list_explored_rooms(&self) -> Vec<RoomId> {
        self.explored_rooms.iter().copied().collect()
    }

    pub fn star_count(&self) -> usize {
        self.collected_stars.len()
    }

    pub fn reset(&mut self) {
        self.unlocked_locks.clear();
        self.collected_keys.clear();
        self.collected_stars.clear();
        self.explored_rooms.clear();
    }
}

#[derive(Default)]
pub struct ProgressRepository {
    slots: Vec<Option<SaveSnapshot>>,
    pending_load_slot: Option<SaveSlot>,
    active_profile: ProgressProfile,
}

impl ProgressRepository {
    fn ensure_slot(&mut self, slot: SaveSlot) {
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }
    }

    pub fn save_checkpoint(
        &mut self,
        slot: SaveSlot,
        room: RoomId,
        position: Vector2,
        checkpoint_key: Option<PersistentKey>,
    ) -> SaveSnapshot {
        self.ensure_slot(slot);
        let snapshot = checkpoint_key
            .map(|key| SaveSnapshot::with_checkpoint_key(room, position, key))
            .unwrap_or_else(|| SaveSnapshot::new(room, position));
        self.slots[slot] = Some(snapshot.clone());
        snapshot
    }

    pub fn peek_checkpoint(&self, slot: SaveSlot) -> Option<SaveSnapshot> {
        self.slots.get(slot).and_then(|slot_data| slot_data.clone())
    }

    pub fn has_save(&self, slot: SaveSlot) -> bool {
        self.slots
            .get(slot)
            .and_then(|slot_data| slot_data.as_ref())
            .is_some()
    }

    pub fn queue_load(&mut self, slot: SaveSlot) -> bool {
        self.ensure_slot(slot);
        if self.slots[slot].is_some() {
            self.pending_load_slot = Some(slot);
            true
        } else {
            false
        }
    }

    pub fn take_pending_load(&mut self) -> Option<SaveSnapshot> {
        let slot = self.pending_load_slot.take()?;
        self.ensure_slot(slot);
        self.slots.get(slot).and_then(|slot_data| slot_data.clone())
    }

    pub fn clear_pending_load(&mut self) {
        self.pending_load_slot = None;
    }

    pub fn mark_entity_key(&mut self, kind: PersistentEntityKind, key: PersistentKey) -> bool {
        self.active_profile.mark_entity_key(kind, key)
    }

    pub fn has_entity_key(&self, kind: PersistentEntityKind, key: &PersistentKey) -> bool {
        self.active_profile.has_entity_key(kind, key)
    }

    pub fn mark_room_explored(&mut self, room: RoomId) -> bool {
        self.active_profile.mark_room_explored(room)
    }

    pub fn is_room_explored(&self, room: RoomId) -> bool {
        self.active_profile.is_room_explored(room)
    }

    pub fn list_explored_rooms(&self) -> Vec<RoomId> {
        self.active_profile.list_explored_rooms()
    }

    pub fn star_count(&self) -> usize {
        self.active_profile.star_count()
    }

    pub fn reset_all(&mut self) {
        self.slots.clear();
        self.pending_load_slot = None;
        self.active_profile.reset();
    }
}

thread_local! {
    static REPOSITORY: RefCell<ProgressRepository> = RefCell::new(ProgressRepository::default());
}

pub fn make_legacy_key(room: RoomId, position: Vector2) -> PersistentKey {
    PersistentKey::Legacy {
        room,
        position: (position.x as i32, position.y as i32),
    }
}

#[cfg(test)]
pub fn save_checkpoint(slot: SaveSlot, room: RoomId, position: Vector2) -> SaveSnapshot {
    REPOSITORY.with_borrow_mut(|repository| repository.save_checkpoint(slot, room, position, None))
}

pub fn save_checkpoint_key(
    slot: SaveSlot,
    room: RoomId,
    position: Vector2,
    checkpoint_key: PersistentKey,
) -> SaveSnapshot {
    REPOSITORY.with_borrow_mut(|repository| {
        repository.save_checkpoint(slot, room, position, Some(checkpoint_key))
    })
}

pub fn peek_checkpoint(slot: SaveSlot) -> Option<SaveSnapshot> {
    REPOSITORY.with_borrow(|repository| repository.peek_checkpoint(slot))
}

pub fn has_save(slot: SaveSlot) -> bool {
    REPOSITORY.with_borrow(|repository| repository.has_save(slot))
}

pub fn queue_load(slot: SaveSlot) -> bool {
    REPOSITORY.with_borrow_mut(|repository| repository.queue_load(slot))
}

pub fn take_pending_load() -> Option<SaveSnapshot> {
    REPOSITORY.with_borrow_mut(ProgressRepository::take_pending_load)
}

pub fn clear_pending_load() {
    REPOSITORY.with_borrow_mut(ProgressRepository::clear_pending_load);
}

#[cfg(test)]
pub fn mark_entity(kind: PersistentEntityKind, room: RoomId, position: Vector2) -> bool {
    mark_entity_key(kind, make_legacy_key(room, position))
}

pub fn mark_entity_key(kind: PersistentEntityKind, key: PersistentKey) -> bool {
    REPOSITORY.with_borrow_mut(|repository| repository.mark_entity_key(kind, key))
}

#[cfg(test)]
pub fn has_entity(kind: PersistentEntityKind, room: RoomId, position: Vector2) -> bool {
    has_entity_key(kind, &make_legacy_key(room, position))
}

pub fn has_entity_key(kind: PersistentEntityKind, key: &PersistentKey) -> bool {
    REPOSITORY.with_borrow(|repository| repository.has_entity_key(kind, key))
}

pub fn mark_room_explored(room: RoomId) -> bool {
    REPOSITORY.with_borrow_mut(|repository| repository.mark_room_explored(room))
}

pub fn is_room_explored(room: RoomId) -> bool {
    REPOSITORY.with_borrow(|repository| repository.is_room_explored(room))
}

pub fn list_explored_rooms() -> Vec<RoomId> {
    REPOSITORY.with_borrow(ProgressRepository::list_explored_rooms)
}

pub fn get_star_count() -> usize {
    REPOSITORY.with_borrow(ProgressRepository::star_count)
}

pub fn reset_all() {
    REPOSITORY.with_borrow_mut(ProgressRepository::reset_all);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_roundtrip_and_pending_load() {
        reset_all();

        let room = (1, 2);
        let position = Vector2::new(10.0, 20.0);
        let saved = save_checkpoint(DEFAULT_SAVE_SLOT, room, position);

        assert_eq!(saved.room, room);
        assert_eq!(saved.position, position);
        assert!(has_save(DEFAULT_SAVE_SLOT));
        assert!(queue_load(DEFAULT_SAVE_SLOT));

        let pending = take_pending_load().expect("expected pending load");
        assert_eq!(pending.room, room);
        assert_eq!(pending.position, position);
        assert!(peek_checkpoint(DEFAULT_SAVE_SLOT).is_some());
    }

    #[test]
    fn tracks_persistent_entities_and_stars() {
        reset_all();

        let room = (3, 4);
        let key_pos = Vector2::new(5.0, 6.0);
        let star_key = PersistentKey::Explicit("star:room_3_4".to_string());

        assert!(mark_entity(PersistentEntityKind::Key, room, key_pos));
        assert!(mark_entity_key(
            PersistentEntityKind::Star,
            star_key.clone()
        ));
        assert!(has_entity(PersistentEntityKind::Key, room, key_pos));
        assert!(has_entity_key(PersistentEntityKind::Star, &star_key));
        assert_eq!(get_star_count(), 1);
    }

    #[test]
    fn checkpoint_key_prefers_explicit_id_over_position() {
        reset_all();

        let checkpoint_key = PersistentKey::Explicit("checkpoint:alpha".to_string());
        let snapshot = save_checkpoint_key(
            DEFAULT_SAVE_SLOT,
            (0, 1),
            Vector2::new(16.0, 24.0),
            checkpoint_key.clone(),
        );

        assert!(snapshot.matches_checkpoint(
            (0, 1),
            Vector2::new(32.0, 48.0),
            0.1,
            Some(&checkpoint_key),
        ));
        assert!(!snapshot.matches_checkpoint(
            (0, 1),
            Vector2::new(16.0, 24.0),
            0.1,
            Some(&PersistentKey::Explicit("checkpoint:beta".to_string())),
        ));
    }

    #[test]
    fn tracks_explored_rooms_and_reset() {
        reset_all();

        assert!(mark_room_explored((2, 3)));
        assert!(is_room_explored((2, 3)));
        assert_eq!(list_explored_rooms(), vec![(2, 3)]);

        reset_all();

        assert!(!has_save(DEFAULT_SAVE_SLOT));
        assert!(!is_room_explored((2, 3)));
    }
}
