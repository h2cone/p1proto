use godot::classes::control::MouseFilter;
use godot::classes::{Control, IControl, Input, InputEvent, InputEventMouseButton};
use godot::global::MouseButton;
use godot::prelude::*;

use crate::game::room_manager::GameRoomManager;
use crate::save;

use super::world_map_model::WorldMapModel;

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
    model: WorldMapModel,
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
            model: WorldMapModel::default(),
            last_size: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        self.base_mut()
            .set_process_mode(godot::classes::node::ProcessMode::ALWAYS);
        self.base_mut().set_process(true);
        self.base_mut().set_mouse_filter(MouseFilter::STOP);
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
            self.model.update_grid_origin(size);
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
        if self.model.select_room_at(pos, self.cell_size) {
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        let rooms = self.model.explored_rooms().to_vec();
        if rooms.is_empty() {
            return;
        }
        let room_color = self.room_color;
        let room_outline_color = self.room_outline_color;
        let selected_color = self.selected_color;

        for room in rooms {
            let pos = self.model.room_to_pos(room);
            let rect = Rect2::new(pos, self.cell_size);
            self.base_mut().draw_rect(rect, room_color);
            self.base_mut()
                .draw_rect_ex(rect, room_outline_color)
                .filled(false)
                .width(1.0)
                .done();
        }

        if let Some(pos) = self.model.selected_room_pos() {
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
        let mut tree = self.base().get_tree();

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
        self.model.refresh_explored(
            save::list_explored_rooms(),
            self.cell_size,
            self.cell_gap,
            self.base().get_size(),
        );
        self.base_mut().queue_redraw();
    }

    fn select_current_room(&mut self) {
        if self.model.select_current_room(self.fetch_current_room()) {
            self.base_mut().queue_redraw();
        }
    }

    fn fetch_current_room(&self) -> Option<Vector2i> {
        let parent = self.base().get_parent()?;
        let room_manager = parent.get_node_or_null(ROOM_MANAGER_NODE)?;
        let room_manager = room_manager.try_cast::<GameRoomManager>().ok()?;
        Some(room_manager.bind().current_room_vector())
    }

    fn is_pause_menu_visible(&self) -> bool {
        self.base()
            .get_parent()
            .and_then(|parent| parent.get_node_or_null(PAUSE_MENU_NODE))
            .and_then(|node| node.try_cast::<Control>().ok())
            .is_some_and(|control| control.is_visible())
    }
}
