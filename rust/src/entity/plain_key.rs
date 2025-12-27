use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

use crate::save;

/// Offset position for the key when following the player (above head)
const FOLLOW_OFFSET: Vector2 = Vector2::new(0.0, -20.0);

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PlainKey {
    #[base]
    base: Base<Area2D>,

    collected: bool,

    sprite: OnReady<Gd<AnimatedSprite2D>>,

    follow_target: Option<Gd<Node2D>>,

    #[export]
    room_coords: Vector2i,

    /// Original position for save state identification
    original_position: Vector2,
}

#[godot_api]
impl IArea2D for PlainKey {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            collected: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            follow_target: None,
            room_coords: Vector2i::ZERO,
            original_position: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.original_position = self.base().get_global_position();

        // Check if already collected
        let room = (self.room_coords.x, self.room_coords.y);
        if save::is_key_collected(room, self.original_position) {
            self.base_mut().queue_free();
            return;
        }

        self.sprite.play();
        self.base_mut().add_to_group("plain_keys");

        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }

    fn process(&mut self, _delta: f64) {
        if let Some(target) = &self.follow_target {
            let target_pos = target.get_global_position();
            self.base_mut()
                .set_global_position(target_pos + FOLLOW_OFFSET);
        }
    }
}

#[godot_api]
impl PlainKey {
    #[signal]
    fn key_collected();

    #[signal]
    fn key_used();

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if self.collected {
            return;
        }
        self.collect(body);
    }

    fn collect(&mut self, body: Gd<Node2D>) {
        self.collected = true;
        self.follow_target = Some(body);
        godot_print!("Key collected!");

        // Save collected state
        let room = (self.room_coords.x, self.room_coords.y);
        save::mark_key_collected(room, self.original_position);

        self.signals().key_collected().emit();

        self.base_mut()
            .set_deferred("monitoring", &false.to_variant());
    }

    #[func]
    pub fn consume(&mut self) {
        godot_print!("Key used!");
        self.signals().key_used().emit();
        self.base_mut().queue_free();
    }

    #[func]
    pub fn is_collected(&self) -> bool {
        self.collected
    }
}
