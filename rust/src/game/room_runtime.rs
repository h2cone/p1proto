use godot::classes::CharacterBody2D;
use godot::prelude::*;

use crate::core::world::RoomId;
use crate::player::Player;
use crate::rooms::RoomLoader;

use super::player_spawner::PlayerSpawner;
use super::room_manager::GameRoomManager;

pub(crate) struct CollisionRestore {
    layer: u32,
    mask: u32,
    frames_remaining: u8,
}

pub(crate) struct RoomRuntime {
    loader: RoomLoader,
    current_room_node: Option<Gd<Node2D>>,
}

impl RoomRuntime {
    pub(crate) fn new(scene_path_pattern: &str) -> Self {
        Self {
            loader: RoomLoader::new(scene_path_pattern.to_string()),
            current_room_node: None,
        }
    }

    pub(crate) fn room_exists(&mut self, room: RoomId) -> bool {
        self.loader.room_exists(room)
    }

    pub(crate) fn load_and_add_room(
        &mut self,
        owner: &mut Gd<Node2D>,
        room: RoomId,
    ) -> Option<Gd<Node2D>> {
        let room_node = self.loader.instantiate_room(room)?;
        owner.add_child(&room_node);
        Some(room_node)
    }

    pub(crate) fn set_current_room(&mut self, room: Gd<Node2D>) {
        self.current_room_node = Some(room);
    }

    pub(crate) fn unload_current_room(&mut self, owner: &mut Gd<Node2D>) {
        if let Some(mut old_room) = self.current_room_node.take() {
            owner.remove_child(&old_room);
            old_room.queue_free();
        }
    }
}

pub(crate) struct PlayerRuntime {
    spawner: PlayerSpawner,
    player: Option<Gd<CharacterBody2D>>,
    pending_collision_restore: Option<CollisionRestore>,
}

impl PlayerRuntime {
    pub(crate) fn new(scene_path: &str) -> Self {
        Self {
            spawner: PlayerSpawner::new(scene_path),
            player: None,
            pending_collision_restore: None,
        }
    }

    pub(crate) fn spawn_into_room(
        &mut self,
        room: &mut Gd<Node2D>,
        spawn_pos: Vector2,
        room_manager: &Gd<GameRoomManager>,
    ) -> bool {
        let Some(mut player) = self.spawner.spawn() else {
            godot_error!("Failed to load player scene for spawn at {:?}", spawn_pos);
            return false;
        };

        player.set_global_position(spawn_pos);
        room.add_child(&player);
        self.connect_death_signal(&player, room_manager);
        self.player = Some(player);
        true
    }

    pub(crate) fn take_player(&mut self) -> Option<Gd<CharacterBody2D>> {
        self.player.take()
    }

    pub(crate) fn store_player(&mut self, player: Gd<CharacterBody2D>) {
        self.player = Some(player);
    }

    pub(crate) fn disable_collision_for_transition(&mut self, player: &mut Gd<CharacterBody2D>) {
        if let Some(state) = &mut self.pending_collision_restore {
            state.frames_remaining = 1;
            return;
        }

        let layer = player.get_collision_layer();
        let mask = player.get_collision_mask();
        player.set_collision_layer(0);
        player.set_collision_mask(0);
        self.pending_collision_restore = Some(CollisionRestore {
            layer,
            mask,
            frames_remaining: 1,
        });
    }

    pub(crate) fn reset_for_room_transition(&self, player: &mut Gd<CharacterBody2D>) {
        let Ok(mut player_script) = player.clone().try_cast::<Player>() else {
            godot_warn!("[RoomManager] player script not found - transition state not reset");
            return;
        };

        player_script.bind_mut().reset_for_room_transition();
    }

    pub(crate) fn tick_collision_restore(&mut self) {
        let Some(state) = &mut self.pending_collision_restore else {
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
        self.pending_collision_restore = None;
    }

    fn connect_death_signal(
        &self,
        player: &Gd<CharacterBody2D>,
        room_manager: &Gd<GameRoomManager>,
    ) {
        let Ok(player_script) = player.clone().try_cast::<Player>() else {
            godot_warn!("[RoomManager] player script not found - death signal not connected");
            return;
        };

        player_script
            .signals()
            .death_finished()
            .connect_other(room_manager, GameRoomManager::on_player_death_finished);
    }
}
