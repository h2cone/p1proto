use std::collections::HashSet;

use godot::classes::control::MouseFilter;
use godot::classes::{Control, IControl, Input, InputEvent, InputEventMouseButton};
use godot::global::MouseButton;
use godot::prelude::*;

use crate::save;

const MAP_ACTION: &str = "ui_map";
const ROOM_MANAGER_NODE: &str = "RoomManager";
const PAUSE_MENU_NODE: &str = "PauseMenu";
const SELECT_OUTLINE_PAD: f32 = 3.0;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct WorldMap {
    base: Base<Control>,
    #[export]
    cell_size: Vector2,
    #[export]
    cell_gap: Vector2,
    #[export]
    room_color: Color,
    #[export]
    room_outline_color: Color,
    #[export]
    selected_color: Color,
    explored_rooms: Vec<Vector2i>,
    explored_set: HashSet<(i32, i32)>,
    selected_room: Option<(i32, i32)>,
    min_room: Vector2i,
    max_room: Vector2i,
    grid_origin: Vector2,
    grid_pitch: Vector2,
    grid_size: Vector2,
    last_size: Vector2,
}

#[godot_api]
impl IControl for WorldMap {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            cell_size: Vector2::new(18.0, 18.0),
            cell_gap: Vector2::new(6.0, 6.0),
            room_color: Color::from_rgba(0.55, 0.6, 0.65, 1.0),
            room_outline_color: Color::from_rgba(0.2, 0.22, 0.25, 1.0),
            selected_color: Color::from_rgba(1.0, 0.85, 0.35, 1.0),
            explored_rooms: Vec::new(),
            explored_set: HashSet::new(),
            selected_room: None,
            min_room: Vector2i::ZERO,
            max_room: Vector2i::ZERO,
            grid_origin: Vector2::ZERO,
            grid_pitch: Vector2::ZERO,
            grid_size: Vector2::ZERO,
            last_size: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        self.base_mut()
            .set_process_mode(godot::classes::node::ProcessMode::ALWAYS);
        self.base_mut().set_process(true);
        self.base_mut().set_mouse_filter(MouseFilter::STOP);
        self.grid_pitch = self.cell_size + self.cell_gap;
        self.last_size = self.base().get_size();
    }

    fn process(&mut self, _delta: f64) {
        let input = Input::singleton();
        if input.is_action_just_pressed(MAP_ACTION) {
            if self.base().is_visible() {
                self.close_map();
            } else if !self.is_pause_menu_visible() {
                self.open_map();
            }
        }

        if !self.base().is_visible() {
            return;
        }

        let size = self.base().get_size();
        if size != self.last_size {
            self.last_size = size;
            self.update_grid_origin();
            self.base_mut().queue_redraw();
        }
    }

    fn gui_input(&mut self, event: Gd<InputEvent>) {
        if !self.base().is_visible() {
            return;
        }

        let pressed = event.is_pressed();
        let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };

        if !pressed || mouse_event.get_button_index() != MouseButton::LEFT {
            return;
        }

        let pos = self.base().get_local_mouse_position();
        self.select_room_at(pos);
    }

    fn draw(&mut self) {
        if self.explored_rooms.is_empty() {
            return;
        }

        let rooms = self.explored_rooms.clone();
        let room_color = self.room_color;
        let room_outline_color = self.room_outline_color;
        let selected_color = self.selected_color;
        let selected_room = self.selected_room.map(|(x, y)| Vector2i::new(x, y));
        let selected_is_explored = selected_room
            .map(|room| self.explored_set.contains(&(room.x, room.y)))
            .unwrap_or(false);
        let selected_pos = if selected_is_explored {
            selected_room.map(|room| self.room_to_pos(room))
        } else {
            None
        };

        for room in rooms {
            let pos = self.room_to_pos(room);
            let rect = Rect2::new(pos, self.cell_size);
            self.base_mut().draw_rect(rect, room_color);
            self.base_mut()
                .draw_rect_ex(rect, room_outline_color)
                .filled(false)
                .width(1.0)
                .done();
        }

        if let Some(pos) = selected_pos {
            let rect = Rect2::new(pos, self.cell_size);
            let fill = Color::from_rgba(selected_color.r, selected_color.g, selected_color.b, 0.25);
            self.base_mut().draw_rect_ex(rect, fill).filled(true).done();

            let pad = Vector2::new(SELECT_OUTLINE_PAD, SELECT_OUTLINE_PAD);
            let outline_rect = Rect2::new(pos - pad, self.cell_size + pad * 2.0);
            self.base_mut()
                .draw_rect_ex(outline_rect, selected_color)
                .filled(false)
                .width(2.0)
                .done();
        }
    }
}

