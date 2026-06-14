use godot::classes::control::MouseFilter;
use godot::classes::{Control, IControl, Input, InputMap};
use godot::prelude::*;

const DEFAULT_GRID_SPACING: f32 = 8.0;
const GRID_EPSILON: f32 = 0.001;
const GRID_ACTION: &str = "ui_grid_view";

#[derive(GodotClass)]
#[class(base=Control)]
pub struct RoomGridOverlay {
    base: Base<Control>,
    #[export]
    room_size: Vector2,
    #[export]
    grid_spacing: f32,
    #[export]
    line_color: Color,
    #[export]
    line_width: f32,
    last_size: Vector2,
}

#[godot_api]
impl IControl for RoomGridOverlay {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            room_size: default_room_size(),
            grid_spacing: DEFAULT_GRID_SPACING,
            line_color: Color::from_rgba(1.0, 1.0, 1.0, 0.24),
            line_width: 1.0,
            last_size: Vector2::ZERO,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        self.base_mut().set_process(true);
        self.base_mut().set_mouse_filter(MouseFilter::IGNORE);
        self.last_size = self.base().get_size();
    }

    fn process(&mut self, _delta: f64) {
        let should_show = is_grid_action_pressed();
        if self.base().is_visible() != should_show {
            self.base_mut().set_visible(should_show);
            self.base_mut().queue_redraw();
        }

        let size = self.base().get_size();
        if size != self.last_size {
            self.last_size = size;
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        if !self.base().is_visible() {
            return;
        }

        let draw_size = self.draw_size();
        if draw_size.x <= GRID_EPSILON || draw_size.y <= GRID_EPSILON {
            return;
        }

        let color = self.line_color;
        let width = normalize_line_width(self.line_width);

        for x in grid_line_positions(draw_size.x, self.grid_spacing) {
            self.base_mut()
                .draw_line_ex(Vector2::new(x, 0.0), Vector2::new(x, draw_size.y), color)
                .width(width)
                .done();
        }

        for y in grid_line_positions(draw_size.y, self.grid_spacing) {
            self.base_mut()
                .draw_line_ex(Vector2::new(0.0, y), Vector2::new(draw_size.x, y), color)
                .width(width)
                .done();
        }
    }
}

impl RoomGridOverlay {
    fn draw_size(&self) -> Vector2 {
        let control_size = self.base().get_size();
        Vector2::new(
            normalize_extent(self.room_size.x).min(normalize_extent(control_size.x)),
            normalize_extent(self.room_size.y).min(normalize_extent(control_size.y)),
        )
    }
}

fn default_room_size() -> Vector2 {
    crate::core::world::DEFAULT_ROOM_SIZE.vector()
}

fn is_grid_action_pressed() -> bool {
    let input_map = InputMap::singleton();
    if !input_map.has_action(GRID_ACTION) {
        return false;
    }

    Input::singleton().is_action_pressed(GRID_ACTION)
}

fn grid_line_positions(extent: f32, spacing: f32) -> Vec<f32> {
    let extent = normalize_extent(extent);
    let spacing = normalize_grid_spacing(spacing);
    let mut positions = Vec::new();
    let mut position = 0.0;

    while position < extent - GRID_EPSILON {
        positions.push(position);
        position += spacing;
    }

    if positions
        .last()
        .is_none_or(|last| (extent - *last).abs() > GRID_EPSILON)
    {
        positions.push(extent);
    }

    positions
}

fn normalize_grid_spacing(value: f32) -> f32 {
    if value.is_finite() && value > GRID_EPSILON {
        value
    } else {
        DEFAULT_GRID_SPACING
    }
}

fn normalize_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn normalize_line_width(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_multiple_includes_both_edges() {
        assert_eq!(
            grid_line_positions(320.0, 8.0),
            vec![
                0.0, 8.0, 16.0, 24.0, 32.0, 40.0, 48.0, 56.0, 64.0, 72.0, 80.0, 88.0, 96.0, 104.0,
                112.0, 120.0, 128.0, 136.0, 144.0, 152.0, 160.0, 168.0, 176.0, 184.0, 192.0, 200.0,
                208.0, 216.0, 224.0, 232.0, 240.0, 248.0, 256.0, 264.0, 272.0, 280.0, 288.0, 296.0,
                304.0, 312.0, 320.0,
            ]
        );
    }

    #[test]
    fn non_multiple_includes_final_room_edge() {
        assert_eq!(grid_line_positions(22.0, 8.0), vec![0.0, 8.0, 16.0, 22.0]);
    }

    #[test]
    fn invalid_spacing_clamps_to_safe_interval() {
        assert_eq!(grid_line_positions(3.0, 0.0), vec![0.0, 3.0]);
        assert_eq!(grid_line_positions(3.0, f32::NAN), vec![0.0, 3.0]);
    }

    #[test]
    fn default_room_size_matches_runtime_room_size() {
        assert_eq!(
            default_room_size(),
            crate::core::world::DEFAULT_ROOM_SIZE.vector()
        );
    }
}
