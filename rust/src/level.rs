use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Level {
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Level {
    fn init(base: Base<Node2D>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("Level ready");
    }
}
