use godot::classes::{AudioStreamPlayer, Input};
use godot::prelude::*;

mod player_spawner;
mod portal_connector;
pub mod room_manager;
mod spawn_resolver;

pub use player_spawner::PlayerSpawner;
pub use portal_connector::{connect_room_portal, find_portal_in_room};
pub use spawn_resolver::SpawnResolver;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Game {
    base: Base<Node>,
    bgm_player: OnReady<Gd<AudioStreamPlayer>>,
    bgm_enabled: bool,
}

#[godot_api]
impl INode for Game {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            bgm_player: OnReady::from_node("AudioStreamPlayer"),
            bgm_enabled: false,
        }
    }

    fn ready(&mut self) {
        godot_print!("[Game] ready");

        self.base_mut()
            .set_process_mode(godot::classes::node::ProcessMode::ALWAYS);
        self.base_mut().set_process(true);

        // Default: BGM off unless player toggles it on.
        self.bgm_player.stop();
        self.bgm_enabled = false;
    }

    fn process(&mut self, _delta: f64) {
        let input = Input::singleton();
        if input.is_action_just_pressed("ui_bgm_toggle") {
            self.toggle_bgm();
        }
    }
}

#[godot_api]
impl Game {
    fn toggle_bgm(&mut self) {
        self.bgm_enabled = !self.bgm_enabled;
        if self.bgm_enabled {
            self.bgm_player.play();
        } else {
            self.bgm_player.stop();
        }
        godot_print!("[Game] bgm_enabled: {}", self.bgm_enabled);
    }
}
