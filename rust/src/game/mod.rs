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
}

#[godot_api]
impl INode for Game {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("[Game] ready")
    }
}
