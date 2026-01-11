//! Spawn point resolver.
//! Determines where to spawn the player based on save state and defaults.

use godot::prelude::*;

use crate::save::{self, DEFAULT_SAVE_SLOT};

/// Resolved spawn point data.
#[derive(Clone, Copy, Debug)]
pub struct SpawnPoint {
    pub room: (i32, i32),
    pub position: Vector2,
}

/// Resolves spawn points from save data or default values.
pub struct SpawnResolver {
    initial_room: (i32, i32),
    initial_position: Vector2,
}

impl SpawnResolver {
    /// Create a new resolver with default spawn point.
    pub fn new(initial_room: (i32, i32), initial_position: Vector2) -> Self {
        Self {
            initial_room,
            initial_position,
        }
    }

    /// Resolve the spawn point.
    /// Checks for pending save load first, falls back to initial spawn.
    /// The `room_exists` callback validates that a room file exists.
    pub fn resolve(&self, mut room_exists: impl FnMut((i32, i32)) -> bool) -> SpawnPoint {
        if let Some(snapshot) = save::take_pending_load() {
            if room_exists(snapshot.room) {
                godot_print!(
                    "[SpawnResolver] loading from save slot {}: room {:?}, position {:?}",
                    DEFAULT_SAVE_SLOT,
                    snapshot.room,
                    snapshot.position
                );
                return SpawnPoint {
                    room: snapshot.room,
                    position: snapshot.position,
                };
            } else {
                godot_warn!(
                    "Saved room {:?} no longer exists; falling back to initial spawn",
                    snapshot.room
                );
            }
        }

        SpawnPoint {
            room: self.initial_room,
            position: self.initial_position,
        }
    }
}
