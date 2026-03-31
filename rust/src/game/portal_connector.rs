//! Portal signal connection utilities.
//! Handles finding and connecting portal entities in rooms.

use godot::meta::conv::UniformObjectDeref;
use godot::prelude::*;

use crate::entity::portal::TeleportPortal;

pub fn find_portal_in_room(room: &Gd<Node2D>, entity_layer: &str) -> Option<Gd<TeleportPortal>> {
    let entities = room.get_node_or_null(entity_layer)?;
    for child in entities.get_children().iter_shared() {
        if let Ok(portal) = child.try_cast::<TeleportPortal>() {
            return Some(portal);
        }
    }
    None
}

pub fn connect_portal_signal<T, Declarer>(
    portal: &Gd<TeleportPortal>,
    target: &Gd<T>,
    method: fn(&mut T, Vector2i),
) where
    T: UniformObjectDeref<Declarer>,
{
    portal
        .signals()
        .teleport_requested()
        .connect_other(target, method);
    godot_print!("[PortalConnector] connected portal teleport signal");
}

/// Returns true if a portal was found and connected.
pub fn connect_room_portal<T, Declarer>(
    room: &Gd<Node2D>,
    entity_layer: &str,
    target: &Gd<T>,
    method: fn(&mut T, Vector2i),
) -> bool
where
    T: UniformObjectDeref<Declarer>,
{
    if let Some(portal) = find_portal_in_room(room, entity_layer) {
        connect_portal_signal(&portal, target, method);
        true
    } else {
        false
    }
}
