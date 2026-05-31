use godot::{classes::Node2D, prelude::*};

use crate::entity::ladder::Ladder;

const LADDER_GROUP: &str = "ladder";
const PLAYER_HALF_HEIGHT_PX: f32 = 12.0;

pub fn is_touching_ladder(player: &Gd<Node2D>) -> bool {
    let tree = player.get_tree();
    let ladders = tree.get_nodes_in_group(LADDER_GROUP);
    let player_position = player.get_global_position();

    for node in ladders.iter_shared() {
        let Ok(ladder) = node.try_cast::<Ladder>() else {
            continue;
        };

        let ladder_position = ladder.clone().upcast::<Node2D>().get_global_position();
        let ladder_size = ladder.bind().climb_size();
        if player_overlaps_ladder(player_position, ladder_position, ladder_size) {
            return true;
        }
    }

    false
}

pub fn player_overlaps_ladder(
    player_position: Vector2,
    ladder_position: Vector2,
    ladder_size: Vector2,
) -> bool {
    let ladder_half_width = ladder_size.x * 0.5;
    let ladder_half_height = ladder_size.y * 0.5;

    (player_position.x - ladder_position.x).abs() <= ladder_half_width
        && ranges_overlap(
            player_position.y - PLAYER_HALF_HEIGHT_PX,
            player_position.y + PLAYER_HALF_HEIGHT_PX,
            ladder_position.y - ladder_half_height,
            ladder_position.y + ladder_half_height,
        )
}

fn ranges_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> bool {
    a_min <= b_max && a_max >= b_min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_player_body_overlapping_ladder_bounds() {
        assert!(player_overlaps_ladder(
            Vector2::new(0.0, 30.0),
            Vector2::ZERO,
            Vector2::new(16.0, 64.0),
        ));
    }

    #[test]
    fn rejects_player_center_outside_ladder_width() {
        assert!(!player_overlaps_ladder(
            Vector2::new(9.0, 0.0),
            Vector2::ZERO,
            Vector2::new(16.0, 64.0),
        ));
    }

    #[test]
    fn allows_player_center_inside_ladder_width() {
        assert!(player_overlaps_ladder(
            Vector2::new(8.0, 0.0),
            Vector2::ZERO,
            Vector2::new(16.0, 64.0),
        ));
    }

    #[test]
    fn rejects_player_after_leaving_ladder() {
        assert!(!player_overlaps_ladder(
            Vector2::new(9.0, 0.0),
            Vector2::ZERO,
            Vector2::new(16.0, 64.0),
        ));
        assert!(!player_overlaps_ladder(
            Vector2::new(0.0, 45.0),
            Vector2::ZERO,
            Vector2::new(16.0, 64.0),
        ));
    }
}
