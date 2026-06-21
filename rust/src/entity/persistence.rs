use godot::{classes::Node, prelude::*};

use crate::core::progress::{
    self, DEFAULT_SAVE_SLOT, PersistentEntityKind, PersistentKey, SaveSnapshot,
};
use crate::core::world::RoomId;

const LDTK_IID_META: &str = "ldtk_iid";

pub(crate) const PLAIN_KEY_GROUP: &str = "plain_keys";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PersistentEntityRef {
    pub room: RoomId,
    pub position: Vector2,
    key: PersistentKey,
}

impl PersistentEntityRef {
    pub(crate) fn new(node: &Gd<Node>, room_coords: Vector2i, position: Vector2) -> Self {
        Self {
            room: room_id(room_coords),
            position,
            key: persistent_key(node, room_coords, position),
        }
    }

    pub(crate) fn is_marked(&self, kind: PersistentEntityKind) -> bool {
        progress::has_entity_key(kind, &self.key)
    }

    pub(crate) fn mark(&self, kind: PersistentEntityKind) -> bool {
        progress::mark_entity_key(kind, self.key.clone())
    }

    pub(crate) fn save_checkpoint(&self) -> SaveSnapshot {
        progress::save_checkpoint_key(
            DEFAULT_SAVE_SLOT,
            self.room,
            self.position,
            self.key.clone(),
        )
    }

    pub(crate) fn find_saved_checkpoint(&self, match_epsilon: f32) -> Option<SaveSnapshot> {
        let snapshot = progress::peek_checkpoint(DEFAULT_SAVE_SLOT)?;
        snapshot
            .matches_checkpoint(self.room, self.position, match_epsilon, Some(&self.key))
            .then_some(snapshot)
    }
}

fn room_id(room_coords: Vector2i) -> RoomId {
    RoomId::from(room_coords)
}

fn persistent_key(node: &Gd<Node>, room_coords: Vector2i, position: Vector2) -> PersistentKey {
    if node.has_meta(LDTK_IID_META) {
        let iid = node.get_meta(LDTK_IID_META).to::<GString>().to_string();
        if !iid.is_empty() {
            return PersistentKey::Explicit(iid);
        }
    }

    progress::make_legacy_key(room_id(room_coords), position)
}
