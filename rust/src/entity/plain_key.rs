use godot::classes::{AnimatedSprite2D, Area2D, IArea2D, Node};
use godot::prelude::*;

use crate::core::progress::PersistentEntityKind;

use super::persistence::{PLAIN_KEY_GROUP, has_persistent_entity, mark_persistent_entity};

const FOLLOW_OFFSET: Vector2 = Vector2::new(0.0, -20.0);

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct PlainKey {
    #[base]
    base: Base<Area2D>,
    collected: bool,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    follow_target: Option<Gd<Node2D>>,
    pending_reparent: bool,
    #[export]
    room_coords: Vector2i,
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
            pending_reparent: false,
            room_coords: Vector2i::ZERO,
            original_position: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.original_position = self.base().get_global_position();
        let node = self.to_gd().upcast::<Node>();
        godot_print!(
            "[PlainKey] ready at {:?}, room {:?}",
            self.original_position,
            self.room_coords
        );

        if has_persistent_entity(
            PersistentEntityKind::Key,
            &node,
            self.room_coords,
            self.original_position,
        ) {
            godot_print!("[PlainKey] already collected, queue_free");
            self.base_mut().queue_free();
            return;
        }

        self.sprite.play();
        self.base_mut().add_to_group(PLAIN_KEY_GROUP);
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }

    fn process(&mut self, _delta: f64) {
        let Some(target) = &self.follow_target else {
            return;
        };

        if self.pending_reparent {
            self.pending_reparent = false;
            if let Some(mut old_parent) = self.base().get_parent() {
                old_parent.remove_child(&self.to_gd());
            }
            target.clone().add_child(&self.to_gd());
        }

        let target_pos = target.get_global_position();
        self.base_mut()
            .set_global_position(target_pos + FOLLOW_OFFSET);
    }
}

#[godot_api]
impl PlainKey {
    #[signal]
    pub(crate) fn key_collected(room_coords: Vector2i, position: Vector2);

    #[signal]
    pub(crate) fn key_used();

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
        self.pending_reparent = true;

        let room_coords = self.room_coords;
        let original_position = self.original_position;
        let node = self.to_gd().upcast::<Node>();
        self.signals()
            .key_collected()
            .emit(room_coords, original_position);
        let _marked = mark_persistent_entity(
            PersistentEntityKind::Key,
            &node,
            room_coords,
            original_position,
        );

        self.base_mut()
            .set_deferred("monitoring", &false.to_variant());
    }

    #[func]
    pub fn consume(&mut self) {
        self.signals().key_used().emit();
        self.base_mut().queue_free();
    }

    #[func]
    pub fn is_collected(&self) -> bool {
        self.collected
    }
}
