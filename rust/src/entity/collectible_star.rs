use godot::classes::{AnimatedSprite2D, Area2D, IArea2D, Node};
use godot::prelude::*;

use crate::core::progress::PersistentEntityKind;

use super::persistence::{has_persistent_entity, mark_persistent_entity};

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct CollectibleStar {
    #[base]
    base: Base<Area2D>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    #[export]
    room_coords: Vector2i,
    original_position: Vector2,
}

#[godot_api]
impl IArea2D for CollectibleStar {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            room_coords: Vector2i::ZERO,
            original_position: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.original_position = self.base().get_global_position();
        let node = self.to_gd().upcast::<Node>();

        if has_persistent_entity(
            PersistentEntityKind::Star,
            &node,
            self.room_coords,
            self.original_position,
        ) {
            self.base_mut().queue_free();
            return;
        }

        self.sprite.play();
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}

#[godot_api]
impl CollectibleStar {
    #[signal]
    pub fn star_collected(room_coords: Vector2i, position: Vector2);

    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        let room_coords = self.room_coords;
        let position = self.original_position;
        let node = self.to_gd().upcast::<Node>();

        self.signals().star_collected().emit(room_coords, position);
        let _marked =
            mark_persistent_entity(PersistentEntityKind::Star, &node, room_coords, position);
        self.base_mut().queue_free();
    }
}
