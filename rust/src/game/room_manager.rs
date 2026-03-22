use godot::classes::CharacterBody2D;
use godot::prelude::*;

use super::portal_connector::{connect_room_portal, find_portal_in_room};
use super::room_runtime::{PlayerRuntime, RoomRuntime};
use crate::core::session::{DeathPlan, RoomSession, RoomTransitionPlan, TransitionSpawn};
use crate::core::world::{BoundaryDetector, RoomId, SpawnResolver};
use crate::save::{self, DEFAULT_SAVE_SLOT};

const INITIAL_ROOM: RoomId = (0, 1);
const INITIAL_PLAYER_POS: Vector2 = Vector2::new(64.0, 64.0);
const PLAYER_SCENE_PATH: &str = "res://player/player.tscn";
const ROOM_SCENE_PATTERN: &str = "res://pipeline/ldtk/levels/Room_{x}_{y}.scn";
const TRANSITION_THRESHOLD: f32 = 0.5;
const ENTITY_LAYER_NAME: &str = "Entities";
const DEFAULT_SPAWN_POS: Vector2 = Vector2::new(64.0, 64.0);

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct GameRoomManager {
    base: Base<Node2D>,
    #[export]
    initial_room: Vector2i,
    #[export]
    initial_player_pos: Vector2,
    room_runtime: RoomRuntime,
    player_runtime: PlayerRuntime,
    boundary_detector: BoundaryDetector,
    spawn_resolver: SpawnResolver,
    room_session: RoomSession,
}

#[godot_api]
impl INode2D for GameRoomManager {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            initial_room: Vector2i::new(INITIAL_ROOM.0, INITIAL_ROOM.1),
            initial_player_pos: INITIAL_PLAYER_POS,
            room_runtime: RoomRuntime::new(ROOM_SCENE_PATTERN),
            player_runtime: PlayerRuntime::new(PLAYER_SCENE_PATH),
            boundary_detector: BoundaryDetector::new(TRANSITION_THRESHOLD),
            spawn_resolver: SpawnResolver::new(INITIAL_ROOM, INITIAL_PLAYER_POS),
            room_session: RoomSession::new(INITIAL_ROOM),
        }
    }

    fn ready(&mut self) {
        godot_print!("[RoomManager] ready - initializing room transition system");

        let initial_room = (self.initial_room.x, self.initial_room.y);
        let initial_pos = self.initial_player_pos;
        self.spawn_resolver = SpawnResolver::new(initial_room, initial_pos);
        self.room_session = RoomSession::new(initial_room);

        let spawn = {
            let room_runtime = &mut self.room_runtime;
            self.room_session
                .resolve_start(&self.spawn_resolver, |room| room_runtime.room_exists(room))
        };

        let mut root = self.to_gd().upcast::<Node2D>();
        match self.room_runtime.load_and_add_room(&mut root, spawn.room) {
            Some(mut room_node) => {
                self.finalize_room_load(&room_node, spawn.room);
                if self.player_runtime.spawn_into_room(
                    &mut room_node,
                    spawn.position,
                    &self.to_gd(),
                ) {
                    godot_print!("[RoomManager] spawned player at {:?}", spawn.position);
                }
                self.room_runtime.set_current_room(room_node);
            }
            None => {
                godot_error!("Failed to load room at {:?}", spawn.room);
            }
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.player_runtime.tick_collision_restore();
        self.check_room_transitions();
    }
}

#[godot_api]
impl GameRoomManager {
    fn finalize_room_load(&mut self, room: &Gd<Node2D>, room_id: RoomId) {
        self.room_session.complete_transition(room_id);
        save::mark_room_explored(room_id);
        self.connect_portal_signals(room);
        godot_print!("[RoomManager] active room set to {:?}", room_id);
    }

