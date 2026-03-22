use godot::classes::{AnimatedSprite2D, Area2D, IArea2D, Node};
use godot::prelude::*;

use super::persistence::{find_saved_checkpoint, save_checkpoint};

const POSITION_MATCH_EPSILON: f32 = 1.0;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Checkpoint {
    #[base]
    base: Base<Area2D>,
    activated: bool,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    #[export]
    room_coords: Vector2i,
}

#[godot_api]
impl IArea2D for Checkpoint {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            activated: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            room_coords: Vector2i::ZERO,
        }
    }

    fn ready(&mut self) {
        self.sprite.set_animation("unchecked");
        self.sprite.stop();
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
        self.restore_if_saved();
    }
}

#[godot_api]
impl Checkpoint {
    #[signal]
    pub(crate) fn checkpoint_activated(room_coords: Vector2i, position: Vector2);

    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        if self.activated {
            return;
        }
        self.activate();
    }

    #[func]
    fn activate(&mut self) {
        if self.activated {
            return;
        }

        self.activated = true;
        self.sprite.set_animation("checked");
        self.sprite.play();

        let room_coords = self.room_coords;
        let position = self.base().get_global_position();
        let node = self.to_gd().upcast::<Node>();
        self.signals()
            .checkpoint_activated()
            .emit(room_coords, position);
        let _snapshot = save_checkpoint(&node, room_coords, position);
    }

    #[func]
    fn is_activated(&self) -> bool {
        self.activated
    }

    #[func]
    fn reset(&mut self) {
        self.activated = false;
        self.sprite.set_animation("unchecked");
        self.sprite.stop();
    }

    fn restore_if_saved(&mut self) {
        let checkpoint_position = self.base().get_global_position();
        let node = self.to_gd().upcast::<Node>();
        if let Some(snapshot) = find_saved_checkpoint(
            &node,
            self.room_coords,
            checkpoint_position,
            POSITION_MATCH_EPSILON,
        ) {
            self.activated = true;
            self.sprite.set_animation("checked");
            self.sprite.play();
            godot_print!(
                "[Checkpoint] restored from saved slot at room {:?}, position {:?}",
                snapshot.room,
                snapshot.position
            );
        }
    }
}
