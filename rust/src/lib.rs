use godot::prelude::*;

mod game;
mod level;
mod player;
mod player_movement;
mod world;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
