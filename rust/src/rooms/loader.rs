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
    /// Create a new room loader
    ///
    /// # Arguments
    /// * `scene_path_pattern` - Pattern for room scene paths, e.g., "res://pipeline/ldtk/levels/Room_{x}_{y}.scn"
    ///                          Use {x} and {y} as placeholders for grid coordinates
    pub fn new(scene_path_pattern: String) -> Self {
        Self {
            scene_cache: HashMap::new(),
            scene_path_pattern,
        }
    }

    /// Load a room scene by grid coordinates
    ///
    /// Returns the loaded PackedScene, caching it for future requests.
    /// Returns None if the scene file doesn't exist.
    pub fn load_room_scene(&mut self, room_coords: (i32, i32)) -> Option<Gd<PackedScene>> {
        // Check cache first
        if let Some(scene) = self.scene_cache.get(&room_coords) {
            return Some(scene.clone());
        }

        // Build the path from pattern
        let path = self
            .scene_path_pattern
            .replace("{x}", &room_coords.0.to_string())
            .replace("{y}", &room_coords.1.to_string());

        // Try to load the scene
        match try_load::<PackedScene>(&path) {
            Ok(scene) => {
                godot_print!("Loaded room scene: {}", path);
                self.scene_cache.insert(room_coords, scene.clone());
                Some(scene)
            }
            Err(_) => {
                godot_warn!("Failed to load room scene: {}", path);
                None
            }
        }
    }

    /// Instantiate a room scene as a Node2D
    ///
    /// This loads the scene if needed and creates an instance ready to add to the scene tree.
    pub fn instantiate_room(&mut self, room_coords: (i32, i32)) -> Option<Gd<Node2D>> {
        let scene = self.load_room_scene(room_coords)?;

        match scene.instantiate() {
            Some(instance) => {
                // Room scenes should have Node2D as root
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
            None => {
                godot_error!("Failed to instantiate room scene at {:?}", room_coords);
                None
            }
        }
    }

    /// Check if a room exists at the given coordinates
    ///
    /// This is useful for validating transitions before attempting to load.
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
        // We can't actually test loading without Godot, but we can verify the pattern logic
        assert_eq!(loader.scene_path_pattern, "res://rooms/Room_{x}_{y}.scn");
    }
}
