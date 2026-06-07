use godot::classes::{
    AnimatedSprite2D, Area2D, CollisionShape2D, IArea2D, RectangleShape2D, Shape2D, Texture2D,
};
use godot::prelude::*;

const DEFAULT_WIDTH_PX: f32 = 16.0;
const DEFAULT_LENGTH_PX: f32 = 64.0;
const DEFAULT_RUNG_PITCH_PX: f32 = 8.0;
// Repeating a clean interior slice avoids stretching caps or sparse moss accents.
const SOURCE_MID_Y_PX: f32 = 16.0;
const SOURCE_BOTTOM_Y_PX: f32 = DEFAULT_LENGTH_PX - DEFAULT_RUNG_PITCH_PX;
const MIN_DIMENSION_PX: f32 = 1.0;
const DRAW_EPSILON_PX: f32 = 0.001;
const COLLISION_SHAPE_PATH: &str = "CollisionShape2D";
const VISUAL_PATH: &str = "AnimatedSprite2D";
const LADDER_GROUP: &str = "ladder";
const DEFAULT_ANIMATION: &str = "default";

/// Climbable ladder bounds driven by LDtk entity size.
#[derive(GodotClass)]
#[class(tool, base=Area2D)]
pub struct Ladder {
    #[base]
    base: Base<Area2D>,

    /// Climbable width in pixels. Usually fixed at 16px from LDtk.
    #[export]
    #[var(get = get_width_px, set = set_width_px)]
    width_px: f32,

    /// Climbable length in pixels. LDtk writes this from the entity rectangle height.
    #[export]
    #[var(get = get_length_px, set = set_length_px)]
    length_px: f32,

    /// Visual rung spacing in pixels. Runtime collision does not depend on this value.
    #[export]
    #[var(get = get_rung_pitch_px, set = set_rung_pitch_px)]
    rung_pitch_px: f32,
}

#[godot_api]
impl IArea2D for Ladder {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            width_px: DEFAULT_WIDTH_PX,
            length_px: DEFAULT_LENGTH_PX,
            rung_pitch_px: DEFAULT_RUNG_PITCH_PX,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group(LADDER_GROUP);
        self.sync_template();
    }

    fn draw(&mut self) {
        self.draw_ladder();
    }
}

#[godot_api]
impl Ladder {
    #[func]
    fn get_width_px(&self) -> f32 {
        self.width_px
    }

    #[func]
    fn set_width_px(&mut self, value: f32) {
        self.width_px = normalize_dimension(value);
        self.sync_template();
    }

    #[func]
    fn get_length_px(&self) -> f32 {
        self.length_px
    }

    #[func]
    fn set_length_px(&mut self, value: f32) {
        self.length_px = normalize_dimension(value);
        self.sync_template();
    }

    #[func]
    fn get_rung_pitch_px(&self) -> f32 {
        self.rung_pitch_px
    }

    #[func]
    fn set_rung_pitch_px(&mut self, value: f32) {
        self.rung_pitch_px = normalize_dimension(value);
        self.base_mut().queue_redraw();
    }

    #[func]
    pub fn climb_size(&self) -> Vector2 {
        Vector2::new(self.width_px, self.length_px)
    }

    fn sync_template(&mut self) {
        self.sync_collision_shape();
        self.sync_visual();
    }

    fn sync_collision_shape(&mut self) {
        let Some(mut collision_shape) = self
            .base()
            .try_get_node_as::<CollisionShape2D>(COLLISION_SHAPE_PATH)
        else {
            return;
        };

        let mut rectangle =
            existing_rectangle_shape(&collision_shape).unwrap_or_else(RectangleShape2D::new_gd);
        rectangle.set_size(self.climb_size());

        let shape = rectangle.upcast::<Shape2D>();
        collision_shape.set_shape(&shape);
    }

    fn sync_visual(&mut self) {
        let Some(mut visual) = self.base().try_get_node_as::<AnimatedSprite2D>(VISUAL_PATH) else {
            return;
        };

        visual.set_animation(DEFAULT_ANIMATION);
        visual.set_scale(Vector2::new(1.0, 1.0));
        visual.hide();
        self.base_mut().queue_redraw();
    }

    fn draw_ladder(&mut self) {
        let Some(texture) = self.template_texture() else {
            return;
        };

        let cap_height = DEFAULT_RUNG_PITCH_PX.min(self.length_px * 0.5);
        let x = -self.width_px * 0.5;
        let top_y = -self.length_px * 0.5;
        let bottom_y = top_y + self.length_px - cap_height;

        self.draw_ladder_region(
            &texture,
            x,
            top_y,
            self.width_px,
            cap_height,
            0.0,
            cap_height,
        );

        if bottom_y > top_y + cap_height + DRAW_EPSILON_PX {
            self.draw_middle_regions(&texture, x, top_y + cap_height, bottom_y);
        }

        if self.length_px > cap_height + DRAW_EPSILON_PX {
            self.draw_ladder_region(
                &texture,
                x,
                bottom_y,
                self.width_px,
                cap_height,
                SOURCE_BOTTOM_Y_PX,
                cap_height,
            );
        }
    }

    fn draw_middle_regions(&mut self, texture: &Gd<Texture2D>, x: f32, start_y: f32, end_y: f32) {
        let mut y = start_y;
        while y < end_y - DRAW_EPSILON_PX {
            let segment_height = self.rung_pitch_px.min(end_y - y);
            let source_height = segment_height.min(DEFAULT_RUNG_PITCH_PX);
            self.draw_ladder_region(
                texture,
                x,
                y,
                self.width_px,
                segment_height,
                SOURCE_MID_Y_PX,
                source_height,
            );
            y += segment_height;
        }
    }

    fn draw_ladder_region(
        &mut self,
        texture: &Gd<Texture2D>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        source_y: f32,
        source_height: f32,
    ) {
        let rect = Rect2::new(Vector2::new(x, y), Vector2::new(width, height));
        let source_rect = Rect2::new(
            Vector2::new(0.0, source_y),
            Vector2::new(DEFAULT_WIDTH_PX, source_height),
        );
        self.base_mut()
            .draw_texture_rect_region(texture, rect, source_rect);
    }

    fn template_texture(&self) -> Option<Gd<Texture2D>> {
        let visual = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>(VISUAL_PATH)?;
        let frames = visual.get_sprite_frames()?;
        frames.get_frame_texture(DEFAULT_ANIMATION, 0)
    }
}

fn existing_rectangle_shape(
    collision_shape: &Gd<CollisionShape2D>,
) -> Option<Gd<RectangleShape2D>> {
    collision_shape
        .get_shape()
        .and_then(|shape| shape.try_cast::<RectangleShape2D>().ok())
}

fn normalize_dimension(value: f32) -> f32 {
    if value.is_finite() {
        value.max(MIN_DIMENSION_PX)
    } else {
        MIN_DIMENSION_PX
    }
}
