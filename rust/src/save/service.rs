//! SaveService - Event-driven save handler.
//! Listens to entity signals and handles persistence.

use godot::prelude::*;

use super::{DEFAULT_SAVE_SLOT, mark_key_collected, mark_lock_unlocked, save_checkpoint};

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
        let base_node = self.base().clone().upcast::<Node>();

        // Connect PlainKey signals
        if node.has_signal("key_collected") {
            let callable = base_node.callable("on_key_collected");
            if !node.is_connected("key_collected", &callable) {
                node.clone().connect("key_collected", &callable);
                godot_print!("[SaveService] connected to key_collected signal");
            }
        }

        // Connect PlainLock signals
        if node.has_signal("lock_unlocked") {
            let callable = base_node.callable("on_lock_unlocked");
            if !node.is_connected("lock_unlocked", &callable) {
                node.clone().connect("lock_unlocked", &callable);
                godot_print!("[SaveService] connected to lock_unlocked signal");
            }
        }

        // Connect Checkpoint signals
        if node.has_signal("checkpoint_activated") {
            let callable = base_node.callable("on_checkpoint_activated");
            if !node.is_connected("checkpoint_activated", &callable) {
                node.clone().connect("checkpoint_activated", &callable);
                godot_print!("[SaveService] connected to checkpoint_activated signal");
            }
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
}
