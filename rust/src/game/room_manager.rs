use godot::classes::CharacterBody2D;
use godot::prelude::*;

use super::{PlayerSpawner, SpawnResolver, connect_room_portal, find_portal_in_room};
use crate::rooms::{BoundaryDetector, RoomLoader};
use crate::save::SaveService;

/// Default initial room grid coordinates (can be overridden in Godot).
const INITIAL_ROOM: (i32, i32) = (0, 1);

/// Default initial player position within the room (can be overridden in Godot).
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

struct CollisionRestore {
    layer: u32,
    mask: u32,
    frames_remaining: u8,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct RoomManager {
    base: Base<Node2D>,
    /// Initial room grid coordinates (editable in Godot).
    #[export]
    initial_room: Vector2i,
    /// Initial player position within the room (editable in Godot).
    #[export]
    initial_player_pos: Vector2,
    /// Room loader for managing scene loading
    room_loader: RoomLoader,
    /// Boundary detector for checking room transitions
    boundary_detector: BoundaryDetector,
    /// Player scene spawner
    player_spawner: PlayerSpawner,
    /// Spawn point resolver
    spawn_resolver: SpawnResolver,
    /// Save service for entity persistence
    save_service: Option<Gd<SaveService>>,
    /// Current room grid coordinates
    current_room: (i32, i32),
    /// Current room node in scene tree
    current_room_node: Option<Gd<Node2D>>,
    /// Player character node
    player: Option<Gd<CharacterBody2D>>,
    /// Restore player collision after room transitions
    pending_player_collision_restore: Option<CollisionRestore>,
}

#[godot_api]
impl INode2D for RoomManager {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            initial_room: Vector2i::new(INITIAL_ROOM.0, INITIAL_ROOM.1),
            initial_player_pos: INITIAL_PLAYER_POS,
            room_loader: RoomLoader::new(ROOM_SCENE_PATTERN.to_string()),
            boundary_detector: BoundaryDetector::new(TRANSITION_THRESHOLD),
            player_spawner: PlayerSpawner::new(PLAYER_SCENE_PATH),
            spawn_resolver: SpawnResolver::new(INITIAL_ROOM, INITIAL_PLAYER_POS),
            save_service: None,
            current_room: INITIAL_ROOM,
            current_room_node: None,
            player: None,
            pending_player_collision_restore: None,
        }
    }

    fn ready(&mut self) {
        godot_print!("[RoomManager] ready - initializing room transition system");

        // Create and add SaveService as child node
        let save_service = Gd::<SaveService>::from_init_fn(SaveService::init);
        self.base_mut().add_child(&save_service);
        self.save_service = Some(save_service);
        godot_print!("[RoomManager] SaveService created");

        // Apply editor overrides for initial room/position before resolving spawn.
        let initial_room = (self.initial_room.x, self.initial_room.y);
        let initial_pos = self.initial_player_pos;
        self.spawn_resolver = SpawnResolver::new(initial_room, initial_pos);

        // Resolve spawn point from save or defaults
        let spawn = self
            .spawn_resolver
            .resolve(|room| self.room_loader.room_exists(room));

        // Load and spawn target room
        match self.load_and_add_room(spawn.room) {
            Some(mut room_node) => {
                self.current_room = spawn.room;
                godot_print!("[RoomManager] spawned room at {:?}", spawn.room);

                // Load and spawn player
                match self.player_spawner.spawn() {
                    Some(mut player) => {
                        player.set_global_position(spawn.position);
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

                // Connect SaveService to entity signals
                self.connect_save_service(&room_node);

                self.current_room_node = Some(room_node);
            }
            None => {
                godot_error!("Failed to load room at {:?}", spawn.room);
            }
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.tick_player_collision_restore();
        self.check_room_transitions();
    }
}

#[godot_api]
impl RoomManager {
    /// Load a room scene and add it to the RoomManager node
    fn load_and_add_room(&mut self, room_coords: (i32, i32)) -> Option<Gd<Node2D>> {
        let room_node = self.room_loader.instantiate_room(room_coords)?;
        self.base_mut().add_child(&room_node);
        Some(room_node)
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

        self.disable_player_collision_for_transition(player);

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
                    SpawnMode::AtPortal => find_portal_in_room(&new_room, ENTITY_LAYER_NAME)
                        .map(|p| p.get_global_position())
                        .unwrap_or(DEFAULT_SPAWN_POS),
                };

                // 5. Add player to new room
                player.set_position(spawn_pos);
                new_room.add_child(&*player);

                // 6. Connect portal signals
                self.connect_portal_signals(&new_room);

                // 7. Connect SaveService to entity signals
                self.connect_save_service(&new_room);

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
        connect_room_portal(
            room,
            ENTITY_LAYER_NAME,
            &self.base().clone().upcast::<Node>(),
            "on_portal_teleport_requested",
        );
    }

    /// Connect SaveService to entity signals in the given room
    fn connect_save_service(&mut self, room: &Gd<Node2D>) {
        if let Some(ref mut service) = self.save_service {
            service.bind_mut().connect_room_entities(room.clone());
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

    fn disable_player_collision_for_transition(&mut self, player: &mut Gd<CharacterBody2D>) {
        if let Some(state) = &mut self.pending_player_collision_restore {
            state.frames_remaining = 1;
            return;
        }

        let layer = player.get_collision_layer();
        let mask = player.get_collision_mask();
        player.set_collision_layer(0);
        player.set_collision_mask(0);
        self.pending_player_collision_restore = Some(CollisionRestore {
            layer,
            mask,
            frames_remaining: 1,
        });
    }

    fn tick_player_collision_restore(&mut self) {
        let Some(state) = &mut self.pending_player_collision_restore else {
            return;
        };

        if state.frames_remaining > 0 {
            state.frames_remaining -= 1;
            return;
        }

        let Some(player) = self.player.as_mut() else {
            return;
        };

        player.set_collision_layer(state.layer);
        player.set_collision_mask(state.mask);
        self.pending_player_collision_restore = None;
    }
}
