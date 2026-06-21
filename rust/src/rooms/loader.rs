use godot::{
    classes::{Node2D, PackedScene},
    prelude::*,
};
use std::{collections::HashMap, fmt};

use crate::core::world::RoomId;

/// Room loader that handles loading and caching room scenes
///
/// Design considerations from spec:
/// - Calculates adjacent rooms from grid coordinates (not hardcoded connections)
/// - Simple, focused responsibility: just loading rooms
pub(crate) struct RoomLoader {
    /// Cache of loaded PackedScene resources indexed by grid coordinates
    scene_cache: HashMap<RoomId, Gd<PackedScene>>,
    /// Base path pattern for room scenes in the Godot project
    scene_path_pattern: String,
}

#[derive(Debug)]
pub(crate) enum RoomLoadError {
    Load { room: RoomId, path: String },
    Instantiate { room: RoomId, path: String },
    WrongRoot { room: RoomId, class: String },
}

impl fmt::Display for RoomLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load { room, path } => write!(f, "failed to load room {room} from {path}"),
            Self::Instantiate { room, path } => {
                write!(f, "failed to instantiate room {room} from {path}")
            }
            Self::WrongRoot { room, class } => {
                write!(f, "room {room} root is not Node2D, got {class}")
            }
        }
    }
}

impl RoomLoader {
    /// Base path pattern for room scenes, e.g. "res://pipeline/ldtk/levels/Room_{x}_{y}.scn".
    /// Use {x} and {y} as placeholders for grid coordinates.
    pub(crate) fn new(scene_path_pattern: String) -> Self {
        Self {
            scene_cache: HashMap::new(),
            scene_path_pattern,
        }
    }

    fn scene_path(&self, room_coords: RoomId) -> String {
        self.scene_path_pattern
            .replace("{x}", &room_coords.x.to_string())
            .replace("{y}", &room_coords.y.to_string())
    }

    fn load_scene_from_disk(&self, room_coords: RoomId) -> Result<Gd<PackedScene>, RoomLoadError> {
        let path = self.scene_path(room_coords);

        match try_load::<PackedScene>(&path) {
            Ok(scene) => {
                godot_print!("[RoomLoader] loaded room scene: {}", path);
                Ok(scene)
            }
            Err(_) => Err(RoomLoadError::Load {
                room: room_coords,
                path,
            }),
        }
    }

    /// Returns the loaded PackedScene, caching it for future requests.
    /// Returns None if the scene file doesn't exist.
    fn load_room_scene(&mut self, room_coords: RoomId) -> Result<Gd<PackedScene>, RoomLoadError> {
        if let Some(scene) = self.scene_cache.get(&room_coords) {
            return Ok(scene.clone());
        }

        let scene = self.load_scene_from_disk(room_coords)?;
        self.scene_cache.insert(room_coords, scene.clone());
        Ok(scene)
    }

    pub(crate) fn instantiate_room(
        &mut self,
        room_coords: RoomId,
    ) -> Result<Gd<Node2D>, RoomLoadError> {
        let path = self.scene_path(room_coords);
        let scene = self.load_room_scene(room_coords)?;
        let instance = scene.instantiate().ok_or(RoomLoadError::Instantiate {
            room: room_coords,
            path,
        })?;
        match instance.try_cast::<Node2D>() {
            Ok(node) => Ok(node),
            Err(instance) => Err(RoomLoadError::WrongRoot {
                room: room_coords,
                class: instance.get_class().to_string(),
            }),
        }
    }

    /// Useful for validating transitions before attempting to load.
    pub(crate) fn room_exists(&mut self, room_coords: RoomId) -> bool {
        if self.scene_cache.contains_key(&room_coords) {
            return true;
        }

        let path = self.scene_path(room_coords);
        match try_load::<PackedScene>(&path) {
            Ok(scene) => {
                self.scene_cache.insert(room_coords, scene);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_path_pattern_substitution() {
        let loader = RoomLoader::new("res://rooms/Room_{x}_{y}.scn".to_string());
        assert_eq!(
            loader.scene_path(RoomId::new(3, 4)),
            "res://rooms/Room_3_4.scn"
        );
    }
}
