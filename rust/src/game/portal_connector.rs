//! Portal signal connection utilities.
//! Handles finding and connecting portal entities in rooms.

use godot::prelude::*;

/// Find Portal node in room's Entities layer.
pub fn find_portal_in_room(room: &Gd<Node2D>, entity_layer: &str) -> Option<Gd<Node2D>> {
    let entities = room.get_node_or_null(entity_layer)?;
    for child in entities.get_children().iter_shared() {
        if child.is_class("Portal") {
            return Some(child.cast::<Node2D>());
        }
    }
    None
}

/// Connect portal's teleport_requested signal to a target node's method.
pub fn connect_portal_signal(portal: &Gd<Node2D>, target: &Gd<Node>, method: &str) {
    let callable = target.callable(method);
    portal
        .clone()
        .upcast::<Node>()
        .connect("teleport_requested", &callable);
    godot_print!("[PortalConnector] connected portal teleport signal");
}

/// Find and connect portal in a room to a handler.
/// Returns true if a portal was found and connected.
pub fn connect_room_portal(
    room: &Gd<Node2D>,
    entity_layer: &str,
    target: &Gd<Node>,
    method: &str,
) -> bool {
    if let Some(portal) = find_portal_in_room(room, entity_layer) {
        connect_portal_signal(&portal, target, method);
        true
    } else {
        false
    }
}
