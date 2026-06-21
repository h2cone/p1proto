use godot::classes::{Area2D, IStaticBody2D, Node, StaticBody2D};
use godot::prelude::*;

use crate::core::progress::PersistentEntityKind;

use super::persistence::{PLAIN_KEY_GROUP, PersistentEntityRef};
use super::plain_key::PlainKey;

#[derive(GodotClass)]
#[class(base=StaticBody2D)]
pub struct PlainLock {
    #[base]
    base: Base<StaticBody2D>,
    detect_area: OnReady<Gd<Area2D>>,
    #[export]
    room_coords: Vector2i,
    persistent_entity: Option<PersistentEntityRef>,
}

#[godot_api]
impl IStaticBody2D for PlainLock {
    fn init(base: Base<StaticBody2D>) -> Self {
        Self {
            base,
            detect_area: OnReady::from_node("DetectArea"),
            room_coords: Vector2i::ZERO,
            persistent_entity: None,
        }
    }

    fn ready(&mut self) {
        let pos = self.base().get_global_position();
        let node = self.to_gd().upcast::<Node>();
        godot_print!(
            "[PlainLock] ready at {:?}, room {:?}",
            pos,
            self.room_coords
        );

        let persistent_entity = PersistentEntityRef::new(&node, self.room_coords, pos);
        if persistent_entity.is_marked(PersistentEntityKind::Lock) {
            godot_print!("[PlainLock] already unlocked, queue_free");
            self.base_mut().queue_free();
            return;
        }
        self.persistent_entity = Some(persistent_entity);

        let plain_lock = self.to_gd();
        self.detect_area
            .signals()
            .body_entered()
            .connect_other(&plain_lock, Self::on_body_entered);
    }
}

#[godot_api]
impl PlainLock {
    #[signal]
    pub(crate) fn lock_unlocked(room_coords: Vector2i, position: Vector2);

    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        if let Some(mut key) = self.find_collected_key() {
            self.unlock(&mut key);
        }
    }

    fn find_collected_key(&self) -> Option<Gd<PlainKey>> {
        let tree = self.base().get_tree();
        let keys = tree.get_nodes_in_group(PLAIN_KEY_GROUP);

        for node in keys.iter_shared() {
            if let Ok(key) = node.try_cast::<PlainKey>()
                && key.bind().is_collected()
            {
                return Some(key);
            }
        }
        None
    }

    fn unlock(&mut self, key: &mut Gd<PlainKey>) {
        let room_coords = self.room_coords;
        let pos = self.base().get_global_position();
        let persistent_entity = self.persistent_entity(pos);

        self.signals().lock_unlocked().emit(room_coords, pos);
        let _marked = persistent_entity.mark(PersistentEntityKind::Lock);
        key.bind_mut().consume();
        self.base_mut().queue_free();
    }

    fn persistent_entity(&self, position: Vector2) -> PersistentEntityRef {
        self.persistent_entity.clone().unwrap_or_else(|| {
            PersistentEntityRef::new(&self.to_gd().upcast::<Node>(), self.room_coords, position)
        })
    }
}
