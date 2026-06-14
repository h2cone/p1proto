use godot::prelude::*;

use super::progress;

pub type RoomId = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomSize {
    pub width: f32,
    pub height: f32,
}

impl RoomSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn vector(self) -> Vector2 {
        Vector2::new(self.width, self.height)
    }
}

pub const ROOM_WIDTH: f32 = 320.0;
pub const ROOM_HEIGHT: f32 = 240.0;
pub const DEFAULT_ROOM_SIZE: RoomSize = RoomSize::new(ROOM_WIDTH, ROOM_HEIGHT);
pub const PLAYER_WIDTH: f32 = 16.0;
pub const PLAYER_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnPoint {
    pub room: RoomId,
    pub position: Vector2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionCheck {
    pub target_room: RoomId,
    pub new_position: Vector2,
}

pub struct SpawnResolver {
    initial_room: RoomId,
    initial_position: Vector2,
}

impl SpawnResolver {
    pub fn new(initial_room: RoomId, initial_position: Vector2) -> Self {
        Self {
            initial_room,
            initial_position,
        }
    }

    pub fn resolve(&self, mut room_exists: impl FnMut(RoomId) -> bool) -> SpawnPoint {
        if let Some(snapshot) = progress::take_pending_load()
            && room_exists(snapshot.room)
        {
            return SpawnPoint {
                room: snapshot.room,
                position: snapshot.position,
            };
        }

        SpawnPoint {
            room: self.initial_room,
            position: self.initial_position,
        }
    }
}

pub struct BoundaryDetector {
    pub cross_threshold: f32,
    room_size: RoomSize,
}

impl BoundaryDetector {
    pub fn new(cross_threshold: f32) -> Self {
        Self::with_room_size(cross_threshold, DEFAULT_ROOM_SIZE)
    }

    pub fn with_room_size(cross_threshold: f32, room_size: RoomSize) -> Self {
        Self {
            cross_threshold,
            room_size,
        }
    }

    pub fn check_transition(
        &self,
        player_pos: Vector2,
        player_velocity: Vector2,
        current_room: RoomId,
    ) -> Option<TransitionCheck> {
        let half_width = PLAYER_WIDTH * 0.5;
        let half_height = PLAYER_HEIGHT * 0.5;

        if player_velocity.x < 0.0 {
            let player_left = player_pos.x - half_width;
            let overflow = -player_left;
            if self.should_trigger(overflow, PLAYER_WIDTH) {
                return Some(TransitionCheck {
                    target_room: (current_room.0 - 1, current_room.1),
                    new_position: Vector2::new(player_pos.x + self.room_size.width, player_pos.y),
                });
            }
        } else if player_velocity.x > 0.0 {
            let player_right = player_pos.x + half_width;
            let overflow = player_right - self.room_size.width;
            if self.should_trigger(overflow, PLAYER_WIDTH) {
                return Some(TransitionCheck {
                    target_room: (current_room.0 + 1, current_room.1),
                    new_position: Vector2::new(player_pos.x - self.room_size.width, player_pos.y),
                });
            }
        }

        if player_velocity.y < 0.0 {
            let player_top = player_pos.y - half_height;
            let overflow = -player_top;
            if self.should_trigger(overflow, PLAYER_HEIGHT) {
                return Some(TransitionCheck {
                    target_room: (current_room.0, current_room.1 - 1),
                    new_position: Vector2::new(player_pos.x, player_pos.y + self.room_size.height),
                });
            }
        } else if player_velocity.y > 0.0 {
            let player_bottom = player_pos.y + half_height;
            let overflow = player_bottom - self.room_size.height;
            if self.should_trigger(overflow, PLAYER_HEIGHT) {
                return Some(TransitionCheck {
                    target_room: (current_room.0, current_room.1 + 1),
                    new_position: Vector2::new(player_pos.x, player_pos.y - self.room_size.height),
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
    use crate::core::progress;

    #[test]
    fn falls_back_to_initial_spawn_when_saved_room_is_missing() {
        progress::reset_all();
        progress::save_checkpoint(progress::DEFAULT_SAVE_SLOT, (9, 9), Vector2::new(1.0, 2.0));
        assert!(progress::queue_load(progress::DEFAULT_SAVE_SLOT));

        let resolver = SpawnResolver::new((0, 1), Vector2::new(64.0, 64.0));
        let spawn = resolver.resolve(|room| room != (9, 9));

        assert_eq!(spawn.room, (0, 1));
        assert_eq!(spawn.position, Vector2::new(64.0, 64.0));
    }

    #[test]
    fn no_transition_when_not_at_boundary() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(ROOM_WIDTH * 0.5, ROOM_HEIGHT * 0.5),
            Vector2::new(10.0, 0.0),
            (0, 1),
        );
        assert!(result.is_none());
    }

    #[test]
    fn transition_right_when_threshold_exceeded() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(ROOM_WIDTH, 180.0),
            Vector2::new(10.0, 0.0),
            (0, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (1, 1));
    }

    #[test]
    fn transition_uses_configured_room_size() {
        let detector = BoundaryDetector::with_room_size(0.5, RoomSize::new(480.0, 360.0));
        let result =
            detector.check_transition(Vector2::new(480.0, 180.0), Vector2::new(10.0, 0.0), (0, 1));

        assert_eq!(
            result,
            Some(TransitionCheck {
                target_room: (1, 1),
                new_position: Vector2::new(0.0, 180.0),
            })
        );
    }

    #[test]
    fn no_transition_when_moving_wrong_direction() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(ROOM_WIDTH, 180.0),
            Vector2::new(-10.0, 0.0),
            (0, 1),
        );
        assert!(result.is_none());
    }

    #[test]
    fn transition_left() {
        let detector = BoundaryDetector::new(0.5);
        let result =
            detector.check_transition(Vector2::new(-8.0, 90.0), Vector2::new(-10.0, 0.0), (1, 1));
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 1));
    }

    #[test]
    fn transition_down() {
        let detector = BoundaryDetector::new(0.5);
        let result = detector.check_transition(
            Vector2::new(ROOM_WIDTH * 0.5, ROOM_HEIGHT + PLAYER_HEIGHT * 0.5),
            Vector2::new(0.0, 10.0),
            (0, 1),
        );
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 2));
    }

    #[test]
    fn transition_up() {
        let detector = BoundaryDetector::new(0.5);
        let result =
            detector.check_transition(Vector2::new(240.0, -12.0), Vector2::new(0.0, -10.0), (0, 1));
        assert!(result.is_some());
        let check = result.unwrap();
        assert_eq!(check.target_room, (0, 0));
    }
}
