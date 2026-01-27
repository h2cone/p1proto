//! SaveService - Event-driven save handler.
//! Listens to entity signals and handles persistence.

use godot::prelude::*;

use crate::entity::{
    checkpoint::Checkpoint, collectible_star::CollectibleStar, plain_key::PlainKey,
    plain_lock::PlainLock,
};

use super::{
    DEFAULT_SAVE_SLOT, mark_key_collected, mark_lock_unlocked, mark_star_collected, save_checkpoint,
};

/// Entity layer name in LDtk imported scenes
const ENTITY_LAYER_NAME: &str = "Entities";

/// SaveService listens to entity signals and handles persistence.
/// Add as a child node to RoomManager to automatically handle save events.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct SaveService {
    base: Base<Node>,
}

#[godot_api]
impl INode for SaveService {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl SaveService {
    /// Connect to all saveable entities in a room.
    /// Call this after loading a new room.
    #[func]
    pub fn connect_room_entities(&mut self, room: Gd<Node2D>) {
        let Some(entities) = room.get_node_or_null(ENTITY_LAYER_NAME) else {
            return;
        };

        for child in entities.get_children().iter_shared() {
            self.try_connect_entity(&child);
        }
    }

    /// Try to connect to an entity's save-related signals.
    fn try_connect_entity(&mut self, node: &Gd<Node>) {
        let save_service = self.to_gd();

        // Connect PlainKey signals
        if let Ok(key) = node.clone().try_cast::<PlainKey>() {
            key.signals()
                .key_collected()
                .connect_other(&save_service, Self::on_key_collected);
            godot_print!("[SaveService] connected to key_collected signal");
            return;
        }

        // Connect PlainLock signals
        if let Ok(lock) = node.clone().try_cast::<PlainLock>() {
            lock.signals()
                .lock_unlocked()
                .connect_other(&save_service, Self::on_lock_unlocked);
            godot_print!("[SaveService] connected to lock_unlocked signal");
            return;
        }

        // Connect Checkpoint signals
        if let Ok(checkpoint) = node.clone().try_cast::<Checkpoint>() {
            checkpoint
                .signals()
                .checkpoint_activated()
                .connect_other(&save_service, Self::on_checkpoint_activated);
            godot_print!("[SaveService] connected to checkpoint_activated signal");
            return;
        }

        // Connect CollectibleStar signals
        if let Ok(star) = node.clone().try_cast::<CollectibleStar>() {
            star.signals()
                .star_collected()
                .connect_other(&save_service, Self::on_star_collected);
            godot_print!("[SaveService] connected to star_collected signal");
        }
    }

    /// Handle key collection event.
    #[func]
    fn on_key_collected(&mut self, room_coords: Vector2i, position: Vector2) {
        let room = (room_coords.x, room_coords.y);
        mark_key_collected(room, position);
        godot_print!(
            "[SaveService] key collected at room {:?}, position {:?}",
            room,
            position
        );
    }

    /// Handle lock unlock event.
    #[func]
    fn on_lock_unlocked(&mut self, room_coords: Vector2i, position: Vector2) {
        let room = (room_coords.x, room_coords.y);
        mark_lock_unlocked(room, position);
        godot_print!(
            "[SaveService] lock unlocked at room {:?}, position {:?}",
            room,
            position
        );
    }

    /// Handle checkpoint activation event.
    #[func]
    fn on_checkpoint_activated(&mut self, room_coords: Vector2i, position: Vector2) {
        let room = (room_coords.x, room_coords.y);
        let snapshot = save_checkpoint(DEFAULT_SAVE_SLOT, room, position);
        godot_print!(
            "[SaveService] checkpoint saved to slot {} at room {:?}, position {:?}",
            DEFAULT_SAVE_SLOT,
            snapshot.room,
            snapshot.position
        );
    }

    /// Handle star collection event.
    #[func]
    fn on_star_collected(&mut self, room_coords: Vector2i, position: Vector2) {
        let room = (room_coords.x, room_coords.y);
        mark_star_collected(room, position);
        godot_print!(
            "[SaveService] star collected at room {:?}, position {:?}",
            room,
            position
        );
    }
}
