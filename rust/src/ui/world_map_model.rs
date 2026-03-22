use std::collections::HashSet;

use godot::prelude::*;

type RoomId = (i32, i32);

#[derive(Default)]
pub struct WorldMapModel {
    explored_rooms: Vec<Vector2i>,
    explored_set: HashSet<RoomId>,
    selected_room: Option<RoomId>,
    min_room: Vector2i,
    grid_origin: Vector2,
    grid_pitch: Vector2,
    grid_size: Vector2,
}

impl WorldMapModel {
    pub fn refresh_explored(
        &mut self,
        mut rooms: Vec<RoomId>,
        cell_size: Vector2,
        cell_gap: Vector2,
        control_size: Vector2,
    ) {
        rooms.sort_by_key(|(x, y)| (*y, *x));

        self.explored_rooms.clear();
        self.explored_set.clear();
        self.selected_room = None;
        self.grid_pitch = cell_size + cell_gap;

        if rooms.is_empty() {
            self.min_room = Vector2i::ZERO;
            self.grid_size = Vector2::ZERO;
            self.update_grid_origin(control_size);
            return;
        }

        let mut min_x = rooms[0].0;
        let mut max_x = rooms[0].0;
        let mut min_y = rooms[0].1;
        let mut max_y = rooms[0].1;

        for (x, y) in rooms {
            self.explored_rooms.push(Vector2i::new(x, y));
            self.explored_set.insert((x, y));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        self.min_room = Vector2i::new(min_x, min_y);
        self.grid_size = Vector2::new(
            (max_x - min_x + 1) as f32 * self.grid_pitch.x - cell_gap.x,
            (max_y - min_y + 1) as f32 * self.grid_pitch.y - cell_gap.y,
        );
        self.update_grid_origin(control_size);
    }

    pub fn update_grid_origin(&mut self, control_size: Vector2) {
        if self.grid_size == Vector2::ZERO {
            self.grid_origin = control_size * 0.5;
            return;
        }

        self.grid_origin = (control_size - self.grid_size) * 0.5;
    }

    pub fn select_current_room(&mut self, room: Option<Vector2i>) -> bool {
        if let Some(room) = room {
            let key = (room.x, room.y);
            if self.explored_set.contains(&key) {
                self.selected_room = Some(key);
                return true;
            }
        }

        if let Some(room) = self.explored_rooms.first() {
            self.selected_room = Some((room.x, room.y));
            return true;
        }

        false
    }

    pub fn select_room_at(&mut self, pos: Vector2, cell_size: Vector2) -> bool {
        if self.explored_rooms.is_empty() {
            return false;
        }

        let local = pos - self.grid_origin;
        if local.x < 0.0 || local.y < 0.0 {
            return false;
        }
        if local.x > self.grid_size.x || local.y > self.grid_size.y {
            return false;
        }

        let cell_x = (local.x / self.grid_pitch.x).floor() as i32;
        let cell_y = (local.y / self.grid_pitch.y).floor() as i32;
        let inside_x = local.x - cell_x as f32 * self.grid_pitch.x;
        let inside_y = local.y - cell_y as f32 * self.grid_pitch.y;

        if inside_x > cell_size.x || inside_y > cell_size.y {
            return false;
        }

        let room = (self.min_room.x + cell_x, self.min_room.y + cell_y);
        if self.explored_set.contains(&room) {
            self.selected_room = Some(room);
            return true;
        }

        false
    }

    pub fn explored_rooms(&self) -> &[Vector2i] {
        &self.explored_rooms
    }

    #[cfg(test)]
    pub fn selected_room(&self) -> Option<RoomId> {
        self.selected_room
    }

    pub fn selected_room_pos(&self) -> Option<Vector2> {
        let room = self.selected_room?;
        self.explored_set
            .contains(&room)
            .then_some(self.room_to_pos(Vector2i::new(room.0, room.1)))
    }

    pub fn room_to_pos(&self, room: Vector2i) -> Vector2 {
        self.grid_origin
            + Vector2::new(
                (room.x - self.min_room.x) as f32 * self.grid_pitch.x,
                (room.y - self.min_room.y) as f32 * self.grid_pitch.y,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_current_room_when_explored() {
        let mut model = WorldMapModel::default();
        model.refresh_explored(
            vec![(0, 1), (1, 1)],
            Vector2::new(18.0, 18.0),
            Vector2::new(6.0, 6.0),
            Vector2::new(200.0, 200.0),
        );

        assert!(model.select_current_room(Some(Vector2i::new(1, 1))));
        assert_eq!(model.selected_room(), Some((1, 1)));
    }

    #[test]
    fn selection_ignores_gap_hits() {
        let mut model = WorldMapModel::default();
        let cell_size = Vector2::new(18.0, 18.0);
        let cell_gap = Vector2::new(6.0, 6.0);
        model.refresh_explored(
            vec![(0, 0), (1, 0)],
            cell_size,
            cell_gap,
            Vector2::new(100.0, 50.0),
        );

        let left_cell = model.room_to_pos(Vector2i::new(0, 0));
        let in_gap = left_cell + Vector2::new(cell_size.x + 2.0, 2.0);

        assert!(!model.select_room_at(in_gap, cell_size));
    }
}
