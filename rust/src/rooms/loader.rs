use godot::{
    classes::{Node2D, PackedScene},
    prelude::*,
};
use std::collections::HashMap;

/// Room loader that handles loading and caching room scenes
///
/// Design considerations from spec:
/// - Calculates adjacent rooms from grid coordinates (not hardcoded connections)
/// - Simple, focused responsibility: just loading rooms
pub struct RoomLoader {
    /// Cache of loaded PackedScene resources indexed by grid coordinates
    scene_cache: HashMap<(i32, i32), Gd<PackedScene>>,
    /// Base path pattern for room scenes in the Godot project
    scene_path_pattern: String,
}

impl RoomLoader {
    /// Base path pattern for room scenes, e.g. "res://pipeline/ldtk/levels/Room_{x}_{y}.scn".
    /// Use {x} and {y} as placeholders for grid coordinates.
    pub fn new(scene_path_pattern: String) -> Self {
        Self {
            scene_cache: HashMap::new(),
            scene_path_pattern,
        }
    }

    fn scene_path(&self, room_coords: (i32, i32)) -> String {
        self.scene_path_pattern
            .replace("{x}", &room_coords.0.to_string())
            .replace("{y}", &room_coords.1.to_string())
    }

    fn load_scene_from_disk(&self, room_coords: (i32, i32)) -> Option<Gd<PackedScene>> {
        let path = self.scene_path(room_coords);

        match try_load::<PackedScene>(&path) {
            Ok(scene) => {
                godot_print!("[RoomLoader] loaded room scene: {}", path);
                Some(scene)
            }
            Err(_) => {
                godot_warn!("Failed to load room scene: {}", path);
                None
            }
        }
    }

    /// Returns the loaded PackedScene, caching it for future requests.
    /// Returns None if the scene file doesn't exist.
    pub fn load_room_scene(&mut self, room_coords: (i32, i32)) -> Option<Gd<PackedScene>> {
        if let Some(scene) = self.scene_cache.get(&room_coords) {
            return Some(scene.clone());
        }

        let scene = self.load_scene_from_disk(room_coords)?;
        self.scene_cache.insert(room_coords, scene.clone());
        Some(scene)
    }

    pub fn instantiate_room(&mut self, room_coords: (i32, i32)) -> Option<Gd<Node2D>> {
        let scene = self.load_room_scene(room_coords)?;
        let instance = scene.instantiate().or_else(|| {
            godot_error!("Failed to instantiate room scene at {:?}", room_coords);
            None
        })?;
        match instance.try_cast::<Node2D>() {
            Ok(node) => Some(node),
            Err(instance) => {
                godot_error!(
                    "Room scene root at {:?} is not a Node2D (got {})",
                    room_coords,
                    instance.get_class()
                );
                None
            }
        }
    }

    /// Useful for validating transitions before attempting to load.
    pub fn room_exists(&mut self, room_coords: (i32, i32)) -> bool {
        self.load_room_scene(room_coords).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_path_pattern_substitution() {
        let loader = RoomLoader::new("res://rooms/Room_{x}_{y}.scn".to_string());
        assert_eq!(loader.scene_path((3, 4)), "res://rooms/Room_3_4.scn");
    }
}
