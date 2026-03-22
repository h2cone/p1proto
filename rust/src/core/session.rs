use godot::prelude::*;

use super::world::{BoundaryDetector, RoomId, SpawnPoint, SpawnResolver};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransitionSpawn {
    Position(Vector2),
    AtPortal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomTransitionPlan {
    pub from_room: RoomId,
    pub to_room: RoomId,
    pub spawn: TransitionSpawn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathPlan {
    ReloadCheckpoint,
    RestartGame,
}

pub struct RoomSession {
    current_room: RoomId,
}

impl RoomSession {
    pub fn new(initial_room: RoomId) -> Self {
        Self {
            current_room: initial_room,
        }
    }

    pub fn resolve_start(
        &mut self,
        resolver: &SpawnResolver,
        room_exists: impl FnMut(RoomId) -> bool,
    ) -> SpawnPoint {
        let spawn = resolver.resolve(room_exists);
        self.current_room = spawn.room;
        spawn
    }

    pub fn current_room(&self) -> RoomId {
        self.current_room
    }

    pub fn plan_boundary_transition(
        &self,
        detector: &BoundaryDetector,
        player_pos: Vector2,
        player_velocity: Vector2,
        mut room_exists: impl FnMut(RoomId) -> bool,
    ) -> Option<RoomTransitionPlan> {
        let transition =
            detector.check_transition(player_pos, player_velocity, self.current_room)?;
        if !room_exists(transition.target_room) {
            return None;
        }

        Some(RoomTransitionPlan {
            from_room: self.current_room,
            to_room: transition.target_room,
            spawn: TransitionSpawn::Position(transition.new_position),
        })
    }

    pub fn plan_portal_transition(
        &self,
        target_room: RoomId,
        mut room_exists: impl FnMut(RoomId) -> bool,
    ) -> Option<RoomTransitionPlan> {
        if !room_exists(target_room) {
            return None;
        }

        Some(RoomTransitionPlan {
            from_room: self.current_room,
            to_room: target_room,
            spawn: TransitionSpawn::AtPortal,
        })
    }

    pub fn complete_transition(&mut self, target_room: RoomId) {
        self.current_room = target_room;
    }

    pub fn plan_death(&self, has_checkpoint: bool) -> DeathPlan {
        if has_checkpoint {
            DeathPlan::ReloadCheckpoint
        } else {
            DeathPlan::RestartGame
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::progress;
    use crate::core::world::SpawnResolver;

    #[test]
    fn resolves_start_from_pending_checkpoint() {
        progress::reset_all();
        progress::save_checkpoint(
            progress::DEFAULT_SAVE_SLOT,
            (2, 1),
            Vector2::new(12.0, 24.0),
        );
        assert!(progress::queue_load(progress::DEFAULT_SAVE_SLOT));

        let resolver = SpawnResolver::new((0, 0), Vector2::new(1.0, 2.0));
        let mut session = RoomSession::new((0, 0));
        let spawn = session.resolve_start(&resolver, |_| true);

        assert_eq!(spawn.room, (2, 1));
        assert_eq!(spawn.position, Vector2::new(12.0, 24.0));
        assert_eq!(session.current_room(), (2, 1));
    }

    #[test]
    fn plans_boundary_transition_when_room_exists() {
        let detector = BoundaryDetector::new(0.5);
        let session = RoomSession::new((0, 1));

        let plan = session.plan_boundary_transition(
            &detector,
            Vector2::new(320.0, 90.0),
            Vector2::new(10.0, 0.0),
            |room| room == (1, 1),
        );

        assert_eq!(
            plan,
            Some(RoomTransitionPlan {
                from_room: (0, 1),
                to_room: (1, 1),
                spawn: TransitionSpawn::Position(Vector2::new(0.0, 90.0)),
            })
        );
    }

    #[test]
    fn death_plan_prefers_checkpoint_reload() {
        let session = RoomSession::new((0, 0));

        assert_eq!(session.plan_death(true), DeathPlan::ReloadCheckpoint);
        assert_eq!(session.plan_death(false), DeathPlan::RestartGame);
    }
}