impl WorldMap {
    fn open_map(&mut self) {
        self.base_mut().set_visible(true);
        self.refresh_explored();
        self.select_current_room();
        self.apply_pause(true);
    }

    fn close_map(&mut self) {
        self.base_mut().set_visible(false);
        self.apply_pause(false);
    }

    fn apply_pause(&mut self, should_pause: bool) {
        let Some(mut tree) = self.base().get_tree() else {
            return;
        };

        if should_pause {
            tree.set_pause(true);
            return;
        }

        if self.is_pause_menu_visible() {
            return;
        }

        tree.set_pause(false);
    }

    fn refresh_explored(&mut self) {
        let mut rooms = save::list_explored_rooms();
        rooms.sort_by_key(|(x, y)| (*y, *x));

        self.explored_rooms.clear();
        self.explored_set.clear();
        self.selected_room = None;

        if rooms.is_empty() {
            self.min_room = Vector2i::ZERO;
            self.max_room = Vector2i::ZERO;
            self.grid_size = Vector2::ZERO;
            self.update_grid_origin();
            self.base_mut().queue_redraw();
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
        self.max_room = Vector2i::new(max_x, max_y);
        self.grid_pitch = self.cell_size + self.cell_gap;
        self.grid_size = Vector2::new(
            (max_x - min_x + 1) as f32 * self.grid_pitch.x - self.cell_gap.x,
            (max_y - min_y + 1) as f32 * self.grid_pitch.y - self.cell_gap.y,
        );
        self.update_grid_origin();
        self.base_mut().queue_redraw();
    }

    fn update_grid_origin(&mut self) {
        if self.grid_size == Vector2::ZERO {
            self.grid_origin = self.base().get_size() * 0.5;
            return;
        }

        let size = self.base().get_size();
        self.grid_origin = (size - self.grid_size) * 0.5;
    }

    fn select_current_room(&mut self) {
        let current_room = self.fetch_current_room();
        if let Some(room) = current_room {
            let key = (room.x, room.y);
            if self.explored_set.contains(&key) {
                self.selected_room = Some(key);
                self.base_mut().queue_redraw();
                return;
            }
        }

        if let Some(room) = self.explored_rooms.first() {
            self.selected_room = Some((room.x, room.y));
            self.base_mut().queue_redraw();
        }
    }

    fn fetch_current_room(&self) -> Option<Vector2i> {
        let parent = self.base().get_parent()?;
        let mut room_manager = parent.get_node_or_null(ROOM_MANAGER_NODE)?;
        if !room_manager.has_method("get_current_room") {
            return None;
        }

        let value = room_manager.call("get_current_room", &[]);
        Some(value.to::<Vector2i>())
    }

    fn select_room_at(&mut self, pos: Vector2) {
        if self.explored_rooms.is_empty() {
            return;
        }

        let local = pos - self.grid_origin;
        if local.x < 0.0 || local.y < 0.0 {
            return;
        }
        if local.x > self.grid_size.x || local.y > self.grid_size.y {
            return;
        }

        let cell_x = (local.x / self.grid_pitch.x).floor() as i32;
        let cell_y = (local.y / self.grid_pitch.y).floor() as i32;
        let inside_x = local.x - cell_x as f32 * self.grid_pitch.x;
        let inside_y = local.y - cell_y as f32 * self.grid_pitch.y;

        if inside_x > self.cell_size.x || inside_y > self.cell_size.y {
            return;
        }

        let room = (self.min_room.x + cell_x, self.min_room.y + cell_y);
        if self.explored_set.contains(&room) {
            self.selected_room = Some(room);
            self.base_mut().queue_redraw();
        }
    }

    fn room_to_pos(&self, room: Vector2i) -> Vector2 {
        self.grid_origin
            + Vector2::new(
                (room.x - self.min_room.x) as f32 * self.grid_pitch.x,
                (room.y - self.min_room.y) as f32 * self.grid_pitch.y,
            )
    }

    fn is_pause_menu_visible(&self) -> bool {
        let Some(parent) = self.base().get_parent() else {
            return false;
        };
        let Some(pause_menu) = parent.get_node_or_null(PAUSE_MENU_NODE) else {
            return false;
        };
        if let Ok(control) = pause_menu.try_cast::<Control>() {
            return control.is_visible();
        }
        false
    }
}
