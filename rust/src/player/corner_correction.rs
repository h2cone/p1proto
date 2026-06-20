use godot::{classes::CharacterBody2D, prelude::*};

const MAX_CORRECTION_PX: i32 = 3;
const SIDE_NORMAL_THRESHOLD: f32 = 0.7;
const INTENT_EPSILON: f32 = 0.01;

pub fn apply_after_slide(
    body: &mut Gd<CharacterBody2D>,
    attempted_velocity: Vector2,
    horizontal_intent: f32,
) {
    if attempted_velocity.y >= 0.0 || body.get_collision_mask() == 0 {
        return;
    }

    let collision_context = collision_context(body);
    if !body.is_on_ceiling() && !collision_context.has_side_collision {
        return;
    }

    try_offsets(
        body,
        correction_directions(horizontal_intent, collision_context.side_normal_x),
        MAX_CORRECTION_PX,
    );
}

#[derive(Default)]
struct CollisionContext {
    has_side_collision: bool,
    side_normal_x: Option<f32>,
}

fn collision_context(body: &mut Gd<CharacterBody2D>) -> CollisionContext {
    let collision_count = body.get_slide_collision_count();
    let mut context = CollisionContext::default();

    for index in 0..collision_count {
        let Some(collision) = body.get_slide_collision(index) else {
            continue;
        };

        let normal = collision.get_normal();
        if normal.x.abs() < SIDE_NORMAL_THRESHOLD {
            continue;
        }

        context.has_side_collision = true;
        context.side_normal_x.get_or_insert(normal.x);
    }

    context
}

fn try_offsets(body: &mut Gd<CharacterBody2D>, directions: [f32; 2], max_px: i32) {
    for px in 1..=max_px {
        for direction in directions {
            let offset = Vector2::new(direction * px as f32, 0.0);
            if can_apply_offset(body, offset) {
                let position = body.get_global_position();
                godot_print!("[Player] corner correction applied offset={:?}", offset);
                body.set_global_position(position + offset);
                return;
            }
        }
    }
}

fn can_apply_offset(body: &mut Gd<CharacterBody2D>, offset: Vector2) -> bool {
    let transform = body.get_global_transform();
    !body
        .call("test_move", &[transform.to_variant(), offset.to_variant()])
        .to::<bool>()
}

fn correction_directions(horizontal_intent: f32, side_normal_x: Option<f32>) -> [f32; 2] {
    if horizontal_intent.abs() > INTENT_EPSILON {
        return ordered_pair(horizontal_intent.signum());
    }

    if let Some(normal_x) = side_normal_x {
        if normal_x.abs() > INTENT_EPSILON {
            return ordered_pair(-normal_x.signum());
        }
    }

    [-1.0, 1.0]
}

fn ordered_pair(preferred: f32) -> [f32; 2] {
    if preferred < 0.0 {
        [-1.0, 1.0]
    } else {
        [1.0, -1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_prefers_horizontal_intent() {
        assert_eq!(correction_directions(-1.0, Some(1.0)), [-1.0, 1.0]);
        assert_eq!(correction_directions(1.0, Some(-1.0)), [1.0, -1.0]);
    }

    #[test]
    fn direction_falls_back_to_opposite_side_normal() {
        assert_eq!(correction_directions(0.0, Some(1.0)), [-1.0, 1.0]);
        assert_eq!(correction_directions(0.0, Some(-1.0)), [1.0, -1.0]);
    }

    #[test]
    fn direction_uses_left_then_right_without_signal() {
        assert_eq!(correction_directions(0.0, None), [-1.0, 1.0]);
    }
}
