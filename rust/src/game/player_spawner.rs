//! Player scene spawner.
//! Handles loading and instantiating player scenes.

use godot::classes::{CharacterBody2D, PackedScene};
use godot::prelude::*;
use std::fmt;

pub(crate) struct PlayerSpawner {
    scene_path: String,
}

#[derive(Debug)]
pub(crate) enum PlayerSpawnError {
    Load { path: String },
    Instantiate { path: String },
    WrongRoot { path: String, class: String },
}

impl fmt::Display for PlayerSpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load { path } => write!(f, "failed to load player scene from {path}"),
            Self::Instantiate { path } => {
                write!(f, "failed to instantiate player scene from {path}")
            }
            Self::WrongRoot { path, class } => {
                write!(
                    f,
                    "player scene root at {path} is not CharacterBody2D, got {class}"
                )
            }
        }
    }
}

impl PlayerSpawner {
    pub(crate) fn new(scene_path: &str) -> Self {
        Self {
            scene_path: scene_path.to_string(),
        }
    }

    pub(crate) fn spawn(&self) -> Result<Gd<CharacterBody2D>, PlayerSpawnError> {
        let scene =
            try_load::<PackedScene>(&self.scene_path).map_err(|_| PlayerSpawnError::Load {
                path: self.scene_path.clone(),
            })?;
        let instance = scene
            .instantiate()
            .ok_or_else(|| PlayerSpawnError::Instantiate {
                path: self.scene_path.clone(),
            })?;
        match instance.try_cast::<CharacterBody2D>() {
            Ok(player) => Ok(player),
            Err(instance) => Err(PlayerSpawnError::WrongRoot {
                path: self.scene_path.clone(),
                class: instance.get_class().to_string(),
            }),
        }
    }
}
