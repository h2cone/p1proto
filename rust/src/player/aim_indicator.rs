use godot::prelude::*;

const INPUT_DEADZONE: f32 = 0.01;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AimDirection {
    Up,
    Down,
    Left,
    #[default]
    Right,
}

impl AimDirection {
    pub fn offset(self, distance: f32) -> Vector2 {
        let distance = distance.max(0.0);
        match self {
            Self::Up => Vector2::new(0.0, -distance),
            Self::Down => Vector2::new(0.0, distance),
            Self::Left => Vector2::new(-distance, 0.0),
            Self::Right => Vector2::new(distance, 0.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AimInput {
    pub horizontal: f32,
    pub vertical: f32,
}

impl AimInput {
    pub fn is_idle(self) -> bool {
        self.horizontal.abs() < INPUT_DEADZONE && self.vertical.abs() < INPUT_DEADZONE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AimIndicatorVisual {
    pub facing: AimDirection,
    pub offset: Vector2,
    pub visible: bool,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct AimIndicator {
    #[base]
    base: Base<Node2D>,
    #[export]
    dot_size: i32,
    #[export]
    dot_color: Color,
}

#[godot_api]
impl INode2D for AimIndicator {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            dot_size: 3,
            dot_color: Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    fn ready(&mut self) {
        self.base_mut().queue_redraw();
    }

    fn draw(&mut self) {
        let dot_rect = centered_dot_rect(self.dot_size);
        let dot_color = self.dot_color;
        self.base_mut().draw_rect(dot_rect, dot_color);
    }
}

impl AimIndicator {
    pub fn apply_visual(&mut self, visual: AimIndicatorVisual) {
        let snapped_offset = Vector2::new(visual.offset.x.round(), visual.offset.y.round());
        self.base_mut().set_position(snapped_offset);
        self.base_mut().set_visible(visual.visible);
    }

    pub fn set_indicator_visible(&mut self, visible: bool) {
        self.base_mut().set_visible(visible);
    }
}

fn centered_dot_rect(dot_size: i32) -> Rect2 {
    let dot_size = normalize_dot_size(dot_size) as f32;
    let half_extent = (dot_size - 1.0) * 0.5;
    Rect2::new(
        Vector2::new(-half_extent, -half_extent),
        Vector2::new(dot_size, dot_size),
    )
}

fn normalize_dot_size(dot_size: i32) -> i32 {
    let dot_size = dot_size.max(1);
    if dot_size % 2 == 0 {
        dot_size + 1
    } else {
        dot_size
    }
}

pub fn resolve_indicator_visual(
    current_facing: AimDirection,
    input: AimInput,
    distance: f32,
) -> AimIndicatorVisual {
    let facing = resolve_next_direction(current_facing, input);
    AimIndicatorVisual {
        facing,
        offset: facing.offset(distance),
        visible: !input.is_idle(),
    }
}

fn resolve_next_direction(current_facing: AimDirection, input: AimInput) -> AimDirection {
    if input.vertical < -INPUT_DEADZONE {
        AimDirection::Up
    } else if input.vertical > INPUT_DEADZONE {
        AimDirection::Down
    } else if input.horizontal < -INPUT_DEADZONE {
        AimDirection::Left
    } else if input.horizontal > INPUT_DEADZONE {
        AimDirection::Right
    } else {
        current_facing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_last_facing_but_hides_when_idle() {
        let visual = resolve_indicator_visual(AimDirection::Left, AimInput::default(), 12.0);

        assert_eq!(visual.facing, AimDirection::Left);
        assert_eq!(visual.offset, Vector2::new(-12.0, 0.0));
        assert!(!visual.visible);
    }

    #[test]
    fn horizontal_input_updates_facing_and_offset() {
        let visual = resolve_indicator_visual(
            AimDirection::Right,
            AimInput {
                horizontal: -1.0,
                vertical: 0.0,
            },
            10.0,
        );

        assert_eq!(visual.facing, AimDirection::Left);
        assert_eq!(visual.offset, Vector2::new(-10.0, 0.0));
        assert!(visual.visible);
    }

    #[test]
    fn vertical_input_updates_facing_and_offset() {
        let visual = resolve_indicator_visual(
            AimDirection::Right,
            AimInput {
                horizontal: 0.0,
                vertical: 1.0,
            },
            8.0,
        );

        assert_eq!(visual.facing, AimDirection::Down);
        assert_eq!(visual.offset, Vector2::new(0.0, 8.0));
        assert!(visual.visible);
    }

    #[test]
    fn vertical_input_wins_over_horizontal_for_four_way_indicator() {
        let visual = resolve_indicator_visual(
            AimDirection::Left,
            AimInput {
                horizontal: 1.0,
                vertical: -1.0,
            },
            6.0,
        );

        assert_eq!(visual.facing, AimDirection::Up);
        assert_eq!(visual.offset, Vector2::new(0.0, -6.0));
        assert!(visual.visible);
    }

    #[test]
    fn negative_distance_clamps_to_zero() {
        let visual = resolve_indicator_visual(
            AimDirection::Right,
            AimInput {
                horizontal: 1.0,
                vertical: 0.0,
            },
            -4.0,
        );

        assert_eq!(visual.offset, Vector2::ZERO);
        assert!(visual.visible);
    }

    #[test]
    fn dot_rect_stays_centered_for_odd_sizes() {
        let rect = centered_dot_rect(3);

        assert_eq!(rect.position, Vector2::new(-1.0, -1.0));
        assert_eq!(rect.size, Vector2::new(3.0, 3.0));
    }

    #[test]
    fn even_dot_sizes_normalize_to_odd_pixel_square() {
        assert_eq!(normalize_dot_size(0), 1);
        assert_eq!(normalize_dot_size(2), 3);
        assert_eq!(normalize_dot_size(5), 5);
    }
}
