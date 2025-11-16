use godot::prelude::*;

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
        godot_print!("Game ready")
    }
}
