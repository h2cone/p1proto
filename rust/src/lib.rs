use godot::prelude::*;

mod core;
mod entity;
mod game;
mod player;
mod rooms;
mod save;
mod ui;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
