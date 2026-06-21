use godot::classes::{
    AnimatedSprite2D, Area2D, CollisionShape2D, IArea2D, Node, Node2D, RectangleShape2D, Shape2D,
};
use godot::prelude::*;

use crate::player::water::WaterEventKind;

const DEFAULT_WIDTH_PX: f32 = 160.0;
const DEFAULT_HEIGHT_PX: f32 = 64.0;
const MIN_DIMENSION_PX: f32 = 1.0;
const COLLISION_SHAPE_PATH: &str = "CollisionShape2D";
const SURFACE_TILES_PATH: &str = "SurfaceTiles";
const FILL_TILES_PATH: &str = "FillTiles";
const SURFACE_TEMPLATE_PATH: &str = "SurfaceTileTemplate";
const FILL_TEMPLATE_PATH: &str = "FillTileTemplate";
const SPLASH_PLAYER_PATH: &str = "SplashPlayer";
const BUBBLE_PLAYER_PATH: &str = "BubblePlayer";
const SURFACE_TILE_WIDTH_PX: f32 = 48.0;
const FILL_TILE_WIDTH_PX: f32 = 48.0;
const FILL_TILE_HEIGHT_PX: f32 = 32.0;
const FILL_SURFACE_OVERLAP_PX: f32 = 24.0;
const SURFACE_LOOP_FRAME_COUNT: i32 = 4;
const FILL_LOOP_FRAME_COUNT: i32 = 2;
pub const WATER_ZONE_GROUP: &str = "water_zone";

struct AnimatedTileStrip<'a> {
    container_path: &'a str,
    template_path: &'a str,
    animation: &'a str,
    count: usize,
    start: Vector2,
    step_x: f32,
}

/// Rectangular water volume driven by an LDtk entity size.
#[derive(GodotClass)]
#[class(tool, base=Area2D)]
pub struct WaterZone {
    #[base]
    base: Base<Area2D>,

    /// Water volume width in pixels. LDtk writes this from the entity rectangle width.
    #[export]
    #[var(get = get_width_px, set = set_width_px)]
    width_px: f32,

    /// Water volume height in pixels. LDtk writes this from the entity rectangle height.
    #[export]
    #[var(get = get_height_px, set = set_height_px)]
    height_px: f32,
}

#[godot_api]
impl IArea2D for WaterZone {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            width_px: DEFAULT_WIDTH_PX,
            height_px: DEFAULT_HEIGHT_PX,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group(WATER_ZONE_GROUP);
        self.sync_template();
    }

    fn process(&mut self, _delta: f64) {
        self.hide_finished_player(SPLASH_PLAYER_PATH);
        self.hide_finished_player(BUBBLE_PLAYER_PATH);
    }
}

#[godot_api]
impl WaterZone {
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
    fn get_height_px(&self) -> f32 {
        self.height_px
    }

    #[func]
    fn set_height_px(&mut self, value: f32) {
        self.height_px = normalize_dimension(value);
        self.sync_template();
    }

    #[func]
    pub fn water_size(&self) -> Vector2 {
        Vector2::new(self.width_px, self.height_px)
    }

    #[func]
    pub fn bounds(&self) -> Rect2 {
        Rect2::new(
            Vector2::new(-self.width_px * 0.5, -self.height_px * 0.5),
            self.water_size(),
        )
    }

    #[func]
    pub fn surface_y(&self) -> f32 {
        self.base().get_global_position().y - self.height_px * 0.5
    }

    pub(crate) fn play_water_event(&mut self, kind: WaterEventKind, global_position: Vector2) {
        let local_position = global_position - self.base().get_global_position();
        let clamped = clamp_local_event_position(local_position, self.bounds());

        match kind {
            WaterEventKind::EnterSurface => {
                self.play_one_shot(SPLASH_PLAYER_PATH, "enter_splash", clamped);
            }
            WaterEventKind::ExitWater => {
                self.play_one_shot(SPLASH_PLAYER_PATH, "exit_splash", clamped);
            }
            WaterEventKind::Dive => {
                self.play_one_shot(BUBBLE_PLAYER_PATH, "dive_bubbles", clamped);
            }
            WaterEventKind::SwimTick => {
                self.play_one_shot(BUBBLE_PLAYER_PATH, "swim_bubbles", clamped);
            }
        }
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
        rectangle.set_size(self.water_size());

        let shape = rectangle.upcast::<Shape2D>();
        collision_shape.set_shape(&shape);
    }

    fn sync_visual(&mut self) {
        self.rebuild_surface_tiles();
        self.rebuild_fill_tiles();
    }

    fn rebuild_surface_tiles(&mut self) {
        let top_y = -self.height_px * 0.5;
        let start_x = -self.width_px * 0.5;
        let count = tile_count_for_dimension(self.width_px, SURFACE_TILE_WIDTH_PX);
        self.rebuild_animated_tiles(AnimatedTileStrip {
            container_path: SURFACE_TILES_PATH,
            template_path: SURFACE_TEMPLATE_PATH,
            animation: "surface_loop",
            count,
            start: Vector2::new(start_x, top_y),
            step_x: SURFACE_TILE_WIDTH_PX,
        });
    }

