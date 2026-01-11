use godot::{
    classes::{CharacterBody2D, PackedScene},
    prelude::*,
};

use crate::rooms::{BoundaryDetector, RoomLoader};
use crate::save::{self, DEFAULT_SAVE_SLOT, SaveSnapshot};

/// Initial room grid coordinates (as specified in SPEC.md)
const INITIAL_ROOM: (i32, i32) = (0, 1);

/// Initial player position within the room (as specified in SPEC.md)
const INITIAL_PLAYER_POS: Vector2 = Vector2::new(64.0, 64.0);

/// Player scene path
const PLAYER_SCENE_PATH: &str = "res://player/player.tscn";

/// Room scene path pattern (LDtk level files)
const ROOM_SCENE_PATTERN: &str = "res://pipeline/ldtk/levels/Room_{x}_{y}.scn";

/// Transition threshold: 50% of player body must cross boundary
const TRANSITION_THRESHOLD: f32 = 0.5;

/// Entity layer name in LDtk imported scenes
const ENTITY_LAYER_NAME: &str = "Entities";

/// Default spawn position when portal is not found
const DEFAULT_SPAWN_POS: Vector2 = Vector2::new(64.0, 64.0);

/// How to determine the player's spawn position in a new room.
enum SpawnMode {
    /// Use a specific position (for boundary transitions).
    Position(Vector2),
    /// Spawn at the portal location in the target room.
    AtPortal,
}

/// Find Portal node in room's Entities layer
fn find_portal_in_room(room: &Gd<Node2D>) -> Option<Gd<Node2D>> {
    let entities = room.get_node_or_null(ENTITY_LAYER_NAME)?;
    for child in entities.get_children().iter_shared() {
        if child.is_class("Portal") {
            return Some(child.cast::<Node2D>());
        }
    }
    None
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct RoomManager {
    base: Base<Node2D>,
    /// Room loader for managing scene loading
    room_loader: RoomLoader,
    /// Boundary detector for checking room transitions
    boundary_detector: BoundaryDetector,
    /// Current room grid coordinates
    current_room: (i32, i32),
    /// Current room node in scene tree
    current_room_node: Option<Gd<Node2D>>,
    /// Player character node
    player: Option<Gd<CharacterBody2D>>,
}

#[godot_api]
impl INode2D for RoomManager {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            room_loader: RoomLoader::new(ROOM_SCENE_PATTERN.to_string()),
            boundary_detector: BoundaryDetector::new(TRANSITION_THRESHOLD),
            current_room: INITIAL_ROOM,
            current_room_node: None,
            player: None,
        }
    }

    fn ready(&mut self) {
        godot_print!("[RoomManager] ready - initializing room transition system");

        let spawn = self.resolve_spawn_point();

        // Load and spawn target room
        match self.load_and_add_room(spawn.room) {
            Some(mut room_node) => {
                self.current_room = spawn.room;
                godot_print!("[RoomManager] spawned room at {:?}", spawn.room);

                // Load and spawn player
                match self.load_player_scene() {
                    Some(mut player) => {
                        // Set spawn position
                        player.set_global_position(spawn.position);

                        // Add player as child of current room
                        room_node.add_child(&player);
                        self.player = Some(player);
                        godot_print!("[RoomManager] spawned player at {:?}", spawn.position);
                    }
                    None => {
                        godot_error!(
                            "Failed to load player scene for spawn at {:?}",
                            spawn.position
                        );
                    }
                }

                // Connect portal signals in the new room
                self.connect_portal_signals(&room_node);

                self.current_room_node = Some(room_node);
            }
            None => {
                godot_error!("Failed to load room at {:?}", spawn.room);
            }
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.check_room_transitions();
    }
}

