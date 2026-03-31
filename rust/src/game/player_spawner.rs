//! Player scene spawner.
//! Handles loading and instantiating player scenes.

use godot::classes::{CharacterBody2D, PackedScene};
use godot::prelude::*;

pub struct PlayerSpawner {
    scene_path: String,
}

impl PlayerSpawner {
    pub fn new(scene_path: &str) -> Self {
        Self {
            scene_path: scene_path.to_string(),
        }
    }

    pub fn spawn(&self) -> Option<Gd<CharacterBody2D>> {
        let scene = try_load::<PackedScene>(&self.scene_path).ok().or_else(|| {
            godot_error!("Failed to load player scene from {}", self.scene_path);
            None
        })?;
        let instance = scene.instantiate().or_else(|| {
            godot_error!("Failed to instantiate player scene");
            None
        })?;
        match instance.try_cast::<CharacterBody2D>() {
            Ok(player) => Some(player),
            Err(instance) => {
                godot_error!(
                    "Player scene root is not CharacterBody2D (got {})",
                    instance.get_class()
                );
                None
            }
        }
    }
}
