use godot::{
    classes::{CollisionObject2D, KinematicCollision2D, Node, Object},
    prelude::*,
};

pub fn is_hazard_collision(
    collision: &Gd<KinematicCollision2D>,
    hazard_layer: i32,
    tilemap_prefixes: &[&str],
) -> bool {
    let Some(mut collider) = collision.get_collider() else {
        return false;
    };

    if let Ok(body) = collider.clone().try_cast::<CollisionObject2D>() {
        return body.get_collision_layer_value(hazard_layer);
    }

    if let Some(is_hazard) = reflected_hazard_layer(&mut collider, hazard_layer) {
        return is_hazard;
    }

    if let Ok(node) = collider.try_cast::<Node>() {
        return is_hazard_tilemap_node(&node, tilemap_prefixes)
            || node
                .get_parent()
                .map(|parent| is_hazard_tilemap_node(&parent, tilemap_prefixes))
                .unwrap_or(false);
    }

    false
}

fn reflected_hazard_layer(collider: &mut Gd<Object>, hazard_layer: i32) -> Option<bool> {
    // Godot can return TileMap-backed colliders whose concrete Rust wrapper does
    // not expose collision-layer methods through a static type here.
    if collider.has_method("get_collision_layer_value") {
        let layer = Variant::from(hazard_layer);
        return Some(
            collider
                .call("get_collision_layer_value", &[layer])
                .to::<bool>(),
        );
    }

    if collider.has_method("get_collision_layer") {
        let layer_bits = collider.call("get_collision_layer", &[]).to::<u32>();
        return Some((layer_bits & (1_u32 << (hazard_layer - 1))) != 0);
    }

    None
}

fn is_hazard_tilemap_node(node: &Gd<Node>, tilemap_prefixes: &[&str]) -> bool {
    let name = node.get_name().to_string();
    tilemap_prefixes
        .iter()
        .any(|prefix| name.starts_with(prefix))
}