#[godot_api]
impl RoomManager {
    /// Load a player scene from the player scene file
    fn load_player_scene(&self) -> Option<Gd<CharacterBody2D>> {
        match try_load::<PackedScene>(PLAYER_SCENE_PATH) {
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
                godot_error!("Failed to load player scene from {}", PLAYER_SCENE_PATH);
                None
            }
        }
    }

    /// Load a room scene and add it to the RoomManager node
    fn load_and_add_room(&mut self, room_coords: (i32, i32)) -> Option<Gd<Node2D>> {
        let room_node = self.room_loader.instantiate_room(room_coords)?;
        self.base_mut().add_child(&room_node);
        Some(room_node)
    }

    /// Determine spawn data for the next game session.
    ///
    /// Prefers a queued save load (e.g., from "Continue") and falls back to the initial spawn.
    fn resolve_spawn_point(&mut self) -> SaveSnapshot {
        if let Some(snapshot) = save::take_pending_load() {
            if self.room_loader.room_exists(snapshot.room) {
                godot_print!(
                    "[RoomManager] loading from save slot {}: room {:?}, position {:?}",
                    DEFAULT_SAVE_SLOT,
                    snapshot.room,
                    snapshot.position
                );
                return snapshot;
            } else {
                godot_warn!(
                    "Saved room {:?} no longer exists; falling back to initial spawn",
                    snapshot.room
                );
            }
        }

        SaveSnapshot::new(INITIAL_ROOM, INITIAL_PLAYER_POS)
    }

    /// Check if player should transition to an adjacent room
    fn check_room_transitions(&mut self) {
        let Some(mut player) = self.player.take() else {
            return;
        };

        // Get player position and velocity
        let player_pos = player.get_global_position();
        let player_velocity = player.get_velocity();

        // Check for boundary crossing
        let check =
            self.boundary_detector
                .check_transition(player_pos, player_velocity, self.current_room);

        if let Some(transition) = check {
            // Validate target room exists before transitioning
            if self.room_loader.room_exists(transition.target_room) {
                self.execute_room_transition(
                    &mut player,
                    transition.target_room,
                    SpawnMode::Position(transition.new_position),
                );
            } else {
                godot_warn!(
                    "Cannot transition to room {:?} - room does not exist",
                    transition.target_room
                );
            }
        }

        self.player = Some(player);
    }

    /// Unified room transition logic.
    ///
    /// Handles both boundary transitions and portal teleports through a common flow:
    /// 1. Remove player from current room
    /// 2. Destroy old room
    /// 3. Load new room
    /// 4. Calculate spawn position based on SpawnMode
    /// 5. Add player to new room
    /// 6. Connect portal signals
    fn execute_room_transition(
        &mut self,
        player: &mut Gd<CharacterBody2D>,
        target_room: (i32, i32),
        spawn_mode: SpawnMode,
    ) {
        godot_print!(
            "[RoomManager] transitioning from {:?} to {:?}",
            self.current_room,
            target_room
        );

        // 1. Remove player from current room
        if let Some(mut parent) = player.get_parent() {
            parent.remove_child(&*player);
        }

        // 2. Destroy old room
        if let Some(mut old_room) = self.current_room_node.take() {
            self.base_mut().remove_child(&old_room);
            old_room.queue_free();
        }

        // 3. Load new room
        match self.load_and_add_room(target_room) {
            Some(mut new_room) => {
                self.current_room = target_room;

                // 4. Calculate spawn position
                let spawn_pos = match spawn_mode {
                    SpawnMode::Position(pos) => pos,
                    SpawnMode::AtPortal => find_portal_in_room(&new_room)
                        .map(|p| p.get_global_position())
                        .unwrap_or(DEFAULT_SPAWN_POS),
                };

                // 5. Add player to new room
                player.set_position(spawn_pos);
                new_room.add_child(&*player);

                // 6. Connect portal signals
                self.connect_portal_signals(&new_room);

                self.current_room_node = Some(new_room);

                godot_print!(
                    "[RoomManager] room transition complete to {:?} at {:?}",
                    target_room,
                    spawn_pos
                );
            }
            None => {
                godot_error!("Failed to load target room {:?}", target_room);
                // Fallback: add player back to RoomManager directly
                self.base_mut().add_child(&*player);
            }
        }
    }

    /// Connect to portal teleport signals in the given room
    fn connect_portal_signals(&mut self, room: &Gd<Node2D>) {
        if let Some(portal) = find_portal_in_room(room) {
            let callable = self.base().callable("on_portal_teleport_requested");
            portal
                .upcast::<Node>()
                .connect("teleport_requested", &callable);
            godot_print!("[RoomManager] connected portal teleport signal");
        }
    }

    /// Handle portal teleport request
    #[func]
    fn on_portal_teleport_requested(&mut self, destination_room: Vector2i) {
        let target = (destination_room.x, destination_room.y);
        godot_print!("[RoomManager] portal teleport requested to {:?}", target);

        // Validate target room exists
        if !self.room_loader.room_exists(target) {
            godot_error!("Portal destination room {:?} does not exist", target);
            return;
        }

        let Some(mut player) = self.player.take() else {
            return;
        };

        self.execute_room_transition(&mut player, target, SpawnMode::AtPortal);
        player.set_velocity(Vector2::ZERO);

        self.player = Some(player);
    }
}
