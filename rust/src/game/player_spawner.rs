//! Player scene spawner.
//! Handles loading and instantiating player scenes.

use godot::classes::{CharacterBody2D, PackedScene};
use godot::prelude::*;

/// Spawns player instances from a scene file.
pub struct PlayerSpawner {
    scene_path: String,
}

impl PlayerSpawner {
    /// Create a new spawner with the given scene path.
    pub fn new(scene_path: &str) -> Self {
        Self {
            scene_path: scene_path.to_string(),
        }
    }

    /// Spawn a new player instance.
    /// Returns None if the scene fails to load or instantiate.
    pub fn spawn(&self) -> Option<Gd<CharacterBody2D>> {
        match try_load::<PackedScene>(&self.scene_path) {
            Ok(scene) => match scene.instantiate() {
                Some(instance) => match instance.try_cast::<CharacterBody2D>() {
                    Ok(player) => Some(player),
                    Err(instance) => {
                        godot_error!(
                            "Player scene root is not CharacterBody2D (got {})",
                            instance.get_class()
                        );
                        None
                    }
                },
                None => {
                    godot_error!("Failed to instantiate player scene");
                    None
                }
            },
            Err(_) => {
                godot_error!("Failed to load player scene from {}", self.scene_path);
                None
            }
        }
    }
}
