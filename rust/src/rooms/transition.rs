use godot::prelude::*;

/// Room dimensions (from LDtk level files)
pub const ROOM_WIDTH: f32 = 320.0;
pub const ROOM_HEIGHT: f32 = 240.0;

/// Player dimensions for collision calculations
pub const PLAYER_WIDTH: f32 = 16.0;
pub const PLAYER_HEIGHT: f32 = 24.0;

/// Result of boundary detection check
#[derive(Clone, Debug)]
pub struct TransitionCheck {
    pub target_room: (i32, i32),
    pub new_position: Vector2,
}

/// Boundary detector for room transitions
///
/// - Checks every frame in the game loop
/// - Uses player direction (velocity) to prevent backtracking
/// - Calculates target room coordinates based on current position
/// - Doesn't use hysteresis/lag thresholds (uses velocity direction instead)
pub struct BoundaryDetector {
    /// Threshold: how much of player body must cross before transition (0.0-1.0)
    pub cross_threshold: f32,
}

impl BoundaryDetector {
    pub fn new(cross_threshold: f32) -> Self {
        Self { cross_threshold }
    }

    /// Check if player should transition to adjacent room
    ///
    /// Arguments:
    /// - player_pos: Player's global position (center point)
    /// - player_velocity: Player's velocity vector
    /// - current_room: Current room grid coordinates (x, y)
    ///
    /// Returns: TransitionCheck with transition info, or None if no transition
    pub fn check_transition(
        &self,
        player_pos: Vector2,
        player_velocity: Vector2,
        current_room: (i32, i32),
    ) -> Option<TransitionCheck> {
        let half_width = PLAYER_WIDTH * 0.5;
        let half_height = PLAYER_HEIGHT * 0.5;

        // Check horizontal boundaries (left/right)
        if player_velocity.x < 0.0 {
            // Moving left
            let player_left = player_pos.x - half_width;
            let overflow = -player_left;
            if self.should_trigger(overflow, PLAYER_WIDTH) {
                let target_room = (current_room.0 - 1, current_room.1);
                let new_position = Vector2::new(player_pos.x + ROOM_WIDTH, player_pos.y);
                return Some(TransitionCheck {
                    target_room,
                    new_position,
                });
            }
        } else if player_velocity.x > 0.0 {
            // Moving right
            let player_right = player_pos.x + half_width;
            let overflow = player_right - ROOM_WIDTH;
            if self.should_trigger(overflow, PLAYER_WIDTH) {
                let target_room = (current_room.0 + 1, current_room.1);
                let new_position = Vector2::new(player_pos.x - ROOM_WIDTH, player_pos.y);
                return Some(TransitionCheck {
                    target_room,
                    new_position,
                });
            }
        }

        // Check vertical boundaries (up/down)
        if player_velocity.y < 0.0 {
            // Moving up
            let player_top = player_pos.y - half_height;
            let overflow = -player_top;
            if self.should_trigger(overflow, PLAYER_HEIGHT) {
                let target_room = (current_room.0, current_room.1 - 1);
                let new_position = Vector2::new(player_pos.x, player_pos.y + ROOM_HEIGHT);
                return Some(TransitionCheck {
                    target_room,
                    new_position,
                });
            }
        } else if player_velocity.y > 0.0 {
            // Moving down
            let player_bottom = player_pos.y + half_height;
            let overflow = player_bottom - ROOM_HEIGHT;
            if self.should_trigger(overflow, PLAYER_HEIGHT) {
                let target_room = (current_room.0, current_room.1 + 1);
                let new_position = Vector2::new(player_pos.x, player_pos.y - ROOM_HEIGHT);
                return Some(TransitionCheck {
                    target_room,
                    new_position,
                });
            }
        }

        None
    }

    fn should_trigger(&self, overflow: f32, player_extent: f32) -> bool {
        if overflow <= 0.0 {
            return false;
        }
        let ratio = overflow / player_extent;
        ratio >= self.cross_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_transition_when_not_at_boundary() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(160.0, 90.0), // Center of room
            Vector2::new(10.0, 0.0),   // Moving right
            (0, 1),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_transition_right_when_threshold_exceeded() {
        let detector = BoundaryDetector::new(0.5);
        // Player at right edge: 320 - 8 (half width) + 8 (50% overflow) = 320
        let result = detector.check_transition(
            Vector2::new(320.0, 90.0),
            Vector2::new(10.0, 0.0), // Moving right
            (0, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (1, 1));
    }

    #[test]
    fn test_no_transition_when_moving_wrong_direction() {
        let detector = BoundaryDetector::new(0.5);
        // At right boundary but moving left
        let result = detector.check_transition(
            Vector2::new(320.0, 90.0),
            Vector2::new(-10.0, 0.0), // Moving left
            (0, 1),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_transition_left() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(-8.0, 90.0), // 50% overflow on left
            Vector2::new(-10.0, 0.0), // Moving left
            (1, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 1));
    }

    #[test]
    fn test_transition_down() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(160.0, 240.0 + 12.0), // 50% overflow downward
            Vector2::new(0.0, 10.0),           // Moving down
            (0, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 2));
    }

    #[test]
    fn test_transition_up() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(160.0, -12.0), // 50% overflow upward
            Vector2::new(0.0, -10.0),   // Moving up
            (0, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 0));
    }
}