    fn rebuild_fill_tiles(&mut self) {
        let start_x = -self.width_px * 0.5;
        let start_y = -self.height_px * 0.5 + FILL_SURFACE_OVERLAP_PX;
        let count_x = tile_count_for_dimension(self.width_px, FILL_TILE_WIDTH_PX);
        let count_y = fill_tile_row_count(self.height_px);

        let Some(mut container) = self.base().try_get_node_as::<Node2D>(FILL_TILES_PATH) else {
            return;
        };
        clear_children(&mut container);

        for y in 0..count_y {
            for x in 0..count_x {
                let initial_frame = ((x + y) as i32) % FILL_LOOP_FRAME_COUNT;
                if let Some(mut tile) =
                    self.duplicate_template(FILL_TEMPLATE_PATH, "fill_loop", initial_frame)
                {
                    tile.set_position(Vector2::new(
                        start_x + x as f32 * FILL_TILE_WIDTH_PX,
                        start_y + y as f32 * FILL_TILE_HEIGHT_PX,
                    ));
                    container.add_child(&tile);
                }
            }
        }
    }

    fn rebuild_animated_tiles(&mut self, strip: AnimatedTileStrip) {
        let Some(mut container) = self.base().try_get_node_as::<Node2D>(strip.container_path)
        else {
            return;
        };
        clear_children(&mut container);

        for index in 0..strip.count {
            let initial_frame = (index as i32) % SURFACE_LOOP_FRAME_COUNT;
            if let Some(mut tile) =
                self.duplicate_template(strip.template_path, strip.animation, initial_frame)
            {
                tile.set_position(strip.start + Vector2::new(index as f32 * strip.step_x, 0.0));
                container.add_child(&tile);
            }
        }
    }

    fn duplicate_template(
        &self,
        template_path: &str,
        animation: &str,
        initial_frame: i32,
    ) -> Option<Gd<AnimatedSprite2D>> {
        let template = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>(template_path)?;
        let mut duplicate = template.duplicate_node();
        duplicate.show();
        duplicate.set_animation(animation);
        duplicate.set_frame(initial_frame);
        duplicate.play();
        Some(duplicate)
    }

    fn play_one_shot(&mut self, player_path: &str, animation: &str, local_position: Vector2) {
        let Some(mut player) = self.base().try_get_node_as::<AnimatedSprite2D>(player_path) else {
            return;
        };

        player.set_position(local_position);
        player.set_animation(animation);
        player.set_frame(0);
        player.show();
        player.play();
    }

    fn hide_finished_player(&mut self, player_path: &str) {
        let Some(mut player) = self.base().try_get_node_as::<AnimatedSprite2D>(player_path) else {
            return;
        };

        if player.is_visible() && !player.is_playing() {
            player.hide();
        }
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

fn tile_count_for_dimension(dimension: f32, tile_size: f32) -> usize {
    (dimension / tile_size).ceil().max(1.0) as usize
}

fn fill_tile_row_count(height_px: f32) -> usize {
    tile_count_for_dimension(
        (height_px - FILL_SURFACE_OVERLAP_PX).max(1.0),
        FILL_TILE_HEIGHT_PX,
    )
}

fn clamp_local_event_position(position: Vector2, bounds: Rect2) -> Vector2 {
    Vector2::new(
        position
            .x
            .clamp(bounds.position.x, bounds.position.x + bounds.size.x),
        position
            .y
            .clamp(bounds.position.y, bounds.position.y + bounds.size.y),
    )
}

fn clear_children(container: &mut Gd<Node2D>) {
    for child in container.get_children().iter_shared() {
        if let Ok(mut node) = child.try_cast::<Node>() {
            container.remove_child(&node);
            node.queue_free();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_water_dimensions_to_positive_finite_size() {
        assert_eq!(normalize_dimension(160.0), 160.0);
        assert_eq!(normalize_dimension(0.0), 1.0);
        assert_eq!(normalize_dimension(-8.0), 1.0);
        assert_eq!(normalize_dimension(f32::NAN), 1.0);
    }

    #[test]
    fn tile_count_covers_partial_dimensions() {
        assert_eq!(tile_count_for_dimension(1.0, 16.0), 1);
        assert_eq!(tile_count_for_dimension(16.0, 16.0), 1);
        assert_eq!(tile_count_for_dimension(17.0, 16.0), 2);
        assert_eq!(tile_count_for_dimension(48.0, 16.0), 3);
    }

    #[test]
    fn fill_tile_rows_cover_water_body_below_surface_overlap() {
        assert_eq!(fill_tile_row_count(32.0), 1);
        assert_eq!(fill_tile_row_count(64.0), 2);
        assert_eq!(fill_tile_row_count(104.0), 3);
    }

    #[test]
    fn clamps_local_event_position_to_water_bounds() {
        let bounds = Rect2::new(Vector2::new(-80.0, -32.0), Vector2::new(160.0, 64.0));

        assert_eq!(
            clamp_local_event_position(Vector2::new(0.0, -32.0), bounds),
            Vector2::new(0.0, -32.0)
        );
        assert_eq!(
            clamp_local_event_position(Vector2::new(-120.0, 64.0), bounds),
            Vector2::new(-80.0, 32.0)
        );
    }
}
