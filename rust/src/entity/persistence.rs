use godot::{classes::Node, prelude::*};

use crate::core::progress::{
    self, DEFAULT_SAVE_SLOT, PersistentEntityKind, PersistentKey, SaveSnapshot,
};
use crate::core::world::RoomId;

const LDTK_IID_META: &str = "ldtk_iid";

pub const PLAIN_KEY_GROUP: &str = "plain_keys";

pub fn room_id(room_coords: Vector2i) -> RoomId {
    (room_coords.x, room_coords.y)
}

pub fn persistent_key(node: &Gd<Node>, room_coords: Vector2i, position: Vector2) -> PersistentKey {
    if node.has_meta(LDTK_IID_META) {
        let iid = node.get_meta(LDTK_IID_META).to::<GString>().to_string();
        if !iid.is_empty() {
            return PersistentKey::Explicit(iid);
        }
    }

    progress::make_legacy_key(room_id(room_coords), position)
}

pub fn has_persistent_entity(
    kind: PersistentEntityKind,
    node: &Gd<Node>,
    room_coords: Vector2i,
    position: Vector2,
) -> bool {
    let key = persistent_key(node, room_coords, position);
    progress::has_entity_key(kind, &key)
}

pub fn mark_persistent_entity(
    kind: PersistentEntityKind,
    node: &Gd<Node>,
    room_coords: Vector2i,
    position: Vector2,
) -> bool {
    progress::mark_entity_key(kind, persistent_key(node, room_coords, position))
}

pub fn save_checkpoint(
    node: &Gd<Node>,
    room_coords: Vector2i,
    checkpoint_position: Vector2,
) -> SaveSnapshot {
    let room = room_id(room_coords);
    let key = persistent_key(node, room_coords, checkpoint_position);
    progress::save_checkpoint_key(DEFAULT_SAVE_SLOT, room, checkpoint_position, key)
}

pub fn find_saved_checkpoint(
    node: &Gd<Node>,
    room_coords: Vector2i,
    checkpoint_position: Vector2,
    match_epsilon: f32,
) -> Option<SaveSnapshot> {
    let snapshot = progress::peek_checkpoint(DEFAULT_SAVE_SLOT)?;
    let key = persistent_key(node, room_coords, checkpoint_position);
    snapshot
        .matches_checkpoint(
            room_id(room_coords),
            checkpoint_position,
            match_epsilon,
            Some(&key),
        )
        .then_some(snapshot)
}
