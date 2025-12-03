use godot::prelude::*;

mod game;
mod player;
mod rooms;
mod world;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
