use godot::{classes::CharacterBody2D, prelude::*};

const MAX_CORRECTION_PX: i32 = 3;
const SIDE_NORMAL_THRESHOLD: f32 = 0.7;
const INTENT_EPSILON: f32 = 0.01;
const UPWARD_CLEARANCE_PX: f32 = -1.0;

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
    let transform = body.get_global_transform();
    for px in 1..=max_px {
        for direction in directions {
            let offset = Vector2::new(direction * px as f32, 0.0);
            if can_apply_offset(body, transform, offset) {
                let position = body.get_global_position();
                godot_print!("[Player] corner correction applied offset={:?}", offset);
                body.set_global_position(position + offset);
                return;
            }
        }
    }
}

fn can_apply_offset(
    body: &mut Gd<CharacterBody2D>,
    transform: Transform2D,
    offset: Vector2,
) -> bool {
    let lateral_blocked = motion_collides(body, transform, offset);
    if lateral_blocked {
        return false;
    }

    let upward_probe = Vector2::new(0.0, UPWARD_CLEARANCE_PX);
    let upward_blocked = motion_collides(body, transform, upward_probe);
    let shifted_upward_blocked = motion_collides(body, transform.translated(offset), upward_probe);

    correction_candidate_is_corner(false, upward_blocked, shifted_upward_blocked)
}

fn motion_collides(
    body: &mut Gd<CharacterBody2D>,
    transform: Transform2D,
    motion: Vector2,
) -> bool {
    // gdext does not expose CharacterBody2D::test_move as a static Rust method.
    body.call("test_move", &[transform.to_variant(), motion.to_variant()])
        .to::<bool>()
}

fn correction_candidate_is_corner(
    lateral_blocked: bool,
    upward_blocked: bool,
    shifted_upward_blocked: bool,
) -> bool {
    !lateral_blocked && upward_blocked && !shifted_upward_blocked
}

fn correction_directions(horizontal_intent: f32, side_normal_x: Option<f32>) -> [f32; 2] {
    if horizontal_intent.abs() > INTENT_EPSILON {
        return ordered_pair(horizontal_intent.signum());
    }

    if let Some(normal_x) = side_normal_x
        && normal_x.abs() > INTENT_EPSILON
    {
        return ordered_pair(-normal_x.signum());
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

    #[test]
    fn candidate_requires_lateral_space_and_upward_clearance() {
        assert!(correction_candidate_is_corner(false, true, false));
    }

    #[test]
    fn candidate_rejects_flat_ceiling_that_remains_blocked_after_shift() {
        assert!(!correction_candidate_is_corner(false, true, true));
    }

    #[test]
    fn candidate_rejects_unrelated_side_slide_without_upward_block() {
        assert!(!correction_candidate_is_corner(false, false, false));
    }

    #[test]
    fn candidate_rejects_blocked_lateral_offset() {
        assert!(!correction_candidate_is_corner(true, true, false));
    }
}