    fn check_room_transitions(&mut self) {
        let Some(mut player) = self.player_runtime.take_player() else {
            return;
        };

        let plan = {
            let room_runtime = &mut self.room_runtime;
            self.room_session.plan_boundary_transition(
                &self.boundary_detector,
                player.get_global_position(),
                player.get_velocity(),
                |room| room_runtime.room_exists(room),
            )
        };

        if let Some(plan) = plan {
            self.execute_room_transition(&mut player, plan);
        }

        self.player_runtime.store_player(player);
    }

    fn execute_room_transition(
        &mut self,
        player: &mut Gd<CharacterBody2D>,
        plan: RoomTransitionPlan,
    ) {
        godot_print!(
            "[RoomManager] transitioning from {:?} to {:?}",
            plan.from_room,
            plan.to_room
        );

        if let Some(mut parent) = player.get_parent() {
            parent.remove_child(&*player);
        }

        self.player_runtime.disable_collision_for_transition(player);
        self.player_runtime.reset_for_room_transition(player);

        let mut root = self.to_gd().upcast::<Node2D>();
        self.room_runtime.unload_current_room(&mut root);

        match self.room_runtime.load_and_add_room(&mut root, plan.to_room) {
            Some(mut new_room) => {
                let spawn_pos = self.resolve_transition_spawn(&new_room, plan.spawn);
                player.set_position(spawn_pos);
                new_room.add_child(&*player);

                self.finalize_room_load(&new_room, plan.to_room);
                self.room_runtime.set_current_room(new_room);

                godot_print!(
                    "[RoomManager] room transition complete to {:?} at {:?}",
                    plan.to_room,
                    spawn_pos
                );
            }
            None => {
                godot_error!("Failed to load target room {:?}", plan.to_room);
                root.add_child(&*player);
            }
        }
    }

    fn resolve_transition_spawn(&self, room: &Gd<Node2D>, spawn: TransitionSpawn) -> Vector2 {
        match spawn {
            TransitionSpawn::Position(pos) => pos,
            TransitionSpawn::AtPortal => find_portal_in_room(room, ENTITY_LAYER_NAME)
                .map(|portal| portal.bind().get_spawn_position())
                .unwrap_or(DEFAULT_SPAWN_POS),
        }
    }

    fn connect_portal_signals(&mut self, room: &Gd<Node2D>) {
        let room_manager = self.to_gd();
        connect_room_portal(
            room,
            ENTITY_LAYER_NAME,
            &room_manager,
            Self::on_portal_teleport_requested,
        );
    }

    #[func]
    fn on_portal_teleport_requested(&mut self, destination_room: Vector2i) {
        let target = (destination_room.x, destination_room.y);
        let plan = {
            let room_runtime = &mut self.room_runtime;
            self.room_session
                .plan_portal_transition(target, |room| room_runtime.room_exists(room))
        };
        let Some(plan) = plan else {
            godot_error!("Portal destination room {:?} does not exist", target);
            return;
        };

        let Some(mut player) = self.player_runtime.take_player() else {
            return;
        };

        self.execute_room_transition(&mut player, plan);
        player.set_velocity(Vector2::ZERO);
        self.player_runtime.store_player(player);
    }

    #[func]
    pub(crate) fn on_player_death_finished(&mut self) {
        match self
            .room_session
            .plan_death(save::has_save(DEFAULT_SAVE_SLOT))
        {
            DeathPlan::ReloadCheckpoint => {
                let _queued = save::queue_load(DEFAULT_SAVE_SLOT);
                godot_print!("[RoomManager] player death - respawn at checkpoint");
            }
            DeathPlan::RestartGame => {
                save::reset_all();
                godot_print!("[RoomManager] player death - restarting");
            }
        }

        let mut tree = self.base().get_tree();
        let _result = tree.change_scene_to_file("res://game.tscn");
    }

    #[func]
    fn get_current_room(&self) -> Vector2i {
        self.current_room_vector()
    }

    pub(crate) fn current_room_vector(&self) -> Vector2i {
        let (x, y) = self.room_session.current_room();
        Vector2i::new(x, y)
    }
}
